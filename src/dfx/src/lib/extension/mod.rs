// use std::collections::HashMap;
use std::fs::DirEntry;

use std::fmt::{Display,Formatter};
pub mod manager;
pub mod manifest;

pub struct Extension {
    pub name: String,
    // pub version: String,
    // pub description: String,
    // pub author: String,
    // pub license: String,
    // pub homepage: String,
    // pub repository: String,
    // pub dependencies: Vec<String>,
    // pub files: Vec<String>,
    // pub scripts: HashMap<String, String>,
}

impl From<DirEntry> for Extension {
    fn from(entry: DirEntry) -> Self {
        let name = entry.file_name().into_string().unwrap();
        Extension { name }
    }
}

impl Display for Extension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

// impl Extension {
//     pub fn from_json_metadata_file(file: File) -> Self {
//         Extension { name }
//     }
// }
