//! Incremental `rid`-based state engine for `sync/maindata` and `sync/torrentPeers`.
//! Pure and UI-agnostic: no Qt, no I/O.

pub mod counts;
pub mod filter;
pub mod maindata;
pub mod store;
pub mod view;

pub use counts::Counts;
pub use filter::{CategoryFilter, StatusFilter, TagFilter, TrackerFilter, ViewQuery};
pub use maindata::{MainChanges, MainDataStore, PeersStore};
pub use store::{Changes, KeyedStore};
pub use view::{RowOp, ViewIndex};
