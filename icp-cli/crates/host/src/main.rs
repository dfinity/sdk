mod bindings;
mod command_config;
mod const_node;
mod host;
mod node;
mod node_state;
mod output_promise;
mod prettify;
mod print_node;
mod runtime;
mod value;

use crate::const_node::ConstNode;
use crate::print_node::PrintNode;
use crate::runtime::Runtime;
use crate::value::OutputValue;
use prettify::*;
use std::sync::Arc;
// #[tokio::main]
// async fn main() {
//     let runtime = GraphRuntime::new();
//
//     let const_node = Arc::new(ConstNode::new("message", "value", "Hello, world!"));
//     runtime.register_node(const_node.clone());
//
//     // Request the output
//     let output = runtime.get_output("message", "value").await.unwrap();
//
//     // Spawn evaluation if not already done
//     runtime.evaluate_node("message").await;
//
//     let value = output.get().await;
//     println!("ConstNode output: {:?}", value);
//
//     if let OutputValue::String(s) = value {
//         let mut prettify = Prettify::new("target/wasm32-wasip2/release/plugin.wasm").unwrap();
//         let r = prettify.prettify(&s).unwrap();
//         println!("{}", r);
//     }
// }

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let const_node = ConstNode::new(OutputValue::String("Hello, World!".to_string()));
    let print_node = PrintNode::new(const_node.output_promise());

    let mut runtime = Runtime::new();
    runtime.add_node(const_node);
    runtime.add_node(print_node);

    runtime.run_graph().await;
}
