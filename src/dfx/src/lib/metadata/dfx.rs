//! A canister metadata with key "dfx"
//!
//! The cli tool dfx should consolidate its usage of canister metadata into this single section
//! It's originally for pulling dependencies. But open to extend for other usage.
use std::{collections::BTreeMap, path::Path};

use crate::lib::{builders::run_command, error::DfxResult};
use anyhow::{bail, Context};
use dfx_core::config::model::dfinity::{Pullable, TechStackItem};
use serde::{Deserialize, Serialize};

/// "dfx" metadata.
/// Standardized metadata for dfx usage.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DfxMetadata {
    /// # Pullable
    /// The required information so that the canister can be pulled using `dfx deps pull`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pullable: Option<Pullable>,

    /// # Tech Stack
    /// A map of the canister name to the tech_stack item version.
    /// The tech_stack item version is optional.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tech_stack: BTreeMap<String, Option<String>>,
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

    pub fn add_tech_stack_item(
        &mut self,
        tech_stack_item: &TechStackItem,
        project_root: &Path,
    ) -> DfxResult<()> {
        let TechStackItem {
            name,
            version,
            version_command,
        } = tech_stack_item;
        if self.tech_stack.contains_key(name) {
            bail!(
                "The tech_stack item with name \"{}\" is defined more than once in dfx.json.",
                name
            );
        }

        let version = match (version, version_command) {
            (Some(_), Some(_)) => {
                bail!("The tech_stack item with name \"{}\" defines both \"version\" and \"version_command\" defined. Please keep at most one of them.", name)
            }
            (Some(_), None) => version.clone(),
            (None, Some(command)) => {
                let bytes = run_command(command, &[], project_root, false).with_context(|| {
                    format!(
                        "Failed to run the \"version_command\" of tech_stack item \"{}\".",
                        name
                    )
                })?;
                Some(
                    String::from_utf8(bytes)
                        .with_context(|| {
                            format!(
                        "The \"version_command\" of tech_stack item \"{}\" didn't return a valid UTF-8 string.",
                        name
                    )
                        })?
                        .trim()
                        .to_string(),
                )
            }
            (None, None) => None,
        };

        self.tech_stack.insert(name.clone(), version);

        Ok(())
    }
}
