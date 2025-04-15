use crate::node::Node;
mod bindings;
mod command_config;
mod graph;
mod host;
mod node;
mod node_state;
mod nodes;
mod output_promise;
mod prettify;
mod registry;
mod runtime;
mod tests;
mod value;
mod workflow;

use crate::nodes::node_types;
use crate::registry::node_type_registry::NodeTypeRegistry;
use crate::runtime::Runtime;
use crate::value::OutputValue;
use nodes::const_node::ConstNode;
use nodes::print_node::PrintNode;
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
    let workflow = r#"
nodes:
    const:
        value: Hello, test!
    print1:
        type: print
        inputs:
            input: const.output
    print2:
        type: print
        inputs:
            input: const.output
"#;
    let workflow: workflow::Workflow = workflow::Workflow::from_string(workflow);

    let mut registry = NodeTypeRegistry::new();
    registry.register(node_types());
    let nodes = graph::build_graph(workflow, &registry);
    let mut runtime = Runtime::new();
    for node in nodes {
        runtime.add_node(node);
    }
    runtime.run_graph().await;
}
