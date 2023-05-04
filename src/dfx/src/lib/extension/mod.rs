#![allow(dead_code)]

pub mod manager;
pub mod manifest;

use std::{
    fmt::{Display, Formatter},
    fs::DirEntry,
};

use clap::Command;

use self::{manager::ExtensionManager, manifest::ExtensionManifest};

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
        let summary = ExtensionManifest::new(&self.name, &manager.dir)
            .map_or_else(|e| e.to_string(), |v| v.summary);
        Command::new(&self.name).about(summary)
    }
}
