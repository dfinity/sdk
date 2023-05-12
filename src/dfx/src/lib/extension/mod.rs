#![allow(dead_code)]

pub mod manager;
pub mod manifest;

use crate::lib::extension::{manager::ExtensionManager, manifest::ExtensionManifest};

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
    pub fn into_clap_command(self, manager: &ExtensionManager) -> Command {
        let mut cmd = Command::new(&self.name)
            .bin_name(&self.name)
            // by default, don't enforce any restrictions
            .allow_missing_positional(true)
            .allow_external_subcommands(true);
        let about = match ExtensionManifest::new(&self.name, &manager.dir) {
            Ok(manifest) => {
                let about = manifest.summary.clone();
                if let Some(subcmds) = manifest.into_clap_commands() {
                    // If the user declared subcommands in the manifest file, only allow
                    // subcommands and arguments specified in the manifest file, disallowing
                    // pass-through of any other values.
                    cmd = cmd.allow_external_subcommands(false).subcommands(subcmds);
                }
                about
            }
            Err(err) => err.to_string(),
        };
        cmd.about(about)
    }
}
