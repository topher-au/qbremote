//! Transfer-list columns: how each torrent field is displayed and sorted. Qt-free.

use qbt_core::fmt;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Queue,
    Text,
    Size,
    Speed,
    Progress,
    State,
    Seeds,
    Peers,
    Eta,
    Ratio,
    Float,
    Date,
    Limit,
    Duration,
    Bool,
}

pub struct Col {
    pub key: &'static str,
    pub title: &'static str,
    pub kind: Kind,
    /// Shown by default.
    pub default_visible: bool,
}

const fn c(key: &'static str, title: &'static str, kind: Kind, vis: bool) -> Col {
    Col {
        key,
        title,
        kind,
        default_visible: vis,
    }
}

pub static COLUMNS: &[Col] = &[
    c("priority", "#", Kind::Queue, true),
    c("name", "Name", Kind::Text, true),
    c("size", "Size", Kind::Size, true),
    c("total_size", "Total Size", Kind::Size, false),
    c("progress", "Progress", Kind::Progress, true),
    c("state", "Status", Kind::State, true),
    c("num_seeds", "Seeds", Kind::Seeds, true),
    c("num_leechs", "Peers", Kind::Peers, true),
    c("dlspeed", "Down Speed", Kind::Speed, true),
    c("upspeed", "Up Speed", Kind::Speed, true),
    c("eta", "ETA", Kind::Eta, true),
    c("ratio", "Ratio", Kind::Ratio, true),
    c("popularity", "Popularity", Kind::Ratio, false),
    c("category", "Category", Kind::Text, true),
    c("tags", "Tags", Kind::Text, true),
    c("added_on", "Added On", Kind::Date, true),
    c("completion_on", "Completed On", Kind::Date, false),
    c("tracker", "Tracker", Kind::Text, false),
    c("dl_limit", "Down Limit", Kind::Limit, false),
    c("up_limit", "Up Limit", Kind::Limit, false),
    c("downloaded", "Downloaded", Kind::Size, false),
    c("uploaded", "Uploaded", Kind::Size, false),
    c("downloaded_session", "Session Download", Kind::Size, false),
    c("uploaded_session", "Session Upload", Kind::Size, false),
    c("amount_left", "Remaining", Kind::Size, false),
    c("time_active", "Time Active", Kind::Duration, false),
    c("seeding_time", "Seeding Time", Kind::Duration, false),
    c("save_path", "Save Path", Kind::Text, false),
    c("download_path", "Incomplete Path", Kind::Text, false),
    c("completed", "Completed", Kind::Size, false),
    c("max_ratio", "Ratio Limit", Kind::Ratio, false),
    c("seen_complete", "Last Seen Complete", Kind::Date, false),
    c("last_activity", "Last Activity", Kind::Date, false),
    c("availability", "Availability", Kind::Float, false),
    c("reannounce", "Reannounce In", Kind::Duration, false),
    c("private", "Private", Kind::Bool, false),
    c("infohash_v1", "Info Hash v1", Kind::Text, false),
    c("infohash_v2", "Info Hash v2", Kind::Text, false),
    c("comment", "Comment", Kind::Text, false),
];

fn i(t: &Map<String, Value>, k: &str) -> i64 {
    t.get(k).and_then(Value::as_i64).unwrap_or(0)
}
fn f(t: &Map<String, Value>, k: &str) -> f64 {
    t.get(k).and_then(Value::as_f64).unwrap_or(0.0)
}
fn s<'a>(t: &'a Map<String, Value>, k: &str) -> &'a str {
    t.get(k).and_then(Value::as_str).unwrap_or("")
}

/// Text shown in the cell.
pub fn display(col: &Col, t: &Map<String, Value>) -> String {
    let k = col.key;
    match col.kind {
        Kind::Queue => match i(t, k) {
            p if p > 0 => p.to_string(),
            _ => String::new(),
        },
        Kind::Text => s(t, k).to_owned(),
        Kind::Size => {
            if t.contains_key(k) {
                fmt::size(i(t, k))
            } else {
                String::new()
            }
        }
        Kind::Speed => fmt::speed_or_empty(i(t, k)),
        Kind::Progress => fmt::percent(f(t, k)),
        Kind::State => fmt::state_label(s(t, k)).to_owned(),
        Kind::Seeds => format!("{} ({})", i(t, "num_seeds"), i(t, "num_complete")),
        Kind::Peers => format!("{} ({})", i(t, "num_leechs"), i(t, "num_incomplete")),
        Kind::Eta => fmt::eta(i(t, k)),
        Kind::Ratio => {
            if t.contains_key(k) {
                fmt::ratio(f(t, k))
            } else {
                String::new()
            }
        }
        Kind::Float => {
            if t.contains_key(k) {
                format!("{:.3}", f(t, k))
            } else {
                String::new()
            }
        }
        Kind::Date => fmt::datetime(i(t, k)),
        Kind::Limit => {
            if t.contains_key(k) {
                fmt::limit(i(t, k))
            } else {
                String::new()
            }
        }
        Kind::Duration => {
            if t.contains_key(k) {
                fmt::duration(i(t, k))
            } else {
                String::new()
            }
        }
        Kind::Bool => match t.get(k).and_then(Value::as_bool) {
            Some(true) => "Yes".into(),
            Some(false) => "No".into(),
            None => String::new(),
        },
    }
}

/// Value for sorting and progress bars: the raw JSON field (seeds/peers sort by the first count).
pub fn raw(col: &Col, t: &Map<String, Value>) -> Value {
    t.get(col.key).cloned().unwrap_or(Value::Null)
}

/// Torrent fields a column's text is derived from (usually just its own key).
fn depends_on(col: &Col, field: &str) -> bool {
    match col.kind {
        Kind::Seeds => matches!(field, "num_seeds" | "num_complete"),
        Kind::Peers => matches!(field, "num_leechs" | "num_incomplete"),
        _ => col.key == field,
    }
}

/// Which columns need repainting when these torrent fields change.
pub fn columns_affected_by<'a>(fields: impl IntoIterator<Item = &'a str>) -> Vec<usize> {
    let mut hit = vec![false; COLUMNS.len()];
    for f in fields {
        for (i, c) in COLUMNS.iter().enumerate() {
            if depends_on(c, f) {
                hit[i] = true;
            }
        }
    }
    hit.iter()
        .enumerate()
        .filter(|(_, h)| **h)
        .map(|(i, _)| i)
        .collect()
}

pub fn index_of(key: &str) -> Option<usize> {
    COLUMNS.iter().position(|c| c.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn t(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn displays() {
        let row = t(
            json!({"name":"A","size":1536,"progress":0.5,"state":"stoppedDL","num_seeds":3,"num_complete":10,
            "dlspeed":0,"upspeed":2048,"eta":8640000,"ratio":1.5,"added_on":1700000000,"priority":0,"private":true,"dl_limit":0}),
        );
        let d = |k: &str| display(&COLUMNS[index_of(k).unwrap()], &row);
        assert_eq!(d("name"), "A");
        assert_eq!(d("size"), "1.50 KiB");
        assert_eq!(d("progress"), "50.0%");
        assert_eq!(d("state"), "Stopped");
        assert_eq!(d("num_seeds"), "3 (10)");
        assert_eq!(d("dlspeed"), "");
        assert_eq!(d("upspeed"), "2.00 KiB/s");
        assert_eq!(d("eta"), "∞");
        assert_eq!(d("priority"), "");
        assert_eq!(d("private"), "Yes");
        assert_eq!(d("dl_limit"), "∞");
        assert_eq!(d("total_size"), "", "absent fields render empty");
    }

    #[test]
    fn only_dependent_columns_are_marked_changed() {
        let cols = columns_affected_by(["dlspeed", "progress", "num_complete", "not_a_column"]);
        let keys: Vec<&str> = cols.iter().map(|&i| COLUMNS[i].key).collect();
        assert_eq!(
            keys,
            ["progress", "num_seeds", "dlspeed"],
            "in column order; seeds depends on num_complete"
        );
        assert!(columns_affected_by(["unknown"]).is_empty());
    }

    #[test]
    fn keys_unique_and_defaults_include_name() {
        let mut k: Vec<_> = COLUMNS.iter().map(|c| c.key).collect();
        k.sort();
        k.dedup();
        assert_eq!(k.len(), COLUMNS.len());
        assert!(COLUMNS[index_of("name").unwrap()].default_visible);
    }
}
