use crate::workflow::nodes::node_descriptors;
use crate::workflow::registry::node_type::NodeDescriptor;
use std::collections::HashMap;

pub struct NodeTypeRegistry {
    types: HashMap<String, NodeDescriptor>,
}

impl NodeTypeRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    pub fn register<I: IntoIterator<Item = NodeDescriptor>>(&mut self, node_types: I) {
        for node_type in node_types {
            self.types.insert(node_type.name.clone(), node_type);
        }
    }

    pub fn get(&self, name: &str) -> Option<&NodeDescriptor> {
        self.types.get(name)
    }
}

impl Default for NodeTypeRegistry {
    fn default() -> Self {
        let mut registry = NodeTypeRegistry::new();
        registry.register(node_descriptors());
        registry
    }
}
