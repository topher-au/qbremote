//! Offline decoding of responses recorded from real qBittorrent servers
//! (`cargo xtask contract --server <tag>` writes `tests/fixtures/<WebAPI version>/`).
//! Every model must decode every fixture and leave no unknown keys behind, except keys that older
//! servers still send and the pinned API dropped.

use std::{fs, path::PathBuf};

use qbt_api::models::{
    Preferences,
    log::LogEntry,
    sync::{MainDataDelta, PeersDelta},
    torrents::{Category, Torrent, TorrentFile, TorrentProperties, Tracker},
    transfer::{SpeedLimits, TransferInfo},
};
use serde::de::DeserializeOwned;

const LEGACY_PREF_KEYS: &[&str] = &[
    "export_dir",
    "export_dir_fin",
    "mail_notification_ssl_enabled",
    "use_subcategories",
];

fn versions() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut v: Vec<PathBuf> = fs::read_dir(root)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    v.sort();
    v
}

fn load<T: DeserializeOwned>(dir: &std::path::Path, name: &str) -> Option<T> {
    let text = fs::read_to_string(dir.join(name)).ok()?;
    Some(serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}/{name}: {e}", dir.display())))
}

#[test]
fn fixtures_exist_for_the_target_server_lines() {
    let names: Vec<String> = versions()
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    for want in ["2.11.2", "2.11.4", "2.16.2", "2.17"] {
        assert!(
            names.iter().any(|n| n == want),
            "no fixtures for WebAPI {want}: run `cargo xtask contract`"
        );
    }
}

#[test]
fn every_fixture_decodes_with_no_unknown_keys() {
    for dir in versions() {
        let tag = dir.file_name().unwrap().to_string_lossy().into_owned();
        let mut unknown: Vec<String> = Vec::new();
        let mut note = |what: &str, keys: Vec<&String>| {
            unknown.extend(keys.into_iter().map(|k| format!("{what}.{k}")))
        };

        if let Some(p) = load::<Preferences>(&dir, "app.preferences.json") {
            let t = p.typed();
            note(
                "preferences",
                t.extra
                    .keys()
                    .filter(|k| !LEGACY_PREF_KEYS.contains(&k.as_str()))
                    .collect(),
            );
        }
        if let Some(list) = load::<Vec<Torrent>>(&dir, "torrents.info.json") {
            assert!(
                !list.is_empty(),
                "{tag}: torrents.info fixture has no torrents"
            );
            for t in &list {
                note("torrent", t.extra.keys().collect());
            }
        }
        if let Some(d) = load::<MainDataDelta>(&dir, "sync.maindata.json") {
            assert!(
                d.full_update,
                "{tag}: maindata fixture must be a full update"
            );
            for (h, fields) in &d.torrents {
                let mut f = fields.clone();
                f.insert("hash".into(), h.clone().into());
                let t: Torrent = serde_json::from_value(serde_json::Value::Object(f)).unwrap();
                note("maindata.torrent", t.extra.keys().collect());
            }
        }
        if let Some(p) = load::<TorrentProperties>(&dir, "torrents.properties.json") {
            note("properties", p.extra.keys().collect());
        }
        for t in load::<Vec<Tracker>>(&dir, "torrents.trackers.json").unwrap_or_default() {
            note("tracker", t.extra.keys().collect());
            for e in t.endpoints.iter().flatten() {
                note("tracker.endpoint", e.extra.keys().collect());
            }
        }
        for f in load::<Vec<TorrentFile>>(&dir, "torrents.files.json").unwrap_or_default() {
            note("file", f.extra.keys().collect());
        }
        if let Some(t) = load::<TransferInfo>(&dir, "transfer.info.json") {
            note("transfer.info", t.extra.keys().collect());
        }
        if let Some(l) = load::<SpeedLimits>(&dir, "transfer.getSpeedLimits.json") {
            note("speedLimits", l.extra.keys().collect());
        }
        if let Some(c) =
            load::<std::collections::BTreeMap<String, Category>>(&dir, "torrents.categories.json")
        {
            for v in c.values() {
                note("category", v.extra.keys().collect());
            }
        }
        for e in load::<Vec<LogEntry>>(&dir, "log.main.json").unwrap_or_default() {
            note("log", e.extra.keys().collect());
        }
        load::<PeersDelta>(&dir, "sync.torrentPeers.json");
        unknown.sort();
        unknown.dedup();
        assert!(
            unknown.is_empty(),
            "WebAPI {tag}: models miss keys the server sends: {unknown:?}"
        );
    }
}

#[test]
fn maindata_fixtures_replay_through_the_sync_engine() {
    // The recorded full update is a valid starting state: counts add up and the typed view works.
    use qbt_sync::{Counts, MainDataStore, StatusFilter};
    for dir in versions() {
        let Some(d) = load::<MainDataDelta>(&dir, "sync.maindata.json") else {
            continue;
        };
        let mut s = MainDataStore::default();
        s.apply(&d);
        let c = Counts::compute(&s);
        assert_eq!(c.total, d.torrents.len());
        assert_eq!(c.status[&StatusFilter::All], c.total);
        assert_eq!(
            c.status[&StatusFilter::Running] + c.status[&StatusFilter::Stopped],
            c.total
        );
        let h = d.torrents.keys().next().unwrap();
        assert!(s.torrent(h).is_some());
    }
}
