//! A canister metadata with key "dfx"
//!
//! The cli tool dfx should consolidate its usage of canister metadata into this single section
//! It's originally for pulling dependencies. But open to extend for other usage.
use std::{collections::BTreeMap, path::Path};

use crate::lib::{builders::run_command, error::DfxResult};
use anyhow::{bail, Context};
use dfx_core::config::model::dfinity::{Pullable, CDK};
use serde::{Deserialize, Serialize};

/// "dfx" metadata.
/// Standardized metadata for dfx usage.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DfxMetadata {
    /// # Pullable
    /// The required information so that the canister can be pulled using `dfx deps pull`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pullable: Option<Pullable>,

    /// # CDK
    /// A map of the canister name to the cdk version.
    /// The cdk version is optional.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub cdk: BTreeMap<String, Option<String>>,
}

impl DfxMetadata {
    pub fn set_pullable(&mut self, pullable: Pullable) {
        self.pullable = Some(pullable);
    }

    pub fn get_pullable(&self) -> DfxResult<&Pullable> {
        match &self.pullable {
            Some(pullable) => Ok(pullable),
            None => bail!("The `dfx` metadata doesn't contain the `pullable` object."),
        }
    }

    pub fn add_cdk(&mut self, cdk_entry: &CDK, project_root: &Path) -> DfxResult<()> {
        let CDK {
            name,
            version,
            version_command,
        } = cdk_entry;
        if self.cdk.contains_key(name) {
            bail!(
                "The cdk with name `{}` is defined more than once in dfx.json.",
                name
            );
        }

        let version = match (version, version_command) {
            (Some(_), Some(_)) => {
                bail!("The cdk with name `{}` has both `version` and `version_command` defined. Please keep at most one of them.", name)
            }
            (Some(_), None) => version.clone(),
            (None, Some(command)) => {
                let output = run_command(command, &[], project_root, false).with_context(|| {
                    format!("Failed to run the version_command for cdk {}", name)
                })?;
                match output {
                    Some(bytes) => Some(
                        String::from_utf8(bytes)
                            .with_context(|| {
                                format!(
                            "The version_command for cdk `{}` didn't return a valid utf8 string.",
                            name
                        )
                            })?
                            .trim()
                            .to_string(),
                    ),
                    None => bail!(
                        "The version_command for cdk `{}` didn't return a version.",
                        name
                    ),
                }
            }
            (None, None) => None,
        };

        self.cdk.insert(name.clone(), version);

        Ok(())
    }
}
