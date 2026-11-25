//! Process-wide state. `APP` is touched only on the Qt thread; everything else is thread-safe.

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use qbt_core::{SessionHandle, SessionInfo};
use qbt_sync::{Counts, MainDataStore};

use crate::tables::Payload;

#[derive(Default)]
pub struct AppState {
    pub store: MainDataStore,
    pub counts: Counts,
    pub info: Option<SessionInfo>,
}

thread_local! {
    /// The mirrored server state. Qt thread only.
    pub static APP: RefCell<AppState> = RefCell::new(AppState::default());
}

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static SESSION: Mutex<Option<SessionHandle>> = Mutex::new(None);
static CACHE: Mutex<Option<HashMap<String, Payload>>> = Mutex::new(None);

pub fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("qbt-rt")
            .build()
            .expect("tokio runtime")
    })
}

pub fn session() -> Option<SessionHandle> {
    SESSION.lock().unwrap().clone()
}

pub fn set_session(s: Option<SessionHandle>) -> Option<SessionHandle> {
    std::mem::replace(&mut *SESSION.lock().unwrap(), s)
}

const MAX_CACHED_ROWS: usize = 20_000;

/// Remember the last data of a table so a model created later can be filled immediately.
pub fn cache_payload(source: &str, p: &Payload) {
    let mut g = CACHE.lock().unwrap();
    let map = g.get_or_insert_with(HashMap::new);
    if p.append {
        let e = map.entry(source.to_owned()).or_default();
        e.keys.extend(p.keys.iter().cloned());
        e.rows.extend(p.rows.iter().cloned());
        if e.rows.len() > MAX_CACHED_ROWS {
            let drop = e.rows.len() - MAX_CACHED_ROWS;
            e.keys.drain(..drop);
            e.rows.drain(..drop);
        }
    } else {
        map.insert(source.to_owned(), p.clone());
    }
}

pub fn cached_payload(source: &str) -> Option<Payload> {
    CACHE.lock().unwrap().as_ref()?.get(source).cloned()
}

pub fn clear_cache() {
    *CACHE.lock().unwrap() = None;
}

static CALL_ID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1_000_000);

/// Ids for `call`, distinct from the session's command ids.
pub fn next_call_id() -> i32 {
    CALL_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
