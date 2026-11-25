use std::{collections::HashMap, sync::Mutex};

const SERVICE: &str = "qbremote";

/// Where passwords and API keys live. Never the config file.
pub trait SecretStore: Send + Sync {
    fn get(&self, profile: &str) -> Option<String>;
    fn set(&self, profile: &str, secret: &str) -> Result<(), String>;
    fn delete(&self, profile: &str);
}

/// OS secret store (Secret Service, Keychain, Credential Manager).
#[derive(Default)]
pub struct KeyringSecrets;

impl SecretStore for KeyringSecrets {
    fn get(&self, profile: &str) -> Option<String> {
        keyring::Entry::new(SERVICE, profile)
            .ok()?
            .get_password()
            .ok()
    }
    fn set(&self, profile: &str, secret: &str) -> Result<(), String> {
        keyring::Entry::new(SERVICE, profile)
            .and_then(|e| e.set_password(secret))
            .map_err(|e| e.to_string())
    }
    fn delete(&self, profile: &str) {
        if let Ok(e) = keyring::Entry::new(SERVICE, profile) {
            let _ = e.delete_credential();
        }
    }
}

/// In-memory store for tests and for sessions where the user chose not to remember secrets.
#[derive(Default)]
pub struct MemorySecrets(Mutex<HashMap<String, String>>);

impl SecretStore for MemorySecrets {
    fn get(&self, profile: &str) -> Option<String> {
        self.0.lock().unwrap().get(profile).cloned()
    }
    fn set(&self, profile: &str, secret: &str) -> Result<(), String> {
        self.0
            .lock()
            .unwrap()
            .insert(profile.to_owned(), secret.to_owned());
        Ok(())
    }
    fn delete(&self, profile: &str) {
        self.0.lock().unwrap().remove(profile);
    }
}
