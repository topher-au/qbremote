use super::lenient;

lenient!(LogEntry {
    id: i64,
    message: String,
    timestamp: i64,
    #[serde(rename = "type")]
    kind: i64
});
lenient!(PeerLogEntry {
    id: i64,
    ip: String,
    timestamp: i64,
    blocked: bool,
    reason: String
});

/// `log/main` severity filter bits.
pub const LOG_NORMAL: i64 = 1;
pub const LOG_INFO: i64 = 2;
pub const LOG_WARNING: i64 = 4;
pub const LOG_CRITICAL: i64 = 8;
