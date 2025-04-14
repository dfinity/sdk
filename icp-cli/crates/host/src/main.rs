mod bindings;
mod command_config;
mod const_node;
mod host;
mod prettify;
mod runtime;

use crate::const_node::ConstNode;
use crate::runtime::{GraphRuntime, OutputValue};
use prettify::*;
use std::sync::Arc;
// pub fn main() {
//     let mut prettify = Prettify::new("target/wasm32-wasip2/release/plugin.wasm").unwrap();
//     let r = prettify.prettify("We will prettify this with a plugin").unwrap();
//     println!("{}", r);
// }

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let runtime = GraphRuntime::new();

    let const_node = Arc::new(ConstNode::new("message", "value", "Hello, world!"));
    runtime.register_node(const_node.clone());

    // Request the output
    let output = runtime.get_output("message", "value").await.unwrap();

    // Spawn evaluation if not already done
    runtime.evaluate_node("message").await;

    let value = output.get().await;
    println!("ConstNode output: {:?}", value);

    if let OutputValue::String(s) = value {
        let mut prettify = Prettify::new("target/wasm32-wasip2/release/plugin.wasm").unwrap();
        let r = prettify.prettify(&s).unwrap();
        println!("{}", r);
    }
}
