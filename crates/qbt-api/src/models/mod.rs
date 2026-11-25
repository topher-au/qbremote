//! Response models. Decoding is lenient: every field is optional and unknown keys land in `extra`.

pub mod app;
pub mod enums;
pub mod log;
pub mod prefs;
pub mod search;
pub mod sync;
pub mod torrents;
pub mod transfer;

pub use app::{BuildInfo, Preferences};
pub use enums::{ContentLayout, FilePriority, ShareLimitAction, TorrentState, TrackerStatus};

/// Declares a lenient response struct: all fields `Option`, plus `extra` for unknown keys.
macro_rules! lenient {
    ($(#[$m:meta])* $name:ident { $($(#[$fm:meta])* $f:ident : $t:ty),* $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct $name {
            $( $(#[$fm])* #[serde(default, skip_serializing_if = "Option::is_none")] pub $f: Option<$t>, )*
            #[serde(flatten)]
            pub extra: serde_json::Map<String, serde_json::Value>,
        }
    };
}
pub(crate) use lenient;
