use crate::nodes::{NodeExecutor};
use crate::nodes::core::{CoreInputNode, CoreOutputNode, CoreTransformNode, CoreMergeNode};
use aether_types::Node;
use uuid::Uuid;


pub struct CoreNodes;

impl CoreNodes {

    pub fn create_input_node(node: Node) -> Box<dyn NodeExecutor + Send + Sync> {
        Box::new(CoreInputNode::new(node))
    }


    pub fn create_output_node(node: Node) -> Box<dyn NodeExecutor + Send + Sync> {
        Box::new(CoreOutputNode::new(node))
    }


    pub fn create_transform_node(node: Node) -> Box<dyn NodeExecutor + Send + Sync> {
        Box::new(CoreTransformNode::new(node))
    }


    pub fn create_merge_node(node: Node) -> Box<dyn NodeExecutor + Send + Sync> {
        Box::new(CoreMergeNode::new(node))
    }


    pub fn get_supported_types() -> Vec<aether_types::NodeType> {
        vec![
            aether_types::NodeType::Input,
            aether_types::NodeType::Output,
            aether_types::NodeType::Transform,
            aether_types::NodeType::Merge,
        ]
    }


    pub fn is_supported(node_type: &aether_types::NodeType) -> bool {
        Self::get_supported_types().contains(node_type)
    }


    pub fn create_node_by_type(node_type: aether_types::NodeType, node: Node) -> Option<Box<dyn NodeExecutor + Send + Sync>> {
        match node_type {
            aether_types::NodeType::Input => Some(Self::create_input_node(node)),
            aether_types::NodeType::Output => Some(Self::create_output_node(node)),
            aether_types::NodeType::Transform => Some(Self::create_transform_node(node)),
            aether_types::NodeType::Merge => Some(Self::create_merge_node(node)),
            _ => None,
        }
    }


    pub fn register_core_nodes(registry: &mut crate::nodes::NodeRegistry) {
        for node_type in Self::get_supported_types() {
            registry.register_node_type(node_type, || {
                let node = aether_types::Node::new(node_type.clone(), "Core Node".to_string());
                Self::create_node_by_type(node_type.clone(), node).unwrap()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{NodeType, PinDataType, InputPin, OutputPin};

    #[test]
    fn test_core_nodes_creation() {

        let input_node = aether_types::Node::new(NodeType::Input, "Test Input".to_string());
        let core_input = CoreNodes::create_input_node(input_node);
        assert!(core_input.node_type() == NodeType::Input);


        let output_node = aether_types::Node::new(NodeType::Output, "Test Output".to_string());
        let core_output = CoreNodes::create_output_node(output_node);
        assert!(core_output.node_type() == NodeType::Output);


        let transform_node = aether_types::Node::new(NodeType::Transform, "Test Transform".to_string());
        let core_transform = CoreNodes::create_transform_node(transform_node);
        assert!(core_transform.node_type() == NodeType::Transform);


        let merge_node = aether_types::Node::new(NodeType::Merge, "Test Merge".to_string());
        let core_merge = CoreNodes::create_merge_node(merge_node);
        assert!(core_merge.node_type() == NodeType::Merge);
    }

    #[test]
    fn test_supported_types() {
        let supported_types = CoreNodes::get_supported_types();

        assert!(supported_types.contains(&NodeType::Input));
        assert!(supported_types.contains(&NodeType::Output));
        assert!(supported_types.contains(&NodeType::Transform));
        assert!(supported_types.contains(&NodeType::Merge));
        assert!(!supported_types.contains(&NodeType::ColorCorrection));
        assert!(!supported_types.contains(&NodeType::Blur));
    }

    #[test]
    fn test_is_supported() {
        assert!(CoreNodes::is_supported(&NodeType::Input));
        assert!(CoreNodes::is_supported(&NodeType::Output));
        assert!(CoreNodes::is_supported(&NodeType::Transform));
        assert!(CoreNodes::is_supported(&NodeType::Merge));
        assert!(!CoreNodes::is_supported(&NodeType::ColorCorrection));
        assert!(!CoreNodes::is_supported(&NodeType::Blur));
    }

    #[test]
    fn test_create_node_by_type() {

        let input_node = CoreNodes::create_node_by_type(
            NodeType::Input,
            aether_types::Node::new(NodeType::Input, "Test".to_string())
        );
        assert!(input_node.is_some());

        let output_node = CoreNodes::create_node_by_type(
            NodeType::Output,
            aether_types::Node::new(NodeType::Output, "Test".to_string())
        );
        assert!(output_node.is_some());


        let unsupported_node = CoreNodes::create_node_by_type(
            NodeType::ColorCorrection,
            aether_types::Node::new(NodeType::ColorCorrection, "Test".to_string())
        );
        assert!(unsupported_node.is_none());
    }
}
