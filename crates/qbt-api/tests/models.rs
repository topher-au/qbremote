use qbt_api::models::{
    FilePriority, TorrentState, TrackerStatus, sync::MainDataDelta, torrents::Torrent,
};

#[test]
fn torrent_decodes_leniently_and_keeps_unknown_keys() {
    let t: Torrent = serde_json::from_str(
        r#"{"hash":"h","name":"n","state":"stoppedUP","progress":1.0,"private":true,"future_key":5}"#).unwrap();
    assert_eq!(t.state, Some(TorrentState::StoppedUp));
    assert_eq!(t.private, Some(true));
    assert!(t.extra.contains_key("future_key"));
    assert!(t.size.is_none());
}

#[test]
fn enum_fallbacks_roundtrip() {
    let s: TorrentState = serde_json::from_str("\"brandNew\"").unwrap();
    assert_eq!(s, TorrentState::Unknown("brandNew".into()));
    assert_eq!(serde_json::to_string(&s).unwrap(), "\"brandNew\"");
    assert_eq!(TrackerStatus::from(9), TrackerStatus::Other(9));
    assert_eq!(i64::from(FilePriority::Maximum), 7);
}

#[test]
fn maindata_partial_decodes() {
    let d: MainDataDelta = serde_json::from_str(
        r#"{"rid":3,"torrents":{"a":{"dlspeed":1}},"torrents_removed":["b"]}"#,
    )
    .unwrap();
    assert_eq!(d.rid, 3);
    assert!(!d.full_update);
    assert_eq!(d.torrents["a"]["dlspeed"], 1);
}

#[test]
fn typed_preferences_cover_every_inventory_key_and_redact_secrets() {
    use qbt_api::models::{
        Preferences,
        prefs::{KEYS, PreferencesPatch, TypedPreferences},
    };
    let inv = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/api-inventory.md"
    ))
    .unwrap();
    let sec = &inv[inv.find("## app/preferences keys").unwrap()..];
    let mut expected: Vec<String> = Vec::new();
    for line in sec
        .lines()
        .filter(|l| l.starts_with("**") && l.contains(":**"))
    {
        expected.extend(
            line.split(":** ")
                .nth(1)
                .unwrap()
                .replace('`', "")
                .split_whitespace()
                .map(str::to_owned),
        );
    }
    assert_eq!(expected.len(), 243);
    assert_eq!(
        KEYS,
        expected.iter().map(String::as_str).collect::<Vec<_>>()
    );

    let raw: Preferences = serde_json::from_str(
        r#"{"save_path":"/d","dht":true,"max_ratio":1.5,"listen_port":"oops","web_ui_api_key":"SECRET","proxy_password":"SECRET","brand_new":1}"#).unwrap();
    let t: TypedPreferences = raw.typed();
    assert_eq!(
        (t.save_path.as_deref(), t.dht, t.max_ratio),
        (Some("/d"), Some(true), Some(1.5))
    );
    assert_eq!(
        t.listen_port, None,
        "mistyped value must not fail the decode"
    );
    assert!(t.extra.contains_key("brand_new"));
    let d = format!("{t:?} {raw:?}");
    assert!(!d.contains("SECRET") && d.contains("/d"));

    let patch = PreferencesPatch::new()
        .save_path("/x")
        .dht(false)
        .web_ui_password("pw")
        .set_raw("future", serde_json::json!(1));
    assert!(!format!("{patch:?}").contains("pw"));
    let m = patch.into_map();
    assert_eq!(m.len(), 4);
    assert_eq!(m["dht"], false);
}
