//! Benchmarks for the sync engine at scale (plan §8 risk 4: "handle 50k torrents").
//! Run with `cargo test -p qbt-sync --release --test perf -- --nocapture`. Bounds are generous so
//! they hold in debug builds on CI too; the printed timings are the useful part.

use std::time::Instant;

use qbt_api::models::sync::MainDataDelta;
use qbt_sync::{Counts, MainDataStore, ViewIndex, ViewQuery};
use serde_json::{Map, Value, json};

fn torrent(i: usize) -> Map<String, Value> {
    let state = [
        "downloading",
        "stalledDL",
        "uploading",
        "stoppedUP",
        "queuedDL",
    ][i % 5];
    let dlspeed = if i.is_multiple_of(5) {
        1000 + i as i64
    } else {
        0
    };
    let tags = if i.is_multiple_of(3) { "a,b" } else { "" };
    let category = ["", "tv", "tv/shows", "movies"][i % 4];
    json!({
        "name": format!("Torrent number {i:06} with a moderately long name.mkv"), "size": 1_000_000 * (i as i64 % 997 + 1),
        "progress": (i % 100) as f64 / 100.0, "state": state, "dlspeed": dlspeed,
        "upspeed": 0, "category": category, "tags": tags,
        "added_on": 1_700_000_000 + i as i64, "num_seeds": 3, "num_leechs": 4, "ratio": 0.5, "eta": 100, "tracker": "http://t.example/a",
        "trackers_count": 1, "priority": i as i64 + 1,
    }).as_object().unwrap().clone()
}

fn full(n: usize) -> MainDataDelta {
    let mut d = MainDataDelta {
        rid: 1,
        full_update: true,
        ..Default::default()
    };
    for i in 0..n {
        d.torrents.insert(format!("{i:040x}"), torrent(i));
    }
    d.trackers.insert(
        "http://t.example/a".into(),
        d.torrents.keys().cloned().collect(),
    );
    d
}

/// A one-second tick: every 5th torrent downloads, so its speed and progress change.
fn tick(n: usize, rid: i64) -> MainDataDelta {
    let mut d = MainDataDelta {
        rid,
        ..Default::default()
    };
    for i in (0..n).step_by(5) {
        d.torrents.insert(format!("{i:040x}"), json!({"dlspeed": 5000 + rid * 7 + i as i64 % 13, "progress": ((i % 100) as f64 + rid as f64 * 0.001).min(0.999)}).as_object().unwrap().clone());
    }
    d
}

fn run(n: usize, bound_ms: u128) {
    let mut s = MainDataStore::default();
    let t = Instant::now();
    s.apply(&full(n));
    let full_ms = t.elapsed().as_millis();

    let t = Instant::now();
    let c = Counts::compute(&s);
    let counts_ms = t.elapsed().as_millis();
    assert_eq!(c.total, n);

    let mut view = ViewIndex::new(ViewQuery {
        sort_by: "name".into(),
        ..Default::default()
    });
    let t = Instant::now();
    view.refresh(&s);
    let first_view_ms = t.elapsed().as_millis();

    let (mut apply_max, mut view_max) = (0, 0);
    for rid in 2..7 {
        let d = tick(n, rid);
        let t = Instant::now();
        let ch = s.apply(&d);
        apply_max = apply_max.max(t.elapsed().as_millis());
        assert_eq!(ch.torrents.changed.len(), n.div_ceil(5));
        let t = Instant::now();
        view.query.sort_by = "name".into();
        view.refresh(&s);
        view_max = view_max.max(t.elapsed().as_millis());
    }
    // Re-sorting by a field that changes every tick (download speed) is the worst realistic case.
    view.query.sort_by = "dlspeed".into();
    view.refresh(&s);
    let t = Instant::now();
    s.apply(&tick(n, 8));
    let ops = view.refresh(&s);
    let resort_ms = t.elapsed().as_millis();

    println!(
        "{n:>6} torrents: full {full_ms} ms | counts {counts_ms} ms | first view {first_view_ms} ms | tick apply ≤ {apply_max} ms | \
              tick view (name sort) ≤ {view_max} ms | tick+resort by speed {resort_ms} ms ({} ops)",
        ops.len()
    );
    for (what, ms) in [
        ("full", full_ms),
        ("counts", counts_ms),
        ("first view", first_view_ms),
        ("tick apply", apply_max),
        ("tick view", view_max),
        ("resort", resort_ms),
    ] {
        assert!(
            ms < bound_ms,
            "{what} took {ms} ms at {n} torrents (bound {bound_ms} ms)"
        );
    }
}

#[test]
fn ten_thousand_torrents() {
    run(10_000, 1500);
}

#[test]
fn fifty_thousand_torrents() {
    run(50_000, 8000);
}
