pub mod manager;
pub mod manifest;
use crate::error::extension::ConvertExtensionIntoClapCommandError;
use crate::extension::{manager::ExtensionManager, manifest::ExtensionManifest};
use clap::Command;
use std::{
    fmt::{Display, Formatter},
    fs::DirEntry,
};

#[derive(Debug, Default)]
pub struct Extension {
    pub name: String,
}

impl From<DirEntry> for Extension {
    fn from(entry: DirEntry) -> Self {
        let name = entry.file_name().to_string_lossy().to_string();
        Extension { name }
    }
}

impl Display for Extension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Extension {
    pub fn into_clap_command(
        self,
        manager: &ExtensionManager,
    ) -> Result<Command, ConvertExtensionIntoClapCommandError> {
        let manifest = ExtensionManifest::load(&self.name, &manager.dir)?;
        let cmd = Command::new(&self.name)
            // don't accept unknown options
            .allow_missing_positional(false)
            // don't accept unknown subcommands
            .allow_external_subcommands(false)
            .about(&manifest.summary)
            .subcommands(manifest.into_clap_commands()?);
        Ok(cmd)
    }
}
