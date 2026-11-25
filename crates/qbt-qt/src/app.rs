//! `AppController`: connection lifecycle, status properties, and every action QML can trigger.
// The QML-facing API takes one argument per form field, so some invokables are wide.
#![allow(clippy::too_many_arguments)]

use core::pin::Pin;
use std::{cell::RefCell, collections::HashMap, path::PathBuf, sync::Mutex};

use cxx_qt::Threading;
use cxx_qt_lib::{QString, QVariant};
use qbt_api::{Error, models::torrents::AddOptions};
use qbt_core::{
    AuthKind, ConnectionState, Data, Event, KeyringSecrets, MemorySecrets, Profile, ProfileStore,
    SecretStore, Session, Watch, WatchGuard, actions, errors, fmt,
};
use qbt_sync::{Counts, MainChanges, PeersStore};
use serde_json::{Map, Value};

use crate::{
    model::{notify_main, push_table},
    state::{self, APP},
    tables::{self, Payload},
};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("shim.h");
        fn qbt_grab_window(path: &QString) -> bool;
        fn qbt_attach_model_testers() -> i32;
        fn qbt_install_translator(extra_dir: &QString) -> bool;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, connection_state)]
        #[qproperty(QString, status_message)]
        #[qproperty(QString, server_version)]
        #[qproperty(QString, webapi_version)]
        #[qproperty(QString, build_info)]
        #[qproperty(QString, dl_speed)]
        #[qproperty(QString, ul_speed)]
        #[qproperty(QString, dl_data)]
        #[qproperty(QString, ul_data)]
        #[qproperty(QString, dl_limit)]
        #[qproperty(QString, ul_limit)]
        #[qproperty(i32, dht_nodes)]
        #[qproperty(QString, free_space)]
        #[qproperty(QString, external_ip)]
        #[qproperty(QString, conn_status)]
        #[qproperty(bool, alt_speed)]
        #[qproperty(bool, session_paused)]
        #[qproperty(bool, queueing)]
        #[qproperty(i32, torrent_count)]
        #[qproperty(QString, title_speed)]
        #[qproperty(QString, speed_history)]
        #[qproperty(QString, piece_states)]
        #[qproperty(QString, piece_availability)]
        #[qproperty(bool, cap_download_file)]
        #[qproperty(bool, cap_api_key)]
        #[qproperty(bool, cap_basic_auth)]
        #[qproperty(bool, cap_process_info)]
        type AppController = super::AppControllerRust;

        // ---- connection & profiles
        #[qinvokable]
        fn load_profiles(self: Pin<&mut AppController>);
        #[qinvokable]
        fn save_profile(
            self: Pin<&mut AppController>,
            name: &QString,
            url: &QString,
            auth: &QString,
            username: &QString,
            secret: &QString,
            remember: bool,
            ca_path: &QString,
        );
        #[qinvokable]
        fn delete_profile(self: Pin<&mut AppController>, name: &QString);
        #[qinvokable]
        fn profile_field(self: &AppController, name: &QString, field: &QString) -> QString;
        #[qinvokable]
        fn last_profile(self: &AppController) -> QString;
        #[qinvokable]
        fn connect_profile(
            self: Pin<&mut AppController>,
            name: &QString,
            secret_override: &QString,
        );
        #[qinvokable]
        fn test_connection(
            self: Pin<&mut AppController>,
            url: &QString,
            auth: &QString,
            username: &QString,
            secret: &QString,
            ca_path: &QString,
        );
        #[qinvokable]
        fn disconnect_server(self: Pin<&mut AppController>);
        #[qinvokable]
        fn set_active(self: &AppController, active: bool);
        /// Whether the connected server has `scope/action` (version table plus runtime detection).
        #[qinvokable]
        fn supports(self: &AppController, scope: &QString, action: &QString) -> bool;
        /// Tooltip for a greyed-out feature, e.g. "requires WebAPI ≥ 2.16.0".
        #[qinvokable]
        fn requirement(self: &AppController, scope: &QString, action: &QString) -> QString;

        // ---- client-only settings
        #[qinvokable]
        fn setting(self: &AppController, key: &QString) -> QString;
        #[qinvokable]
        fn set_setting(self: Pin<&mut AppController>, key: &QString, value: &QString);
        /// Push the layout to the server's `clientdata` store when the user opted in.
        #[qinvokable]
        fn sync_ui_store(self: &AppController);

        // ---- actions
        #[qinvokable]
        fn torrent_action(
            self: &AppController,
            action: &QString,
            hashes: &QString,
            arg: &QString,
        ) -> i32;
        #[qinvokable]
        fn add_torrents(
            self: &AppController,
            urls: &QString,
            files: &QString,
            options_json: &QString,
        ) -> i32;
        #[qinvokable]
        fn call(
            self: &AppController,
            tag: &QString,
            scope: &QString,
            action: &QString,
            params_json: &QString,
        ) -> i32;
        #[qinvokable]
        fn save_to_file(
            self: &AppController,
            scope: &QString,
            action: &QString,
            params_json: &QString,
            path: &QString,
        ) -> i32;
        #[qinvokable]
        fn toggle_alt_speed(self: &AppController) -> i32;
        #[qinvokable]
        fn set_global_limits(
            self: &AppController,
            dl: i32,
            ul: i32,
            alt_dl: i32,
            alt_ul: i32,
        ) -> i32;
        #[qinvokable]
        fn pause_session(self: &AppController, pause: bool) -> i32;
        #[qinvokable]
        fn shutdown_server(self: &AppController) -> i32;
        #[qinvokable]
        fn rss_select(self: &AppController, path: &QString);
        #[qinvokable]
        fn rss_import_rules(self: &AppController, path: &QString) -> i32;
        /// One value from the server's `server_state` (raw JSON text), for the statistics dialog.
        #[qinvokable]
        fn server_stat(self: &AppController, key: &QString) -> QString;
        /// Empty a table (e.g. the log view's Clear button).
        #[qinvokable]
        fn clear_table(self: &AppController, source: &QString);

        // ---- detail pollers
        #[qinvokable]
        fn watch(self: &AppController, kind: &QString, arg: &QString);
        #[qinvokable]
        fn unwatch(self: &AppController, kind: &QString);

        // ---- server preferences
        #[qinvokable]
        fn prefs_load(self: &AppController);
        #[qinvokable]
        fn pref(self: &AppController, key: &QString) -> QString;
        #[qinvokable]
        fn pref_is_secret(self: &AppController, key: &QString) -> bool;
        /// `bool`, `int`, `float`, `string`, `json` or `missing`.
        #[qinvokable]
        fn pref_type(self: &AppController, key: &QString) -> QString;
        // ---- add-dialog metadata preview
        #[qinvokable]
        fn preview_files(self: &AppController, files: &QString);
        #[qinvokable]
        fn preview_magnet(self: &AppController, source: &QString);
        #[qinvokable]
        fn preview_cancel(self: &AppController, source: &QString);
        #[qinvokable]
        fn pref_set(self: &AppController, key: &QString, value: &QString);
        #[qinvokable]
        fn pref_dirty_count(self: &AppController) -> i32;
        #[qinvokable]
        fn prefs_discard(self: &AppController);
        #[qinvokable]
        fn prefs_apply(self: &AppController) -> i32;

        // ---- helpers
        #[qinvokable]
        fn format_size(self: &AppController, bytes: i64) -> QString;
        #[qinvokable]
        fn format_speed(self: &AppController, bps: i64) -> QString;
        #[qinvokable]
        fn local_path(self: &AppController, url: &QString) -> QString;
        /// Attach fatal-mode `QAbstractItemModelTester`s to all models (test hook).
        #[qinvokable]
        fn attach_model_testers(self: &AppController) -> i32;
        /// Magnet links and .torrent paths passed on the command line, one per line.
        #[qinvokable]
        fn startup_args(self: &AppController) -> QString;
        /// Opt-in: make this app the system handler for magnet links (Linux; elsewhere the installer does it).
        #[qinvokable]
        fn register_magnet_handler(self: &AppController) -> QString;
        /// Save the window as a PNG (test hook).
        #[qinvokable]
        fn grab_window(self: &AppController, path: &QString) -> bool;
        /// Environment variable (for the headless screenshot hook used by tests).
        #[qinvokable]
        fn env(self: &AppController, name: &QString) -> QString;

        #[qsignal]
        fn command_finished(self: Pin<&mut AppController>, id: i32, ok: bool, message: &QString);
        #[qsignal]
        fn call_result(self: Pin<&mut AppController>, tag: &QString, ok: bool, body: &QString);
        #[qsignal]
        fn toast(self: Pin<&mut AppController>, message: &QString);
        #[qsignal]
        fn torrent_completed(self: Pin<&mut AppController>, name: &QString);
        #[qsignal]
        fn test_result(self: Pin<&mut AppController>, ok: bool, message: &QString);
        #[qsignal]
        fn prefs_loaded(self: Pin<&mut AppController>);
        #[qsignal]
        fn connected(self: Pin<&mut AppController>);
        /// The layout was replaced by the copy stored on the server.
        #[qsignal]
        fn ui_state_loaded(self: Pin<&mut AppController>);
    }
    impl cxx_qt::Threading for AppController {}
}

#[derive(Default)]
pub struct AppControllerRust {
    connection_state: QString,
    status_message: QString,
    server_version: QString,
    webapi_version: QString,
    build_info: QString,
    dl_speed: QString,
    ul_speed: QString,
    dl_data: QString,
    ul_data: QString,
    dl_limit: QString,
    ul_limit: QString,
    dht_nodes: i32,
    free_space: QString,
    external_ip: QString,
    conn_status: QString,
    alt_speed: bool,
    session_paused: bool,
    queueing: bool,
    torrent_count: i32,
    title_speed: QString,
    speed_history: QString,
    piece_states: QString,
    piece_availability: QString,
    cap_download_file: bool,
    cap_api_key: bool,
    cap_basic_auth: bool,
    cap_process_info: bool,
}

fn q(s: &str) -> QString {
    QString::from(s)
}

// ----------------------------------------------------------------------------- globals

/// Key under which the layout is stored in the server's `clientdata`.
const UI_STATE_KEY: &str = "qbremote.ui";

static PROFILES: Mutex<Option<ProfileStore>> = Mutex::new(None);
static SECRETS: Mutex<Option<Box<dyn SecretStore>>> = Mutex::new(None);

thread_local! {
    static WATCHES: RefCell<HashMap<String, WatchGuard>> = RefCell::new(HashMap::new());
    static PEERS: RefCell<(String, PeersStore)> = RefCell::new((String::new(), PeersStore::default()));
    static SPEED: RefCell<std::collections::VecDeque<(i64, i64)>> = const { RefCell::new(std::collections::VecDeque::new()) };
    static RSS_LAST: RefCell<(Value, String)> = const { RefCell::new((Value::Null, String::new())) };
    static PREFS: RefCell<(Map<String, Value>, Map<String, Value>)> = RefCell::new((Map::new(), Map::new()));
}

fn with_profiles<R>(f: impl FnOnce(&mut ProfileStore) -> R) -> R {
    let mut g = PROFILES.lock().unwrap();
    if g.is_none() {
        let path = std::env::var_os("QBREMOTE_CONFIG")
            .map(PathBuf::from)
            .or_else(ProfileStore::default_path)
            .unwrap_or_else(|| PathBuf::from("qbremote.toml"));
        *g = Some(ProfileStore::load(&path).unwrap_or_else(|e| {
            eprintln!("config: {e}; starting with defaults");
            ProfileStore::load(path.with_extension("fresh")).expect("empty store")
        }));
    }
    f(g.as_mut().unwrap())
}

fn with_secrets<R>(f: impl FnOnce(&dyn SecretStore) -> R) -> R {
    let mut g = SECRETS.lock().unwrap();
    let s = g.get_or_insert_with(|| {
        if std::env::var_os("QBREMOTE_MEMORY_SECRETS").is_some() {
            // Test hook: QBREMOTE_TEST_SECRETS="profile=secret;other=secret2" pre-seeds the store.
            let m = MemorySecrets::default();
            for pair in std::env::var("QBREMOTE_TEST_SECRETS")
                .unwrap_or_default()
                .split(';')
            {
                if let Some((k, v)) = pair.split_once('=') {
                    let _ = m.set(k, v);
                }
            }
            Box::new(m)
        } else {
            Box::new(KeyringSecrets)
        }
    });
    f(s.as_ref())
}

fn auth_kind(s: &str) -> AuthKind {
    match s {
        "apikey" => AuthKind::ApiKey,
        "basic" => AuthKind::Basic,
        _ => AuthKind::Cookie,
    }
}

fn auth_name(a: AuthKind) -> &'static str {
    match a {
        AuthKind::Cookie => "cookie",
        AuthKind::ApiKey => "apikey",
        AuthKind::Basic => "basic",
    }
}

fn strip_file_url(p: &str) -> String {
    p.strip_prefix("file://")
        .map(percent_decode)
        .unwrap_or_else(|| p.to_owned())
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let Some(hex) = s.get(i + 1..i + 3)
            && let Ok(v) = u8::from_str_radix(hex, 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// QML uses camelCase setting names; `ClientSettings` serializes as snake_case.
pub fn snake_case(key: &str) -> String {
    let mut out = String::with_capacity(key.len() + 4);
    for c in key.chars() {
        if c.is_ascii_uppercase() {
            out.push('_');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Command-line arguments that name something to add: magnet links, http(s) torrent URLs, and
/// `.torrent` paths (plain or `file://`). Flags are ignored.
pub fn startup_sources(args: impl Iterator<Item = String>) -> Vec<String> {
    args.filter(|a| {
        !a.starts_with('-')
            && (a.starts_with("magnet:")
                || a.starts_with("http://")
                || a.starts_with("https://")
                || a.starts_with("file://")
                || a.to_ascii_lowercase().ends_with(".torrent"))
    })
    .collect()
}

fn is_secret(key: &str) -> bool {
    qbt_api::models::app::is_secret_key(key)
}

// ----------------------------------------------------------------------------- impl

impl qobject::AppController {
    // ---- profiles
    fn load_profiles(self: Pin<&mut Self>) {
        let list = with_profiles(|s| s.profiles().to_vec());
        push_table("profiles", tables::profiles(&list));
    }
    fn save_profile(
        self: Pin<&mut Self>,
        name: &QString,
        url: &QString,
        auth: &QString,
        username: &QString,
        secret: &QString,
        remember: bool,
        ca_path: &QString,
    ) {
        let (name, secret) = (name.to_string(), secret.to_string());
        let mut p = Profile::new(&name, &url.to_string());
        p.auth = auth_kind(&auth.to_string());
        p.username = username.to_string();
        let ca = ca_path.to_string();
        p.ca_path = (!ca.is_empty()).then(|| PathBuf::from(strip_file_url(&ca)));
        if let Some(old) = with_profiles(|s| s.get(&name).cloned()) {
            p.extra_headers = old.extra_headers;
            p.timeout_secs = old.timeout_secs;
            p.ui = old.ui;
        }
        let res = with_profiles(|s| {
            s.upsert(p);
            s.set_last_used(&name);
            s.save()
        });
        if remember
            && !secret.is_empty()
            && let Err(e) = with_secrets(|st| st.set(&name, &secret))
        {
            self.toast_later(format!(
                "Could not store the secret in the system keyring: {e}"
            ));
        }
        if let Err(e) = res {
            self.toast_later(format!("Could not save configuration: {e}"));
        }
        self.load_profiles();
    }
    fn delete_profile(self: Pin<&mut Self>, name: &QString) {
        let name = name.to_string();
        with_profiles(|s| {
            with_secrets(|st| s.remove(&name, st));
            let _ = s.save();
        });
        self.load_profiles();
    }
    fn profile_field(&self, name: &QString, field: &QString) -> QString {
        let name = name.to_string();
        let v = with_profiles(|s| {
            s.get(&name).map(|p| match field.to_string().as_str() {
                "url" => p.url.clone(),
                "auth" => auth_name(p.auth).to_owned(),
                "username" => p.username.clone(),
                "ca_path" => p
                    .ca_path
                    .as_ref()
                    .map(|x| x.display().to_string())
                    .unwrap_or_default(),
                "has_secret" => with_secrets(|st| st.get(&name).is_some()).to_string(),
                _ => String::new(),
            })
        });
        q(&v.unwrap_or_default())
    }
    fn last_profile(&self) -> QString {
        q(&with_profiles(|s| s.last_used().unwrap_or("").to_owned()))
    }

    fn connect_profile(mut self: Pin<&mut Self>, name: &QString, secret_override: &QString) {
        let name = name.to_string();
        let secret = {
            let o = secret_override.to_string();
            if o.is_empty() {
                with_secrets(|st| st.get(&name)).unwrap_or_default()
            } else {
                o
            }
        };
        let Some((profile, mult)) = with_profiles(|s| {
            s.get(&name)
                .cloned()
                .map(|p| (p, s.settings().polling_multiplier))
        }) else {
            self.as_mut().set_status_message(q("Unknown profile"));
            self.as_mut().set_connection_state(q("failed"));
            return;
        };
        with_profiles(|s| {
            s.set_last_used(&name);
            let _ = s.save();
        });
        let cfg = match profile.client_config(&secret) {
            Ok(c) => c,
            Err(e) => {
                self.as_mut().set_status_message(q(&e));
                self.as_mut().set_connection_state(q("failed"));
                return;
            }
        };
        self.as_mut().teardown();
        match Session::spawn(state::runtime().handle().clone(), cfg, mult) {
            Ok((handle, mut rx)) => {
                state::set_session(Some(handle));
                let qt = self.qt_thread();
                state::runtime().spawn(async move {
                    while let Some(ev) = rx.recv().await {
                        if qt.queue(move |c| c.handle_event(ev)).is_err() {
                            break;
                        }
                    }
                });
            }
            Err(e) => {
                self.as_mut().set_status_message(q(&errors::friendly(&e)));
                self.as_mut().set_connection_state(q("failed"));
            }
        }
    }

    /// Stop the current session and clear everything derived from it.
    fn teardown(mut self: Pin<&mut Self>) {
        WATCHES.with(|w| w.borrow_mut().clear());
        if let Some(old) = state::set_session(None) {
            state::runtime().spawn(async move { old.close().await });
        }
        APP.with(|a| *a.borrow_mut() = Default::default());
        SPEED.with(|h| h.borrow_mut().clear());
        state::clear_cache();
        PREFS.with(|p| *p.borrow_mut() = Default::default());
        notify_main(MainChanges {
            full_update: true,
            ..Default::default()
        });
        push_table("sidebar", Payload::default());
        for s in [
            "trackers", "webseeds", "files", "general", "peers", "log", "peerlog",
        ] {
            push_table(s, Payload::default());
        }
        self.as_mut().set_connection_state(q("disconnected"));
        self.as_mut().set_torrent_count(0);
        self.as_mut().set_title_speed(q(""));
    }

    fn disconnect_server(self: Pin<&mut Self>) {
        self.teardown();
    }

    fn test_connection(
        self: Pin<&mut Self>,
        url: &QString,
        auth: &QString,
        username: &QString,
        secret: &QString,
        ca_path: &QString,
    ) {
        let mut p = Profile::new("test", &url.to_string());
        p.auth = auth_kind(&auth.to_string());
        p.username = username.to_string();
        let ca = ca_path.to_string();
        p.ca_path = (!ca.is_empty()).then(|| PathBuf::from(strip_file_url(&ca)));
        let secret = secret.to_string();
        let qt = self.qt_thread();
        state::runtime().spawn(async move {
            let result: Result<String, String> = async {
                let cfg = p.client_config(&secret)?;
                let c = qbt_api::Client::new(cfg).map_err(|e| errors::friendly(&e))?;
                c.login().await.map_err(|e| errors::friendly(&e))?;
                let v = c.app_version().await.map_err(|e| errors::friendly(&e))?;
                let w = c
                    .app_webapi_version()
                    .await
                    .map_err(|e| errors::friendly(&e))?;
                let _ = c.logout().await;
                Ok(format!("Connected: qBittorrent {v} (WebAPI {w})"))
            }
            .await;
            let _ = qt.queue(move |c| {
                let (ok, m) = match result {
                    Ok(m) => (true, m),
                    Err(m) => (false, m),
                };
                c.test_result(ok, &q(&m));
            });
        });
    }

    fn supports(&self, scope: &QString, action: &QString) -> bool {
        let (scope, action) = (scope.to_string(), action.to_string());
        let by_version = APP.with(|a| {
            a.borrow()
                .info
                .as_ref()
                .is_none_or(|i| i.capabilities.supports(&scope, &action))
        });
        by_version && !state::session().is_some_and(|s| s.client().is_unsupported(&scope, &action))
    }
    fn requirement(&self, scope: &QString, action: &QString) -> QString {
        q(
            &qbt_api::Capabilities::requirement(&scope.to_string(), &action.to_string())
                .unwrap_or_default(),
        )
    }

    fn set_active(&self, active: bool) {
        if let Some(s) = state::session() {
            s.set_active(active);
        }
    }

    // ---- settings
    fn setting(&self, key: &QString) -> QString {
        let key = snake_case(&key.to_string());
        let v = with_profiles(|s| {
            serde_json::to_value(s.settings())
                .ok()
                .and_then(|v| v.get(&key).cloned())
        });
        q(&match v {
            Some(Value::String(s)) => s,
            Some(Value::Null) | None => String::new(),
            Some(o) => o.to_string(),
        })
    }
    fn set_setting(self: Pin<&mut Self>, key: &QString, value: &QString) {
        let (key, value) = (snake_case(&key.to_string()), value.to_string());
        let mult = with_profiles(|s| {
            if let Ok(Value::Object(mut m)) = serde_json::to_value(s.settings())
                && let Some(old) = m.get(&key)
            {
                let new = match old {
                    Value::Bool(_) => Value::Bool(value == "true"),
                    Value::Number(_) => value
                        .parse::<f64>()
                        .ok()
                        .and_then(serde_json::Number::from_f64)
                        .map(Value::Number)
                        .unwrap_or(old.clone()),
                    _ => Value::String(value.clone()),
                };
                m.insert(key.clone(), new);
                if let Ok(st) = serde_json::from_value(Value::Object(m)) {
                    *s.settings_mut() = st;
                    let _ = s.save();
                }
            }
            s.settings().polling_multiplier
        });
        if let Some(s) = state::session() {
            s.set_polling_multiplier(mult);
        }
    }

    fn sync_ui_store(&self) {
        let (on, cols, widths, order) = with_profiles(|s| {
            (
                s.settings().sync_ui_state,
                s.settings().transfer_columns.clone(),
                s.settings().column_widths.clone(),
                s.settings().column_order.clone(),
            )
        });
        let supported = APP.with(|a| {
            a.borrow()
                .info
                .as_ref()
                .is_some_and(|i| i.capabilities.client_data)
        });
        if !on || !supported {
            return;
        }
        let mut data = Map::new();
        data.insert(
            UI_STATE_KEY.into(),
            serde_json::json!({ "transferColumns": cols, "columnWidths": widths, "columnOrder": order }),
        );
        self.run(move |c| async move { c.clientdata_store(&data).await });
    }

    // ---- actions
    fn run<F, Fut>(&self, f: F) -> i32
    where
        F: FnOnce(std::sync::Arc<qbt_api::Client>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Error>> + Send + 'static,
    {
        match state::session() {
            Some(s) => s.run(f) as i32,
            None => -1,
        }
    }

    fn torrent_action(&self, action: &QString, hashes: &QString, arg: &QString) -> i32 {
        let (a, h, g) = (action.to_string(), hashes.to_string(), arg.to_string());
        self.run(move |c| async move { actions::torrent_action(&c, &a, &h, &g).await })
    }

    fn add_torrents(&self, urls: &QString, files: &QString, options_json: &QString) -> i32 {
        let (urls, files, opts) = (
            urls.to_string(),
            files.to_string(),
            options_json.to_string(),
        );
        let Some(session) = state::session() else {
            return -1;
        };
        let id = session.run(move |c| async move {
            let mut o = AddOptions {
                urls: urls
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(str::to_owned)
                    .collect(),
                ..Default::default()
            };
            for f in files.lines().map(str::trim).filter(|l| !l.is_empty()) {
                let path = PathBuf::from(strip_file_url(f));
                let bytes = tokio::fs::read(&path)
                    .await
                    .map_err(|e| Error::BadRequest(format!("{}: {e}", path.display())))?;
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file.torrent".into());
                o.files.push((name, bytes));
            }
            if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&opts) {
                for (k, v) in m {
                    let s = match v {
                        Value::String(s) => s,
                        Value::Null => continue,
                        other => other.to_string(),
                    };
                    o = o.put(&k, s);
                }
            }
            let (status, r) = c.torrents_add(&o).await?;
            if status == 202
                || r.failure_count.unwrap_or(0) > 0 && r.success_count.unwrap_or(0) == 0
            {
                return Err(Error::Conflict(format!(
                    "added {}, pending {}, failed {}",
                    r.success_count.unwrap_or(0),
                    r.pending_count.unwrap_or(0),
                    r.failure_count.unwrap_or(0)
                )));
            }
            Ok(())
        });
        id as i32
    }

    /// Run any inventoried endpoint. The raw body comes back through `call_result`; known
    /// responses also feed tables.
    fn call(&self, tag: &QString, scope: &QString, action: &QString, params_json: &QString) -> i32 {
        let (tag, scope, action, params) = (
            tag.to_string(),
            scope.to_string(),
            action.to_string(),
            params_json.to_string(),
        );
        let Some(session) = state::session() else {
            return -1;
        };
        let client = session.client();
        let qt = self.qt_thread();
        let id = state::next_call_id();
        state::runtime().spawn(async move {
            let r = actions::call_json(&client, &scope, &action, &params).await;
            let (ok, body) = match &r {
                Ok(resp) => (true, resp.text()),
                Err(e) => (false, errors::friendly(e)),
            };
            let table = if ok {
                feed_tables(&scope, &action, &params, &body)
            } else {
                None
            };
            let _ = qt.queue(move |mut c| {
                if let Some((src, p)) = table {
                    push_table(src, p);
                }
                if ok
                    && scope == "rss"
                    && action == "items"
                    && let Ok(v) = serde_json::from_str::<Value>(&body)
                {
                    let sel = RSS_LAST.with(|r| r.borrow().1.clone());
                    push_table("rss_tree", tables::rss_tree(&v));
                    push_table("rss_articles", tables::rss_articles(&v, &sel));
                    RSS_LAST.with(|r| r.borrow_mut().0 = v);
                }
                c.as_mut().call_result(&q(&tag), ok, &q(&body));
            });
        });
        id
    }

    fn save_to_file(
        &self,
        scope: &QString,
        action: &QString,
        params_json: &QString,
        path: &QString,
    ) -> i32 {
        let (scope, action, params, path) = (
            scope.to_string(),
            action.to_string(),
            params_json.to_string(),
            strip_file_url(&path.to_string()),
        );
        self.run(move |c| async move {
            let mut p = qbt_api::Params::new();
            if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&params) {
                for (k, v) in m {
                    p = p.str(
                        &k,
                        match v {
                            Value::String(s) => s,
                            o => o.to_string(),
                        },
                    );
                }
            }
            c.download_to(&scope, &action, p, std::path::Path::new(&path))
                .await
                .map(|_| ())
        })
    }

    fn toggle_alt_speed(&self) -> i32 {
        self.run(|c| async move { c.transfer_toggle_speed_limits_mode().await })
    }
    fn set_global_limits(&self, dl: i32, ul: i32, alt_dl: i32, alt_ul: i32) -> i32 {
        self.run(move |c| async move {
            let limits = qbt_api::models::transfer::SpeedLimits {
                dl_limit: Some(dl.into()),
                up_limit: Some(ul.into()),
                alt_dl_limit: Some(alt_dl.into()),
                alt_up_limit: Some(alt_ul.into()),
                ..Default::default()
            };
            match c.transfer_set_speed_limits(&limits).await {
                // Servers older than WebAPI 2.16.0 only have the per-direction endpoints
                // (which act on whichever limit set is currently active).
                Err(Error::NotFound(_) | Error::MethodNotAllowed(_)) => {
                    c.transfer_set_download_limit(dl.into()).await?;
                    c.transfer_set_upload_limit(ul.into()).await
                }
                other => other,
            }
        })
    }
    fn pause_session(&self, pause: bool) -> i32 {
        self.run(move |c| async move {
            if pause {
                c.transfer_pause_session().await
            } else {
                c.transfer_resume_session().await
            }
        })
    }
    fn shutdown_server(&self) -> i32 {
        self.run(|c| async move { c.app_shutdown().await })
    }
    fn rss_import_rules(&self, path: &QString) -> i32 {
        let path = PathBuf::from(strip_file_url(&path.to_string()));
        self.run(move |c| async move {
            let bytes = tokio::fs::read(&path)
                .await
                .map_err(|e| Error::BadRequest(format!("{}: {e}", path.display())))?;
            c.rss_import_rules(
                &path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                &bytes,
            )
            .await
        })
    }
    fn server_stat(&self, key: &QString) -> QString {
        let key = key.to_string();
        let v = APP.with(|a| a.borrow().store.server_state.get(&key).cloned());
        q(&match v {
            Some(Value::String(s)) => s,
            Some(Value::Null) | None => String::new(),
            Some(o) => o.to_string(),
        })
    }
    fn clear_table(&self, source: &QString) {
        push_table(&source.to_string(), Payload::default());
    }
    fn rss_select(&self, path: &QString) {
        let p = path.to_string();
        RSS_LAST.with(|r| {
            let mut r = r.borrow_mut();
            r.1 = p.clone();
            push_table("rss_articles", tables::rss_articles(&r.0, &p));
        });
    }

    // ---- watches
    fn watch(&self, kind: &QString, arg: &QString) {
        let (kind, arg) = (kind.to_string(), arg.to_string());
        let Some(session) = state::session() else {
            return;
        };
        let w = match kind.as_str() {
            "general" => Watch::General(arg),
            "trackers" => Watch::Trackers(arg),
            "webseeds" => Watch::WebSeeds(arg),
            "content" => Watch::Content(arg),
            "peers" => {
                PEERS.with(|p| *p.borrow_mut() = (arg.clone(), PeersStore::default()));
                push_table("peers", Payload::default());
                Watch::Peers(arg)
            }
            "log" => Watch::MainLog,
            "peerlog" => Watch::PeerLog,
            "searchstatus" => Watch::SearchStatus,
            "searchresults" => {
                push_table("search_results", Payload::default());
                Watch::SearchResults(arg.parse().unwrap_or(0))
            }
            "rss" => Watch::Rss,
            "creator" => Watch::Creator,
            _ => return,
        };
        if matches!(kind.as_str(), "log" | "peerlog") {
            // Append-only tables restart cleanly each time the tab is shown.
        }
        let guard = session.watch(w);
        WATCHES.with(|m| m.borrow_mut().insert(kind, guard));
    }
    fn unwatch(&self, kind: &QString) {
        WATCHES.with(|m| m.borrow_mut().remove(&kind.to_string()));
    }

    // ---- preferences
    fn prefs_load(&self) {
        let Some(session) = state::session() else {
            return;
        };
        let client = session.client();
        let qt = self.qt_thread();
        state::runtime().spawn(async move {
            let r = client.app_preferences().await;
            let _ = qt.queue(move |mut c| match r {
                Ok(p) => {
                    PREFS.with(|x| *x.borrow_mut() = (p.0, Map::new()));
                    c.as_mut().prefs_loaded();
                }
                Err(e) => c.as_mut().toast(&q(&errors::friendly(&e))),
            });
        });
    }
    fn pref(&self, key: &QString) -> QString {
        let key = key.to_string();
        PREFS.with(|p| {
            let p = p.borrow();
            let v = p.1.get(&key).or_else(|| p.0.get(&key));
            q(&match v {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Null) | None => String::new(),
                Some(o) => o.to_string(),
            })
        })
    }
    fn pref_is_secret(&self, key: &QString) -> bool {
        is_secret(&key.to_string()) || key.to_string() == "web_ui_password"
    }
    fn pref_type(&self, key: &QString) -> QString {
        let key = key.to_string();
        PREFS.with(|p| {
            q(match p.borrow().0.get(&key) {
                None => "missing",
                Some(Value::Bool(_)) => "bool",
                Some(Value::Number(n)) if n.is_f64() => "float",
                Some(Value::Number(_)) => "int",
                Some(Value::String(_)) => "string",
                Some(_) => "json",
            })
        })
    }

    fn preview_files(&self, files: &QString) {
        let paths: Vec<PathBuf> = files
            .to_string()
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|l| PathBuf::from(strip_file_url(l)))
            .collect();
        let Some(session) = state::session() else {
            return;
        };
        let client = session.client();
        let qt = self.qt_thread();
        state::runtime().spawn(async move {
            let mut files = Vec::new();
            for p in &paths {
                if let Ok(b) = tokio::fs::read(p).await {
                    files.push((
                        p.file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                        b,
                    ));
                }
            }
            let r = client.torrents_parse_metadata(&files).await;
            let payload = r.ok().map(|v| tables::preview_from_meta(&v));
            let _ = qt.queue(move |mut c| match payload {
                Some(p) => push_table("preview", p),
                None => c.as_mut().toast(&q("Could not read the torrent file(s)")),
            });
        });
    }

    fn preview_magnet(&self, source: &QString) {
        let source = source.to_string();
        let Some(session) = state::session() else {
            return;
        };
        let client = session.client();
        state::runtime().spawn(async move {
            // Poll until metadata arrives (the server answers 202 while pending); give up after ~2 min.
            for _ in 0..120 {
                match client.torrents_fetch_metadata(&source, None).await {
                    Ok(Some(v)) => {
                        push_table("preview", tables::preview_from_meta(&v));
                        return;
                    }
                    Ok(None) => tokio::time::sleep(std::time::Duration::from_secs(1)).await,
                    Err(_) => return,
                }
            }
        });
    }
    fn preview_cancel(&self, source: &QString) {
        let source = source.to_string();
        if let Some(session) = state::session() {
            let client = session.client();
            state::runtime().spawn(async move {
                let _ = client.torrents_cancel_fetch_metadata(&source).await;
            });
        }
        push_table("preview", Payload::default());
    }

    fn pref_set(&self, key: &QString, value: &QString) {
        let (key, value) = (key.to_string(), value.to_string());
        PREFS.with(|p| {
            let mut p = p.borrow_mut();
            let new = match p.0.get(&key) {
                Some(Value::Bool(_)) => Value::Bool(value == "true"),
                Some(Value::Number(n)) if n.is_f64() => value
                    .parse::<f64>()
                    .ok()
                    .and_then(serde_json::Number::from_f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null),
                Some(Value::Number(_)) => {
                    value.parse::<i64>().map(Value::from).unwrap_or(Value::Null)
                }
                Some(Value::Array(_) | Value::Object(_)) => {
                    serde_json::from_str(&value).unwrap_or(Value::Null)
                }
                _ if key == "web_ui_password" => Value::String(value),
                _ => Value::String(value),
            };
            if new.is_null() {
                return;
            }
            if p.0.get(&key) == Some(&new) {
                p.1.remove(&key);
            } else {
                p.1.insert(key, new);
            }
        });
    }
    fn pref_dirty_count(&self) -> i32 {
        PREFS.with(|p| p.borrow().1.len() as i32)
    }
    fn prefs_discard(&self) {
        PREFS.with(|p| p.borrow_mut().1.clear());
    }
    fn prefs_apply(&self) -> i32 {
        let diff = PREFS.with(|p| p.borrow().1.clone());
        if diff.is_empty() {
            return 0;
        }
        let id = self.run(move |c| async move { c.app_set_preferences(&diff).await });
        // Merge locally so a re-read shows the applied values; the diff clears.
        PREFS.with(|p| {
            let mut p = p.borrow_mut();
            let d = std::mem::take(&mut p.1);
            for (k, v) in d {
                p.0.insert(k, v);
            }
        });
        id
    }

    // ---- helpers
    fn format_size(&self, bytes: i64) -> QString {
        q(&fmt::size(bytes))
    }
    fn format_speed(&self, bps: i64) -> QString {
        q(&fmt::speed(bps))
    }
    fn startup_args(&self) -> QString {
        q(&startup_sources(std::env::args().skip(1)).join("\n"))
    }
    fn register_magnet_handler(&self) -> QString {
        if !cfg!(target_os = "linux") {
            return q("On this system the installer registers the magnet: handler.");
        }
        let ok = std::process::Command::new("xdg-mime")
            .args([
                "default",
                "org.qbremote.qbremote.desktop",
                "x-scheme-handler/magnet",
            ])
            .status()
            .is_ok_and(|s| s.success());
        q(if ok {
            "qbremote is now the handler for magnet links."
        } else {
            "Could not run xdg-mime. Install the desktop file (packaging/org.qbremote.qbremote.desktop) first."
        })
    }
    fn attach_model_testers(&self) -> i32 {
        qobject::qbt_attach_model_testers()
    }
    fn grab_window(&self, path: &QString) -> bool {
        qobject::qbt_grab_window(path)
    }
    fn env(&self, name: &QString) -> QString {
        q(&std::env::var(name.to_string()).unwrap_or_default())
    }
    fn local_path(&self, url: &QString) -> QString {
        q(&strip_file_url(&url.to_string()))
    }

    fn toast_later(&self, msg: String) {
        let qt = self.qt_thread();
        let _ = qt.queue(move |mut c| c.as_mut().toast(&q(&msg)));
    }

    // ---- event handling (Qt thread)
    fn handle_event(mut self: Pin<&mut Self>, ev: Event) {
        match ev {
            Event::State(s) => {
                let (name, msg) = match &s {
                    ConnectionState::Disconnected => ("disconnected", String::new()),
                    ConnectionState::Connecting => ("connecting", String::new()),
                    ConnectionState::Connected => ("connected", String::new()),
                    ConnectionState::Retrying { attempt, error } => {
                        ("retrying", format!("Retrying (attempt {attempt}): {error}"))
                    }
                    ConnectionState::Failed(m) => ("failed", m.clone()),
                };
                // Message first: QML reacts to `connectionStateChanged` and reads the message.
                self.as_mut().set_status_message(q(&msg));
                self.as_mut().set_connection_state(q(name));
                if name == "connected" {
                    self.as_mut().connected();
                }
            }
            Event::Connected(info) => {
                self.as_mut().sync_ui_load(info.capabilities.client_data);
                APP.with(|a| {
                    let mut a = a.borrow_mut();
                    *a = Default::default();
                    a.info = Some((*info).clone());
                });
                state::clear_cache();
                notify_main(MainChanges {
                    full_update: true,
                    ..Default::default()
                });
                self.as_mut().set_server_version(q(&info.version));
                self.as_mut().set_webapi_version(q(&info.webapi_version));
                let b = &info.build;
                self.as_mut().set_build_info(q(&format!(
                    "Qt {}, libtorrent {}, Boost {}, OpenSSL {}, zlib {}, {}-bit {}",
                    b.qt.clone().unwrap_or_default(),
                    b.libtorrent.clone().unwrap_or_default(),
                    b.boost.clone().unwrap_or_default(),
                    b.openssl.clone().unwrap_or_default(),
                    b.zlib.clone().unwrap_or_default(),
                    b.bitness.unwrap_or(0),
                    b.platform.clone().unwrap_or_default()
                )));
                let c = &info.capabilities;
                self.as_mut().set_cap_download_file(c.download_file);
                self.as_mut().set_cap_api_key(c.api_key_auth);
                self.as_mut().set_cap_basic_auth(c.basic_auth);
                self.as_mut().set_cap_process_info(c.process_info);
            }
            Event::MainData(delta) => self.as_mut().on_main_data(&delta),
            Event::Data(d) => self.as_mut().on_data(d),
            Event::CommandFinished(r) => {
                self.as_mut()
                    .command_finished(r.id as i32, r.ok, &q(&r.message))
            }
        }
    }

    /// Opt-in: replace the local layout with the copy stored on the server.
    fn sync_ui_load(self: Pin<&mut Self>, supported: bool) {
        if !supported || !with_profiles(|s| s.settings().sync_ui_state) {
            return;
        }
        let Some(session) = state::session() else {
            return;
        };
        let client = session.client();
        let qt = self.qt_thread();
        state::runtime().spawn(async move {
            let Ok(v) = client.clientdata_load(&[UI_STATE_KEY.to_owned()]).await else {
                return;
            };
            let Some(s) = v.get(UI_STATE_KEY) else { return };
            let cols = s
                .get("transferColumns")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            let widths = s
                .get("columnWidths")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            let order = s
                .get("columnOrder")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            let _ = qt.queue(move |mut c| {
                with_profiles(|st| {
                    st.settings_mut().transfer_columns = cols;
                    st.settings_mut().column_widths = widths;
                    st.settings_mut().column_order = order;
                    let _ = st.save();
                });
                c.as_mut().ui_state_loaded();
            });
        });
    }

    fn on_main_data(self: Pin<&mut Self>, delta: &qbt_api::models::sync::MainDataDelta) {
        if std::env::var_os("QBREMOTE_PERF").is_none() {
            return self.on_main_data_inner(delta);
        }
        // Perf probe: time the whole Qt-thread update (store merge, counts, sidebar, model ops).
        let t = std::time::Instant::now();
        self.on_main_data_inner(delta);
        let n = APP.with(|a| a.borrow().store.torrents.len());
        eprintln!(
            "perf: main_data {} ms ({} torrents, {} changed)",
            t.elapsed().as_millis(),
            n,
            delta.torrents.len()
        );
    }

    fn on_main_data_inner(mut self: Pin<&mut Self>, delta: &qbt_api::models::sync::MainDataDelta) {
        let (changes, n) = APP.with(|a| {
            let mut a = a.borrow_mut();
            let ch = a.store.apply(delta);
            let t = &ch.torrents;
            let recount = ch.full_update
                || !t.inserted.is_empty()
                || !t.removed.is_empty()
                || ch.tags_changed
                || ch.trackers_changed
                || !ch.categories.is_empty()
                || t.changed.iter().any(|(_, f)| {
                    f.iter().any(|x| {
                        matches!(
                            x.as_str(),
                            "state"
                                | "category"
                                | "tags"
                                | "tracker"
                                | "trackers_count"
                                | "dlspeed"
                                | "upspeed"
                                | "has_tracker_error"
                                | "has_tracker_warning"
                                | "has_other_announce_error"
                        )
                    })
                });
            let n = a.store.torrents.len();
            if recount {
                a.counts = Counts::compute(&a.store);
                let payload = tables::sidebar(&a.counts, &a.store);
                let (cats, tags) = tables::name_lists(&a.store, &a.counts);
                // Release the borrow before the model pushes: they may read `APP` again.
                drop(a);
                push_table("sidebar", payload);
                push_table("category_names", cats);
                push_table("tag_names", tags);
            }
            (ch, n)
        });
        self.as_mut().set_torrent_count(n as i32);
        notify_main(changes.clone());

        // Completion notifications: `completion_on` turning positive on an existing torrent.
        if !changes.full_update {
            for (hash, fields) in &changes.torrents.changed {
                if fields.iter().any(|f| f == "completion_on") {
                    let name = APP.with(|a| {
                        let a = a.borrow();
                        let t = a.store.torrents.get(hash)?;
                        (t.get("completion_on")?.as_i64()? > 0).then(|| {
                            t.get("name")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_owned()
                        })
                    });
                    if let Some(name) = name {
                        self.as_mut().torrent_completed(&q(&name));
                    }
                }
            }
        }

        if !changes.server_state.is_empty() || changes.full_update {
            let (dls, uls, dld, uld, dll, ull, dht, free, ip, cs, alt, paused, queueing) = APP
                .with(|a| {
                    let st = &a.borrow().store.server_state;
                    let i = |k: &str| st.get(k).and_then(Value::as_i64).unwrap_or(0);
                    let s = |k: &str| st.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
                    (
                        i("dl_info_speed"),
                        i("up_info_speed"),
                        i("dl_info_data"),
                        i("up_info_data"),
                        i("dl_rate_limit"),
                        i("up_rate_limit"),
                        i("dht_nodes"),
                        i("free_space_on_disk"),
                        s("last_external_address_v4"),
                        s("connection_status"),
                        st.get("use_alt_speed_limits")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        st.get("session_state")
                            .and_then(Value::as_str)
                            .map(|x| x == "paused")
                            .unwrap_or(false),
                        st.get("queueing").and_then(Value::as_bool).unwrap_or(false),
                    )
                });
            self.as_mut().set_dl_speed(q(&fmt::speed(dls)));
            self.as_mut().set_ul_speed(q(&fmt::speed(uls)));
            self.as_mut().set_dl_data(q(&fmt::size(dld)));
            self.as_mut().set_ul_data(q(&fmt::size(uld)));
            self.as_mut().set_dl_limit(q(&fmt::limit(dll)));
            self.as_mut().set_ul_limit(q(&fmt::limit(ull)));
            self.as_mut().set_dht_nodes(dht as i32);
            self.as_mut().set_free_space(q(&fmt::size(free)));
            self.as_mut().set_external_ip(q(&ip));
            self.as_mut().set_conn_status(q(&cs));
            self.as_mut().set_alt_speed(alt);
            self.as_mut().set_session_paused(paused);
            self.as_mut().set_queueing(queueing);
            self.as_mut().set_title_speed(q(&format!(
                "[D: {}, U: {}]",
                fmt::speed(dls),
                fmt::speed(uls)
            )));
            let hist = SPEED.with(|h| {
                let mut h = h.borrow_mut();
                h.push_back((dls, uls));
                while h.len() > 300 {
                    h.pop_front();
                }
                h.iter()
                    .map(|(d, u)| format!("{d},{u}"))
                    .collect::<Vec<_>>()
                    .join(";")
            });
            self.as_mut().set_speed_history(q(&hist));
        }
    }

    fn on_data(mut self: Pin<&mut Self>, d: Data) {
        match &d {
            Data::PieceStates(v) => {
                self.as_mut().set_piece_states(q(&v
                    .iter()
                    .map(|x| char::from(b'0' + (*x).min(9)))
                    .collect::<String>()));
            }
            Data::PieceAvailability(v) => {
                self.as_mut().set_piece_availability(q(&v
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")));
            }
            Data::Peers { hash, delta } => {
                let payload = PEERS.with(|p| {
                    let mut p = p.borrow_mut();
                    if &p.0 != hash {
                        return None; // stale response for a previously selected torrent
                    }
                    p.1.apply(delta);
                    let mut out = Payload::default();
                    for (k, o) in p.1.peers.iter() {
                        out.keys.push(k.clone());
                        out.rows.push(tables::peers_row(k, o));
                    }
                    let mut idx: Vec<usize> = (0..out.keys.len()).collect();
                    idx.sort_by(|&a, &b| out.keys[a].cmp(&out.keys[b]));
                    out.keys = idx.iter().map(|&i| out.keys[i].clone()).collect();
                    out.rows = idx.iter().map(|&i| out.rows[i].clone()).collect();
                    Some(out)
                });
                if let Some(p) = payload {
                    push_table("peers", p);
                }
            }
            Data::RssItems(v) => {
                let sel = RSS_LAST.with(|r| r.borrow().1.clone());
                push_table("rss_tree", tables::rss_tree(v));
                push_table("rss_articles", tables::rss_articles(v, &sel));
                RSS_LAST.with(|r| r.borrow_mut().0 = v.clone());
            }
            other => {
                if let Some((src, p)) = tables::from_data(other) {
                    push_table(src, p);
                }
            }
        }
    }
}

/// Table feeds for responses to `call`. Runs off the Qt thread.
fn feed_tables(
    scope: &str,
    action: &str,
    params: &str,
    body: &str,
) -> Option<(&'static str, Payload)> {
    let v: Value = serde_json::from_str(body).ok()?;
    Some(match (scope, action) {
        ("app", "getDirectoryContent") => {
            let parent = serde_json::from_str::<Value>(params)
                .ok()?
                .get("dirPath")?
                .as_str()?
                .to_owned();
            ("dir", tables::dir_listing(&parent, &v))
        }
        ("torrents", "categories") => ("categories", tables::categories(&v)),
        ("search", "plugins") => {
            let list: Vec<qbt_api::models::search::SearchPlugin> =
                serde_json::from_value(v).ok()?;
            ("plugins", tables::plugins(&list))
        }
        ("rss", "rules") => ("rss_rules", tables::rss_rules(&v)),
        ("app", "cookies") => ("cookies", tables::cookies(&v)),
        ("torrentcreator", "status") => ("creator", tables::creator_tasks(&v)),
        _ => return None,
    })
}

#[allow(dead_code)]
fn _variant_unused(_: QVariant) {}

/// The user's saved style override (read before the QML engine starts).
pub fn configured_style() -> String {
    with_profiles(|s| s.settings().style.clone())
}

#[cfg(test)]
mod tests {
    use super::snake_case;
    use qbt_core::ClientSettings;

    #[test]
    fn camel_case_setting_names_map_to_real_fields() {
        assert_eq!(snake_case("transferColumns"), "transfer_columns");
        assert_eq!(snake_case("syncUiState"), "sync_ui_state");
        // Every name QML uses must exist in the serialized settings, or it would be ignored silently.
        let v = serde_json::to_value(ClientSettings::default()).unwrap();
        for k in [
            "transferColumns",
            "columnWidths",
            "columnOrder",
            "showToolbar",
            "showStatusBar",
            "showSidebar",
            "speedInTitle",
            "showSearchTab",
            "showRssTab",
            "showLogTab",
            "syncUiState",
            "doubleClickAction",
            "confirmDelete",
            "notifyOnCompletion",
            "pollingMultiplier",
        ] {
            assert!(
                v.get(snake_case(k)).is_some(),
                "QML setting {k} has no matching ClientSettings field"
            );
        }
    }
}

#[cfg(test)]
mod arg_tests {
    use super::startup_sources;

    #[test]
    fn only_addable_sources_are_taken_from_argv() {
        let a = [
            "--style",
            "Fusion",
            "magnet:?xt=urn:btih:abc",
            "/tmp/a.TORRENT",
            "notes.txt",
            "https://x/y.torrent",
        ]
        .map(String::from);
        assert_eq!(
            startup_sources(a.into_iter()),
            [
                "magnet:?xt=urn:btih:abc",
                "/tmp/a.TORRENT",
                "https://x/y.torrent"
            ]
        );
    }
}

#[cfg(test)]
mod decode_tests {
    use super::percent_decode;

    #[test]
    fn percent_decoding_handles_edges() {
        assert_eq!(
            percent_decode("/home/a%20b/%E2%9C%93.torrent"),
            "/home/a b/✓.torrent"
        );
        assert_eq!(percent_decode("100%"), "100%", "a trailing % is literal");
        assert_eq!(
            percent_decode("%zz%4"),
            "%zz%4",
            "invalid escapes stay literal"
        );
        assert_eq!(percent_decode(""), "");
    }
}
