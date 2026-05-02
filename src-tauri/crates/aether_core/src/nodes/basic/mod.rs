pub mod input;
pub mod output;
pub mod merge;
pub mod transform;
pub mod color_correction;
pub mod blur;

pub use input::*;
pub use output::*;
pub use merge::*;
pub use transform::*;
pub use color_correction::*;
pub use blur::*;

use crate::nodes::NodeExecutor;
use aether_types::{Node, NodeType};
use uuid::Uuid;

/// Factory for creating all basic node types
pub struct BasicNodeFactory;

impl BasicNodeFactory {
    /// Create a basic node based on the node type
    pub fn create_basic_node(node_type: NodeType, node: Node) -> crate::nodes::NodeResult<Box<dyn NodeExecutor + Send + Sync>> {
        match node_type {
            NodeType::Input => Ok(InputNode::new(node)),
            NodeType::Output => Ok(OutputNode::new(node)),
            NodeType::Merge => Ok(MergeNode::new(node)),
            NodeType::Transform => Ok(TransformNode::new(node)),
            NodeType::ColorCorrection => Ok(ColorCorrectionNode::new(node)),
            NodeType::Blur => Ok(BlurNode::new(node)),
            _ => Err(crate::nodes::NodeError::ExecutionFailed(format!("Unsupported basic node type: {:?}", node_type))),
        }
    }
    
    /// Register all basic node types with a registry
    pub fn register_basic_nodes(registry: &mut crate::nodes::NodeRegistry) {
        registry.register_node_type(NodeType::Input, || {
            let node = aether_types::Node::new(NodeType::Input, "Input".to_string());
            Self::create_basic_node(NodeType::Input, node).unwrap()
        });
        
        registry.register_node_type(NodeType::Output, || {
            let node = aether_types::Node::new(NodeType::Output, "Output".to_string());
            Self::create_basic_node(NodeType::Output, node).unwrap()
        });
        
        registry.register_node_type(NodeType::Merge, || {
            let node = aether_types::Node::new(NodeType::Merge, "Merge".to_string());
            Self::create_basic_node(NodeType::Merge, node).unwrap()
        });
        
        registry.register_node_type(NodeType::Transform, || {
            let node = aether_types::Node::new(NodeType::Transform, "Transform".to_string());
            Self::create_basic_node(NodeType::Transform, node).unwrap()
        });
        
        registry.register_node_type(NodeType::ColorCorrection, || {
            let node = aether_types::Node::new(NodeType::ColorCorrection, "Color Correction".to_string());
            Self::create_basic_node(NodeType::ColorCorrection, node).unwrap()
        });
        
        registry.register_node_type(NodeType::Blur, || {
            let node = aether_types::Node::new(NodeType::Blur, "Blur".to_string());
            Self::create_basic_node(NodeType::Blur, node).unwrap()
        });
    }
    
    /// Get all supported basic node types
    pub fn get_supported_types() -> Vec<NodeType> {
        vec![
            NodeType::Input,
            NodeType::Output,
            NodeType::Merge,
            NodeType::Transform,
            NodeType::ColorCorrection,
            NodeType::Blur,
        ]
    }
}

/// Helper function to create a basic node with standard pins
pub fn create_basic_node_with_pins(
    node_type: NodeType,
    name: String,
    input_count: usize,
    output_count: usize,
) -> Node {
    let mut node = Node::new(node_type, name);
    
    // Add input pins
    for i in 0..input_count {
        let input_pin = aether_types::InputPin {
            id: Uuid::new_v4(),
            name: format!("input_{}", i),
            data_type: aether_types::PinDataType::Image,
            required: i == 0, // First input is required
            default_value: aether_types::ParameterValue::None,
            current_value: aether_types::ParameterValue::None,
            connection: None,
        };
        node.add_input(input_pin);
    }
    
    // Add output pins
    for i in 0..output_count {
        let output_pin = aether_types::OutputPin {
            id: Uuid::new_v4(),
            name: format!("output_{}", i),
            data_type: aether_types::PinDataType::Image,
            value: aether_types::ParameterValue::None,
        };
        node.add_output(output_pin);
    }
    
    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::NodeRegistry;
    
    #[test]
    fn test_basic_node_factory() {
        let mut registry = NodeRegistry::new();
        BasicNodeFactory::register_basic_nodes(&mut registry);
        
        // Test that all basic node types are registered
        let supported_types = BasicNodeFactory::get_supported_types();
        for node_type in supported_types {
            assert!(registry.create_node(&node_type).is_ok());
        }
    }
    
    #[test]
    fn test_create_basic_node_with_pins() {
        let node = create_basic_node_with_pins(
            NodeType::Transform,
            "Test Transform".to_string(),
            1,
            1,
        );
        
        assert_eq!(node.node_type, NodeType::Transform);
        assert_eq!(node.name, "Test Transform");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert!(node.inputs[0].required);
    }
}
