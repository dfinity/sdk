//! A canister metadata with key "dfx"
//!
//! The cli tool dfx should consolidate its usage of canister metadata into this single section
//! It's originally for pulling dependencies. But open to extend for other usage.
use std::{collections::HashMap, path::Path};

use crate::lib::{builders::run_command, error::DfxResult};
use anyhow::{bail, Context};
use dfx_core::config::model::dfinity::{Pullable, TechStackCategory, TechStackConfigItem};
use serde::{Deserialize, Serialize};

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
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub tech_stack: HashMap<TechStackCategory, Vec<TechStackItem>>,
}

type TechStackItem = HashMap<String, String>;

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
        tech_stack_config: &HashMap<TechStackCategory, Vec<TechStackConfigItem>>,
        project_root: &Path,
    ) -> DfxResult<()> {
        for (category, config_items) in tech_stack_config {
            let mut items = vec![];
            for config_item in config_items {
                let mut fields = HashMap::new();
                fields.insert("name".to_string(), config_item.name.clone());
                for custom_field in &config_item.custom_fields {
                    let triple = format!(
                        "{:?}->{}->{}",
                        category, config_item.name, custom_field.field
                    );
                    let value = match (&custom_field.value, &custom_field.value_command) {
                        (Some(value), None) => value.to_string(),
                        (None, Some(command)) => {
                            let bytes = run_command(command, &[], project_root, false)
                                .with_context(|| {
                                    format!("Failed to run the value_command: {triple}.")
                                })?;
                            String::from_utf8(bytes)
                                .with_context(|| {
                                      format!("The value_command didn't return a valid UTF-8 string: {triple}.")
                                })?
                                .trim()
                                .to_string()
                        }
                        (_, _) => {
                            bail!("A custom_field should define only one of value/value_command: {triple}.")
                        }
                    };
                    fields.insert(custom_field.field.clone(), value);
                }
                items.push(fields);
            }
            self.tech_stack.insert(category.clone(), items);
        }

        Ok(())
    }
}
