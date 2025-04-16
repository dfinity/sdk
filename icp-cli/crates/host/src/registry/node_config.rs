use crate::execution::promise::{AnyPromise, Promise};
use std::collections::HashMap;
use std::sync::Arc;

pub struct NodeConfig {
    pub params: HashMap<String, String>,
    pub inputs: HashMap<String, AnyPromise>,
    pub outputs: HashMap<String, AnyPromise>,
}

impl NodeConfig {
    pub fn string_param(&self, name: &str) -> String {
        self.params
            .get(name)
            .expect("missing parameter")
            .clone()
    }
    pub fn string_input(&self, name: &str) -> Arc<Promise<String>> {
        self.inputs
            .get(name)
            .expect("missing input")
            .string()
            .expect("type mismatch for input")
    }

    pub fn string_output(&self, name: &str) -> Arc<Promise<String>> {
        self.outputs
            .get("output")
            .expect("missing 'output' output")
            .string()
            .expect("type mismatch for 'output' output")
    }
}
