use std::collections::{BTreeMap, HashMap};

use serde_json::{Map, Value};

pub type Fields = Map<String, Value>;

/// What a merge changed, for emitting minimal model signals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Changes {
    pub inserted: Vec<String>,
    pub removed: Vec<String>,
    /// Existing rows whose listed fields changed value.
    pub changed: Vec<(String, Vec<String>)>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.inserted.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }
}

/// A keyed table of JSON objects that is merged field by field.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KeyedStore {
    rows: HashMap<String, Fields>,
}

impl KeyedStore {
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn get(&self, key: &str) -> Option<&Fields> {
        self.rows.get(key)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Fields)> {
        self.rows.iter()
    }
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.rows.keys()
    }
    pub fn contains(&self, key: &str) -> bool {
        self.rows.contains_key(key)
    }
    pub fn to_btree(&self) -> BTreeMap<String, Fields> {
        self.rows
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Apply one delta. `full` replaces everything (changes are computed against the old state).
    pub fn apply(
        &mut self,
        full: bool,
        upserts: &BTreeMap<String, Fields>,
        removed: &[String],
    ) -> Changes {
        let mut ch = Changes::default();
        if full {
            let old = std::mem::take(&mut self.rows);
            for k in old.keys() {
                if !upserts.contains_key(k) {
                    ch.removed.push(k.clone());
                }
            }
            for (k, new) in upserts {
                match old.get(k) {
                    None => ch.inserted.push(k.clone()),
                    Some(o) => {
                        let diff = changed_fields(o, new);
                        if !diff.is_empty() {
                            ch.changed.push((k.clone(), diff));
                        }
                    }
                }
                self.rows.insert(k.clone(), new.clone());
            }
            ch.removed.sort();
            return ch;
        }
        for k in removed {
            if self.rows.remove(k).is_some() {
                ch.removed.push(k.clone());
            }
        }
        for (k, partial) in upserts {
            match self.rows.get_mut(k) {
                None => {
                    self.rows.insert(k.clone(), partial.clone());
                    ch.inserted.push(k.clone());
                }
                Some(row) => {
                    let mut diff = Vec::new();
                    for (f, v) in partial {
                        if row.get(f) != Some(v) {
                            row.insert(f.clone(), v.clone());
                            diff.push(f.clone());
                        }
                    }
                    if !diff.is_empty() {
                        ch.changed.push((k.clone(), diff));
                    }
                }
            }
        }
        ch
    }
}

fn changed_fields(old: &Fields, new: &Fields) -> Vec<String> {
    let mut out: Vec<String> = new
        .iter()
        .filter(|(k, v)| old.get(*k) != Some(*v))
        .map(|(k, _)| k.clone())
        .collect();
    out.extend(old.keys().filter(|k| !new.contains_key(*k)).cloned());
    out
}
