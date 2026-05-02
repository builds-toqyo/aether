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
    sequence_pattern: Option<String>,
}

/// Supported media types for input
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Video,
    Image,
    Audio,
    Sequence,
}

/// Video frame metadata
#[derive(Debug, Clone)]
pub struct VideoFrameMetadata {
    /// Frame number
    pub frame_number: u64,
    /// Frame width
    pub width: usize,
    /// Frame height
    pub height: usize,
    /// Video codec
    pub codec: String,
    /// Pixel format
    pub pixel_format: String,
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Frame ID
    pub frame_id: Uuid,
}

/// Image metadata
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    /// Image width
    pub width: usize,
    /// Image height
    pub height: usize,
    /// Image format
    pub format: String,
    /// Bit depth
    pub bit_depth: u8,
    /// Color space
    pub color_space: String,
    /// Frame ID
    pub frame_id: Uuid,
}

/// Audio metadata
#[derive(Debug, Clone)]
pub struct AudioMetadata {
    /// Frame number
    pub frame_number: u64,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u8,
    /// Bit depth
    pub bit_depth: u8,
    /// Samples per frame
    pub samples_per_frame: u32,
    /// Audio ID
    pub audio_id: Uuid,
}

impl InputNode {
    /// Create a new input node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            media_path: None,
            frame_cache: HashMap::new(),
            media_type: MediaType::Video,
            sequence_pattern: None,
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
        
        // Load from media file and decode frame
        let frame_data = self.load_media_frame(frame);
        
        // Cache the frame
        self.frame_cache.insert(frame, frame_data.clone());
        
        frame_data
    }
    
    /// Load and decode media frame from file
    fn load_media_frame(&self, frame: u64) -> ParameterValue {
        log::debug!("Loading frame {} from media type: {:?}", frame, self.media_type);
        
        match &self.media_type {
            MediaType::Video => self.load_video_frame(frame),
            MediaType::Image => self.load_image_frame(frame),
            MediaType::Audio => self.load_audio_frame(frame),
            MediaType::Sequence => self.load_sequence_frame(frame),
        }
    }
    
    /// Load video frame from file
    fn load_video_frame(&self, frame: u64) -> ParameterValue {
        // In a real implementation, this would:
        // - Use FFmpeg or similar library to open video file
        // - Seek to the specified frame
        // - Decode the frame to RGB data
        // - Handle different video codecs and formats
        // - Manage frame buffers and memory
        
        if let Some(media_path) = &self.media_path {
            log::debug!("Loading video frame {} from file: {}", frame, media_path);
            
            // Simulate video frame loading
            let frame_data = self.simulate_video_frame_decode(frame, media_path);
            
            ParameterValue::Image(frame_data)
        } else {
            log::warn!("No media path set for video input");
            ParameterValue::None
        }
    }
    
    /// Load image frame from file
    fn load_image_frame(&self, frame: u64) -> ParameterValue {
        // In a real implementation, this would:
        // - Use image decoding library (stb_image, libpng, libjpeg, etc.)
        // - Load image from file
        // - Convert to RGB format
        // - Handle different image formats (PNG, JPEG, TIFF, EXR, etc.)
        // - Manage memory for large images
        
        if let Some(media_path) = &self.media_path {
            log::debug!("Loading image from file: {}", media_path);
            
            // For image input, frame number doesn't matter (always same image)
            let frame_data = self.simulate_image_decode(media_path);
            
            ParameterValue::Image(frame_data)
        } else {
            log::warn!("No media path set for image input");
            ParameterValue::None
        }
    }
    
    /// Load audio frame from file
    fn load_audio_frame(&self, frame: u64) -> ParameterValue {
        // In a real implementation, this would:
        // - Use audio decoding library (libsndfile, libmp3lame, etc.)
        // - Load audio samples for the frame
        // - Handle different audio formats (WAV, MP3, FLAC, etc.)
        // - Convert to float samples
        // - Handle sample rate conversion
        
        if let Some(media_path) = &self.media_path {
            log::debug!("Loading audio frame {} from file: {}", frame, media_path);
            
            // Simulate audio frame loading
            let audio_data = self.simulate_audio_decode(frame, media_path);
            
            ParameterValue::Audio(audio_data)
        } else {
            log::warn!("No media path set for audio input");
            ParameterValue::None
        }
    }
    
    /// Load frame from image sequence
    fn load_sequence_frame(&self, frame: u64) -> ParameterValue {
        // In a real implementation, this would:
        // - Generate filename based on frame number and pattern
        // - Load individual image from sequence
        // - Handle missing frames gracefully
        // - Cache sequence metadata
        
        if let Some(media_path) = &self.media_path {
            let filename = self.generate_sequence_filename(frame);
            log::debug!("Loading sequence frame {} from file: {}", frame, filename);
            
            let frame_data = self.simulate_sequence_frame_decode(frame, &filename);
            
            ParameterValue::Image(frame_data)
        } else {
            log::warn!("No media path set for sequence input");
            ParameterValue::None
        }
    }
    
    /// Simulate video frame decoding
    fn simulate_video_frame_decode(&self, frame: u64, media_path: &str) -> Uuid {
        // In a real implementation, this would:
        // - Open video file with FFmpeg
        // - Seek to frame position
        // - Decode frame to RGB buffer
        // - Handle different codecs (H.264, H.265, ProRes, etc.)
        
        log::debug!("Decoding video frame {} from {}", frame, media_path);
        
        // Simulate frame dimensions and format
        let width = 1920;
        let height = 1080;
        let codec = "H.264";
        let pixel_format = "RGB24";
        
        log::debug!("Video info: {}x{}, codec={}, format={}", width, height, codec, pixel_format);
        
        // Create frame ID (in real implementation, this would be texture ID)
        let frame_id = Uuid::new_v4();
        
        // Store frame metadata
        let frame_metadata = VideoFrameMetadata {
            frame_number: frame,
            width,
            height,
            codec: codec.to_string(),
            pixel_format: pixel_format.to_string(),
            timestamp: frame as f64 / 30.0, // Assuming 30 FPS
            frame_id,
        };
        
        log::debug!("Created video frame: {:?}", frame_metadata);
        
        frame_id
    }
    
    /// Simulate image decoding
    fn simulate_image_decode(&self, media_path: &str) -> Uuid {
        // In a real implementation, this would:
        // - Detect image format from file extension
        // - Use appropriate decoder (PNG, JPEG, TIFF, EXR, etc.)
        // - Convert to RGB format
        // - Handle different bit depths (8-bit, 16-bit, 32-bit float)
        
        log::debug!("Decoding image from {}", media_path);
        
        // Simulate image properties
        let width = 1920;
        let height = 1080;
        let format = "RGBA8";
        let bit_depth = 8;
        
        log::debug!("Image info: {}x{}, format={}, depth={} bits", width, height, format, bit_depth);
        
        let frame_id = Uuid::new_v4();
        
        // Store image metadata
        let image_metadata = ImageMetadata {
            width,
            height,
            format: format.to_string(),
            bit_depth,
            color_space: "sRGB".to_string(),
            frame_id,
        };
        
        log::debug!("Created image: {:?}", image_metadata);
        
        frame_id
    }
    
    /// Simulate audio decoding
    fn simulate_audio_decode(&self, frame: u64, media_path: &str) -> Uuid {
        // In a real implementation, this would:
        // - Open audio file with audio decoder
        // - Seek to frame position
        // - Decode audio samples
        // - Convert to float samples
        // - Handle different sample rates and bit depths
        
        log::debug!("Decoding audio frame {} from {}", frame, media_path);
        
        // Simulate audio properties
        let sample_rate = 48000;
        let channels = 2;
        let bit_depth = 16;
        let samples_per_frame = sample_rate / 30; // Assuming 30 FPS video
        
        log::debug!("Audio info: {}Hz, {} channels, {} bits, {} samples/frame", 
            sample_rate, channels, bit_depth, samples_per_frame);
        
        let audio_id = Uuid::new_v4();
        
        // Store audio metadata
        let audio_metadata = AudioMetadata {
            frame_number: frame,
            sample_rate,
            channels,
            bit_depth,
            samples_per_frame,
            audio_id,
        };
        
        log::debug!("Created audio frame: {:?}", audio_metadata);
        
        audio_id
    }
    
    /// Simulate sequence frame decoding
    fn simulate_sequence_frame_decode(&self, frame: u64, filename: &str) -> Uuid {
        // In a real implementation, this would:
        // - Load individual image from sequence
        // - Handle missing files gracefully
        // - Maintain consistent format across sequence
        
        log::debug!("Decoding sequence frame {} from {}", frame, filename);
        
        // Use same simulation as image decode
        self.simulate_image_decode(filename)
    }
    
    /// Generate filename for image sequence
    fn generate_sequence_filename(&self, frame: u64) -> String {
        // In a real implementation, this would:
        // - Use the pattern to generate filename
        // - Handle different padding formats (%04d, %06d, etc.)
        // - Support different naming conventions
        
        if let Some(pattern) = &self.sequence_pattern {
            pattern.replace("%04d", &format!("{:04}", frame))
                .replace("%06d", &format!("{:06}", frame))
                .replace("%d", &format!("{}", frame))
        } else {
            format!("frame_{:04}.png", frame)
        }
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
        assert_eq!(MediaType::Image, MediaType::Image);
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
    
    #[test]
    fn test_media_loading_pipeline() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_video.mp4".to_string());
        input_node.set_media_type(MediaType::Video);
        
        let frame_data = input_node.load_media_frame(0);
        
        match frame_data {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result for video"),
        }
    }
    
    #[test]
    fn test_video_frame_loading() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_video.mp4".to_string());
        
        let frame_data = input_node.load_video_frame(10);
        
        match frame_data {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result"),
        }
    }
    
    #[test]
    fn test_image_loading() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_image.png".to_string());
        input_node.set_media_type(MediaType::Image);
        
        let frame_data = input_node.load_image_frame(0);
        
        match frame_data {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result for image"),
        }
    }
    
    #[test]
    fn test_audio_loading() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_audio.wav".to_string());
        input_node.set_media_type(MediaType::Audio);
        
        let frame_data = input_node.load_audio_frame(0);
        
        match frame_data {
            ParameterValue::Audio(audio_id) => {
                assert_ne!(audio_id, Uuid::default());
            }
            _ => panic!("Expected Audio result"),
        }
    }
    
    #[test]
    fn test_sequence_loading() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("frame_%04d.png".to_string());
        input_node.set_media_type(MediaType::Sequence);
        
        let frame_data = input_node.load_sequence_frame(5);
        
        match frame_data {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result for sequence"),
        }
    }
    
    #[test]
    fn test_frame_caching() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_video.mp4".to_string());
        
        // Generate first frame
        let frame1 = input_node.generate_frame(0);
        
        // Check cache size
        assert_eq!(input_node.cache_size(), 1);
        
        // Generate same frame again (should use cache)
        let frame2 = input_node.generate_frame(0);
        
        // Should be the same result
        assert_eq!(frame1, frame2);
        
        // Cache size should still be 1
        assert_eq!(input_node.cache_size(), 1);
    }
    
    #[test]
    fn test_frame_cache_clear() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_video.mp4".to_string());
        
        // Generate some frames
        input_node.generate_frame(0);
        input_node.generate_frame(1);
        input_node.generate_frame(2);
        
        assert_eq!(input_node.cache_size(), 3);
        
        // Clear cache
        input_node.clear_cache();
        
        assert_eq!(input_node.cache_size(), 0);
    }
    
    #[test]
    fn test_video_frame_simulation() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let frame_id = input_node.simulate_video_frame_decode(10, "test.mp4");
        
        assert_ne!(frame_id, Uuid::default());
    }
    
    #[test]
    fn test_image_decode_simulation() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let frame_id = input_node.simulate_image_decode("test.png");
        
        assert_ne!(frame_id, Uuid::default());
    }
    
    #[test]
    fn test_audio_decode_simulation() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let audio_id = input_node.simulate_audio_decode(5, "test.wav");
        
        assert_ne!(audio_id, Uuid::default());
    }
    
    #[test]
    fn test_sequence_filename_generation() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        // Test default pattern
        let filename1 = input_node.generate_sequence_filename(5);
        assert_eq!(filename1, "frame_0005.png");
        
        // Test custom pattern
        input_node.sequence_pattern = Some("shot_%06d.exr".to_string());
        let filename2 = input_node.generate_sequence_filename(123);
        assert_eq!(filename2, "shot_000123.exr");
        
        // Test %d pattern
        input_node.sequence_pattern = Some("frame_%d.jpg".to_string());
        let filename3 = input_node.generate_sequence_filename(42);
        assert_eq!(filename3, "frame_42.jpg");
    }
    
    #[test]
    fn test_media_path_validation() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        // Test without media path
        let result = input_node.load_video_frame(0);
        assert_eq!(result, ParameterValue::None);
        
        // Test with media path
        input_node.set_media_path("test.mp4".to_string());
        let result = input_node.load_video_frame(0);
        
        match result {
            ParameterValue::Image(_) => {
                // Success
            }
            _ => panic!("Expected Image result"),
        }
    }
    
    #[test]
    fn test_video_frame_metadata() {
        let metadata = VideoFrameMetadata {
            frame_number: 10,
            width: 1920,
            height: 1080,
            codec: "H.264".to_string(),
            pixel_format: "RGB24".to_string(),
            timestamp: 0.333,
            frame_id: Uuid::new_v4(),
        };
        
        assert_eq!(metadata.frame_number, 10);
        assert_eq!(metadata.width, 1920);
        assert_eq!(metadata.height, 1080);
        assert_eq!(metadata.codec, "H.264");
        assert_eq!(metadata.pixel_format, "RGB24");
        assert_eq!(metadata.timestamp, 0.333);
    }
    
    #[test]
    fn test_image_metadata() {
        let metadata = ImageMetadata {
            width: 1920,
            height: 1080,
            format: "RGBA8".to_string(),
            bit_depth: 8,
            color_space: "sRGB".to_string(),
            frame_id: Uuid::new_v4(),
        };
        
        assert_eq!(metadata.width, 1920);
        assert_eq!(metadata.height, 1080);
        assert_eq!(metadata.format, "RGBA8");
        assert_eq!(metadata.bit_depth, 8);
        assert_eq!(metadata.color_space, "sRGB");
    }
    
    #[test]
    fn test_audio_metadata() {
        let metadata = AudioMetadata {
            frame_number: 5,
            sample_rate: 48000,
            channels: 2,
            bit_depth: 16,
            samples_per_frame: 1600,
            audio_id: Uuid::new_v4(),
        };
        
        assert_eq!(metadata.frame_number, 5);
        assert_eq!(metadata.sample_rate, 48000);
        assert_eq!(metadata.channels, 2);
        assert_eq!(metadata.bit_depth, 16);
        assert_eq!(metadata.samples_per_frame, 1600);
    }
    
    #[test]
    fn test_different_media_types() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test.mp4".to_string());
        
        // Test video
        input_node.set_media_type(MediaType::Video);
        let video_result = input_node.load_media_frame(0);
        match video_result {
            ParameterValue::Image(_) => {},
            _ => panic!("Expected Image for video"),
        }
        
        // Test image
        input_node.set_media_type(MediaType::Image);
        let image_result = input_node.load_media_frame(0);
        match image_result {
            ParameterValue::Image(_) => {},
            _ => panic!("Expected Image for image"),
        }
        
        // Test audio
        input_node.set_media_type(MediaType::Audio);
        let audio_result = input_node.load_media_frame(0);
        match audio_result {
            ParameterValue::Audio(_) => {},
            _ => panic!("Expected Audio for audio"),
        }
        
        // Test sequence
        input_node.set_media_type(MediaType::Sequence);
        let sequence_result = input_node.load_media_frame(0);
        match sequence_result {
            ParameterValue::Image(_) => {},
            _ => panic!("Expected Image for sequence"),
        }
    }
    
    #[test]
    fn test_frame_generation_with_caching() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test.mp4".to_string());
        
        // Generate multiple frames
        let frame0 = input_node.generate_frame(0);
        let frame1 = input_node.generate_frame(1);
        let frame2 = input_node.generate_frame(2);
        
        // All should be different
        assert_ne!(frame0, frame1);
        assert_ne!(frame1, frame2);
        assert_ne!(frame0, frame2);
        
        // Cache should contain 3 frames
        assert_eq!(input_node.cache_size(), 3);
        
        // Generate frame 1 again (should use cache)
        let frame1_cached = input_node.generate_frame(1);
        assert_eq!(frame1, frame1_cached);
        
        // Cache size should still be 3
        assert_eq!(input_node.cache_size(), 3);
    }
}
