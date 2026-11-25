//! Live contract tests. Ignored by default; run with `cargo xtask contract --server <tag>`
//! (which starts the official container) or point them at any server:
//!
//!     QBT_CONTRACT_URL=http://host:8080/ QBT_CONTRACT_USER=admin QBT_CONTRACT_PASSWORD=… \
//!       cargo test -p qbt-api --test contract -- --ignored --nocapture
//!
//! `QBT_CONTRACT_API_KEY` replaces user/password. `QBT_CONTRACT_STRICT=1` additionally fails on any
//! key the typed models do not know (`extra`), which flags API drift.

use qbt_api::{
    AuthMode, Client, ClientConfig, Error, Params, endpoints::ENDPOINTS,
    models::torrents::InfoQuery,
};
use qbt_sync::{Counts, MainDataStore, StatusFilter};

fn client() -> Option<Client> {
    let url = std::env::var("QBT_CONTRACT_URL").ok()?;
    let auth = match std::env::var("QBT_CONTRACT_API_KEY") {
        Ok(k) => AuthMode::ApiKey(k),
        Err(_) => AuthMode::Cookie {
            username: std::env::var("QBT_CONTRACT_USER").unwrap_or_else(|_| "admin".into()),
            password: std::env::var("QBT_CONTRACT_PASSWORD").unwrap_or_default(),
        },
    };
    let mut cfg = ClientConfig::new(&url, auth).unwrap();
    // The server rejects requests whose Host header is not its own (answering 401): when the
    // container's port is published elsewhere, present the Host it expects.
    if let Ok(h) = std::env::var("QBT_CONTRACT_HOST_HEADER") {
        cfg.extra_headers.push(("Host".into(), h));
    }
    Some(Client::new(cfg).unwrap())
}

/// Preference keys that older servers still send but the pinned 2.17.0 API dropped or renamed
/// (seen on real 5.0/5.1 servers); strict mode tolerates them.
const LEGACY_PREF_KEYS: &[&str] = &[
    "export_dir",
    "export_dir_fin",
    "mail_notification_ssl_enabled",
    "use_subcategories",
];

/// Give the server a few torrents (stopped magnets, so nothing touches the network) in different
/// categories, so filter counts and decoding are exercised on real data.
async fn ensure_seeded(c: &Client) {
    use qbt_api::models::torrents::AddOptions;
    if c.torrents_count().await.unwrap_or(0) >= 3 {
        return;
    }
    for (i, (cat, tag)) in [("linux", "iso"), ("tv", ""), ("", "misc")]
        .iter()
        .enumerate()
    {
        let hash = format!("{:040x}", 0xC0FFEE00u64 + i as u64);
        let mut o = AddOptions {
            urls: vec![format!("magnet:?xt=urn:btih:{hash}&dn=contract-seed-{i}")],
            ..Default::default()
        }
        .stopped(true);
        if !cat.is_empty() {
            let _ = c.torrents_create_category(cat, "", &[]).await;
            o = o.category(cat);
        }
        if !tag.is_empty() {
            o = o.tags(&[tag.to_string()]);
        }
        let _ = c.torrents_add(&o).await;
    }
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
}

fn strict() -> bool {
    std::env::var("QBT_CONTRACT_STRICT").is_ok_and(|v| v == "1")
}

/// Every read endpoint answers (never 404/405) with parameters of the right shape.
#[tokio::test]
#[ignore]
async fn every_read_endpoint_is_served() {
    let Some(c) = client() else { return };
    c.login().await.unwrap();
    let caps = c.capabilities().await.unwrap();
    assert!(caps.supported, "server too old: {}", caps.version);
    ensure_seeded(&c).await;
    let main = c.sync_maindata(0).await.unwrap();
    let hash = main
        .torrents
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| "0".repeat(40));
    let mut failures = vec![];
    for e in ENDPOINTS.iter().filter(|e| !e.post) {
        if !caps.supports(e.scope, e.action) {
            continue; // older server: capability gating covers it
        }
        let p = match (e.scope, e.action) {
            (
                "torrents",
                "properties" | "trackers" | "webseeds" | "files" | "pieceHashes" | "pieceStates"
                | "pieceAvailability" | "SSLParameters" | "export" | "downloadFile",
            ) => Params::new().str("hash", hash.clone()).str("file", "0"),
            ("torrents", "uploadLimit" | "downloadLimit") => Params::new().str("hashes", "all"),
            ("sync", "maindata") => Params::new().int("rid", 0),
            ("sync", "torrentPeers") => Params::new().str("hash", hash.clone()).int("rid", 0),
            ("app", "getDirectoryContent") => Params::new().str("dirPath", "/").str("mode", "dirs"),
            ("app", "getFreeSpaceAtPath") => Params::new().str("path", "/"),
            ("app", "networkInterfaceAddressList") => Params::new().str("iface", ""),
            ("rss", "items") => Params::new().bool("withData", true),
            ("rss", "matchingArticles") => Params::new().str("ruleName", "nonexistent"),
            ("clientdata", "load") => Params::new().str("keys", "[]"),
            ("torrentcreator", "torrentFile") => Params::new().str("taskID", "nonexistent"),
            ("torrents", "saveMetadata") => Params::new().str("source", &hash),
            ("search", "results") => Params::new().int("id", 0).int("limit", 1).int("offset", 0),
            ("log", _) => Params::new().int("last_known_id", -1),
            _ => Params::new(),
        };
        match c.call(e.scope, e.action, p).await {
            // 404 with a message (e.g. unknown torrent or task) proves the route exists.
            Ok(_) | Err(Error::BadRequest(_) | Error::Conflict(_) | Error::UnsupportedMedia(_)) => {
            }
            Err(Error::NotFound(m)) if !m.is_empty() => {}
            Err(e2) => failures.push(format!("{}/{}: {e2}", e.scope, e.action)),
        }
    }
    assert!(
        failures.is_empty(),
        "endpoints failed:\n{}",
        failures.join("\n")
    );
}

/// Wrong method returns 405: the endpoint table's method column matches the server, both ways.
#[tokio::test]
#[ignore]
async fn method_table_matches_the_server() {
    let Some(c) = client() else { return };
    c.login().await.unwrap();
    let caps = c.capabilities().await.unwrap();
    let mut wrong = vec![];
    for e in ENDPOINTS {
        // Only POST-only actions are strict: GET-capable actions also accept POST, and probing a
        // mutating action with POST would change server state.
        if !e.post || !caps.supports(e.scope, e.action) || (e.scope, e.action) == ("auth", "login")
        {
            continue;
        }
        let status = c.probe_status(e.scope, e.action, false).await.unwrap();
        // 404: the action does not exist on this (older) server.
        if status == 404 {
            println!(
                "  {}/{}: not present on WebAPI {}",
                e.scope, e.action, caps.version
            );
            continue;
        }
        // The POST-only list grew over time, so it is only exact for the pinned version.
        if caps.version < qbt_api::WebApiVersion::new(2, 17, 0) {
            continue;
        }
        if status != 405 {
            wrong.push(format!(
                "{}/{}: GET answered {status}, expected 405",
                e.scope, e.action
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "method table disagrees with the server:\n{}",
        wrong.join("\n")
    );
}

/// Models decode real responses; in strict mode no unknown keys remain.
#[tokio::test]
#[ignore]
async fn models_decode_real_responses() {
    let Some(c) = client() else { return };
    c.login().await.unwrap();
    ensure_seeded(&c).await;
    let list = c.torrents_info(&InfoQuery::default()).await.unwrap();
    let mut unknown = vec![];
    for t in &list {
        unknown.extend(t.extra.keys().cloned());
    }
    let prefs = c.app_preferences().await.unwrap().typed();
    unknown.extend(
        prefs
            .extra
            .keys()
            .filter(|k| !LEGACY_PREF_KEYS.contains(&k.as_str()))
            .map(|k| format!("preferences.{k}")),
    );
    if let Some(h) = list.first().and_then(|t| t.hash.clone()) {
        let props = c.torrents_properties(&h).await.unwrap();
        unknown.extend(props.extra.keys().map(|k| format!("properties.{k}")));
        for t in c.torrents_trackers(&h).await.unwrap() {
            unknown.extend(t.extra.keys().map(|k| format!("tracker.{k}")));
        }
    }
    unknown.sort();
    unknown.dedup();
    println!("unknown keys: {unknown:?}");
    assert!(
        !strict() || unknown.is_empty(),
        "the models miss keys the server sends: {unknown:?}"
    );
}

/// Client-side status counts equal the server's `torrents/info?filter=` counts.
#[tokio::test]
#[ignore]
async fn filter_counts_match_the_server() {
    let Some(c) = client() else { return };
    c.login().await.unwrap();
    ensure_seeded(&c).await;
    let mut store = MainDataStore::default();
    store.apply(&c.sync_maindata(0).await.unwrap());
    let counts = Counts::compute(&store);
    let mut mismatches = vec![];
    for f in StatusFilter::ALL {
        let q = InfoQuery {
            filter: Some(f.api_name().to_owned()),
            ..Default::default()
        };
        let server = c.torrents_info(&q).await.unwrap().len();
        let client_side = counts.status[&f];
        println!(
            "{:>20}: client {client_side:>4}  server {server:>4}",
            f.api_name()
        );
        if client_side != server {
            mismatches.push(format!(
                "{}: client {client_side} vs server {server}",
                f.api_name()
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "filter rules differ from the server: {mismatches:?}"
    );
}

/// Setting a preference and reading it back returns the same value (round trip).
#[tokio::test]
#[ignore]
async fn preferences_round_trip() {
    let Some(c) = client() else { return };
    c.login().await.unwrap();
    let before = c.app_preferences().await.unwrap();
    // Toggle harmless booleans and restore them afterwards.
    let keys = [
        "resolve_peer_countries",
        "recheck_completed_torrents",
        "queueing_enabled",
    ];
    for k in keys {
        let Some(serde_json::Value::Bool(old)) = before.get(k).cloned() else {
            continue;
        };
        let mut m = serde_json::Map::new();
        m.insert(k.into(), serde_json::Value::Bool(!old));
        c.app_set_preferences(&m).await.unwrap();
        let now = c.app_preferences().await.unwrap();
        assert_eq!(
            now.get(k),
            Some(&serde_json::Value::Bool(!old)),
            "{k} did not round-trip"
        );
        m.insert(k.into(), serde_json::Value::Bool(old));
        c.app_set_preferences(&m).await.unwrap();
    }
}
