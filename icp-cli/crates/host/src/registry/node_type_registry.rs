use crate::registry::node_type::NodeType;
use std::collections::HashMap;

pub struct NodeTypeRegistry {
    types: HashMap<String, NodeType>,
}

impl NodeTypeRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    pub fn register0(&mut self, node_types: Vec<NodeType>) {
        for node_type in node_types {
            self.types.insert(node_type.name.clone(), node_type);
        }
    }

    pub fn register<I: IntoIterator<Item = NodeType>>(&mut self, node_types: I) {
        for node_type in node_types {
            self.types.insert(node_type.name.clone(), node_type);
        }
    }

    pub fn get(&self, name: &str) -> Option<&NodeType> {
        self.types.get(name)
    }
}
