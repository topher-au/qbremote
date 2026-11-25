use serde::{Deserialize, Serialize};

macro_rules! string_enum {
    ($(#[$m:meta])* $name:ident { $($v:ident = $s:literal),* $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[serde(from = "String", into = "String")]
        pub enum $name { $($v,)* Unknown(String) }
        impl From<String> for $name {
            fn from(s: String) -> Self { match s.as_str() { $($s => Self::$v,)* _ => Self::Unknown(s) } }
        }
        impl From<$name> for String {
            fn from(v: $name) -> String { match v { $($name::$v => $s.to_owned(),)* $name::Unknown(s) => s } }
        }
        impl $name {
            pub fn as_str(&self) -> &str { match self { $(Self::$v => $s,)* Self::Unknown(s) => s } }
        }
    };
}

string_enum!(
    /// `state` values from the pinned source (5.x names).
    TorrentState {
        Error = "error", MissingFiles = "missingFiles", Uploading = "uploading",
        StoppedUp = "stoppedUP", QueuedUp = "queuedUP", StalledUp = "stalledUP",
        CheckingUp = "checkingUP", ForcedUp = "forcedUP", Downloading = "downloading",
        MetaDl = "metaDL", ForcedMetaDl = "forcedMetaDL", StoppedDl = "stoppedDL",
        QueuedDl = "queuedDL", StalledDl = "stalledDL", CheckingDl = "checkingDL",
        ForcedDl = "forcedDL", CheckingResumeData = "checkingResumeData", Moving = "moving",
        Unknown_ = "unknown",
    }
);

string_enum!(
    ContentLayout { Original = "Original", Subfolder = "Subfolder", NoSubfolder = "NoSubfolder" }
);
string_enum!(
    ShareLimitAction {
        Default = "Default", Stop = "Stop", Remove = "Remove",
        RemoveWithContent = "RemoveWithContent", EnableSuperSeeding = "EnableSuperSeeding",
    }
);

impl TorrentState {
    pub fn is_stopped(&self) -> bool {
        matches!(self, Self::StoppedUp | Self::StoppedDl)
    }
    pub fn is_downloading(&self) -> bool {
        matches!(
            self,
            Self::Downloading
                | Self::MetaDl
                | Self::ForcedMetaDl
                | Self::StalledDl
                | Self::CheckingDl
                | Self::ForcedDl
                | Self::QueuedDl
                | Self::StoppedDl
        )
    }
    pub fn is_uploading(&self) -> bool {
        matches!(
            self,
            Self::Uploading
                | Self::StalledUp
                | Self::CheckingUp
                | Self::ForcedUp
                | Self::QueuedUp
                | Self::StoppedUp
        )
    }
}

/// Tracker status codes: 0 disabled … 6 unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(from = "i64", into = "i64")]
pub enum TrackerStatus {
    Disabled,
    NotContacted,
    Working,
    Updating,
    NotWorking,
    TrackerError,
    Unreachable,
    Other(i64),
}
impl From<i64> for TrackerStatus {
    fn from(v: i64) -> Self {
        match v {
            0 => Self::Disabled,
            1 => Self::NotContacted,
            2 => Self::Working,
            3 => Self::Updating,
            4 => Self::NotWorking,
            5 => Self::TrackerError,
            6 => Self::Unreachable,
            o => Self::Other(o),
        }
    }
}
impl From<TrackerStatus> for i64 {
    fn from(v: TrackerStatus) -> i64 {
        match v {
            TrackerStatus::Disabled => 0,
            TrackerStatus::NotContacted => 1,
            TrackerStatus::Working => 2,
            TrackerStatus::Updating => 3,
            TrackerStatus::NotWorking => 4,
            TrackerStatus::TrackerError => 5,
            TrackerStatus::Unreachable => 6,
            TrackerStatus::Other(o) => o,
        }
    }
}

/// File priorities: 0 skip, 1 normal, 6 high, 7 maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(from = "i64", into = "i64")]
pub enum FilePriority {
    DoNotDownload,
    Normal,
    High,
    Maximum,
    Other(i64),
}
impl From<i64> for FilePriority {
    fn from(v: i64) -> Self {
        match v {
            0 => Self::DoNotDownload,
            1 => Self::Normal,
            6 => Self::High,
            7 => Self::Maximum,
            o => Self::Other(o),
        }
    }
}
impl From<FilePriority> for i64 {
    fn from(v: FilePriority) -> i64 {
        match v {
            FilePriority::DoNotDownload => 0,
            FilePriority::Normal => 1,
            FilePriority::High => 6,
            FilePriority::Maximum => 7,
            FilePriority::Other(o) => o,
        }
    }
}
