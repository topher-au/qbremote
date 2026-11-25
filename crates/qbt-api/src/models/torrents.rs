use serde_json::{Map, Value};
use std::collections::BTreeMap;

use super::{FilePriority, TorrentState, TrackerStatus, lenient};

lenient!(
    /// One entry of `torrents/info` / `sync/maindata` (`serialize_torrent.h`).
    Torrent {
        hash: String, infohash_v1: String, infohash_v2: String, name: String, magnet_uri: String,
        size: i64, progress: f64, dlspeed: i64, upspeed: i64, priority: i64,
        num_seeds: i64, num_complete: i64, num_leechs: i64, num_incomplete: i64,
        ratio: f64, popularity: f64, eta: i64, state: TorrentState, error: String,
        seq_dl: bool, f_l_piece_prio: bool, category: String, tags: String,
        super_seeding: bool, force_start: bool, save_path: String, download_path: String,
        content_path: String, root_path: String, added_on: i64, completion_on: i64,
        tracker: String, trackers_count: i64, dl_limit: i64, up_limit: i64,
        downloaded: i64, uploaded: i64, downloaded_session: i64, uploaded_session: i64,
        amount_left: i64, completed: i64, max_ratio: f64, max_seeding_time: i64,
        max_inactive_seeding_time: i64, ratio_limit: f64, seeding_time_limit: i64,
        inactive_seeding_time_limit: i64, share_limits_mode: String, share_limit_action: String,
        seen_complete: i64, last_activity: i64, total_size: i64, auto_tmm: bool,
        time_active: i64, seeding_time: i64, availability: f64, reannounce: i64,
        comment: String, private: bool, has_metadata: bool, connections_count: i64,
        connections_limit: i64, total_wasted: i64, pieces_num: i64, piece_size: i64,
        pieces_have: i64, created_by: String, creation_date: i64,
        has_tracker_warning: bool, has_tracker_error: bool, has_other_announce_error: bool,
    }
);

lenient!(
    /// `torrents/properties`.
    TorrentProperties {
        addition_date: i64, comment: String, completion_date: i64, created_by: String,
        creation_date: i64, dl_limit: i64, dl_speed: i64, dl_speed_avg: i64, download_path: String,
        eta: i64, hash: String, infohash_v1: String, infohash_v2: String,
        is_private: bool, private: bool, last_seen: i64, name: String, nb_connections: i64,
        nb_connections_limit: i64, peers: i64, peers_total: i64, piece_size: i64, pieces_have: i64,
        pieces_num: i64, reannounce: i64, save_path: String, seeding_time: i64, seeds: i64,
        seeds_total: i64, share_ratio: f64, time_elapsed: i64, total_downloaded: i64,
        total_downloaded_session: i64, total_size: i64, total_uploaded: i64,
        total_uploaded_session: i64, total_wasted: i64, up_limit: i64, up_speed: i64, up_speed_avg: i64,
        has_metadata: bool, popularity: f64, progress: f64, availability: f64,
    }
);

lenient!(TrackerEndpoint {
    name: String,
    updating: bool,
    status: TrackerStatus,
    msg: String,
    bt_version: i64,
    num_peers: i64,
    num_seeds: i64,
    num_leeches: i64,
    num_downloaded: i64,
    next_announce: i64,
    min_announce: i64,
});

lenient!(Tracker {
    url: String, status: TrackerStatus, tier: i64, msg: String, num_peers: i64, num_seeds: i64,
    num_leeches: i64, num_downloaded: i64, next_announce: i64, min_announce: i64,
    disabled_for_torrent: bool, endpoints: Vec<TrackerEndpoint>,
});

lenient!(WebSeed { url: String });

lenient!(TorrentFile {
    index: i64, name: String, size: i64, progress: f64, priority: FilePriority,
    is_seed: bool, piece_range: Vec<i64>, availability: f64,
});

lenient!(Category {
    name: String,
    #[serde(rename = "savePath")]
    save_path: String,
    download_path: Value,
    #[serde(rename = "ratio_limit")]
    ratio_limit: f64,
    seeding_time_limit: i64,
    inactive_seeding_time_limit: i64,
    share_limits_mode: String,
    share_limit_action: String,
});

lenient!(SslParameters {
    ssl_certificate: String,
    ssl_private_key: String,
    ssl_dh_params: String
});

lenient!(
    /// `torrents/add` result.
    AddResult { success_count: i64, failure_count: i64, pending_count: i64, added_torrent_ids: Vec<String> }
);

pub type Categories = BTreeMap<String, Category>;

/// Filters accepted by `torrents/info`.
#[derive(Debug, Clone, Default)]
pub struct InfoQuery {
    pub filter: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub sort: Option<String>,
    pub reverse: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub hashes: Option<Vec<String>>,
    pub private: Option<bool>,
    pub include_files: Option<bool>,
    pub include_trackers: Option<bool>,
}

/// Options for `torrents/add`; every field is optional and encoded under the server's key names.
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    pub urls: Vec<String>,
    /// `(file name, bytes)` for each local .torrent.
    pub files: Vec<(String, Vec<u8>)>,
    pub fields: Vec<(String, String)>,
}

impl AddOptions {
    pub fn put(mut self, k: &str, v: String) -> Self {
        self.fields.push((k.to_owned(), v));
        self
    }
    pub fn savepath(self, v: &str) -> Self {
        self.put("savepath", v.into())
    }
    pub fn download_path(self, v: &str) -> Self {
        self.put("downloadPath", v.into())
    }
    pub fn use_download_path(self, v: bool) -> Self {
        self.put("useDownloadPath", v.to_string())
    }
    pub fn category(self, v: &str) -> Self {
        self.put("category", v.into())
    }
    pub fn tags(self, v: &[String]) -> Self {
        self.put("tags", v.join(","))
    }
    pub fn rename(self, v: &str) -> Self {
        self.put("rename", v.into())
    }
    pub fn up_limit(self, v: i64) -> Self {
        self.put("upLimit", v.to_string())
    }
    pub fn dl_limit(self, v: i64) -> Self {
        self.put("dlLimit", v.to_string())
    }
    pub fn ratio_limit(self, v: f64) -> Self {
        self.put("ratioLimit", v.to_string())
    }
    pub fn seeding_time_limit(self, v: i64) -> Self {
        self.put("seedingTimeLimit", v.to_string())
    }
    pub fn inactive_seeding_time_limit(self, v: i64) -> Self {
        self.put("inactiveSeedingTimeLimit", v.to_string())
    }
    pub fn share_limits_mode(self, v: &str) -> Self {
        self.put("shareLimitsMode", v.into())
    }
    pub fn share_limit_action(self, v: &str) -> Self {
        self.put("shareLimitAction", v.into())
    }
    pub fn auto_tmm(self, v: bool) -> Self {
        self.put("autoTMM", v.to_string())
    }
    pub fn stopped(self, v: bool) -> Self {
        self.put("stopped", v.to_string())
    }
    pub fn forced(self, v: bool) -> Self {
        self.put("forced", v.to_string())
    }
    pub fn add_to_top_of_queue(self, v: bool) -> Self {
        self.put("addToTopOfQueue", v.to_string())
    }
    pub fn stop_condition(self, v: &str) -> Self {
        self.put("stopCondition", v.into())
    }
    pub fn content_layout(self, v: &str) -> Self {
        self.put("contentLayout", v.into())
    }
    pub fn seed_mode(self, v: bool) -> Self {
        self.put("seedMode", v.to_string())
    }
    pub fn sequential_download(self, v: bool) -> Self {
        self.put("sequentialDownload", v.to_string())
    }
    pub fn first_last_piece_prio(self, v: bool) -> Self {
        self.put("firstLastPiecePrio", v.to_string())
    }
    /// Only valid with a single source and metadata available.
    pub fn file_priorities(self, v: &[FilePriority]) -> Self {
        let s: Vec<String> = v.iter().map(|p| i64::from(*p).to_string()).collect();
        self.put("filePriorities", s.join(","))
    }
    pub fn downloader(self, v: &str) -> Self {
        self.put("downloader", v.into())
    }
}

pub type RawMap = Map<String, Value>;
