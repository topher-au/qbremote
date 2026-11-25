use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BuildInfo {
    pub qt: Option<String>,
    pub libtorrent: Option<String>,
    pub boost: Option<String>,
    pub openssl: Option<String>,
    pub zlib: Option<String>,
    pub bitness: Option<u32>,
    pub platform: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// `app/preferences`: a key/value map. `Debug` redacts secrets.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Preferences(pub Map<String, Value>);

const SECRET_KEYS: &[&str] = &[
    "web_ui_api_key",
    "web_ui_password",
    "proxy_password",
    "dyndns_password",
    "mail_notification_password",
];

pub fn is_secret_key(k: &str) -> bool {
    SECRET_KEYS.contains(&k) || k.ends_with("_password")
}

impl fmt::Debug for Preferences {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut m = f.debug_map();
        for (k, v) in &self.0 {
            if is_secret_key(k) {
                m.entry(k, &"<redacted>");
            } else {
                m.entry(k, v);
            }
        }
        m.finish()
    }
}

impl Preferences {
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Typed view of the values (lenient: a mistyped key becomes `None`).
    pub fn typed(&self) -> super::prefs::TypedPreferences {
        serde_json::from_value(Value::Object(self.0.clone())).unwrap_or_default()
    }
}
