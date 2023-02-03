#![allow(dead_code)]

pub mod manager;
pub mod manifest;

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
