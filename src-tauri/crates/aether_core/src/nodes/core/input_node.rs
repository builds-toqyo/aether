use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue};
use std::collections::HashMap;
use uuid::Uuid;
use log::debug;

#[derive(Debug)]
pub struct CoreInputNode {
    node: Node,
    media_path: Option<String>,
    frame_cache: HashMap<u64, ParameterValue>,
}

impl CoreInputNode {

    pub fn new(node: Node) -> Self {
        Self {
            node,
            media_path: None,
            frame_cache: HashMap::new(),
        }
    }

    pub fn set_media_path(&mut self, path: String) {
        self.media_path = Some(path);
    }

    pub fn get_media_path(&self) -> Option<&String> {
        self.media_path.as_ref()
    }


    pub fn clear_cache(&mut self) {
        self.frame_cache.clear();
    }


    pub fn cache_size(&self) -> usize {
        self.frame_cache.len()
    }


    pub fn generate_frame(&mut self, frame: u64) -> ParameterValue {

        if let Some(cached_frame) = self.frame_cache.get(&frame) {
            return cached_frame.clone();
        }


        let frame_data = self.load_media_frame(frame);


        self.frame_cache.insert(frame, frame_data.clone());

        frame_data
    }


    fn load_media_frame(&self, frame: u64) -> ParameterValue {
        debug!("Loading media frame {} from path: {:?}", frame, self.media_path);


        ParameterValue::Image(Uuid::new_v4())
    }
}

impl NodeExecutor for CoreInputNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }


        let frame_data = self.generate_frame(context.frame);


        if let Some(output_pin) = self.node.outputs.first() {
            context.set_output(output_pin.id, frame_data);
        }

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Input
    }

    fn get_inputs(&self) -> Vec<Uuid> {
        vec![]
    }

    fn get_outputs(&self) -> Vec<Uuid> {
        self.node.outputs.iter().map(|pin| pin.id).collect()
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


        input_node.set_media_path("/path/to/media.mp4".to_string());
        assert_eq!(input_node.get_media_path(), Some(&"/path/to/media.mp4".to_string()));


        input_node.set_media_path("".to_string());
        assert_eq!(input_node.get_media_path(), Some(&"".to_string()));
    }

    #[test]
    fn test_cache_operations() {
        let node = Node::new(NodeType::Input, "Test".to_string());
        let mut input_node = CoreInputNode::new(node);


        assert_eq!(input_node.cache_size(), 0);


        input_node.clear_cache();
        assert_eq!(input_node.cache_size(), 0);


        let frame_data = input_node.generate_frame(1);
        assert_eq!(input_node.cache_size(), 1);
        assert!(matches!(frame_data, ParameterValue::Image(_)));


        let frame_data2 = input_node.generate_frame(1);
        assert_eq!(input_node.cache_size(), 1);
        assert_eq!(frame_data, frame_data2);


        input_node.generate_frame(2);
        assert_eq!(input_node.cache_size(), 2);
    }

    #[test]
    fn test_node_executor_interface() {
        let mut node = Node::new(NodeType::Input, "Test Input".to_string());


        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);

        let mut input_node = CoreInputNode::new(node);


        assert_eq!(input_node.node_type(), NodeType::Input);


        assert!(input_node.get_inputs().is_empty());
        assert_eq!(input_node.get_outputs().len(), 1);


        let context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };
        assert!(input_node.can_execute(&context));


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
