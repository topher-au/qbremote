//! Client-side filters mirroring the server's `torrents/info?filter=` rules.
//! The M2 contract test compares these against a live server.

use serde_json::Value;

use crate::store::Fields;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StatusFilter {
    #[default]
    All,
    Downloading,
    Seeding,
    Completed,
    Stopped,
    Running,
    Active,
    Inactive,
    Stalled,
    StalledUploading,
    StalledDownloading,
    Checking,
    Moving,
    Errored,
}

impl StatusFilter {
    pub const ALL: [StatusFilter; 14] = [
        Self::All,
        Self::Downloading,
        Self::Seeding,
        Self::Completed,
        Self::Stopped,
        Self::Running,
        Self::Active,
        Self::Inactive,
        Self::Stalled,
        Self::StalledUploading,
        Self::StalledDownloading,
        Self::Checking,
        Self::Moving,
        Self::Errored,
    ];

    /// The server's `filter` parameter value.
    pub fn api_name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Downloading => "downloading",
            Self::Seeding => "seeding",
            Self::Completed => "completed",
            Self::Stopped => "stopped",
            Self::Running => "running",
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Stalled => "stalled",
            Self::StalledUploading => "stalled_uploading",
            Self::StalledDownloading => "stalled_downloading",
            Self::Checking => "checking",
            Self::Moving => "moving",
            Self::Errored => "errored",
        }
    }

    /// Rules mirror upstream `TorrentFilter::matchState` and `TorrentImpl::isDownloading/
    /// isUploading/isCompleted/isActive` (src/base/bittorrent/torrentimpl.cpp). Note that
    /// `downloading` includes stopped-but-incomplete torrents and `active` means payload traffic.
    pub fn matches(self, t: &Fields) -> bool {
        let (state, dl, up) = Self::parts(t);
        self.matches_parts(state, dl, up)
    }

    /// The fields the status filters look at, extracted once so callers can test many filters.
    pub fn parts(t: &Fields) -> (&str, i64, i64) {
        (
            t.get("state").and_then(Value::as_str).unwrap_or(""),
            t.get("dlspeed").and_then(Value::as_i64).unwrap_or(0),
            t.get("upspeed").and_then(Value::as_i64).unwrap_or(0),
        )
    }

    pub fn matches_parts(self, state: &str, dl: i64, up: i64) -> bool {
        let stopped = matches!(state, "stoppedUP" | "stoppedDL");
        let downloading = matches!(
            state,
            "downloading"
                | "metaDL"
                | "forcedMetaDL"
                | "stalledDL"
                | "checkingDL"
                | "stoppedDL"
                | "queuedDL"
                | "forcedDL"
        );
        let uploading = matches!(
            state,
            "uploading" | "stalledUP" | "checkingUP" | "queuedUP" | "forcedUP"
        );
        match self {
            Self::All => true,
            Self::Downloading => downloading,
            Self::Seeding => uploading,
            Self::Completed => uploading || state == "stoppedUP",
            Self::Stopped => stopped,
            Self::Running => !stopped,
            Self::Active => dl > 0 || up > 0,
            Self::Inactive => !(dl > 0 || up > 0),
            Self::Stalled => matches!(state, "stalledUP" | "stalledDL"),
            Self::StalledUploading => state == "stalledUP",
            Self::StalledDownloading => state == "stalledDL",
            Self::Checking => matches!(state, "checkingUP" | "checkingDL" | "checkingResumeData"),
            Self::Moving => state == "moving",
            Self::Errored => matches!(state, "error" | "missingFiles"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CategoryFilter {
    #[default]
    All,
    Uncategorized,
    /// Includes sub-categories (`name/…`).
    Named(String),
}

impl CategoryFilter {
    pub fn matches(&self, t: &Fields) -> bool {
        let c = t.get("category").and_then(Value::as_str).unwrap_or("");
        match self {
            Self::All => true,
            Self::Uncategorized => c.is_empty(),
            Self::Named(n) => {
                c == n
                    || c.strip_prefix(n.as_str())
                        .is_some_and(|r| r.starts_with('/'))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TagFilter {
    #[default]
    All,
    Untagged,
    Named(String),
}

impl TagFilter {
    pub fn matches(&self, t: &Fields) -> bool {
        let tags = t.get("tags").and_then(Value::as_str).unwrap_or("");
        match self {
            Self::All => true,
            Self::Untagged => tags.trim().is_empty(),
            Self::Named(n) => tags.split(',').any(|x| x.trim() == n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TrackerFilter {
    #[default]
    All,
    Trackerless,
    Error,
    Warning,
    OtherError,
    /// A tracker host; resolved through the `trackers` URL→hashes map.
    Host(String),
}

pub fn tracker_host(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let auth = rest.split(['/', '?']).next().unwrap_or(rest);
    let auth = auth.rsplit('@').next().unwrap_or(auth);
    if let Some(stripped) = auth.strip_prefix('[') {
        return stripped.split(']').next().unwrap_or(stripped);
    }
    auth.rsplit_once(':').map_or(auth, |(h, p)| {
        if p.chars().all(|c| c.is_ascii_digit()) {
            h
        } else {
            auth
        }
    })
}

impl TrackerFilter {
    pub fn matches(
        &self,
        hash: &str,
        t: &Fields,
        trackers: &std::collections::BTreeMap<String, Vec<String>>,
    ) -> bool {
        let flag = |k: &str| t.get(k).and_then(Value::as_bool).unwrap_or(false);
        match self {
            Self::All => true,
            Self::Trackerless => t.get("trackers_count").and_then(Value::as_i64).map_or_else(
                || {
                    t.get("tracker")
                        .and_then(Value::as_str)
                        .is_none_or(str::is_empty)
                },
                |n| n == 0,
            ),
            Self::Error => flag("has_tracker_error"),
            Self::Warning => flag("has_tracker_warning"),
            Self::OtherError => flag("has_other_announce_error"),
            Self::Host(h) => trackers
                .iter()
                .any(|(u, hs)| tracker_host(u) == h && hs.iter().any(|x| x == hash)),
        }
    }
}

/// Everything that decides which rows the transfer list shows, and in what order.
#[derive(Debug, Clone, Default)]
pub struct ViewQuery {
    pub status: StatusFilter,
    pub category: CategoryFilter,
    pub tag: TagFilter,
    pub tracker: TrackerFilter,
    pub text: String,
    pub regex: bool,
    /// Field name as in the torrent object (`name`, `size`, `progress`, …).
    pub sort_by: String,
    pub reverse: bool,
}
