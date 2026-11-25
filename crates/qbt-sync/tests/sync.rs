use std::collections::BTreeMap;

use proptest::prelude::*;
use qbt_api::models::sync::MainDataDelta;
use qbt_sync::{
    Counts, MainDataStore, StatusFilter, ViewIndex, ViewQuery,
    filter::{CategoryFilter, TagFilter, TrackerFilter, tracker_host},
    view::{apply_ops, diff_orders},
};
use serde_json::{Map, Value, json};

type Model = BTreeMap<String, Map<String, Value>>;

#[derive(Debug, Clone)]
enum Op {
    Add(u8, i64, i64),
    Remove(u8),
    SetA(u8, i64),
    SetB(u8, i64),
}

fn key(k: u8) -> String {
    format!("h{}", k % 6)
}

fn apply_model(m: &mut Model, op: &Op) {
    match op {
        Op::Add(k, a, b) => {
            m.entry(key(*k)).or_insert_with(|| {
                json!({"a": a, "b": b, "s": "x"})
                    .as_object()
                    .unwrap()
                    .clone()
            });
        }
        Op::Remove(k) => {
            m.remove(&key(*k));
        }
        Op::SetA(k, v) => {
            if let Some(r) = m.get_mut(&key(*k)) {
                r.insert("a".into(), json!(v));
            }
        }
        Op::SetB(k, v) => {
            if let Some(r) = m.get_mut(&key(*k)) {
                r.insert("b".into(), json!(v));
            }
        }
    }
}

/// What the server would send: new keys in full, changed keys as changed fields only.
fn delta_between(old: &Model, new: &Model, rid: i64) -> MainDataDelta {
    let mut d = MainDataDelta {
        rid,
        ..Default::default()
    };
    for k in old.keys().filter(|k| !new.contains_key(*k)) {
        d.torrents_removed.push(k.clone());
    }
    for (k, nv) in new {
        match old.get(k) {
            None => {
                d.torrents.insert(k.clone(), nv.clone());
            }
            Some(ov) => {
                let part: Map<String, Value> = nv
                    .iter()
                    .filter(|(f, v)| ov.get(*f) != Some(*v))
                    .map(|(f, v)| (f.clone(), v.clone()))
                    .collect();
                if !part.is_empty() {
                    d.torrents.insert(k.clone(), part);
                }
            }
        }
    }
    d
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        (any::<u8>(), 0..5i64, 0..5i64).prop_map(|(k, a, b)| Op::Add(k, a, b)),
        any::<u8>().prop_map(Op::Remove),
        (any::<u8>(), 0..5i64).prop_map(|(k, v)| Op::SetA(k, v)),
        (any::<u8>(), 0..5i64).prop_map(|(k, v)| Op::SetB(k, v)),
    ]
}

proptest! {
    /// Applying a sequence of deltas equals a fresh full snapshot of the final state.
    #[test]
    fn deltas_equal_fresh_snapshot(batches in prop::collection::vec(prop::collection::vec(op_strategy(), 0..6), 1..12)) {
        let mut model = Model::new();
        let mut store = MainDataStore::default();
        // First response is always a full update.
        let first = MainDataDelta { rid: 1, full_update: true, ..Default::default() };
        store.apply(&first);
        for (i, batch) in batches.iter().enumerate() {
            let old = model.clone();
            for op in batch { apply_model(&mut model, op); }
            let d = delta_between(&old, &model, i as i64 + 2);
            store.apply(&d);
            prop_assert_eq!(store.torrents.to_btree(), model.clone());
        }
        let mut fresh = MainDataStore::default();
        fresh.apply(&MainDataDelta { rid: 99, full_update: true, torrents: model.clone(), ..Default::default() });
        prop_assert_eq!(store.torrents.to_btree(), fresh.torrents.to_btree());
    }

    /// A full update over existing state reports exactly the differences.
    #[test]
    fn full_update_changes_are_consistent(a in prop::collection::vec(op_strategy(), 0..10), b in prop::collection::vec(op_strategy(), 0..10)) {
        let (mut m1, mut m2) = (Model::new(), Model::new());
        for op in &a { apply_model(&mut m1, op); }
        for op in &b { apply_model(&mut m2, op); }
        let mut s = MainDataStore::default();
        s.apply(&MainDataDelta { rid: 1, full_update: true, torrents: m1.clone(), ..Default::default() });
        let ch = s.apply(&MainDataDelta { rid: 2, full_update: true, torrents: m2.clone(), ..Default::default() });
        prop_assert_eq!(s.torrents.to_btree(), m2.clone());
        for k in &ch.torrents.inserted { prop_assert!(!m1.contains_key(k) && m2.contains_key(k)); }
        for k in &ch.torrents.removed { prop_assert!(m1.contains_key(k) && !m2.contains_key(k)); }
        for (k, _) in &ch.torrents.changed { prop_assert!(m1.contains_key(k) && m2.contains_key(k) && m1[k] != m2[k]); }
    }

    /// Row ops always turn the old order into the new order.
    #[test]
    fn row_ops_transform_old_into_new(old in prop::collection::vec(0u8..12, 0..12), new in prop::collection::vec(0u8..12, 0..12)) {
        let dedup = |v: Vec<u8>| { let mut seen = std::collections::HashSet::new(); v.into_iter().filter(|x| seen.insert(*x)).map(|x| format!("k{x}")).collect::<Vec<_>>() };
        let (old, new) = (dedup(old), dedup(new));
        let ops = diff_orders(&old, &new);
        let mut list = old.clone();
        apply_ops(&mut list, &ops, &new);
        prop_assert_eq!(list, new);
    }
}

fn delta(v: Value) -> MainDataDelta {
    serde_json::from_value(v).unwrap()
}

#[test]
fn partial_merge_keeps_untouched_fields_and_reports_changes() {
    let mut s = MainDataStore::default();
    s.apply(&delta(json!({"rid":1,"full_update":true,"torrents":{"a":{"name":"A","dlspeed":5,"state":"downloading"}},
        "tags":["t1"],"server_state":{"dl_info_speed":5,"refresh_interval":1500}})));
    let ch = s.apply(&delta(json!({"rid":2,"torrents":{"a":{"dlspeed":9}},"tags":["t2"],"server_state":{"dl_info_speed":9}})));
    assert_eq!(s.rid(), 2);
    assert_eq!(
        ch.torrents.changed,
        vec![("a".to_string(), vec!["dlspeed".to_string()])]
    );
    assert_eq!(s.torrent("a").unwrap().name.as_deref(), Some("A"));
    assert_eq!(s.torrent("a").unwrap().hash.as_deref(), Some("a"));
    assert!(ch.tags_changed && s.tags.len() == 2);
    assert_eq!(s.server_i64("refresh_interval"), Some(1500));
    // Unchanged repeat is a no-op.
    assert!(
        s.apply(&delta(json!({"rid":3,"torrents":{"a":{"dlspeed":9}}})))
            .is_empty()
    );
    let ch = s.apply(&delta(
        json!({"rid":4,"torrents_removed":["a"],"tags_removed":["t1"]}),
    ));
    assert_eq!(ch.torrents.removed, vec!["a"]);
    assert_eq!(s.torrents.len(), 0);
}

fn sample_store() -> MainDataStore {
    let mut s = MainDataStore::default();
    s.apply(&delta(json!({"rid":1,"full_update":true,"torrents":{
        "a":{"name":"Alpha","state":"downloading","dlspeed":10,"upspeed":0,"category":"tv","tags":"x, y","size":30,"trackers_count":1},
        "b":{"name":"beta","state":"stalledUP","dlspeed":0,"upspeed":0,"category":"tv/shows","tags":"","size":10,"trackers_count":1},
        "c":{"name":"Gamma","state":"stoppedUP","dlspeed":0,"upspeed":0,"category":"","tags":"y","size":20,"trackers_count":0},
        "d":{"name":"Delta","state":"error","dlspeed":0,"upspeed":0,"category":"movies","tags":"","size":5,"trackers_count":1,"has_tracker_error":true}},
        "trackers":{"http://t.example:6969/announce":["a","b"],"udp://other.org:80":["d"]}})));
    s
}

#[test]
fn status_filters_and_counts() {
    let s = sample_store();
    let c = Counts::compute(&s);
    assert_eq!(c.total, 4);
    assert_eq!(
        c.status[&StatusFilter::Downloading],
        1,
        "no stoppedDL in the sample; stopped incomplete torrents would count too"
    );
    assert_eq!(c.status[&StatusFilter::Stopped], 1);
    assert_eq!(c.status[&StatusFilter::Running], 3);
    assert_eq!(c.status[&StatusFilter::Errored], 1);
    assert_eq!(c.status[&StatusFilter::StalledUploading], 1);
    assert_eq!(c.status[&StatusFilter::Completed], 2);
    assert_eq!(
        c.status[&StatusFilter::Active] + c.status[&StatusFilter::Inactive],
        4
    );
    assert_eq!(c.uncategorized, 1);
    assert_eq!(c.categories["tv"], 2, "parents include sub-categories");
    assert_eq!(c.categories["tv/shows"], 1);
    assert_eq!(c.untagged, 2);
    assert_eq!(c.tags["y"], 2);
    assert_eq!((c.trackerless, c.tracker_error), (1, 1));
    assert_eq!(c.tracker_hosts["t.example"], 2);
}

#[test]
fn view_filters_sorts_and_diffs() {
    let s = sample_store();
    let mut v = ViewIndex::new(ViewQuery {
        sort_by: "name".into(),
        ..Default::default()
    });
    let ops = v.refresh(&s);
    assert_eq!(
        v.rows(),
        ["a", "b", "d", "c"],
        "case-insensitive name order"
    );
    assert_eq!(ops.len(), 4);
    v.query.reverse = true;
    v.query.sort_by = "size".into();
    v.refresh(&s);
    assert_eq!(v.rows(), ["a", "c", "b", "d"]);
    v.query = ViewQuery {
        category: CategoryFilter::Named("tv".into()),
        tag: TagFilter::All,
        ..Default::default()
    };
    v.refresh(&s);
    assert_eq!(v.rows(), ["a", "b"]);
    v.query = ViewQuery {
        tracker: TrackerFilter::Host("t.example".into()),
        text: "^b".into(),
        regex: true,
        ..Default::default()
    };
    v.refresh(&s);
    assert_eq!(v.rows(), ["b"]);
    assert_eq!(v.row_of("b"), Some(0));
}

#[test]
fn tracker_host_parsing() {
    assert_eq!(tracker_host("http://t.example:6969/announce"), "t.example");
    assert_eq!(tracker_host("udp://[::1]:80"), "::1");
    assert_eq!(tracker_host("https://user@h.org/a?b=c"), "h.org");
}

#[test]
fn downloading_filter_includes_stopped_incomplete_like_the_server() {
    use qbt_sync::filter::StatusFilter;
    let mut s = MainDataStore::default();
    s.apply(&delta(json!({"rid":1,"full_update":true,"torrents":{
        "a":{"state":"stoppedDL","dlspeed":0,"upspeed":0},"b":{"state":"stoppedUP","dlspeed":0,"upspeed":0},
        "c":{"state":"stalledUP","dlspeed":0,"upspeed":7}}})));
    let m = |f: StatusFilter, k: &str| f.matches(s.torrents.get(k).unwrap());
    assert!(m(StatusFilter::Downloading, "a") && m(StatusFilter::Stopped, "a"));
    assert!(!m(StatusFilter::Seeding, "b") && m(StatusFilter::Completed, "b"));
    assert!(
        m(StatusFilter::Active, "c") && !m(StatusFilter::Inactive, "c"),
        "stalled but uploading is active"
    );
}
