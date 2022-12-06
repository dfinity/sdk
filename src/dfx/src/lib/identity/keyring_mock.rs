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
pub const USE_KEYRING_MOCK_ENV_VAR: &str = "DFX_CI_MOCK_KEYRING_LOCATION";
fn keyring_identity_name_from_suffix(suffix: &str) -> String {
    format!("{}{}", KEYRING_IDENTITY_PREFIX, suffix)
}

enum KeyringMockMode {
    /// Use system keyring
    NoMock,
    /// Simulate keyring where access is granted
    MockAvailable,
    /// Simulate keyring where access is rejected
    MockReject,
}

impl KeyringMockMode {
    fn current_mode() -> Self {
        match std::env::var(USE_KEYRING_MOCK_ENV_VAR) {
            Err(_) => Self::NoMock,
            Ok(location) => match location.as_str() {
                "" => Self::MockReject,
                _ => Self::MockAvailable,
            },
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct KeyringMock {
    pub kv_store: HashMap<String, String>,
}

impl KeyringMock {
    fn get_location() -> DfxResult<PathBuf> {
        match std::env::var(USE_KEYRING_MOCK_ENV_VAR) {
            Ok(filename) => match filename.as_str() {
                "" => bail!("Mock keyring unavailable - access rejected."),
                _ => Ok(PathBuf::from(filename)),
            },
            _ => bail!("Mock keyring unavailable."),
        }
    }

    #[context("Failed to load mock keyring.")]
    pub fn load() -> DfxResult<Self> {
        let location = Self::get_location()?;
        if location.exists() {
            let serialized_mock = std::fs::read(&location).with_context(|| {
                format!(
                    "Failed to read existing keyring mock at {}",
                    location.to_string_lossy()
                )
            })?;
            let mock = serde_json::from_slice(serialized_mock.as_slice())
                .context("Failed to deserialize mock keyring.")?;
            Ok(mock)
        } else {
            Ok(Self::default())
        }
    }

    #[context("Failed to load mock keyring.")]
    pub fn save(&self) -> DfxResult {
        let location = Self::get_location()?;
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize mock keyring")?;
        std::fs::write(&location, content).with_context(|| {
            format!(
                "Failed to save mock keyring to {}",
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
    match KeyringMockMode::current_mode() {
        KeyringMockMode::NoMock => {
            let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &keyring_identity_name);
            let encoded_pem = entry.get_password()?;
            let pem = hex::decode(&encoded_pem)?;
            Ok(pem)
        }
        KeyringMockMode::MockAvailable => {
            let mock = KeyringMock::load()?;
            let encoded_pem = mock.kv_store.get(&keyring_identity_name).with_context(|| {
                format!("Mock Keyring: key {} not found", &keyring_identity_name)
            })?;
            let pem = hex::decode(encoded_pem)?;
            Ok(pem)
        }
        KeyringMockMode::MockReject => bail!("Mock Keyring not available."),
    }
}

#[context(
    "Failed to write PEM file to keyring for identity '{}'.",
    identity_name_suffix
)]
pub fn write_pem_to_keyring(identity_name_suffix: &str, pem_content: &[u8]) -> DfxResult<()> {
    let keyring_identity_name = keyring_identity_name_from_suffix(identity_name_suffix);
    let encoded_pem = hex::encode(pem_content);
    match KeyringMockMode::current_mode() {
        KeyringMockMode::NoMock => {
            let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &keyring_identity_name);
            entry.set_password(&encoded_pem)?;
            Ok(())
        }
        KeyringMockMode::MockAvailable => {
            let mut mock = KeyringMock::load()?;
            mock.kv_store.insert(keyring_identity_name, encoded_pem);
            mock.save()?;
            Ok(())
        }
        KeyringMockMode::MockReject => bail!("Mock Keyring not available."),
    }
}

/// Determines if keyring is available by trying to write a dummy entry.
pub fn keyring_available(log: &Logger) -> bool {
    match KeyringMockMode::current_mode() {
        KeyringMockMode::NoMock => {
            trace!(log, "Checking for keyring availability.");
            // by using the temp identity prefix this will not clash with real identities since that would be an invalid identity name
            let dummy_entry_name = format!(
                "{}{}{}",
                KEYRING_IDENTITY_PREFIX, TEMP_IDENTITY_PREFIX, "dummy"
            );
            let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &dummy_entry_name);
            entry.set_password("dummy entry").is_ok()
        }
        KeyringMockMode::MockReject => false,
        KeyringMockMode::MockAvailable => true,
    }
}

pub fn delete_pem_from_keyring(identity_name_suffix: &str) -> DfxResult {
    let keyring_identity_name = keyring_identity_name_from_suffix(identity_name_suffix);
    match KeyringMockMode::current_mode() {
        KeyringMockMode::NoMock => {
            let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &keyring_identity_name);
            if entry.get_password().is_ok() {
                entry.delete_password()?;
            }
        }
        KeyringMockMode::MockAvailable => {
            let mut mock = KeyringMock::load()?;
            mock.kv_store.remove(&keyring_identity_name);
            mock.save()?;
        }
        KeyringMockMode::MockReject => bail!("Mock Keyring not available."),
    }
    Ok(())
}
