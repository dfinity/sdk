use crate::execute::execute::{Execute, SharedExecuteResult};
use crate::execute::promise::OutputRef;
use crate::payload::wasm::Wasm;
use crate::registry::edge::EdgeType;
use crate::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ConstWasmNode {
    value: Wasm,
    output: OutputRef<Wasm>,
}

#[async_trait]
impl Execute for ConstWasmNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        println!("ConstWasmNode producing {} bytes", self.value.0.len());
        self.output.set(self.value.clone());
        Ok(())
    }
}

impl ConstWasmNode {
    pub fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            name: "const-wasm".to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::from([("output".to_string(), EdgeType::Wasm)]),
            produces_side_effect: false,
            constructor: Box::new(|config| {
                let hex_string = config.string_param("value");
                let wasm_bytes =
                    hex::decode(&hex_string).expect("invalid hex string for wasm module");
                let output = config.wasm_output("output");
                Ok(Arc::new(ConstWasmNode {
                    value: Wasm(wasm_bytes),
                    output,
                }))
            }),
        }
    }
}
