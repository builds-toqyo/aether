use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;

/// Color correction node for basic color adjustments
pub struct ColorCorrectionNode {
    node: Node,
    brightness: f32,
    contrast: f32,
    saturation: f32,
    gamma: f32,
    temperature: f32,
    tint: f32,
    hue: f32,
    lift: f32,
    gamma_gain: f32,
    gain: f32,
}

impl ColorCorrectionNode {
    /// Create a new color correction node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            temperature: 6500.0,
            tint: 0.0,
            hue: 0.0,
            lift: 0.0,
            gamma_gain: 1.0,
            gain: 1.0,
        }
    }
    
    /// Set brightness (-1.0 to 1.0)
    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness.clamp(-1.0, 1.0);
    }
    
    /// Get brightness
    pub fn get_brightness(&self) -> f32 {
        self.brightness
    }
    
    /// Set contrast (0.0 to 2.0)
    pub fn set_contrast(&mut self, contrast: f32) {
        self.contrast = contrast.clamp(0.0, 2.0);
    }
    
    /// Get contrast
    pub fn get_contrast(&self) -> f32 {
        self.contrast
    }
    
    /// Set saturation (0.0 to 2.0)
    pub fn set_saturation(&mut self, saturation: f32) {
        self.saturation = saturation.clamp(0.0, 2.0);
    }
    
    /// Get saturation
    pub fn get_saturation(&self) -> f32 {
        self.saturation
    }
    
    /// Set gamma (0.1 to 3.0)
    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma = gamma.clamp(0.1, 3.0);
    }
    
    /// Get gamma
    pub fn get_gamma(&self) -> f32 {
        self.gamma
    }
    
    /// Set temperature in Kelvin (2000 to 12000)
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature.clamp(2000.0, 12000.0);
    }
    
    /// Get temperature
    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }
    
    /// Set tint (-100 to 100)
    pub fn set_tint(&mut self, tint: f32) {
        self.tint = tint.clamp(-100.0, 100.0);
    }
    
    /// Get tint
    pub fn get_tint(&self) -> f32 {
        self.tint
    }
    
    /// Set hue (-180 to 180 degrees)
    pub fn set_hue(&mut self, hue: f32) {
        self.hue = hue.clamp(-180.0, 180.0);
    }
    
    /// Get hue
    pub fn get_hue(&self) -> f32 {
        self.hue
    }
    
    /// Set lift (-0.5 to 0.5)
    pub fn set_lift(&mut self, lift: f32) {
        self.lift = lift.clamp(-0.5, 0.5);
    }
    
    /// Get lift
    pub fn get_lift(&self) -> f32 {
        self.lift
    }
    
    /// Set gamma gain (0.0 to 2.0)
    pub fn set_gamma_gain(&mut self, gamma_gain: f32) {
        self.gamma_gain = gamma_gain.clamp(0.0, 2.0);
    }
    
    /// Get gamma gain
    pub fn get_gamma_gain(&self) -> f32 {
        self.gamma_gain
    }
    
    /// Set gain (0.0 to 2.0)
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 2.0);
    }
    
    /// Get gain
    pub fn get_gain(&self) -> f32 {
        self.gain
    }
    
    /// Apply color corrections to an image
    fn apply_color_correction(&self, input_value: ParameterValue) -> ParameterValue {
        match input_value {
            ParameterValue::Image(input_id) => {
                // In a real implementation, this would:
                // - Load the input texture
                // - Apply brightness, contrast, saturation adjustments
                // - Apply gamma correction
                // - Apply white balance (temperature/tint)
                // - Apply hue shift
                // - Apply lift/gamma/gain adjustments
                // - Return corrected texture ID
                
                log::debug!("Applying color correction: brightness={:.2}, contrast={:.2}, saturation={:.2}, gamma={:.2}, temp={:.0}K, tint={:.1}, hue={:.1}°, lift={:.2}, gamma_gain={:.2}, gain={:.2}", 
                    self.brightness, self.contrast, self.saturation, self.gamma, 
                    self.temperature, self.tint, self.hue, self.lift, self.gamma_gain, self.gain);
                
                // Return a corrected image ID
                let corrected_id = Uuid::new_v4();
                ParameterValue::Image(corrected_id)
            }
            _ => {
                // No valid image input
                ParameterValue::None
            }
        }
    }
    
    /// Convert temperature to RGB white balance
    fn temperature_to_rgb(&self, temperature: f32) -> (f32, f32, f32) {
        // Simplified black body radiation color temperature conversion
        // In a real implementation, this would use more precise algorithms
        let temp = temperature / 100.0;
        
        let (r, g, b) = if temp <= 66.0 {
            let r = 1.0;
            let g = if temp <= 19.0 {
                0.0
            } else {
                let g = temp - 10.0;
                let g_log = g.log10();
                let g_pow = g_log * 174.50984 - 123.61775;
                g_pow.exp()
            };
            let b = if temp <= 19.0 {
                0.0
            } else if temp >= 66.0 {
                1.0
            } else {
                let b = temp - 10.0;
                let b_log = b.log10();
                let b_pow = b_log * 138.517731 - 74.744531;
                b_pow.exp()
            };
            (r, g, b)
        } else {
            let r = temp - 60.0;
            let r_log = r.log10();
            let r_pow = r_log * 92.708053 - 31.049992;
            let r = r_pow.exp();
            
            let g = temp - 60.0;
            let g_log = g.log10();
            let g_pow = g_log * 134.969777 - 60.138036;
            let g = g_pow.exp();
            
            let b = if temp >= 66.0 {
                1.0
            } else {
                let b = temp - 2.0;
                let b_log = b.log10();
                let b_pow = b_log * 99.470803 - 61.917364;
                b_pow.exp()
            };
            (r, g, b)
        };
        
        // Apply tint adjustment (green-magenta shift)
        let tint_factor = self.tint / 100.0;
        let r = r * (1.0 - tint_factor * 0.5);
        let g = g * (1.0 + tint_factor);
        let b = b * (1.0 - tint_factor * 0.5);
        
        (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }
    
    /// Create a color correction node with standard configuration
    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::ColorCorrection, name);
        
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
        
        // Add basic correction parameters
        let brightness_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "brightness".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: Some(-1.0),
            max_value: Some(1.0),
            animatable: true,
            description: Some("Brightness adjustment (-1.0 to 1.0)".to_string()),
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
            description: Some("Contrast adjustment (0.0 to 2.0)".to_string()),
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
            description: Some("Saturation adjustment (0.0 to 2.0)".to_string()),
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
            description: Some("Gamma correction (0.1 to 3.0)".to_string()),
        };
        node.add_parameter(gamma_param);
        
        // Add white balance parameters
        let temperature_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "temperature".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(6500.0),
            default_value: ParameterValue::Float(6500.0),
            min_value: Some(2000.0),
            max_value: Some(12000.0),
            animatable: true,
            description: Some("Color temperature in Kelvin (2000-12000)".to_string()),
        };
        node.add_parameter(temperature_param);
        
        let tint_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "tint".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: Some(-100.0),
            max_value: Some(100.0),
            animatable: true,
            description: Some("Color tint (-100 to 100)".to_string()),
        };
        node.add_parameter(tint_param);
        
        // Add advanced parameters
        let hue_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "hue".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: Some(-180.0),
            max_value: Some(180.0),
            animatable: true,
            description: Some("Hue shift in degrees (-180 to 180)".to_string()),
        };
        node.add_parameter(hue_param);
        
        // Add lift/gamma/gain parameters
        let lift_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "lift".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: Some(-0.5),
            max_value: Some(0.5),
            animatable: true,
            description: Some("Shadow lift (-0.5 to 0.5)".to_string()),
        };
        node.add_parameter(lift_param);
        
        let gamma_gain_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "gamma_gain".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(0.0),
            max_value: Some(2.0),
            animatable: true,
            description: Some("Midtone gamma gain (0.0 to 2.0)".to_string()),
        };
        node.add_parameter(gamma_gain_param);
        
        let gain_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "gain".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(0.0),
            max_value: Some(2.0),
            animatable: true,
            description: Some("Highlight gain (0.0 to 2.0)".to_string()),
        };
        node.add_parameter(gain_param);
        
        node
    }
    
    /// Update parameters from node metadata
    fn update_parameters(&mut self) {
        if let Some(brightness_param) = self.node.parameters.get("brightness") {
            if let ParameterValue::Float(brightness) = brightness_param.value {
                self.brightness = brightness.clamp(-1.0, 1.0);
            }
        }
        
        if let Some(contrast_param) = self.node.parameters.get("contrast") {
            if let ParameterValue::Float(contrast) = contrast_param.value {
                self.contrast = contrast.clamp(0.0, 2.0);
            }
        }
        
        if let Some(saturation_param) = self.node.parameters.get("saturation") {
            if let ParameterValue::Float(saturation) = saturation_param.value {
                self.saturation = saturation.clamp(0.0, 2.0);
            }
        }
        
        if let Some(gamma_param) = self.node.parameters.get("gamma") {
            if let ParameterValue::Float(gamma) = gamma_param.value {
                self.gamma = gamma.clamp(0.1, 3.0);
            }
        }
        
        if let Some(temperature_param) = self.node.parameters.get("temperature") {
            if let ParameterValue::Float(temperature) = temperature_param.value {
                self.temperature = temperature.clamp(2000.0, 12000.0);
            }
        }
        
        if let Some(tint_param) = self.node.parameters.get("tint") {
            if let ParameterValue::Float(tint) = tint_param.value {
                self.tint = tint.clamp(-100.0, 100.0);
            }
        }
        
        if let Some(hue_param) = self.node.parameters.get("hue") {
            if let ParameterValue::Float(hue) = hue_param.value {
                self.hue = hue.clamp(-180.0, 180.0);
            }
        }
        
        if let Some(lift_param) = self.node.parameters.get("lift") {
            if let ParameterValue::Float(lift) = lift_param.value {
                self.lift = lift.clamp(-0.5, 0.5);
            }
        }
        
        if let Some(gamma_gain_param) = self.node.parameters.get("gamma_gain") {
            if let ParameterValue::Float(gamma_gain) = gamma_gain_param.value {
                self.gamma_gain = gamma_gain.clamp(0.0, 2.0);
            }
        }
        
        if let Some(gain_param) = self.node.parameters.get("gain") {
            if let ParameterValue::Float(gain) = gain_param.value {
                self.gain = gain.clamp(0.0, 2.0);
            }
        }
    }
}

impl NodeExecutor for ColorCorrectionNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Get input image
        let input_value = if let Some(input_pin) = self.node.inputs.first() {
            context.get_input(&input_pin.id).cloned()
        } else {
            None
        };
        
        let result = if let Some(input_value) = input_value {
            self.apply_color_correction(input_value)
        } else {
            ParameterValue::None
        };
        
        // Set output
        if let Some(output_pin) = self.node.outputs.first() {
            context.set_output(output_pin.id, result);
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::ColorCorrection
    }
    
    fn validate(&self) -> NodeResult<()> {
        // Validate parameter ranges
        if self.brightness < -1.0 || self.brightness > 1.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Brightness must be between -1.0 and 1.0".to_string()
            ));
        }
        
        if self.contrast < 0.0 || self.contrast > 2.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Contrast must be between 0.0 and 2.0".to_string()
            ));
        }
        
        if self.saturation < 0.0 || self.saturation > 2.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Saturation must be between 0.0 and 2.0".to_string()
            ));
        }
        
        if self.gamma < 0.1 || self.gamma > 3.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Gamma must be between 0.1 and 3.0".to_string()
            ));
        }
        
        if self.temperature < 2000.0 || self.temperature > 12000.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Temperature must be between 2000K and 12000K".to_string()
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
            .map(|pin| context.get_input(&pin.id).is_some())
            .unwrap_or(false)
    }
}

/// Basic color correction node (simplified)
pub struct BasicColorCorrectionNode {
    color_node: ColorCorrectionNode,
}

impl BasicColorCorrectionNode {
    pub fn new(node: Node) -> Self {
        Self {
            color_node: ColorCorrectionNode::new(node),
        }
    }
    
    /// Create a standard basic color correction node
    pub fn create_standard(name: String) -> Node {
        let mut node = ColorCorrectionNode::create_standard(name);
        
        // Remove advanced parameters for basic node
        node.parameters.retain(|name, _| !["hue", "lift", "gamma_gain", "gain"].contains(&name.as_str()));
        
        node
    }
}

impl NodeExecutor for BasicColorCorrectionNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.color_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::ColorCorrection
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.color_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.color_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.color_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.color_node.can_execute(context)
    }
}

/// White balance node (temperature and tint only)
pub struct WhiteBalanceNode {
    color_node: ColorCorrectionNode,
}

impl WhiteBalanceNode {
    pub fn new(node: Node) -> Self {
        Self {
            color_node: ColorCorrectionNode::new(node),
        }
    }
    
    /// Create a standard white balance node
    pub fn create_standard(name: String) -> Node {
        let mut node = ColorCorrectionNode::create_standard(name);
        
        // Keep only temperature and tint parameters
        node.parameters.retain(|name, _| ["temperature", "tint"].contains(&name.as_str()));
        
        node
    }
}

impl NodeExecutor for WhiteBalanceNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.color_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::ColorCorrection
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.color_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.color_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.color_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.color_node.can_execute(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_correction_node_creation() {
        let node = ColorCorrectionNode::create_standard("Test Color".to_string());
        assert_eq!(node.node_type, NodeType::ColorCorrection);
        assert_eq!(node.name, "Test Color");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.parameters.len(), 10);
    }
    
    #[test]
    fn test_color_correction_properties() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);
        
        color_node.set_brightness(0.5);
        assert_eq!(color_node.get_brightness(), 0.5);
        
        color_node.set_contrast(1.5);
        assert_eq!(color_node.get_contrast(), 1.5);
        
        color_node.set_saturation(1.2);
        assert_eq!(color_node.get_saturation(), 1.2);
        
        color_node.set_temperature(5500.0);
        assert_eq!(color_node.get_temperature(), 5500.0);
    }
    
    #[test]
    fn test_temperature_to_rgb() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let color_node = ColorCorrectionNode::new(node);
        
        let rgb_6500 = color_node.temperature_to_rgb(6500.0);
        assert!(rgb_6500.0 > 0.9 && rgb_6500.0 < 1.1); // R should be near 1.0
        assert!(rgb_6500.1 > 0.9 && rgb_6500.1 < 1.1); // G should be near 1.0
        assert!(rgb_6500.2 > 0.9 && rgb_6500.2 < 1.1); // B should be near 1.0
        
        let rgb_3200 = color_node.temperature_to_rgb(3200.0);
        assert!(rgb_3200.0 > rgb_3200.2); // R should be greater than B (warm light)
    }
    
    #[test]
    fn test_specialized_color_nodes() {
        let basic_node = BasicColorCorrectionNode::create_standard("Basic".to_string());
        let wb_node = WhiteBalanceNode::create_standard("White Balance".to_string());
        
        assert_eq!(basic_node.node_type, NodeType::ColorCorrection);
        assert_eq!(wb_node.node_type, NodeType::ColorCorrection);
        
        // Basic node should have fewer parameters
        assert!(basic_node.parameters.len() < 10);
        // White balance node should have only 2 parameters
        assert_eq!(wb_node.parameters.len(), 2);
    }
    
    #[test]
    fn test_color_correction_execution() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let color_node = ColorCorrectionNode::new(node);
        
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert!(color_node.execute(&mut context).is_ok());
    }
}
