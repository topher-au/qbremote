use std::{cmp::Ordering, fmt, str::FromStr};

/// A `major.minor.patch` WebAPI version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebApiVersion(pub u32, pub u32, pub u32);

impl WebApiVersion {
    pub const fn new(a: u32, b: u32, c: u32) -> Self {
        Self(a, b, c)
    }
}

impl Ord for WebApiVersion {
    fn cmp(&self, o: &Self) -> Ordering {
        (self.0, self.1, self.2).cmp(&(o.0, o.1, o.2))
    }
}
impl PartialOrd for WebApiVersion {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl fmt::Display for WebApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}
impl FromStr for WebApiVersion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        let mut it = s
            .trim()
            .trim_start_matches('v')
            .split('.')
            .map(|p| p.parse::<u32>());
        let a = it.next().ok_or(())?.map_err(|_| ())?;
        let b = it.next().transpose().map_err(|_| ())?.unwrap_or(0);
        let c = it.next().transpose().map_err(|_| ())?.unwrap_or(0);
        Ok(Self(a, b, c))
    }
}

/// First WebAPI version with the `start`/`stop` endpoints and `stopped*` states (qBittorrent 5.0).
///
/// Verified against the source: commit `5d1c249606` ("Use Start/Stop instead of Resume/Pause",
/// 2024-03-25) bumped `API_VERSION` from 2.10.4 to 2.11.0; tag `release-5.0.0beta1` is 2.10.4
/// (still `pauseAction`), `release-5.0.0rc1` is 2.11.2 (`stopAction`). Older servers are unsupported.
pub const MIN_SUPPORTED: WebApiVersion = WebApiVersion(2, 11, 0);

/// Minimum WebAPI version per endpoint, from `WebAPI_Changelog.md` ("Add … endpoint" entries).
/// Endpoints absent here predate the changelog (2.11.6) or are undocumented in it; for those the
/// client falls back to runtime detection (a 404 marks the action unavailable).
pub static MIN_VERSIONS: &[(&str, &str, WebApiVersion)] = &[
    ("torrents", "cancelFetchMetadata", WebApiVersion(2, 17, 0)),
    ("rss", "exportRules", WebApiVersion(2, 16, 2)),
    ("rss", "importRules", WebApiVersion(2, 16, 2)),
    ("transfer", "pauseSession", WebApiVersion(2, 16, 2)),
    ("transfer", "resumeSession", WebApiVersion(2, 16, 2)),
    ("transfer", "getSpeedLimits", WebApiVersion(2, 16, 0)),
    ("transfer", "setSpeedLimits", WebApiVersion(2, 16, 0)),
    ("torrents", "downloadFile", WebApiVersion(2, 16, 0)),
    ("rss", "cloneRule", WebApiVersion(2, 15, 4)),
    ("app", "getFreeSpaceAtPath", WebApiVersion(2, 15, 2)),
    ("app", "processInfo", WebApiVersion(2, 15, 1)),
    ("torrents", "pieceAvailability", WebApiVersion(2, 15, 1)),
    ("torrents", "setComment", WebApiVersion(2, 12, 1)),
    // Bounds observed on real servers (404 on 5.0.5 = WebAPI 2.11.2, served by 5.1.4 = 2.11.4);
    // the changelog does not go back that far. `app/setCookies` is documented as 2.11.3.
    ("app", "setCookies", WebApiVersion(2, 11, 3)),
    ("torrents", "addWebSeeds", WebApiVersion(2, 11, 4)),
    ("torrents", "editWebSeed", WebApiVersion(2, 11, 4)),
    ("torrents", "removeWebSeeds", WebApiVersion(2, 11, 4)),
    ("torrents", "setTags", WebApiVersion(2, 11, 4)),
    ("app", "rotateAPIKey", WebApiVersion(2, 14, 1)),
    ("app", "deleteAPIKey", WebApiVersion(2, 14, 1)),
    ("clientdata", "load", WebApiVersion(2, 13, 1)),
    ("clientdata", "store", WebApiVersion(2, 13, 1)),
    ("torrents", "fetchMetadata", WebApiVersion(2, 11, 9)),
    ("torrents", "parseMetadata", WebApiVersion(2, 11, 9)),
    ("torrents", "saveMetadata", WebApiVersion(2, 11, 9)),
];

/// Feature switches derived from the server's WebAPI version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub version: WebApiVersion,
    /// Server is new enough for the `start`/`stop` API (qBittorrent 5.0+).
    pub supported: bool,
    pub api_key_auth: bool,
    pub basic_auth: bool,
    pub download_file: bool,
    pub process_info: bool,
    pub speed_limits: bool,
    pub session_pause: bool,
    pub client_data: bool,
}

impl Capabilities {
    pub fn from_version(v: WebApiVersion) -> Self {
        let mut c = Self {
            version: v,
            supported: v >= MIN_SUPPORTED,
            api_key_auth: false,
            basic_auth: v >= WebApiVersion(2, 15, 0),
            download_file: false,
            process_info: false,
            speed_limits: false,
            session_pause: false,
            client_data: false,
        };
        c.api_key_auth = v >= WebApiVersion(2, 14, 1);
        c.download_file = c.supports("torrents", "downloadFile");
        c.process_info = c.supports("app", "processInfo");
        c.speed_limits = c.supports("transfer", "getSpeedLimits");
        c.session_pause = c.supports("transfer", "pauseSession");
        c.client_data = c.supports("clientdata", "load");
        c
    }

    /// Whether the server's version is high enough for `scope/action`. Actions without a
    /// documented minimum are assumed present (see [`MIN_VERSIONS`]).
    pub fn supports(&self, scope: &str, action: &str) -> bool {
        MIN_VERSIONS
            .iter()
            .find(|(s, a, _)| *s == scope && *a == action)
            .is_none_or(|(_, _, min)| self.version >= *min)
    }

    /// Tooltip text for a greyed-out feature, e.g. "requires WebAPI ≥ 2.16.0".
    pub fn requirement(scope: &str, action: &str) -> Option<String> {
        MIN_VERSIONS
            .iter()
            .find(|(s, a, _)| *s == scope && *a == action)
            .map(|(_, _, v)| format!("requires WebAPI ≥ {v}"))
    }
}
