use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use qbt_api::{AuthMode, ClientConfig};
use serde::{Deserialize, Serialize};

use crate::{ClientSettings, SecretStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthKind {
    #[default]
    Cookie,
    ApiKey,
    Basic,
}

/// A saved server connection. The secret (password or API key) is kept in the [`SecretStore`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub auth: AuthKind,
    #[serde(default)]
    pub username: String,
    /// Extra trusted CA certificate (PEM file).
    #[serde(default)]
    pub ca_path: Option<PathBuf>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub extra_headers: Vec<(String, String)>,
    /// Transfer-list column layout etc., saved per profile.
    #[serde(default)]
    pub ui: toml::Table,
}

fn default_timeout() -> u64 {
    30
}

impl Profile {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            auth: AuthKind::Cookie,
            username: String::new(),
            ca_path: None,
            timeout_secs: default_timeout(),
            extra_headers: Vec::new(),
            ui: toml::Table::new(),
        }
    }

    pub fn client_config(&self, secret: &str) -> Result<ClientConfig, String> {
        let auth = match self.auth {
            AuthKind::Cookie => AuthMode::Cookie {
                username: self.username.clone(),
                password: secret.into(),
            },
            AuthKind::ApiKey => AuthMode::ApiKey(secret.into()),
            AuthKind::Basic => AuthMode::Basic {
                username: self.username.clone(),
                password: secret.into(),
            },
        };
        let mut cfg = ClientConfig::new(&self.url, auth).map_err(|e| e.to_string())?;
        cfg.timeout = Duration::from_secs(self.timeout_secs);
        cfg.extra_headers = self.extra_headers.clone();
        if let Some(p) = &self.ca_path {
            cfg.ca_pem =
                Some(fs::read(p).map_err(|e| format!("reading CA file {}: {e}", p.display()))?);
        }
        Ok(cfg)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Disk {
    #[serde(default)]
    last_used: Option<String>,
    #[serde(default)]
    settings: ClientSettings,
    #[serde(default, rename = "profile")]
    profiles: Vec<Profile>,
}

/// Profiles and client settings, persisted to a TOML file with crash-safe writes.
#[derive(Debug)]
pub struct ProfileStore {
    path: PathBuf,
    disk: Disk,
}

impl ProfileStore {
    /// Default location under the platform's config directory.
    pub fn default_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "qbremote")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn load(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let disk = match fs::read_to_string(&path) {
            Ok(s) => toml::from_str(&s).map_err(|e| format!("{}: {e}", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Disk::default(),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        Ok(Self { path, disk })
    }

    /// Write to a temporary file, fsync, then rename over the real one.
    pub fn save(&self) -> Result<(), String> {
        let text = toml::to_string_pretty(&self.disk).map_err(|e| e.to_string())?;
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let tmp = self.path.with_extension("toml.tmp");
        let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
        f.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&tmp, &self.path).map_err(|e| e.to_string())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn profiles(&self) -> &[Profile] {
        &self.disk.profiles
    }
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.disk.profiles.iter().find(|p| p.name == name)
    }
    pub fn upsert(&mut self, p: Profile) {
        match self.disk.profiles.iter_mut().find(|x| x.name == p.name) {
            Some(x) => *x = p,
            None => self.disk.profiles.push(p),
        }
    }
    pub fn remove(&mut self, name: &str, secrets: &dyn SecretStore) {
        self.disk.profiles.retain(|p| p.name != name);
        if self.disk.last_used.as_deref() == Some(name) {
            self.disk.last_used = None;
        }
        secrets.delete(name);
    }
    pub fn last_used(&self) -> Option<&str> {
        self.disk.last_used.as_deref()
    }
    pub fn set_last_used(&mut self, name: &str) {
        self.disk.last_used = Some(name.to_owned());
    }
    pub fn settings(&self) -> &ClientSettings {
        &self.disk.settings
    }
    pub fn settings_mut(&mut self) -> &mut ClientSettings {
        &mut self.disk.settings
    }
}
