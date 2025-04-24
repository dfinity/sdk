use crate::workflow::execute::promise::{AnyPromise, Input, InputRef, Output, OutputRef};
use crate::workflow::payload::wasm::Wasm;
use crate::workflow::registry::error::StringSourceError;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NodeConfig {
    pub inputs: HashMap<String, AnyPromise>,
    pub outputs: HashMap<String, AnyPromise>,
}

impl NodeConfig {
    pub fn get_input(&self, name: &str) -> Result<&AnyPromise, StringSourceError> {
        self.inputs
            .get(name)
            .ok_or_else(|| StringSourceError::MissingInput(name.to_string()))
    }

    pub fn string_input(&self, name: &str) -> Result<InputRef<String>, StringSourceError> {
        let input = self.get_input(name)?.string_input()?;
        Ok(input)
    }

    pub fn string_output(&self, name: &str) -> OutputRef<String> {
        self.outputs
            .get("output")
            .expect("missing 'output' output")
            .string_output()
            .expect("type mismatch for 'output' output")
    }

    pub fn wasm_output(&self, name: &str) -> OutputRef<Wasm> {
        self.outputs
            .get("output")
            .expect("missing 'output' output")
            .wasm_output()
            .expect("type mismatch for 'output' output")
    }
}
