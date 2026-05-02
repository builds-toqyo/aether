use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, PinDataType, ParameterValue};
use std::collections::HashMap;
use uuid::Uuid;
use log::debug;

/// Core input node - provides media source input
#[derive(Debug)]
pub struct CoreInputNode {
    node: Node,
    media_path: Option<String>,
    frame_cache: HashMap<u64, ParameterValue>,
}

impl CoreInputNode {
    /// Create a new core input node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            media_path: None,
            frame_cache: HashMap::new(),
        }
    }
    
    /// Set the media path
    pub fn set_media_path(&mut self, path: String) {
        self.media_path = Some(path);
    }
    
    /// Get the media path
    pub fn get_media_path(&self) -> Option<&String> {
        self.media_path.as_ref()
    }
    
    /// Clear the frame cache
    pub fn clear_cache(&mut self) {
        self.frame_cache.clear();
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.frame_cache.len()
    }
    
    /// Generate a frame for the given frame number
    pub fn generate_frame(&mut self, frame: u64) -> ParameterValue {
        // Check cache first
        if let Some(cached_frame) = self.frame_cache.get(&frame) {
            return cached_frame.clone();
        }
        
        // Generate frame (in real implementation, this would load from media)
        let frame_data = self.load_media_frame(frame);
        
        // Cache the result
        self.frame_cache.insert(frame, frame_data.clone());
        
        frame_data
    }
    
    /// Load media frame (placeholder implementation)
    fn load_media_frame(&self, frame: u64) -> ParameterValue {
        debug!("Loading media frame {} from path: {:?}", frame, self.media_path);
        
        // In real implementation, this would:
        // - Load media file from self.media_path
        // - Seek to frame position
        // - Decode frame data
        // - Return frame data as ParameterValue
        
        // Placeholder: return a test image
        ParameterValue::Image(Uuid::new_v4())
    }
}

impl NodeExecutor for CoreInputNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Generate frame for current context frame
        let frame_data = self.generate_frame(context.frame);
        
        // Set output value
        if let Some(output_pin) = self.node.outputs.first() {
            context.set_output(output_pin.id, frame_data);
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        &[]
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        &self.node.outputs.iter().map(|pin| pin.id).collect::<Vec<_>>()
    }
    
    fn can_execute(&self, _context: &ExecutionContext) -> bool {
        self.node.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{OutputPin};
    
    #[test]
    fn test_core_input_node_creation() {
        let mut node = Node::new(NodeType::Input, "Test Input".to_string());
        
        // Add output pin
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);
        
        let input_node = CoreInputNode::new(node);
        
        assert_eq!(input_node.get_media_path(), None);
        assert_eq!(input_node.cache_size(), 0);
        assert_eq!(input_node.node_type(), NodeType::Input);
    }
    
    #[test]
    fn test_media_path_operations() {
        let node = Node::new(NodeType::Input, "Test".to_string());
        let mut input_node = CoreInputNode::new(node);
        
        // Test setting media path
        input_node.set_media_path("/path/to/media.mp4".to_string());
        assert_eq!(input_node.get_media_path(), Some(&"/path/to/media.mp4".to_string()));
        
        // Test clearing media path
        input_node.set_media_path("".to_string());
        assert_eq!(input_node.get_media_path(), Some(&"".to_string()));
    }
    
    #[test]
    fn test_cache_operations() {
        let node = Node::new(NodeType::Input, "Test".to_string());
        let mut input_node = CoreInputNode::new(node);
        
        // Test initial cache state
        assert_eq!(input_node.cache_size(), 0);
        
        // Test cache clearing
        input_node.clear_cache();
        assert_eq!(input_node.cache_size(), 0);
        
        // Test frame generation (should add to cache)
        let frame_data = input_node.generate_frame(1);
        assert_eq!(input_node.cache_size(), 1);
        assert!(matches!(frame_data, ParameterValue::Image(_)));
        
        // Test cache hit (should not increase cache size)
        let frame_data2 = input_node.generate_frame(1);
        assert_eq!(input_node.cache_size(), 1);
        assert_eq!(frame_data, frame_data2);
        
        // Test cache miss (should increase cache size)
        input_node.generate_frame(2);
        assert_eq!(input_node.cache_size(), 2);
    }
    
    #[test]
    fn test_node_executor_interface() {
        let mut node = Node::new(NodeType::Input, "Test Input".to_string());
        
        // Add output pin
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);
        
        let mut input_node = CoreInputNode::new(node);
        
        // Test node type
        assert_eq!(input_node.node_type(), NodeType::Input);
        
        // Test inputs and outputs
        assert!(input_node.get_inputs().is_empty());
        assert_eq!(input_node.get_outputs().len(), 1);
        
        // Test can execute
        let context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };
        assert!(input_node.can_execute(&context));
        
        // Test execution
        let mut context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };
        
        let result = input_node.execute(&mut context);
        assert!(result.is_ok());
        assert_eq!(input_node.cache_size(), 1);
    }
}
