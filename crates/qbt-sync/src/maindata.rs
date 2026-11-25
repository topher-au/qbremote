use std::collections::{BTreeMap, BTreeSet};

use qbt_api::models::sync::{MainDataDelta, PeersDelta};
use serde_json::Value;

use crate::store::{Changes, Fields, KeyedStore};

/// Result of merging one `sync/maindata` response.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MainChanges {
    pub full_update: bool,
    pub torrents: Changes,
    pub categories: Changes,
    pub tags_changed: bool,
    pub trackers_changed: bool,
    /// Names of `server_state` keys whose value changed.
    pub server_state: Vec<String>,
}

impl MainChanges {
    pub fn is_empty(&self) -> bool {
        self.torrents.is_empty()
            && self.categories.is_empty()
            && !self.tags_changed
            && !self.trackers_changed
            && self.server_state.is_empty()
    }
}

/// Client-side mirror of the server's `sync/maindata` state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MainDataStore {
    rid: i64,
    pub torrents: KeyedStore,
    pub categories: KeyedStore,
    pub tags: BTreeSet<String>,
    /// tracker URL → torrent hashes.
    pub trackers: BTreeMap<String, Vec<String>>,
    pub server_state: Fields,
}

impl MainDataStore {
    /// The `rid` to send with the next request (0 until the first response).
    pub fn rid(&self) -> i64 {
        self.rid
    }

    pub fn apply(&mut self, d: &MainDataDelta) -> MainChanges {
        let full = d.full_update;
        let mut ch = MainChanges {
            full_update: full,
            ..Default::default()
        };
        ch.torrents = self.torrents.apply(full, &d.torrents, &d.torrents_removed);
        ch.categories = self
            .categories
            .apply(full, &d.categories, &d.categories_removed);

        let before_tags = self.tags.clone();
        if full {
            self.tags.clear();
        }
        for t in &d.tags_removed {
            self.tags.remove(t);
        }
        self.tags.extend(d.tags.iter().cloned());
        ch.tags_changed = self.tags != before_tags;

        let before_trackers = self.trackers.clone();
        if full {
            self.trackers.clear();
        }
        for u in &d.trackers_removed {
            self.trackers.remove(u);
        }
        for (u, h) in &d.trackers {
            self.trackers.insert(u.clone(), h.clone());
        }
        ch.trackers_changed = self.trackers != before_trackers;

        if full {
            let old = std::mem::replace(&mut self.server_state, d.server_state.clone());
            ch.server_state = diff_keys(&old, &self.server_state);
        } else {
            for (k, v) in &d.server_state {
                if self.server_state.get(k) != Some(v) {
                    self.server_state.insert(k.clone(), v.clone());
                    ch.server_state.push(k.clone());
                }
            }
        }
        self.rid = d.rid;
        ch
    }

    /// Typed view of one torrent (the hash key is injected, since deltas omit it).
    pub fn torrent(&self, hash: &str) -> Option<qbt_api::models::torrents::Torrent> {
        let mut f = self.torrents.get(hash)?.clone();
        f.insert("hash".into(), Value::String(hash.to_owned()));
        serde_json::from_value(Value::Object(f)).ok()
    }

    pub fn server_i64(&self, key: &str) -> Option<i64> {
        self.server_state.get(key).and_then(Value::as_i64)
    }
}

fn diff_keys(old: &Fields, new: &Fields) -> Vec<String> {
    let mut v: Vec<String> = new
        .iter()
        .filter(|(k, x)| old.get(*k) != Some(*x))
        .map(|(k, _)| k.clone())
        .collect();
    v.extend(old.keys().filter(|k| !new.contains_key(*k)).cloned());
    v
}

/// Mirror of `sync/torrentPeers`, keyed by `ip:port`. Reset when the selected torrent changes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PeersStore {
    rid: i64,
    pub peers: KeyedStore,
    pub show_flags: bool,
}

impl PeersStore {
    pub fn rid(&self) -> i64 {
        self.rid
    }
    pub fn apply(&mut self, d: &PeersDelta) -> Changes {
        let ch = self.peers.apply(d.full_update, &d.peers, &d.peers_removed);
        if let Some(s) = d.show_flags {
            self.show_flags = s;
        }
        self.rid = d.rid;
        ch
    }
}
