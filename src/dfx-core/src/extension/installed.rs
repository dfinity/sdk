use crate::error::extension::ConvertExtensionIntoClapCommandError;
use crate::extension::manifest::ExtensionManifest;
use crate::extension::ExtensionName;
use clap::Command;
use std::collections::HashMap;

pub type InstalledExtensionList = Vec<ExtensionName>;
pub struct InstalledExtensionManifests(pub HashMap<ExtensionName, ExtensionManifest>);

impl InstalledExtensionManifests {
    pub fn as_clap_commands(&self) -> Result<Vec<Command>, ConvertExtensionIntoClapCommandError> {
        let commands = self
            .0
            .values()
            .map(|manifest| {
                manifest.into_clap_commands().map(|subcommands| {
                    Command::new(&manifest.name)
                        .allow_missing_positional(false) // don't accept unknown options
                        .allow_external_subcommands(false) // don't accept unknown subcommands
                        .about(&manifest.summary)
                        .subcommands(subcommands)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(commands)
    }

    pub fn contains(&self, extension: &str) -> bool {
        self.0.contains_key(extension)
    }
}
