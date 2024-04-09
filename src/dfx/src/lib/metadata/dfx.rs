//! A canister metadata with key "dfx"
//!
//! The cli tool dfx should consolidate its usage of canister metadata into this single section
//! It's originally for pulling dependencies. But open to extend for other usage.
use crate::lib::{builders::command_output, error::DfxResult};
use anyhow::{bail, Context};
use dfx_core::config::model::dfinity::{Pullable, TechStack, TechStackCategoryMap};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// # "dfx" metadata.
/// Standardized metadata for dfx usage.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DfxMetadata {
    /// # Pullable
    /// The required information so that the canister can be pulled using `dfx deps pull`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pullable: Option<Pullable>,

    /// # Tech Stack
    /// The tech stack information of the canister.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub tech_stack: Option<TechStack>,
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

    pub fn set_tech_stack(
        &mut self,
        tech_stack_config: &TechStack,
        project_root: &Path,
    ) -> DfxResult<()> {
        let mut tech_stack = tech_stack_config.clone();
        overwrite_field_from_command("cdk", tech_stack.cdk.as_mut(), project_root)?;
        overwrite_field_from_command("language", tech_stack.language.as_mut(), project_root)?;
        overwrite_field_from_command("lib", tech_stack.lib.as_mut(), project_root)?;
        overwrite_field_from_command("tool", tech_stack.tool.as_mut(), project_root)?;
        overwrite_field_from_command("other", tech_stack.other.as_mut(), project_root)?;
        self.tech_stack = Some(tech_stack);
        Ok(())
    }
}

// If the value of a field starts with "$(", and ends with ")", then it's a command to calculate the value.
// The field value will be overwritten by the output of the command.
fn overwrite_field_from_command(
    category: &str,
    category_map: Option<&mut TechStackCategoryMap>,
    project_root: &Path,
) -> DfxResult<()> {
    if let Some(category_map) = category_map {
        for (name, fields) in category_map.iter_mut() {
            for (field, value) in fields.iter_mut() {
                if value.starts_with("$(") && value.ends_with(')') {
                    let triple = format!("{}->{}->{}", category, name, field);
                    let command = &value[2..value.len() - 1];
                    let bytes = command_output(command, &[], project_root)
                        .with_context(|| format!("Failed to run the value_command: {}.", triple))?;
                    let calculated_value = String::from_utf8(bytes).with_context(|| {
                        format!(
                            "The value_command didn't return a valid UTF-8 string: {}.",
                            triple
                        )
                    })?;
                    *value = calculated_value.trim().to_string();
                }
            }
        }
    }
    Ok(())
}
