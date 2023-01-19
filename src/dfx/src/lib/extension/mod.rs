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
    // type Error = <T as TryFrom<_>>::Error: Into<anyhow::Error>;

    // fn try_from(value: DirEntry) -> Result<Self, Self::Error> {

    // }
    fn from(entry: DirEntry) -> Self {
        let name = entry.file_name().to_string_lossy().to_string();
        Extension {
            name,
            ..Default::default()
        }
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
