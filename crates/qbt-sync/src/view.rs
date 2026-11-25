use std::{cmp::Ordering, collections::HashMap};

use regex::RegexBuilder;
use serde_json::Value;

use crate::{filter::ViewQuery, maindata::MainDataStore, store::Fields};

/// Minimal model operations to turn the old row order into the new one.
/// Apply in sequence; indexes refer to the list as it is at that step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowOp {
    Remove {
        row: usize,
    },
    Insert {
        row: usize,
        key: String,
    },
    Move {
        from: usize,
        to: usize,
    },
    /// Too many differences: reset the whole model.
    Reset,
}

const MAX_OPS: usize = 500;

/// Sorted, filtered list of torrent hashes for the current [`ViewQuery`].
#[derive(Debug, Default)]
pub struct ViewIndex {
    pub query: ViewQuery,
    order: Vec<String>,
    pos: HashMap<String, usize>,
}

impl ViewIndex {
    pub fn new(query: ViewQuery) -> Self {
        Self {
            query,
            ..Default::default()
        }
    }
    pub fn rows(&self) -> &[String] {
        &self.order
    }
    pub fn len(&self) -> usize {
        self.order.len()
    }
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
    pub fn row_of(&self, key: &str) -> Option<usize> {
        self.pos.get(key).copied()
    }

    /// Whether `changes` can alter which rows are shown or their order under the current query.
    /// When `false`, the cached order is still correct and only cell values need repainting.
    pub fn affected_by(&self, changes: &crate::MainChanges) -> bool {
        use crate::filter::{CategoryFilter, StatusFilter, TagFilter, TrackerFilter};
        let t = &changes.torrents;
        if changes.full_update || !t.inserted.is_empty() || !t.removed.is_empty() {
            return true;
        }
        let q = &self.query;
        let mut relevant: Vec<&str> = vec![if q.sort_by.is_empty() {
            "name"
        } else {
            q.sort_by.as_str()
        }];
        if q.status != StatusFilter::All {
            relevant.extend(["state", "dlspeed", "upspeed"]);
        }
        if q.category != CategoryFilter::All {
            relevant.push("category");
        }
        if q.tag != TagFilter::All {
            relevant.push("tags");
        }
        if q.tracker != TrackerFilter::All {
            relevant.extend([
                "tracker",
                "trackers_count",
                "has_tracker_error",
                "has_tracker_warning",
                "has_other_announce_error",
            ]);
        }
        if !q.text.is_empty() {
            relevant.push("name");
        }
        // Tracker-host filters depend on the URL→hashes map, which moves independently.
        if matches!(q.tracker, TrackerFilter::Host(_)) && changes.trackers_changed {
            return true;
        }
        t.changed
            .iter()
            .any(|(_, fields)| fields.iter().any(|f| relevant.contains(&f.as_str())))
    }

    /// Recompute the ordering from the store and return the operations that transform the old
    /// order into the new one.
    pub fn refresh(&mut self, s: &MainDataStore) -> Vec<RowOp> {
        let new = compute_order(&self.query, s);
        let ops = diff_orders(&self.order, &new);
        self.pos = new
            .iter()
            .enumerate()
            .map(|(i, k)| (k.clone(), i))
            .collect();
        self.order = new;
        ops
    }
}

fn text_matcher(q: &ViewQuery) -> Box<dyn Fn(&str) -> bool> {
    if q.text.is_empty() {
        return Box::new(|_| true);
    }
    if q.regex
        && let Ok(re) = RegexBuilder::new(&q.text).case_insensitive(true).build()
    {
        return Box::new(move |s| re.is_match(s));
    }
    let needle = q.text.to_lowercase();
    Box::new(move |s| s.to_lowercase().contains(&needle))
}

fn compute_order(q: &ViewQuery, s: &MainDataStore) -> Vec<String> {
    let text = text_matcher(q);
    let rows: Vec<(&String, &Fields)> = s
        .torrents
        .iter()
        .filter(|(h, t)| {
            q.status.matches(t)
                && q.category.matches(t)
                && q.tag.matches(t)
                && q.tracker.matches(h, t, &s.trackers)
                && text(t.get("name").and_then(Value::as_str).unwrap_or(""))
        })
        .collect();
    let key = if q.sort_by.is_empty() {
        "name"
    } else {
        q.sort_by.as_str()
    };
    // Extract each sort key once (lower-casing strings once) instead of on every comparison.
    enum K {
        Num(f64),
        Text(String),
        Bool(bool),
        Missing,
    }
    let mut keyed: Vec<(K, &String)> = rows
        .into_iter()
        .map(|(h, t)| {
            let k = match t.get(key) {
                Some(Value::Number(n)) => K::Num(n.as_f64().unwrap_or(0.0)),
                Some(Value::String(s)) => K::Text(s.to_lowercase()),
                Some(Value::Bool(b)) => K::Bool(*b),
                _ => K::Missing,
            };
            (k, h)
        })
        .collect();
    keyed.sort_unstable_by(|(ka, ha), (kb, hb)| {
        let o = match (ka, kb) {
            (K::Num(x), K::Num(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
            (K::Text(x), K::Text(y)) => x.cmp(y),
            (K::Bool(x), K::Bool(y)) => x.cmp(y),
            (K::Missing, K::Missing) => Ordering::Equal,
            (K::Missing, _) => Ordering::Less,
            (_, K::Missing) => Ordering::Greater,
            _ => Ordering::Equal,
        };
        let o = if q.reverse { o.reverse() } else { o };
        o.then_with(|| ha.cmp(hb))
    });
    keyed.into_iter().map(|(_, h)| h.clone()).collect()
}

/// Transform `old` into `new` with removes, then moves/inserts. Falls back to `Reset` when large.
pub fn diff_orders(old: &[String], new: &[String]) -> Vec<RowOp> {
    let new_set: std::collections::HashSet<&String> = new.iter().collect();
    let mut cur: Vec<String> = old.to_vec();
    let mut ops = Vec::new();
    // Removals, highest index first so earlier indexes stay valid.
    for i in (0..cur.len()).rev() {
        if !new_set.contains(&cur[i]) {
            cur.remove(i);
            ops.push(RowOp::Remove { row: i });
            if ops.len() > MAX_OPS {
                return vec![RowOp::Reset];
            }
        }
    }
    for (i, k) in new.iter().enumerate() {
        if cur.get(i) == Some(k) {
            continue;
        }
        match cur.iter().skip(i).position(|x| x == k) {
            Some(off) => {
                let from = i + off;
                let item = cur.remove(from);
                cur.insert(i, item);
                ops.push(RowOp::Move { from, to: i });
            }
            None => {
                cur.insert(i, k.clone());
                ops.push(RowOp::Insert {
                    row: i,
                    key: k.clone(),
                });
            }
        }
        if ops.len() > MAX_OPS {
            return vec![RowOp::Reset];
        }
    }
    ops
}

/// Apply ops to a list (used by tests and by model-less consumers).
pub fn apply_ops(list: &mut Vec<String>, ops: &[RowOp], target: &[String]) {
    for op in ops {
        match op {
            RowOp::Remove { row } => {
                list.remove(*row);
            }
            RowOp::Insert { row, key } => list.insert(*row, key.clone()),
            RowOp::Move { from, to } => {
                let x = list.remove(*from);
                list.insert(*to, x);
            }
            RowOp::Reset => *list = target.to_vec(),
        }
    }
}
