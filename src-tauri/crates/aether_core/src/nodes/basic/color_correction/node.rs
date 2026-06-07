use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use crate::nodes::basic::color_correction::{ColorCorrectionParams, ColorProcessor, GpuOperations};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;
use log::debug;

pub struct ColorCorrectionNode {
    node: Node,
    parameters: ColorCorrectionParams,
    processor: ColorProcessor,
    gpu_ops: GpuOperations,
}

impl ColorCorrectionNode {

    pub fn new(node: Node) -> Self {
        let parameters = ColorCorrectionParams::default();
        let processor = ColorProcessor::new(parameters.clone());
        let gpu_ops = GpuOperations::new();

        Self {
            node,
            parameters,
            processor,
            gpu_ops,
        }
    }

    pub fn set_brightness(&mut self, brightness: f32) {
        self.parameters.brightness = brightness.clamp(-1.0, 1.0);
        self.update_processor();
    }

    pub fn get_brightness(&self) -> f32 {
        self.parameters.brightness
    }

    pub fn set_contrast(&mut self, contrast: f32) {
        self.parameters.contrast = contrast.clamp(0.0, 2.0);
        self.update_processor();
    }

    pub fn get_contrast(&self) -> f32 {
        self.parameters.contrast
    }

    pub fn set_saturation(&mut self, saturation: f32) {
        self.parameters.saturation = saturation.clamp(0.0, 2.0);
        self.update_processor();
    }

    pub fn get_saturation(&self) -> f32 {
        self.parameters.saturation
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        self.parameters.gamma = gamma.clamp(0.1, 3.0);
        self.update_processor();
    }

    pub fn get_gamma(&self) -> f32 {
        self.parameters.gamma
    }

    pub fn set_temperature(&mut self, temperature: f32) {
        self.parameters.temperature = temperature.clamp(2000.0, 12000.0);
        self.update_processor();
    }

    pub fn get_temperature(&self) -> f32 {
        self.parameters.temperature
    }

    pub fn set_tint(&mut self, tint: f32) {
        self.parameters.tint = tint.clamp(-100.0, 100.0);
        self.update_processor();
    }

    pub fn get_tint(&self) -> f32 {
        self.parameters.tint
    }

    pub fn set_hue(&mut self, hue: f32) {
        self.parameters.hue = hue.clamp(-180.0, 180.0);
        self.update_processor();
    }

    pub fn get_hue(&self) -> f32 {
        self.parameters.hue
    }

    pub fn set_lift(&mut self, lift: f32) {
        self.parameters.lift = lift.clamp(-1.0, 1.0);
        self.update_processor();
    }

    pub fn get_lift(&self) -> f32 {
        self.parameters.lift
    }

    pub fn set_gamma_gain(&mut self, gamma_gain: f32) {
        self.parameters.gamma_gain = gamma_gain.clamp(0.1, 3.0);
        self.update_processor();
    }

    pub fn get_gamma_gain(&self) -> f32 {
        self.parameters.gamma_gain
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.parameters.gain = gain.clamp(0.0, 2.0);
        self.update_processor();
    }

    pub fn get_gain(&self) -> f32 {
        self.parameters.gain
    }

    pub fn set_parameters(&mut self, params: ColorCorrectionParams) {
        self.parameters = params;
        self.parameters.validate();
        self.update_processor();
    }

    pub fn get_parameters(&self) -> &ColorCorrectionParams {
        &self.parameters
    }

    fn update_processor(&mut self) {
        self.processor.set_parameters(self.parameters.clone());
    }

    pub fn process_image_texture(&mut self, input_id: Uuid, corrected_id: Uuid) -> Result<Uuid, String> {

        let texture_info = self.gpu_ops.bind_texture(input_id)?;

        let raw_data = self.gpu_ops.read_pixel_data(&texture_info);

        let rgb_data = self.gpu_ops.convert_to_rgb_float(&raw_data, &texture_info);

        let corrected_image = self.processor.apply_corrections(&rgb_data, corrected_id);

        let corrected_raw_data = self.gpu_ops.convert_from_rgb_float(&corrected_image.data, &texture_info);

        self.gpu_ops.upload_corrected_texture(corrected_id, &corrected_raw_data, &texture_info)?;

        debug!("Color correction applied successfully: {:?}", corrected_id);

        Ok(corrected_id)
    }


    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::ColorCorrection, name);


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
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);


        let brightness_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "brightness".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: Some(-1.0),
            max_value: Some(1.0),
            animatable: true,
            description: Some("Brightness adjustment".to_string()),
        };
        node.add_parameter(brightness_param);

        let contrast_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "contrast".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(0.0),
            max_value: Some(2.0),
            animatable: true,
            description: Some("Contrast adjustment".to_string()),
        };
        node.add_parameter(contrast_param);

        let saturation_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "saturation".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(0.0),
            max_value: Some(2.0),
            animatable: true,
            description: Some("Saturation adjustment".to_string()),
        };
        node.add_parameter(saturation_param);

        let gamma_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "gamma".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(0.1),
            max_value: Some(3.0),
            animatable: true,
            description: Some("Gamma correction".to_string()),
        };
        node.add_parameter(gamma_param);

        node
    }


    pub fn reset_parameters(&mut self) {
        self.parameters = ColorCorrectionParams::default();
        self.update_processor();
    }


    pub fn is_active(&self) -> bool {
        self.parameters.is_active()
    }


    pub fn clear_cache(&mut self) {
        self.gpu_ops.clear_cache();
    }


    pub fn cache_size(&self) -> usize {
        self.gpu_ops.cache_size()
    }
}

impl NodeExecutor for ColorCorrectionNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {

        let input_value = self.node.get_input_value("input").unwrap_or(ParameterValue::None);

        match input_value {
            ParameterValue::Image(input_id) => {
                debug!("Executing color correction: brightness={:.2}, contrast={:.2}, saturation={:.2}, gamma={:.2}, temp={:.0}K, tint={:.1}, hue={:.1}°, lift={:.2}, gamma_gain={:.2}, gain={:.2}",
                    self.parameters.brightness, self.parameters.contrast, self.parameters.saturation, self.parameters.gamma,
                    self.parameters.temperature, self.parameters.tint, self.parameters.hue, self.parameters.lift, self.parameters.gamma_gain, self.parameters.gain);

                let corrected_id = Uuid::new_v4();

                match self.process_image_texture(input_id, corrected_id) {
                    Ok(_) => {

                        self.node.set_output_value("output", ParameterValue::Image(corrected_id));
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Color correction failed: {}", e);
                        Err(crate::nodes::NodeError::ExecutionFailed(e))
                    }
                }
            }
            _ => {

                self.node.set_output_value("output", ParameterValue::None);
                Ok(())
            }
        }
    }

    fn node_type(&self) -> NodeType {
        NodeType::ColorCorrection
    }

    fn get_inputs(&self) -> Vec<Uuid> {
        self.node.inputs.iter().map(|pin| pin.id).collect()
    }

    fn get_outputs(&self) -> Vec<Uuid> {
        self.node.outputs.iter().map(|pin| pin.id).collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_correction_node_creation() {
        let node = ColorCorrectionNode::create_standard("Test Color Correction".to_string());
        let color_node = ColorCorrectionNode::new(node);

        assert_eq!(color_node.get_brightness(), 0.0);
        assert_eq!(color_node.get_contrast(), 1.0);
        assert_eq!(color_node.get_saturation(), 1.0);
        assert_eq!(color_node.get_gamma(), 1.0);
        assert_eq!(color_node.get_temperature(), 6500.0);
        assert_eq!(color_node.get_tint(), 0.0);
        assert_eq!(color_node.get_hue(), 0.0);
        assert_eq!(color_node.get_lift(), 0.0);
        assert_eq!(color_node.get_gamma_gain(), 1.0);
        assert_eq!(color_node.get_gain(), 1.0);
    }

    #[test]
    fn test_parameter_setting() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);

        color_node.set_brightness(0.5);
        assert_eq!(color_node.get_brightness(), 0.5);

        color_node.set_contrast(1.5);
        assert_eq!(color_node.get_contrast(), 1.5);

        color_node.set_saturation(1.2);
        assert_eq!(color_node.get_saturation(), 1.2);

        color_node.set_gamma(1.1);
        assert_eq!(color_node.get_gamma(), 1.1);

        color_node.set_temperature(5500.0);
        assert_eq!(color_node.get_temperature(), 5500.0);

        color_node.set_tint(10.0);
        assert_eq!(color_node.get_tint(), 10.0);

        color_node.set_hue(45.0);
        assert_eq!(color_node.get_hue(), 45.0);

        color_node.set_lift(0.2);
        assert_eq!(color_node.get_lift(), 0.2);

        color_node.set_gamma_gain(1.2);
        assert_eq!(color_node.get_gamma_gain(), 1.2);

        color_node.set_gain(1.1);
        assert_eq!(color_node.get_gain(), 1.1);
    }

    #[test]
    fn test_parameter_clamping() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);


        color_node.set_brightness(2.0);
        assert_eq!(color_node.get_brightness(), 1.0);

        color_node.set_brightness(-2.0);
        assert_eq!(color_node.get_brightness(), -1.0);

        color_node.set_contrast(3.0);
        assert_eq!(color_node.get_contrast(), 2.0);

        color_node.set_contrast(-1.0);
        assert_eq!(color_node.get_contrast(), 0.0);

        color_node.set_temperature(1000.0);
        assert_eq!(color_node.get_temperature(), 2000.0);

        color_node.set_temperature(20000.0);
        assert_eq!(color_node.get_temperature(), 12000.0);
    }

    #[test]
    fn test_reset_parameters() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);


        color_node.set_brightness(0.5);
        color_node.set_contrast(1.5);
        color_node.set_saturation(1.2);


        color_node.reset_parameters();


        assert_eq!(color_node.get_brightness(), 0.0);
        assert_eq!(color_node.get_contrast(), 1.0);
        assert_eq!(color_node.get_saturation(), 1.0);
    }

    #[test]
    fn test_is_active() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);


        assert!(!color_node.is_active());


        color_node.set_brightness(0.5);
        assert!(color_node.is_active());


        color_node.reset_parameters();
        assert!(!color_node.is_active());
    }
}
