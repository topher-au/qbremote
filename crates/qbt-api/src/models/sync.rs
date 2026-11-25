use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Raw `sync/maindata` delta. Torrent/peer/category objects stay as JSON maps so the sync engine
/// can merge them field by field.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct MainDataDelta {
    #[serde(default)]
    pub rid: i64,
    #[serde(default)]
    pub full_update: bool,
    #[serde(default)]
    pub torrents: BTreeMap<String, Map<String, Value>>,
    #[serde(default)]
    pub torrents_removed: Vec<String>,
    #[serde(default)]
    pub categories: BTreeMap<String, Map<String, Value>>,
    #[serde(default)]
    pub categories_removed: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub tags_removed: Vec<String>,
    #[serde(default)]
    pub trackers: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub trackers_removed: Vec<String>,
    #[serde(default)]
    pub server_state: Map<String, Value>,
}

/// Raw `sync/torrentPeers` delta.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct PeersDelta {
    #[serde(default)]
    pub rid: i64,
    #[serde(default)]
    pub full_update: bool,
    #[serde(default)]
    pub peers: BTreeMap<String, Map<String, Value>>,
    #[serde(default)]
    pub peers_removed: Vec<String>,
    #[serde(default)]
    pub show_flags: Option<bool>,
}
