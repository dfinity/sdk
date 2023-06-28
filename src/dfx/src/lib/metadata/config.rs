use dfx_core::config::model::dfinity::CanisterMetadataSection;

use std::collections::BTreeMap;

#[derive(Debug)]
pub struct CanisterMetadataConfig {
    pub sections: BTreeMap<String, CanisterMetadataSection>,
}

impl CanisterMetadataConfig {
    pub fn new(sections: &Vec<CanisterMetadataSection>, network: &str) -> Self {
        let mut map = BTreeMap::new();
        for section in sections {
            if section.applies_to_network(network) && !map.contains_key(&section.name) {
                map.insert(section.name.clone(), section.clone());
            }
        }

        CanisterMetadataConfig { sections: map }
    }

    pub fn get(&self, name: &str) -> Option<&CanisterMetadataSection> {
        self.sections.get(name)
    }
}
