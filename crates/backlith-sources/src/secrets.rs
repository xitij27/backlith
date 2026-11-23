// SPDX-License-Identifier: GPL-3.0-or-later
//! The secret store. A calendar link gives full read access to a calendar,
//! so it is kept here and nowhere else: on Windows in Credential Manager,
//! on Linux in the Secret Service (GNOME Keyring).

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the secret store failed: {0}")]
pub struct SecretError(pub String);

pub trait SecretStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, SecretError>;
    fn set(&self, key: &str, value: &str) -> Result<(), SecretError>;
    fn delete(&self, key: &str) -> Result<(), SecretError>;
}

/// The system's own secret store, under the service name "Backlith".
pub struct KeyringStore;

pub const KEYRING_SERVICE: &str = "Backlith";

impl SecretStore for KeyringStore {
    fn get(&self, key: &str) -> Result<Option<String>, SecretError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, key).map_err(|e| SecretError(e.to_string()))?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretError(e.to_string())),
        }
    }
    fn set(&self, key: &str, value: &str) -> Result<(), SecretError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, key).map_err(|e| SecretError(e.to_string()))?;
        entry.set_password(value).map_err(|e| SecretError(e.to_string()))
    }
    fn delete(&self, key: &str) -> Result<(), SecretError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, key).map_err(|e| SecretError(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecretError(e.to_string())),
        }
    }
}

/// An in-memory store, for tests and for running without a system store.
#[derive(Default)]
pub struct MemoryStore {
    values: Mutex<HashMap<String, String>>,
}

impl MemoryStore {
    fn values(&self) -> std::sync::MutexGuard<'_, HashMap<String, String>> {
        self.values.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl SecretStore for MemoryStore {
    fn get(&self, key: &str) -> Result<Option<String>, SecretError> {
        Ok(self.values().get(key).cloned())
    }
    fn set(&self, key: &str, value: &str) -> Result<(), SecretError> {
        self.values().insert(key.to_string(), value.to_string());
        Ok(())
    }
    fn delete(&self, key: &str) -> Result<(), SecretError> {
        self.values().remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trip() {
        let s = MemoryStore::default();
        assert_eq!(s.get("calendar-link:a").unwrap(), None);
        s.set("calendar-link:a", "https://example.com/a.ics").unwrap();
        assert_eq!(s.get("calendar-link:a").unwrap().as_deref(), Some("https://example.com/a.ics"));
        s.delete("calendar-link:a").unwrap();
        assert_eq!(s.get("calendar-link:a").unwrap(), None);
        s.delete("calendar-link:a").unwrap();
    }
}
