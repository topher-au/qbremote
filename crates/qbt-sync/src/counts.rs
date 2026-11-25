use std::collections::{BTreeMap, HashMap};

use serde_json::Value;

use crate::{
    filter::{StatusFilter, TrackerFilter, tracker_host},
    maindata::MainDataStore,
};

/// Sidebar badge counts, recomputed after each merge that changed torrents.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    pub total: usize,
    pub status: HashMap<StatusFilter, usize>,
    pub uncategorized: usize,
    pub categories: BTreeMap<String, usize>,
    pub untagged: usize,
    pub tags: BTreeMap<String, usize>,
    pub trackerless: usize,
    pub tracker_error: usize,
    pub tracker_warning: usize,
    pub tracker_other_error: usize,
    pub tracker_hosts: BTreeMap<String, usize>,
}

impl Counts {
    pub fn compute(s: &MainDataStore) -> Self {
        let mut c = Counts {
            total: s.torrents.len(),
            ..Default::default()
        };
        for f in StatusFilter::ALL {
            c.status.insert(f, 0);
        }
        for (hash, t) in s.torrents.iter() {
            let (state, dl, up) = StatusFilter::parts(t);
            for f in StatusFilter::ALL {
                if f.matches_parts(state, dl, up) {
                    *c.status.entry(f).or_default() += 1;
                }
            }
            let cat = t.get("category").and_then(Value::as_str).unwrap_or("");
            if cat.is_empty() {
                c.uncategorized += 1;
            } else {
                // A torrent counts for its category and each ancestor.
                let mut p = String::new();
                for part in cat.split('/') {
                    if !p.is_empty() {
                        p.push('/');
                    }
                    p.push_str(part);
                    *c.categories.entry(p.clone()).or_default() += 1;
                }
            }
            let tags = t.get("tags").and_then(Value::as_str).unwrap_or("");
            if tags.trim().is_empty() {
                c.untagged += 1;
            }
            for tag in tags.split(',').map(str::trim).filter(|x| !x.is_empty()) {
                *c.tags.entry(tag.to_owned()).or_default() += 1;
            }
            if TrackerFilter::Trackerless.matches(hash, t, &s.trackers) {
                c.trackerless += 1;
            }
            let flag = |k: &str| t.get(k).and_then(Value::as_bool).unwrap_or(false);
            c.tracker_error += flag("has_tracker_error") as usize;
            c.tracker_warning += flag("has_tracker_warning") as usize;
            c.tracker_other_error += flag("has_other_announce_error") as usize;
        }
        // Hosts: count distinct torrents per host from the URL→hashes map.
        let mut per_host: BTreeMap<&str, std::collections::HashSet<&str>> = BTreeMap::new();
        for (url, hashes) in &s.trackers {
            per_host
                .entry(tracker_host(url))
                .or_default()
                .extend(hashes.iter().map(String::as_str));
        }
        c.tracker_hosts = per_host
            .into_iter()
            .map(|(h, set)| (h.to_owned(), set.len()))
            .collect();
        c
    }
}
