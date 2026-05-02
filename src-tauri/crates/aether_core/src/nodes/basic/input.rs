use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use std::collections::HashMap;
use uuid::Uuid;

/// Input node for media source input
pub struct InputNode {
    node: Node,
    media_path: Option<String>,
    frame_cache: HashMap<u64, ParameterValue>,
    media_type: MediaType,
}

/// Supported media types for input
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Video,
    Image,
    Audio,
    Sequence,
}

impl InputNode {
    /// Create a new input node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            media_path: None,
            frame_cache: HashMap::new(),
            media_type: MediaType::Video,
        }
    }
    
    /// Set the media path for this input
    pub fn set_media_path(&mut self, path: String) {
        self.media_path = Some(path);
    }
    
    /// Set the media type
    pub fn set_media_type(&mut self, media_type: MediaType) {
        self.media_type = media_type;
    }
    
    /// Get the media type
    pub fn get_media_type(&self) -> MediaType {
        self.media_type.clone()
    }
    
    /// Generate a frame for the given frame number
    fn generate_frame(&self, frame: u64) -> ParameterValue {
        // Check cache first
        if let Some(cached_frame) = self.frame_cache.get(&frame) {
            return cached_frame.clone();
        }
        
        // Generate a dummy frame for now
        // In a real implementation, this would:
        // - Load from media file if media_path is set
        // - Decode video frame or load image
        // - Handle different media types appropriately
        let frame_id = Uuid::new_v4();
        ParameterValue::Image(frame_id)
    }
    
    /// Clear the frame cache
    pub fn clear_cache(&mut self) {
        self.frame_cache.clear();
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.frame_cache.len()
    }
    
    /// Create an input node with standard configuration
    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::Input, name);
        
        // Add output pin
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);
        
        // Add parameters
        let media_path_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "media_path".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String(String::new()),
            default_value: ParameterValue::String(String::new()),
            min_value: None,
            max_value: None,
            animatable: false,
            description: Some("Path to the media file".to_string()),
        };
        node.add_parameter(media_path_param);
        
        let media_type_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "media_type".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("video".to_string()),
            default_value: ParameterValue::String("video".to_string()),
            min_value: None,
            max_value: None,
            animatable: false,
            description: Some("Type of media (video, image, audio, sequence)".to_string()),
        };
        node.add_parameter(media_type_param);
        
        let start_frame_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "start_frame".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(0),
            default_value: ParameterValue::Integer(0),
            min_value: Some(0.0),
            max_value: None,
            animatable: false,
            description: Some("Start frame for video/sequence".to_string()),
        };
        node.add_parameter(start_frame_param);
        
        node
    }
}

impl NodeExecutor for InputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Generate frame for current time
        let frame = self.generate_frame(context.frame);
        
        // Set output
        if let Some(output_pin) = self.node.outputs.first() {
            context.set_output(output_pin.id, frame);
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn validate(&self) -> NodeResult<()> {
        // Validate that we have a media path for non-sequence types
        if self.media_path.is_none() && self.media_type != MediaType::Sequence {
            log::warn!("Input node has no media path set");
        }
        
        // Validate cache size
        if self.frame_cache.len() > 1000 {
            log::warn!("Input node cache size is large: {}", self.frame_cache.len());
        }
        
        Ok(())
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

/// Video input node specialized for video files
pub struct VideoInputNode {
    input_node: InputNode,
    frame_rate: f32,
    duration_frames: u64,
}

impl VideoInputNode {
    pub fn new(node: Node) -> Self {
        let mut input_node = InputNode::new(node);
        input_node.set_media_type(MediaType::Video);
        
        Self {
            input_node,
            frame_rate: 30.0,
            duration_frames: 0,
        }
    }
    
    pub fn set_frame_rate(&mut self, frame_rate: f32) {
        self.frame_rate = frame_rate;
    }
    
    pub fn set_duration(&mut self, duration_frames: u64) {
        self.duration_frames = duration_frames;
    }
    
    pub fn get_duration(&self) -> u64 {
        self.duration_frames
    }
}

impl NodeExecutor for VideoInputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.input_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.input_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.input_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.input_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        // Check if frame is within duration
        if self.duration_frames > 0 && context.frame >= self.duration_frames {
            return false;
        }
        self.input_node.can_execute(context)
    }
}

/// Image input node specialized for static images
pub struct ImageInputNode {
    input_node: InputNode,
}

impl ImageInputNode {
    pub fn new(node: Node) -> Self {
        let mut input_node = InputNode::new(node);
        input_node.set_media_type(MediaType::Image);
        
        Self { input_node }
    }
}

impl NodeExecutor for ImageInputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.input_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.input_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.input_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.input_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.input_node.can_execute(context)
    }
}

/// Sequence input node specialized for image sequences
pub struct SequenceInputNode {
    input_node: InputNode,
    sequence_pattern: Option<String>,
    start_frame: u64,
}

impl SequenceInputNode {
    pub fn new(node: Node) -> Self {
        let mut input_node = InputNode::new(node);
        input_node.set_media_type(MediaType::Sequence);
        
        Self {
            input_node,
            sequence_pattern: None,
            start_frame: 0,
        }
    }
    
    pub fn set_sequence_pattern(&mut self, pattern: String) {
        self.sequence_pattern = Some(pattern);
    }
    
    pub fn set_start_frame(&mut self, start_frame: u64) {
        self.start_frame = start_frame;
    }
    
    fn get_sequence_frame(&self, frame: u64) -> u64 {
        self.start_frame + frame
    }
}

impl NodeExecutor for SequenceInputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        // Calculate actual sequence frame
        let sequence_frame = self.get_sequence_frame(context.frame);
        
        // Create a modified context for the underlying input node
        let mut modified_context = ExecutionContext::new(
            sequence_frame,
            context.time,
            context.frame_rate,
            context.resolution,
        );
        
        self.input_node.execute(&mut modified_context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn validate(&self) -> NodeResult<()> {
        if self.sequence_pattern.is_none() {
            log::warn!("Sequence input node has no pattern set");
        }
        self.input_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.input_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.input_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.input_node.can_execute(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_node_creation() {
        let node = InputNode::create_standard("Test Input".to_string());
        assert_eq!(node.node_type, NodeType::Input);
        assert_eq!(node.name, "Test Input");
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.parameters.len(), 3);
    }
    
    #[test]
    fn test_input_node_execution() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert!(input_node.execute(&mut context).is_ok());
        
        // Should have output
        assert!(!context.outputs.is_empty());
    }
    
    #[test]
    fn test_media_types() {
        assert_eq!(MediaType::Video, MediaType::Video);
        assert_ne!(MediaType::Video, MediaType::Image);
    }
    
    #[test]
    fn test_video_input_node() {
        let node = InputNode::create_standard("Video".to_string());
        let mut video_node = VideoInputNode::new(node);
        
        video_node.set_frame_rate(24.0);
        video_node.set_duration(100);
        
        assert_eq!(video_node.frame_rate, 24.0);
        assert_eq!(video_node.get_duration(), 100);
    }
    
    #[test]
    fn test_sequence_input_node() {
        let node = InputNode::create_standard("Sequence".to_string());
        let mut sequence_node = SequenceInputNode::new(node);
        
        sequence_node.set_sequence_pattern("frame_%04d.png".to_string());
        sequence_node.set_start_frame(10);
        
        assert_eq!(sequence_node.get_sequence_frame(5), 15);
    }
}
