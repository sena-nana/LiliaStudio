//! Secret storage for the native host.
//!
//! The real store is the platform credential manager via the `keyring`
//! facade (service `ameya.ai`): Windows Credential Manager, macOS Keychain,
//! or the Linux secret service. This mirrors the Tauri implementation in
//! `src-tauri/src/windows/credential.rs` and extends it to non-Windows
//! platforms. [`InMemorySecretStore`] backs headless tests.

use std::collections::HashMap;
use std::sync::Mutex;

use ameya_core::ai::settings::SecretStore;

const SERVICE_NAME: &str = "ameya.ai";

/// 平台相关错误文案：Windows 沿用 Tauri 的逐字契约，其余平台用系统凭据表述。
fn store_label() -> &'static str {
    if cfg!(windows) {
        "Windows 凭据"
    } else {
        "系统凭据"
    }
}

/// System credential manager store (platform-native via `keyring`).
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemCredentialStore;

impl SystemCredentialStore {
    fn credential_entry(name: &str) -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICE_NAME, name)
            .map_err(|error| format!("创建{}项失败：{error}", store_label()))
    }
}

impl SecretStore for SystemCredentialStore {
    fn read_secret(&self, name: &str) -> Result<Option<String>, String> {
        match Self::credential_entry(name)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(format!("读取{}失败：{error}", store_label())),
        }
    }

    fn write_secret(&self, name: &str, secret: &str) -> Result<(), String> {
        Self::credential_entry(name)?
            .set_password(secret)
            .map_err(|error| format!("保存{}失败：{error}", store_label()))
    }

    fn delete_secret(&self, name: &str) -> Result<(), String> {
        match Self::credential_entry(name)?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!("删除{}失败：{error}", store_label())),
        }
    }
}

/// Shared secret-store handle: clones like an `Arc` while staying a concrete
/// (sized) type so it can flow through the core settings API.
#[derive(Clone)]
pub struct SharedSecretStore {
    inner: std::sync::Arc<dyn SecretStore + Send + Sync>,
}

impl SharedSecretStore {
    pub fn new(store: impl SecretStore + 'static) -> Self {
        Self {
            inner: std::sync::Arc::new(store),
        }
    }
}

impl SecretStore for SharedSecretStore {
    fn read_secret(&self, name: &str) -> Result<Option<String>, String> {
        self.inner.read_secret(name)
    }

    fn write_secret(&self, name: &str, secret: &str) -> Result<(), String> {
        self.inner.write_secret(name, secret)
    }

    fn delete_secret(&self, name: &str) -> Result<(), String> {
        self.inner.delete_secret(name)
    }
}

#[allow(dead_code)] // exercised by host tests via SharedSecretStore
#[derive(Debug, Default)]
pub struct InMemorySecretStore {
    secrets: Mutex<HashMap<String, String>>,
}

#[allow(dead_code)] // exercised by host tests via SharedSecretStore
impl InMemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.secrets
            .lock()
            .map(|secrets| secrets.is_empty())
            .unwrap_or(true)
    }
}

impl SecretStore for InMemorySecretStore {
    fn read_secret(&self, name: &str) -> Result<Option<String>, String> {
        let secrets = self
            .secrets
            .lock()
            .map_err(|_| "secret store lock is poisoned".to_string())?;
        Ok(secrets.get(name).cloned())
    }

    fn write_secret(&self, name: &str, secret: &str) -> Result<(), String> {
        let mut secrets = self
            .secrets
            .lock()
            .map_err(|_| "secret store lock is poisoned".to_string())?;
        secrets.insert(name.to_string(), secret.to_string());
        Ok(())
    }

    fn delete_secret(&self, name: &str) -> Result<(), String> {
        let mut secrets = self
            .secrets
            .lock()
            .map_err(|_| "secret store lock is poisoned".to_string())?;
        secrets.remove(name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemorySecretStore, SERVICE_NAME};
    use ameya_core::ai::settings::SecretStore;

    #[test]
    fn service_name_matches_tauri_contract() {
        assert_eq!(SERVICE_NAME, "ameya.ai");
    }

    #[test]
    fn in_memory_store_round_trips_secrets() {
        let store = InMemorySecretStore::new();

        assert_eq!(store.read_secret("ameya_openAiCompatibleApiKey"), Ok(None));

        store
            .write_secret("ameya_openAiCompatibleApiKey", "sk-live-1234")
            .expect("write succeeds");
        assert_eq!(
            store.read_secret("ameya_openAiCompatibleApiKey"),
            Ok(Some("sk-live-1234".into()))
        );

        store
            .delete_secret("ameya_openAiCompatibleApiKey")
            .expect("delete succeeds");
        assert_eq!(store.read_secret("ameya_openAiCompatibleApiKey"), Ok(None));
        assert!(store.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn windows_error_label_matches_tauri_contract() {
        // Tauri 逐字契约：错误文案使用「Windows 凭据」。
        assert_eq!(super::store_label(), "Windows 凭据");
    }
}
