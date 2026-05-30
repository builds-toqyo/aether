use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use crate::nodes::basic::output::{OutputFormat, QualitySettings, OutputEncoders};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;
use log::debug;


pub struct OutputNode {
    node: Node,
    output_format: OutputFormat,
    quality_settings: QualitySettings,
    encoders: OutputEncoders,
}

impl OutputNode {

    pub fn new(node: Node) -> Self {
        let output_format = OutputFormat::Raw;
        let quality_settings = QualitySettings::default();
        let encoders = OutputEncoders::new(quality_settings.clone());

        Self {
            node,
            output_format,
            quality_settings,
            encoders,
        }
    }


    pub fn set_output_format(&mut self, format: OutputFormat) {
        self.output_format = format;
    }


    pub fn get_output_format(&self) -> OutputFormat {
        self.output_format.clone()
    }


    pub fn get_quality_settings(&self) -> &QualitySettings {
        &self.quality_settings
    }


    pub fn set_quality_settings(&mut self, settings: QualitySettings) {
        self.quality_settings = settings.clone();
        self.quality_settings.validate();
        self.encoders.update_quality_settings(self.quality_settings.clone());
    }


    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::Output, name);


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


        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Binary,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);


        let format_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "output_format".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("raw".to_string()),
            default_value: ParameterValue::String("raw".to_string()),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(format_param);

        let jpeg_quality_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "jpeg_quality".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(95),
            default_value: ParameterValue::Integer(95),
            min_value: Some(ParameterValue::Integer(1)),
            max_value: Some(ParameterValue::Integer(100)),
        };
        node.add_parameter(jpeg_quality_param);

        let png_compression_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "png_compression".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(6),
            default_value: ParameterValue::Integer(6),
            min_value: Some(ParameterValue::Integer(0)),
            max_value: Some(ParameterValue::Integer(9)),
        };
        node.add_parameter(png_compression_param);

        let video_bitrate_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "video_bitrate".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(50),
            default_value: ParameterValue::Integer(50),
            min_value: Some(ParameterValue::Integer(1)),
            max_value: Some(ParameterValue::Integer(1000)),
        };
        node.add_parameter(video_bitrate_param);

        let color_depth_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "color_depth".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(8),
            default_value: ParameterValue::Integer(8),
            min_value: Some(ParameterValue::Integer(8)),
            max_value: Some(ParameterValue::Integer(32)),
        };
        node.add_parameter(color_depth_param);

        node
    }


    pub fn reset_quality_settings(&mut self) {
        self.quality_settings = QualitySettings::default();
        self.encoders.update_quality_settings(self.quality_settings.clone());
    }


    pub fn is_active(&self) -> bool {
        self.output_format != OutputFormat::Raw
    }
}

impl NodeExecutor for OutputNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {

        let input_value = self.node.get_input_value("input", context);


        let encoding_result = self.encoders.process_output(input_value, context.frame);

        if encoding_result.success {
            debug!("Output processed successfully: {:?}", encoding_result.metadata);
            self.node.set_output_value("output", encoding_result.output_data);
        } else {
            if let Some(error) = &encoding_result.error {
                log::error!("Output processing failed: {}", error);
            }
            self.node.set_output_value("output", ParameterValue::None);
        }

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Output
    }

    fn get_inputs(&self) -> &[Uuid] {
        &self.node.inputs
    }

    fn get_outputs(&self) -> &[Uuid] {
        &self.node.outputs
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_node_creation() {
        let node = OutputNode::create_standard("Test Output".to_string());
        let output_node = OutputNode::new(node);

        assert_eq!(output_node.get_output_format(), OutputFormat::Raw);
        assert_eq!(output_node.get_quality_settings().jpeg_quality, 95);
        assert_eq!(output_node.get_quality_settings().png_compression, 6);
        assert_eq!(output_node.get_quality_settings().video_bitrate, 50);
        assert_eq!(output_node.get_quality_settings().color_depth, 8);
    }

    #[test]
    fn test_output_format_setting() {
        let node = OutputNode::create_standard("Test".to_string());
        let mut output_node = OutputNode::new(node);

        output_node.set_output_format(OutputFormat::PngSequence);
        assert_eq!(output_node.get_output_format(), OutputFormat::PngSequence);

        output_node.set_output_format(OutputFormat::Mp4);
        assert_eq!(output_node.get_output_format(), OutputFormat::Mp4);

        output_node.set_output_format(OutputFormat::ExrSequence);
        assert_eq!(output_node.get_output_format(), OutputFormat::ExrSequence);
    }

    #[test]
    fn test_quality_settings() {
        let node = OutputNode::create_standard("Test".to_string());
        let mut output_node = OutputNode::new(node);

        let mut settings = QualitySettings::default();
        settings.set_jpeg_quality(85);
        settings.set_png_compression(3);
        settings.set_video_bitrate(100);
        settings.set_color_depth(16);

        output_node.set_quality_settings(settings);

        assert_eq!(output_node.get_quality_settings().jpeg_quality, 85);
        assert_eq!(output_node.get_quality_settings().png_compression, 3);
        assert_eq!(output_node.get_quality_settings().video_bitrate, 100);
        assert_eq!(output_node.get_quality_settings().color_depth, 16);
    }

    #[test]
    fn test_quality_settings_validation() {
        let mut settings = QualitySettings::default();


        settings.set_jpeg_quality(150);
        assert_eq!(settings.jpeg_quality, 100);

        settings.set_jpeg_quality(0);
        assert_eq!(settings.jpeg_quality, 1);

        settings.set_png_compression(15);
        assert_eq!(settings.png_compression, 9);

        settings.set_png_compression(-1);
        assert_eq!(settings.png_compression, 0);

        settings.set_video_bitrate(2000);
        assert_eq!(settings.video_bitrate, 1000);

        settings.set_video_bitrate(0);
        assert_eq!(settings.video_bitrate, 1);

        settings.set_color_depth(12);
        assert_eq!(settings.color_depth, 8);

        settings.set_color_depth(64);
        assert_eq!(settings.color_depth, 8);

        settings.set_color_depth(16);
        assert_eq!(settings.color_depth, 16);

        settings.set_color_depth(32);
        assert_eq!(settings.color_depth, 32);
    }

    #[test]
    fn test_is_active() {
        let node = OutputNode::create_standard("Test".to_string());
        let mut output_node = OutputNode::new(node);


        assert!(!output_node.is_active());


        output_node.set_output_format(OutputFormat::PngSequence);
        assert!(output_node.is_active());

        output_node.set_output_format(OutputFormat::Mp4);
        assert!(output_node.is_active());

        output_node.set_output_format(OutputFormat::ExrSequence);
        assert!(output_node.is_active());


        output_node.set_output_format(OutputFormat::Raw);
        assert!(!output_node.is_active());
    }

    #[test]
    fn test_reset_quality_settings() {
        let node = OutputNode::create_standard("Test".to_string());
        let mut output_node = OutputNode::new(node);


        let mut settings = QualitySettings::default();
        settings.set_jpeg_quality(50);
        settings.set_png_compression(9);
        settings.set_video_bitrate(200);
        settings.set_color_depth(32);

        output_node.set_quality_settings(settings);


        output_node.reset_quality_settings();


        assert_eq!(output_node.get_quality_settings().jpeg_quality, 95);
        assert_eq!(output_node.get_quality_settings().png_compression, 6);
        assert_eq!(output_node.get_quality_settings().video_bitrate, 50);
        assert_eq!(output_node.get_quality_settings().color_depth, 8);
    }

    #[test]
    fn test_standard_node_creation() {
        let node = OutputNode::create_standard("Test Output".to_string());

        assert_eq!(node.node_type, NodeType::Output);
        assert_eq!(node.name, "Test Output");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert!(node.inputs[0].required);


        let param_names: Vec<String> = node.parameters.iter()
            .map(|p| p.name.clone())
            .collect();
        assert!(param_names.contains(&"output_format".to_string()));
        assert!(param_names.contains(&"jpeg_quality".to_string()));
        assert!(param_names.contains(&"png_compression".to_string()));
        assert!(param_names.contains(&"video_bitrate".to_string()));
        assert!(param_names.contains(&"color_depth".to_string()));
    }
}
