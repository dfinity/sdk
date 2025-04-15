use crate::nodes::const_node::ConstNode;
use crate::nodes::print_node::PrintNode;
use crate::registry::node_type::NodeDescriptor;

pub fn node_types() -> Vec<NodeDescriptor> {
    vec![ConstNode::node_type(), PrintNode::node_type()]
}
