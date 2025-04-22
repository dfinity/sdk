use crate::workflow::nodes::prettify::PrettifyNode;
use crate::workflow::nodes::print::PrintNode;
use crate::workflow::nodes::scalar::string::ConstStringNode;
use crate::workflow::nodes::wasm::r#const::ConstWasmNode;
use crate::workflow::registry::node_type::NodeDescriptor;

pub fn node_descriptors() -> Vec<NodeDescriptor> {
    vec![
        ConstStringNode::descriptor(),
        ConstWasmNode::descriptor(),
        PrettifyNode::descriptor(),
        PrintNode::descriptor(),
    ]
}
