use crate::ai::settings::SecretStore;

const SERVICE_NAME: &str = "ameya.ai";

#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsCredentialStore;

impl SecretStore for WindowsCredentialStore {
    fn read_secret(&self, name: &str) -> Result<Option<String>, String> {
        credential_entry(name)?
            .get_password()
            .map(Some)
            .or_else(|error| {
                if is_missing_credential(&error) {
                    Ok(None)
                } else {
                    Err(format!("读取 Windows 凭据失败：{error}"))
                }
            })
    }

    fn write_secret(&self, name: &str, secret: &str) -> Result<(), String> {
        credential_entry(name)?
            .set_password(secret)
            .map_err(|error| format!("保存 Windows 凭据失败：{error}"))
    }

    fn delete_secret(&self, name: &str) -> Result<(), String> {
        credential_entry(name)?
            .delete_credential()
            .or_else(|error| {
                if is_missing_credential(&error) {
                    Ok(())
                } else {
                    Err(error)
                }
            })
            .map_err(|error| format!("删除 Windows 凭据失败：{error}"))
    }
}

fn credential_entry(name: &str) -> Result<keyring_core::Entry, String> {
    ensure_windows_store()?;
    keyring_core::Entry::new(SERVICE_NAME, name)
        .map_err(|error| format!("创建 Windows 凭据项失败：{error}"))
}

fn ensure_windows_store() -> Result<(), String> {
    if keyring_core::get_default_store().is_none() {
        let store = windows_native_keyring_store::Store::new()
            .map_err(|error| format!("初始化 Windows Credential Manager 失败：{error}"))?;
        keyring_core::set_default_store(store);
    }
    Ok(())
}

fn is_missing_credential(error: &keyring_core::Error) -> bool {
    matches!(error, keyring_core::Error::NoEntry)
}
