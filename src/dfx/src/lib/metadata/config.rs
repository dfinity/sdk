use crate::config::dfinity::{CanisterMetadataSection, CanisterTypeProperties};
use crate::lib::metadata::names::CANDID_SERVICE;

use crate::config::dfinity::MetadataVisibility::Public;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct CanisterMetadataConfig {
    pub sections: BTreeMap<String, CanisterMetadataSection>,
}

impl CanisterMetadataConfig {
    pub fn new(
        type_properties: &CanisterTypeProperties,
        sections: &Vec<CanisterMetadataSection>,
        network: &str,
    ) -> Self {
        let mut map = BTreeMap::new();
        for section in sections {
            if section.applies_to_network(network) && !map.contains_key(&section.name) {
                map.insert(section.name.clone(), section.clone());
            }
        }

        let default_candid_service = matches!(
            type_properties,
            CanisterTypeProperties::Rust { .. } | CanisterTypeProperties::Motoko
        );
        if default_candid_service && !map.contains_key(CANDID_SERVICE) {
            map.insert(
                CANDID_SERVICE.to_string(),
                CanisterMetadataSection {
                    name: CANDID_SERVICE.to_string(),
                    visibility: Public,
                    networks: None,
                    path: None,
                },
            );
        }

        CanisterMetadataConfig { sections: map }
    }

    pub fn get(&self, name: &str) -> Option<&CanisterMetadataSection> {
        self.sections.get(name)
    }
}
