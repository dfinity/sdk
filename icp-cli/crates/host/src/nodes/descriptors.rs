use crate::nodes::prettify::PrettifyNode;
use crate::nodes::print::PrintNode;
use crate::nodes::scalar::string::ConstStringNode;
use crate::registry::node_type::NodeDescriptor;

pub fn node_descriptors() -> Vec<NodeDescriptor> {
    vec![
        ConstStringNode::descriptor(),
        PrettifyNode::descriptor(),
        PrintNode::descriptor(),
    ]
}
