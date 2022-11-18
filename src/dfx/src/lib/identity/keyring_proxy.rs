use std::{collections::HashMap, path::PathBuf};

use crate::lib::error::DfxResult;

use super::TEMP_IDENTITY_PREFIX;
use anyhow::{bail, Context};
use fn_error_context::context;
use keyring;
use serde::{Deserialize, Serialize};
use slog::{trace, Logger};

pub const KEYRING_SERVICE_NAME: &str = "internet_computer_identities";
pub const KEYRING_IDENTITY_PREFIX: &str = "internet_computer_identity_";
pub const USE_KEYRING_PROXY_ENV_VAR: &str = "DFX_CI_USE_PROXY_KEYRING";
fn keyring_identity_name_from_suffix(suffix: &str) -> String {
    format!("{}{}", KEYRING_IDENTITY_PREFIX, suffix)
}

enum KeyringProxyMode {
    /// Use system keyring
    NoProxy,
    /// Simulate keyring where access is granted
    ProxyAvailable,
    /// Simulate keyring where access is rejected
    ProxyReject,
}

impl KeyringProxyMode {
    fn current_mode() -> Self {
        match std::env::var(USE_KEYRING_PROXY_ENV_VAR) {
            Err(_) => Self::NoProxy,
            Ok(mode) => match mode.as_str() {
                "available" => Self::ProxyAvailable,
                _ => Self::ProxyReject,
            },
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct KeyringProxy {
    pub kv_store: HashMap<String, String>,
}

impl KeyringProxy {
    fn get_location() -> DfxResult<PathBuf> {
        Ok(PathBuf::from(std::env::var("HOME")?).join("mock_keyring.json"))
    }

    #[context("Failed to load proxy keyring.")]
    pub fn load() -> DfxResult<Self> {
        let location = Self::get_location()?;
        if location.exists() {
            let serialized_proxy = std::fs::read(&location).with_context(|| {
                format!(
                    "Failed to read existing keyring proxy at {}",
                    location.to_string_lossy()
                )
            })?;
            let proxy = serde_json::from_slice(serialized_proxy.as_slice())
                .context("Failed to deserialize proxy keyring.")?;
            Ok(proxy)
        } else {
            Ok(Self::default())
        }
    }

    #[context("Failed to load proxy keyring.")]
    pub fn save(&self) -> DfxResult {
        let location = Self::get_location()?;
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize proxy keyring")?;
        std::fs::write(&location, content).with_context(|| {
            format!(
                "Failed to save proxy keyring to {}",
                location.to_string_lossy()
            )
        })
    }
}

#[context(
    "Failed to load PEM file from keyring for identity '{}'.",
    identity_name_suffix
)]
pub fn load_pem_from_keyring(identity_name_suffix: &str) -> DfxResult<Vec<u8>> {
    let keyring_identity_name = keyring_identity_name_from_suffix(identity_name_suffix);
    match KeyringProxyMode::current_mode() {
        KeyringProxyMode::NoProxy => {
            let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &keyring_identity_name);
            let encoded_pem = entry.get_password()?;
            let pem = hex::decode(&encoded_pem)?;
            Ok(pem)
        }
        KeyringProxyMode::ProxyAvailable => {
            let proxy = KeyringProxy::load()?;
            let encoded_pem = proxy
                .kv_store
                .get(&keyring_identity_name)
                .with_context(|| {
                    format!("Proxy Keyring: key {} not found", &keyring_identity_name)
                })?;
            let pem = hex::decode(&encoded_pem)?;
            Ok(pem)
        }
        KeyringProxyMode::ProxyReject => bail!("Proxy Keyring not available."),
    }
}

#[context(
    "Failed to write PEM file to keyring for identity '{}'.",
    identity_name_suffix
)]
pub fn write_pem_to_keyring(identity_name_suffix: &str, pem_content: &[u8]) -> DfxResult<()> {
    let keyring_identity_name = keyring_identity_name_from_suffix(identity_name_suffix);
    let encoded_pem = hex::encode(pem_content);
    match KeyringProxyMode::current_mode() {
        KeyringProxyMode::NoProxy => {
            let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &keyring_identity_name);
            entry.set_password(&encoded_pem)?;
            Ok(())
        }
        KeyringProxyMode::ProxyAvailable => {
            let mut proxy = KeyringProxy::load()?;
            proxy.kv_store.insert(keyring_identity_name, encoded_pem);
            proxy.save()?;
            Ok(())
        }
        KeyringProxyMode::ProxyReject => bail!("Proxy Keyring not available."),
    }
}

/// Determines if keyring is available by trying to write a dummy entry.
pub fn keyring_available(log: &Logger) -> bool {
    match KeyringProxyMode::current_mode() {
        KeyringProxyMode::NoProxy => {
            trace!(log, "Checking for keyring availability.");
            // by using the temp identity prefix this will not clash with real identities since that would be an invalid identity name
            let dummy_entry_name = format!(
                "{}{}{}",
                KEYRING_IDENTITY_PREFIX, TEMP_IDENTITY_PREFIX, "dummy"
            );
            let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &dummy_entry_name);
            entry.set_password("dummy entry").is_ok()
        }
        KeyringProxyMode::ProxyReject => false,
        KeyringProxyMode::ProxyAvailable => true,
    }
}

pub fn delete_pem_from_keyring(identity_name_suffix: &str) -> DfxResult {
    let keyring_identity_name = keyring_identity_name_from_suffix(identity_name_suffix);
    match KeyringProxyMode::current_mode() {
        KeyringProxyMode::NoProxy => {
            let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &keyring_identity_name);
            if entry.get_password().is_ok() {
                entry.delete_password()?;
            }
        }
        KeyringProxyMode::ProxyAvailable => {
            let mut proxy = KeyringProxy::load()?;
            proxy.kv_store.remove(&keyring_identity_name);
            proxy.save()?;
        }
        KeyringProxyMode::ProxyReject => bail!("Proxy Keyring not available."),
    }
    Ok(())
}
