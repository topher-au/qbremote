//! Coverage gate (plan §2.4): every action has a typed client method AND is reachable from the UI.
//!
//! "Reachable from the UI" means one of:
//!  * `core`      the connection/poller layer calls the typed method (sync, watches, login);
//!  * `invokable` a QML-visible `AppController` method exists, QML calls it, and it uses the typed method;
//!  * `action`    a named torrent action handled by `qbt_core::actions` that QML triggers;
//!  * `call`      QML names the endpoint literally in `AppController.call/saveToFile(...)`;
//!  * `superseded` replaced by `sync/maindata` (explicit allowlist with a reason).

use std::{collections::BTreeSet, fs, path::Path};

use qbt_api::endpoints::ENDPOINTS;

/// Endpoints driven by the session and pollers rather than by a user command: (scope, action, typed method).
const CORE: &[(&str, &str, &str)] = &[
    ("auth", "login", "login"),
    ("auth", "logout", "logout"),
    ("app", "version", "app_version"),
    ("app", "webapiVersion", "app_webapi_version"),
    ("app", "buildInfo", "app_build_info"),
    ("sync", "maindata", "sync_maindata"),
    ("sync", "torrentPeers", "sync_torrent_peers"),
    ("torrents", "properties", "torrents_properties"),
    ("torrents", "trackers", "torrents_trackers"),
    ("torrents", "webseeds", "torrents_webseeds"),
    ("torrents", "files", "torrents_files"),
    ("torrents", "pieceStates", "torrents_piece_states"),
    (
        "torrents",
        "pieceAvailability",
        "torrents_piece_availability",
    ),
    ("log", "main", "log_main"),
    ("log", "peers", "log_peers"),
    ("search", "status", "search_status"),
    ("search", "results", "search_results"),
    ("rss", "items", "rss_items"),
    ("torrentcreator", "status", "creator_status"),
];

/// Endpoints with a dedicated bridge method: (scope, action, QML-visible name, typed method).
const INVOKABLE: &[(&str, &str, &str, &str)] = &[
    ("app", "shutdown", "shutdownServer", "app_shutdown"),
    ("app", "preferences", "prefsLoad", "app_preferences"),
    ("app", "setPreferences", "prefsApply", "app_set_preferences"),
    ("torrents", "add", "addTorrents", "torrents_add"),
    (
        "torrents",
        "parseMetadata",
        "previewFiles",
        "torrents_parse_metadata",
    ),
    (
        "torrents",
        "fetchMetadata",
        "previewMagnet",
        "torrents_fetch_metadata",
    ),
    (
        "torrents",
        "cancelFetchMetadata",
        "previewCancel",
        "torrents_cancel_fetch_metadata",
    ),
    ("rss", "importRules", "rssImportRules", "rss_import_rules"),
    (
        "transfer",
        "toggleSpeedLimitsMode",
        "toggleAltSpeed",
        "transfer_toggle_speed_limits_mode",
    ),
    (
        "transfer",
        "pauseSession",
        "pauseSession",
        "transfer_pause_session",
    ),
    (
        "transfer",
        "resumeSession",
        "pauseSession",
        "transfer_resume_session",
    ),
    (
        "transfer",
        "setSpeedLimits",
        "setGlobalLimits",
        "transfer_set_speed_limits",
    ),
    (
        "transfer",
        "setDownloadLimit",
        "setGlobalLimits",
        "transfer_set_download_limit",
    ),
    (
        "transfer",
        "setUploadLimit",
        "setGlobalLimits",
        "transfer_set_upload_limit",
    ),
    ("clientdata", "load", "syncUiStore", "clientdata_load"),
    ("clientdata", "store", "syncUiStore", "clientdata_store"),
];

/// Named torrent actions (`qbt_core::actions::torrent_action`): (UI action name, action it calls).
const ACTIONS: &[(&str, &str)] = &[
    ("start", "start"),
    ("stop", "stop"),
    ("forceStart", "setForceStart"),
    ("recheck", "recheck"),
    ("reannounce", "reannounce"),
    ("delete", "delete"),
    ("rename", "rename"),
    ("setComment", "setComment"),
    ("setCategory", "setCategory"),
    ("addTags", "addTags"),
    ("removeTags", "removeTags"),
    ("setTags", "setTags"),
    ("setLocation", "setLocation"),
    ("setSavePath", "setSavePath"),
    ("setDownloadPath", "setDownloadPath"),
    ("setAutoManagement", "setAutoManagement"),
    ("setUploadLimit", "setUploadLimit"),
    ("setDownloadLimit", "setDownloadLimit"),
    ("setShareLimits", "setShareLimits"),
    ("setSuperSeeding", "setSuperSeeding"),
    ("toggleSequentialDownload", "toggleSequentialDownload"),
    ("toggleFirstLastPiecePrio", "toggleFirstLastPiecePrio"),
    ("increasePrio", "increasePrio"),
    ("decreasePrio", "decreasePrio"),
    ("topPrio", "topPrio"),
    ("bottomPrio", "bottomPrio"),
    ("addTrackers", "addTrackers"),
    ("editTracker", "editTracker"),
    ("removeTrackers", "removeTrackers"),
    ("addWebSeeds", "addWebSeeds"),
    ("editWebSeed", "editWebSeed"),
    ("removeWebSeeds", "removeWebSeeds"),
    ("addPeers", "addPeers"),
    ("filePrio", "filePrio"),
    ("renameFile", "renameFile"),
    ("renameFolder", "renameFolder"),
    ("setSSLParameters", "setSSLParameters"),
];

/// Replaced by `sync/maindata`, which carries the same data incrementally.
const SUPERSEDED: &[(&str, &str, &str)] = &[
    (
        "torrents",
        "info",
        "the transfer list is built from sync/maindata deltas",
    ),
    (
        "torrents",
        "tags",
        "tags arrive in sync/maindata (`tags`, `tags_removed`)",
    ),
];

pub struct Report {
    pub uncovered: Vec<String>,
    pub covered: usize,
}

fn read_dir_text(dir: &Path, ext: &str) -> String {
    let mut out = String::new();
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == ext) {
            out.push_str(&fs::read_to_string(&p).unwrap_or_default());
            out.push('\n');
        }
    }
    // Collapse whitespace so multi-line calls still match.
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn check() -> Report {
    let root = crate::repo_root();
    let qml = read_dir_text(&root.join("crates/qbt-qt/qml"), "qml");
    let rust_ui = read_dir_text(&root.join("crates/qbt-qt/src"), "rs");
    let rust_core = read_dir_text(&root.join("crates/qbt-core/src"), "rs");
    let api = read_dir_text(&root.join("crates/qbt-api/src"), "rs");
    let actions_src =
        fs::read_to_string(root.join("crates/qbt-core/src/actions.rs")).unwrap_or_default();

    let mut uncovered = Vec::new();
    let mut covered = 0;
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for e in ENDPOINTS {
        let id = format!("{}/{}", e.scope, e.action);
        seen.insert((e.scope.into(), e.action.into()));
        // 1. A typed client method exists.
        let literal = format!("\"{}\"", e.action);
        let wired = api.contains(&format!("\"{}\", {}", e.scope, literal))
            || api.contains(&format!("=> {literal}"));
        if !wired {
            uncovered.push(format!("{id}: no typed client method"));
            continue;
        }
        // 2. Reachable from the UI.
        let ok = if let Some((_, _, m)) = CORE
            .iter()
            .find(|(s, a, _)| *s == e.scope && *a == e.action)
        {
            rust_core.contains(&format!(".{m}(")) || rust_ui.contains(&format!(".{m}("))
        } else if let Some((_, _, q, m)) = INVOKABLE
            .iter()
            .find(|(s, a, _, _)| *s == e.scope && *a == e.action)
        {
            qml.contains(&format!("AppController.{q}(")) && rust_ui.contains(&format!(".{m}("))
        } else if e.scope == "torrents" && ACTIONS.iter().any(|(_, a)| *a == e.action) {
            ACTIONS
                .iter()
                .filter(|(_, a)| *a == e.action)
                .any(|(n, _)| {
                    actions_src.contains(&format!("\"{n}\" =>"))
                        && (qml.contains(&format!("\"{n}\""))
                            || (e.action == "delete" && qml.contains("\"deleteFiles\"")))
                })
        } else if SUPERSEDED
            .iter()
            .any(|(s, a, _)| *s == e.scope && *a == e.action)
        {
            true
        } else {
            // Named literally in a QML call: call("tag", "scope", "action", ...) or saveToFile("scope", "action", ...),
            // or (RSS page only) through its `op("action", ...)` helper, which forwards to `call`.
            qml.contains(&format!("\"{}\", \"{}\"", e.scope, e.action))
                || (e.scope == "rss" && qml.contains(&format!("op(\"{}\"", e.action)))
        };
        if ok {
            covered += 1;
        } else {
            uncovered.push(format!("{id}: no UI path"));
        }
    }
    // Stale table entries are bugs in the gate itself.
    for (s, a, _) in SUPERSEDED.iter().map(|(s, a, r)| (s, a, r)) {
        if !seen.contains(&(s.to_string(), a.to_string())) {
            uncovered.push(format!("gate table mentions unknown endpoint {s}/{a}"));
        }
    }
    Report { uncovered, covered }
}

pub fn run() -> anyhow::Result<()> {
    let r = check();
    println!("{} of {} endpoints covered", r.covered, ENDPOINTS.len());
    for (s, a, why) in SUPERSEDED {
        println!("  superseded: {s}/{a} — {why}");
    }
    if r.uncovered.is_empty() {
        println!("coverage gate passed");
        Ok(())
    } else {
        for u in &r.uncovered {
            eprintln!("  UNCOVERED {u}");
        }
        anyhow::bail!("{} endpoint(s) not covered", r.uncovered.len())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_endpoint_has_a_typed_method_and_a_ui_path() {
        let r = super::check();
        assert!(
            r.uncovered.is_empty(),
            "uncovered endpoints:\n{}",
            r.uncovered.join("\n")
        );
        assert_eq!(r.covered, qbt_api::endpoints::ENDPOINTS.len());
    }
}
