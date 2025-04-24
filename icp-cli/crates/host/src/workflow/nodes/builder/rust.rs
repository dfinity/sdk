use crate::workflow::execute::execute::{Execute, SharedExecuteResult};
use crate::workflow::execute::promise::{InputRef, OutputRef};
use crate::workflow::payload::wasm::Wasm;
use crate::workflow::registry::edge::EdgeType;
use crate::workflow::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RustBuilderNode {
    package: InputRef<String>,
    output: OutputRef<Wasm>,
}

#[async_trait]
impl Execute for RustBuilderNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        let package = self.package.get().await?;
        eprintln!("Building Rust package: {}", package);
        // self.output.set(self.value.clone());
        Ok(())
    }
}

impl RustBuilderNode {
    pub fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            name: "rust-builder".to_string(),
            inputs: HashMap::from([("package".to_string(), EdgeType::String)]),
            outputs: HashMap::from([("output".to_string(), EdgeType::Wasm)]),
            produces_side_effect: true,
            constructor: Box::new(|config| {
                let package = config.string_input("package")?;
                let output = config.wasm_output("output");
                Ok(Arc::new(RustBuilderNode { package, output }))
            }),
        }
    }
}
