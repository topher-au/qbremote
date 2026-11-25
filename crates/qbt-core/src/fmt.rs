//! Display formatting for sizes, speeds, durations and dates (pure; unit tested).

const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

pub fn size(bytes: i64) -> String {
    if bytes < 0 {
        return String::new();
    }
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else if v >= 100.0 {
        format!("{v:.0} {}", UNITS[i])
    } else if v >= 10.0 {
        format!("{v:.1} {}", UNITS[i])
    } else {
        format!("{v:.2} {}", UNITS[i])
    }
}

pub fn speed(bps: i64) -> String {
    format!("{}/s", size(bps.max(0)))
}

/// Speed column: empty when idle.
pub fn speed_or_empty(bps: i64) -> String {
    if bps <= 0 { String::new() } else { speed(bps) }
}

/// Limit column: `∞` for none (qBittorrent uses 0 or -1 for unlimited).
pub fn limit(bps: i64) -> String {
    if bps <= 0 { "∞".into() } else { speed(bps) }
}

pub fn ratio(r: f64) -> String {
    // Negative means "no ratio yet"; the server clamps absurd values, so treat both as infinite.
    if !(0.0..100.0).contains(&r) {
        "∞".into()
    } else {
        format!("{r:.2}")
    }
}

pub fn percent(p: f64) -> String {
    let pct = (p * 1000.0).floor() / 10.0;
    format!("{pct:.1}%")
}

/// ETA in seconds; 8640000 is the server's "infinity".
pub fn eta(secs: i64) -> String {
    if !(0..8_640_000).contains(&secs) {
        return "∞".into();
    }
    duration(secs)
}

pub fn duration(secs: i64) -> String {
    let secs = secs.max(0);
    let (d, h, m) = (secs / 86400, secs % 86400 / 3600, secs % 3600 / 60);
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else if m > 0 {
        format!("{m}m")
    } else {
        format!("{secs}s")
    }
}

/// Unix timestamp → `YYYY-MM-DD HH:MM:SS` (UTC). Negative or zero timestamps are "never".
pub fn datetime(ts: i64) -> String {
    if ts <= 0 {
        return String::new();
    }
    let days = ts.div_euclid(86400);
    let rem = ts.rem_euclid(86400);
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Human label for a torrent `state` value.
pub fn state_label(state: &str) -> &'static str {
    match state {
        "error" => "Errored",
        "missingFiles" => "Missing files",
        "uploading" => "Seeding",
        "stoppedUP" => "Completed",
        "queuedUP" | "queuedDL" => "Queued",
        "stalledUP" => "Seeding (stalled)",
        "checkingUP" | "checkingDL" => "Checking",
        "forcedUP" => "Forced seeding",
        "downloading" => "Downloading",
        "metaDL" | "forcedMetaDL" => "Downloading metadata",
        "stoppedDL" => "Stopped",
        "stalledDL" => "Stalled",
        "forcedDL" => "Forced downloading",
        "checkingResumeData" => "Checking resume data",
        "moving" => "Moving",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(size(0), "0 B");
        assert_eq!(size(1023), "1023 B");
        assert_eq!(size(1536), "1.50 KiB");
        assert_eq!(size(10 * 1024 * 1024), "10.0 MiB");
        assert_eq!(size(500 * 1024 * 1024 * 1024), "500 GiB");
        assert_eq!(speed(2048), "2.00 KiB/s");
        assert_eq!(speed_or_empty(0), "");
        assert_eq!(limit(0), "∞");
    }

    #[test]
    fn times() {
        assert_eq!(eta(8_640_000), "∞");
        assert_eq!(eta(59), "59s");
        assert_eq!(eta(3700), "1h 1m");
        assert_eq!(eta(90_000), "1d 1h");
        assert_eq!(datetime(0), "");
        assert_eq!(datetime(1_700_000_000), "2023-11-14 22:13:20");
        assert_eq!(datetime(951_782_400), "2000-02-29 00:00:00");
    }

    #[test]
    fn ratios() {
        assert_eq!(percent(0.9999), "99.9%");
        assert_eq!(percent(1.0), "100.0%");
        assert_eq!(ratio(1.234), "1.23");
        assert_eq!(ratio(-1.0), "∞");
        assert_eq!(state_label("stoppedDL"), "Stopped");
    }
}
