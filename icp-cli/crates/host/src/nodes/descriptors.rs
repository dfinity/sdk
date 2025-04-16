use crate::nodes::prettify::PrettifyNode;
use crate::nodes::print::PrintNode;
use crate::nodes::r#const::ConstNode;
use crate::registry::node_type::NodeDescriptor;

pub fn node_descriptors() -> Vec<NodeDescriptor> {
    vec![
        ConstNode::descriptor(),
        PrettifyNode::descriptor(),
        PrintNode::descriptor(),
    ]
}
