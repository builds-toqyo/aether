use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;

/// Output node for final result output
pub struct OutputNode {
    node: Node,
    output_format: OutputFormat,
    quality_settings: QualitySettings,
}

/// Output format options
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    /// Raw frame buffer
    Raw,
    /// PNG image sequence
    PngSequence,
    /// JPEG image sequence
    JpegSequence,
    /// MP4 video
    Mp4,
    /// ProRes video
    ProRes,
    /// EXR image sequence (HDR)
    ExrSequence,
}

/// Quality settings for output
#[derive(Debug, Clone)]
pub struct QualitySettings {
    /// JPEG quality (1-100)
    pub jpeg_quality: u8,
    /// PNG compression level (0-9)
    pub png_compression: u8,
    /// Video bitrate in Mbps
    pub video_bitrate: u32,
    /// Color depth (8, 16, 32)
    pub color_depth: u8,
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            jpeg_quality: 95,
            png_compression: 6,
            video_bitrate: 50,
            color_depth: 8,
        }
    }
}

impl OutputNode {
    /// Create a new output node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            output_format: OutputFormat::Raw,
            quality_settings: QualitySettings::default(),
        }
    }
    
    /// Set the output format
    pub fn set_output_format(&mut self, format: OutputFormat) {
        self.output_format = format;
    }
    
    /// Get the output format
    pub fn get_output_format(&self) -> OutputFormat {
        self.output_format.clone()
    }
    
    /// Set quality settings
    pub fn set_quality_settings(&mut self, settings: QualitySettings) {
        self.quality_settings = settings;
    }
    
    /// Get quality settings
    pub fn get_quality_settings(&self) -> QualitySettings {
        self.quality_settings.clone()
    }
    
    /// Process the final output based on format
    fn process_output(&self, input_value: ParameterValue, context: &ExecutionContext) -> ParameterValue {
        match &self.output_format {
            OutputFormat::Raw => {
                // Pass through raw frame buffer
                input_value
            }
            OutputFormat::PngSequence => {
                // In a real implementation, this would:
                // - Convert frame to PNG
                // - Save to file with frame number
                // - Return the saved file path or buffer
                log::debug!("Processing PNG output for frame {}", context.frame);
                input_value
            }
            OutputFormat::JpegSequence => {
                // In a real implementation, this would:
                // - Convert frame to JPEG with quality settings
                // - Save to file with frame number
                // - Return the saved file path or buffer
                log::debug!("Processing JPEG output for frame {} (quality: {})", 
                    context.frame, self.quality_settings.jpeg_quality);
                input_value
            }
            OutputFormat::Mp4 => {
                // In a real implementation, this would:
                // - Encode frame to video stream
                // - Handle video codec settings
                // - Return encoded frame data
                log::debug!("Processing MP4 output for frame {} (bitrate: {} Mbps)", 
                    context.frame, self.quality_settings.video_bitrate);
                input_value
            }
            OutputFormat::ProRes => {
                // In a real implementation, this would:
                // - Encode frame to ProRes format
                // - Handle professional video settings
                log::debug!("Processing ProRes output for frame {}", context.frame);
                input_value
            }
            OutputFormat::ExrSequence => {
                // In a real implementation, this would:
                // - Convert frame to EXR (HDR format)
                // - Handle floating point color data
                // - Save to file with frame number
                log::debug!("Processing EXR output for frame {} (depth: {} bits)", 
                    context.frame, self.quality_settings.color_depth);
                input_value
            }
        }
    }
    
    /// Create an output node with standard configuration
    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::Output, name);
        
        // Add input pin
        let input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node.add_input(input_pin);
        
        // Add output pin
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);
        
        // Add parameters
        let output_format_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "output_format".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("raw".to_string()),
            default_value: ParameterValue::String("raw".to_string()),
            min_value: None,
            max_value: None,
            animatable: false,
            description: Some("Output format (raw, png_sequence, jpeg_sequence, mp4, prores, exr_sequence)".to_string()),
        };
        node.add_parameter(output_format_param);
        
        let jpeg_quality_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "jpeg_quality".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(95),
            default_value: ParameterValue::Integer(95),
            min_value: Some(1.0),
            max_value: Some(100.0),
            animatable: false,
            description: Some("JPEG quality (1-100)".to_string()),
        };
        node.add_parameter(jpeg_quality_param);
        
        let video_bitrate_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "video_bitrate".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(50),
            default_value: ParameterValue::Integer(50),
            min_value: Some(1.0),
            max_value: Some(1000.0),
            animatable: false,
            description: Some("Video bitrate in Mbps".to_string()),
        };
        node.add_parameter(video_bitrate_param);
        
        let color_depth_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "color_depth".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(8),
            default_value: ParameterValue::Integer(8),
            min_value: Some(8.0),
            max_value: Some(32.0),
            animatable: false,
            description: Some("Color depth (8, 16, 32)".to_string()),
        };
        node.add_parameter(color_depth_param);
        
        node
    }
    
    /// Update parameters from node metadata
    fn update_parameters(&mut self) {
        if let Some(format_param) = self.node.parameters.get("output_format") {
            if let ParameterValue::String(format_str) = &format_param.value {
                self.output_format = match format_str.as_str() {
                    "png_sequence" => OutputFormat::PngSequence,
                    "jpeg_sequence" => OutputFormat::JpegSequence,
                    "mp4" => OutputFormat::Mp4,
                    "prores" => OutputFormat::ProRes,
                    "exr_sequence" => OutputFormat::ExrSequence,
                    _ => OutputFormat::Raw,
                };
            }
        }
        
        if let Some(jpeg_quality_param) = self.node.parameters.get("jpeg_quality") {
            if let ParameterValue::Integer(quality) = jpeg_quality_param.value {
                self.quality_settings.jpeg_quality = quality as u8;
            }
        }
        
        if let Some(video_bitrate_param) = self.node.parameters.get("video_bitrate") {
            if let ParameterValue::Integer(bitrate) = video_bitrate_param.value {
                self.quality_settings.video_bitrate = bitrate as u32;
            }
        }
        
        if let Some(color_depth_param) = self.node.parameters.get("color_depth") {
            if let ParameterValue::Integer(depth) = color_depth_param.value {
                self.quality_settings.color_depth = depth as u8;
            }
        }
    }
}

impl NodeExecutor for OutputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Get input from connected node
        let input_value = if let Some(input_pin) = self.node.inputs.first() {
            context.get_input(&input_pin.id).cloned()
        } else {
            None
        };
        
        let final_output = if let Some(input_value) = input_value {
            // Process the output based on format and quality settings
            self.process_output(input_value, context)
        } else {
            // No input, return none
            ParameterValue::None
        };
        
        // Set as final output
        if let Some(output_pin) = self.node.outputs.first() {
            context.set_output(output_pin.id, final_output);
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Output
    }
    
    fn validate(&self) -> NodeResult<()> {
        // Validate quality settings
        if self.quality_settings.jpeg_quality > 100 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "JPEG quality must be between 1 and 100".to_string()
            ));
        }
        
        if self.quality_settings.png_compression > 9 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "PNG compression must be between 0 and 9".to_string()
            ));
        }
        
        if self.quality_settings.video_bitrate == 0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Video bitrate must be greater than 0".to_string()
            ));
        }
        
        // Validate color depth
        if ![8, 16, 32].contains(&self.quality_settings.color_depth) {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Color depth must be 8, 16, or 32".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        &self.node.inputs.iter().map(|pin| pin.id).collect::<Vec<_>>()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        &self.node.outputs.iter().map(|pin| pin.id).collect::<Vec<_>>()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.node.enabled && 
        self.node.inputs.first()
            .and_then(|pin| pin.connection)
            .is_some()
    }
}

/// Video output node specialized for video encoding
pub struct VideoOutputNode {
    output_node: OutputNode,
    encoder_settings: EncoderSettings,
}

/// Video encoder settings
#[derive(Debug, Clone)]
pub struct EncoderSettings {
    /// Codec type
    pub codec: VideoCodec,
    /// Profile (for H.264/H.265)
    pub profile: Option<String>,
    /// Level (for H.264/H.265)
    pub level: Option<String>,
    /// Keyframe interval
    pub keyframe_interval: u32,
    /// Enable B-frames
    pub enable_b_frames: bool,
}

/// Video codec types
#[derive(Debug, Clone, PartialEq)]
pub enum VideoCodec {
    H264,
    H265,
    ProRes422,
    ProRes444,
    AppleProRes,
}

impl VideoOutputNode {
    pub fn new(node: Node) -> Self {
        let mut output_node = OutputNode::new(node);
        output_node.set_output_format(OutputFormat::Mp4);
        
        Self {
            output_node,
            encoder_settings: EncoderSettings {
                codec: VideoCodec::H264,
                profile: Some("high".to_string()),
                level: Some("4.1".to_string()),
                keyframe_interval: 30,
                enable_b_frames: true,
            },
        }
    }
    
    pub fn set_encoder_settings(&mut self, settings: EncoderSettings) {
        self.encoder_settings = settings;
    }
}

impl NodeExecutor for VideoOutputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.output_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Output
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.output_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.output_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.output_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.output_node.can_execute(context)
    }
}

/// Image sequence output node specialized for image sequences
pub struct ImageSequenceOutputNode {
    output_node: OutputNode,
    file_pattern: String,
}

impl ImageSequenceOutputNode {
    pub fn new(node: Node, format: OutputFormat) -> Self {
        let mut output_node = OutputNode::new(node);
        output_node.set_output_format(format);
        
        Self {
            output_node,
            file_pattern: "output_%04d.png".to_string(),
        }
    }
    
    pub fn set_file_pattern(&mut self, pattern: String) {
        self.file_pattern = pattern;
    }
    
    fn generate_filename(&self, frame: u64) -> String {
        self.file_pattern.replace("%04d", &format!("{:04}", frame))
    }
}

impl NodeExecutor for ImageSequenceOutputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        // Generate filename for this frame
        let filename = self.generate_filename(context.frame);
        log::debug!("Output filename: {}", filename);
        
        self.output_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Output
    }
    
    fn validate(&self) -> NodeResult<()> {
        if self.file_pattern.is_empty() {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "File pattern cannot be empty".to_string()
            ));
        }
        self.output_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.output_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.output_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.output_node.can_execute(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_output_node_creation() {
        let node = OutputNode::create_standard("Test Output".to_string());
        assert_eq!(node.node_type, NodeType::Output);
        assert_eq!(node.name, "Test Output");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.parameters.len(), 4);
    }
    
    #[test]
    fn test_output_formats() {
        assert_eq!(OutputFormat::Raw, OutputFormat::Raw);
        assert_ne!(OutputFormat::Raw, OutputFormat::Mp4);
    }
    
    #[test]
    fn test_quality_settings() {
        let settings = QualitySettings::default();
        assert_eq!(settings.jpeg_quality, 95);
        assert_eq!(settings.png_compression, 6);
        assert_eq!(settings.video_bitrate, 50);
        assert_eq!(settings.color_depth, 8);
    }
    
    #[test]
    fn test_video_output_node() {
        let node = OutputNode::create_standard("Video Out".to_string());
        let mut video_node = VideoOutputNode::new(node);
        
        assert_eq!(video_node.encoder_settings.codec, VideoCodec::H264);
        assert_eq!(video_node.encoder_settings.keyframe_interval, 30);
    }
    
    #[test]
    fn test_image_sequence_output_node() {
        let node = OutputNode::create_standard("Sequence Out".to_string());
        let mut seq_node = ImageSequenceOutputNode::new(node, OutputFormat::PngSequence);
        
        seq_node.set_file_pattern("frame_%06d.exr".to_string());
        assert_eq!(seq_node.generate_filename(123), "frame_000123.exr");
    }
}
