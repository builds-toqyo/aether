use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;

/// Metadata for a corrected image
#[derive(Debug, Clone)]
pub struct CorrectedImage {
    /// Original image ID
    pub original_id: Uuid,
    /// Corrected image ID
    pub corrected_id: Uuid,
    /// Brightness adjustment
    pub brightness: f32,
    /// Contrast adjustment
    pub contrast: f32,
    /// Saturation adjustment
    pub saturation: f32,
    /// Gamma correction
    pub gamma: f32,
    /// Color temperature
    pub temperature: f32,
    /// Color tint
    pub tint: f32,
    /// Hue shift
    pub hue: f32,
    /// Shadow lift
    pub lift: f32,
    /// Midtone gamma gain
    pub gamma_gain: f32,
    /// Highlight gain
    pub gain: f32,
}

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
                log::debug!("Applying color correction: brightness={:.2}, contrast={:.2}, saturation={:.2}, gamma={:.2}, temp={:.0}K, tint={:.1}, hue={:.1}°, lift={:.2}, gamma_gain={:.2}, gain={:.2}", 
                    self.brightness, self.contrast, self.saturation, self.gamma, 
                    self.temperature, self.tint, self.hue, self.lift, self.gamma_gain, self.gain);
                
                // Create a new corrected image ID
                let corrected_id = Uuid::new_v4();
                
                // In a real implementation, this would load the texture and process pixel data
                // For now, we'll create a metadata structure that represents the corrected image
                let corrected_image = CorrectedImage {
                    original_id: input_id,
                    corrected_id,
                    brightness: self.brightness,
                    contrast: self.contrast,
                    saturation: self.saturation,
                    gamma: self.gamma,
                    temperature: self.temperature,
                    tint: self.tint,
                    hue: self.hue,
                    lift: self.lift,
                    gamma_gain: self.gamma_gain,
                    gain: self.gain,
                };
                
                // Store the corrected image metadata (in a real implementation, this would be a texture cache)
                log::debug!("Created corrected image: {:?}", corrected_image);
                
                ParameterValue::Image(corrected_id)
            }
            _ => {
                // No valid image input
                ParameterValue::None
            }
        }
    }
    
    /// Apply color correction to individual pixel values
    fn apply_pixel_correction(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        // Step 1: Apply white balance (temperature/tint)
        let (r, g, b) = self.apply_white_balance(r, g, b);
        
        // Step 2: Apply lift/gamma/gain adjustments
        let (r, g, b) = self.apply_lift_gamma_gain(r, g, b);
        
        // Step 3: Apply brightness and contrast
        let (r, g, b) = self.apply_brightness_contrast(r, g, b);
        
        // Step 4: Apply gamma correction
        let (r, g, b) = self.apply_gamma_correction(r, g, b);
        
        // Step 5: Apply hue and saturation adjustments
        let (r, g, b) = self.apply_hue_saturation(r, g, b);
        
        // Clamp values to valid range
        (
            r.clamp(0.0, 1.0),
            g.clamp(0.0, 1.0),
            b.clamp(0.0, 1.0)
        )
    }
    
    /// Apply white balance adjustments
    fn apply_white_balance(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let (wb_r, wb_g, wb_b) = self.temperature_to_rgb(self.temperature);
        
        // Apply white balance multiplication
        let r = r * wb_r;
        let g = g * wb_g;
        let b = b * wb_b;
        
        // Apply tint adjustment (green-magenta shift)
        let tint_factor = self.tint / 100.0;
        let r = r * (1.0 - tint_factor * 0.5);
        let g = g * (1.0 + tint_factor);
        let b = b * (1.0 - tint_factor * 0.5);
        
        (r, g, b)
    }
    
    /// Apply lift/gamma/gain adjustments
    fn apply_lift_gamma_gain(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        // Apply lift (shadow adjustment)
        let r = r + self.lift;
        let g = g + self.lift;
        let b = b + self.lift;
        
        // Apply gamma gain (midtone adjustment)
        if self.gamma_gain > 0.0 {
            let gamma = 1.0 / self.gamma_gain;
            let r = r.powf(gamma);
            let g = g.powf(gamma);
            let b = b.powf(gamma);
        }
        
        // Apply gain (highlight adjustment)
        let r = r * self.gain;
        let g = g * self.gain;
        let b = b * self.gain;
        
        (r, g, b)
    }
    
    /// Apply brightness and contrast adjustments
    fn apply_brightness_contrast(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        // Apply brightness
        let r = r + self.brightness;
        let g = g + self.brightness;
        let b = b + self.brightness;
        
        // Apply contrast
        let r = (r - 0.5) * self.contrast + 0.5;
        let g = (g - 0.5) * self.contrast + 0.5;
        let b = (b - 0.5) * self.contrast + 0.5;
        
        (r, g, b)
    }
    
    /// Apply gamma correction
    fn apply_gamma_correction(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        if self.gamma > 0.0 {
            let gamma = 1.0 / self.gamma;
            (
                r.powf(gamma),
                g.powf(gamma),
                b.powf(gamma)
            )
        } else {
            (r, g, b)
        }
    }
    
    /// Apply hue and saturation adjustments
    fn apply_hue_saturation(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        // Convert RGB to HSL
        let (h, s, l) = self.rgb_to_hsl(r, g, b);
        
        // Apply hue shift
        let h = (h + self.hue / 360.0).rem_euclid(1.0);
        
        // Apply saturation adjustment
        let s = (s * self.saturation).clamp(0.0, 1.0);
        
        // Convert back to RGB
        self.hsl_to_rgb(h, s, l)
    }
    
    /// Convert RGB to HSL color space
    fn rgb_to_hsl(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        
        if max == min {
            (0.0, 0.0, l) // Achromatic
        } else {
            let d = max - min;
            let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
            
            let h = match max {
                x if x == r => ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0,
                x if x == g => ((b - r) / d + 2.0) / 6.0,
                _ => ((r - g) / d + 4.0) / 6.0,
            };
            
            (h, s, l)
        }
    }
    
    /// Convert HSL to RGB color space
    fn hsl_to_rgb(&self, h: f32, s: f32, l: f32) -> (f32, f32, f32) {
        if s == 0.0 {
            (l, l, l) // Achromatic
        } else {
            let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
            let p = 2.0 * l - q;
            
            let r = self.hue_to_rgb(p, q, h + 1.0 / 3.0);
            let g = self.hue_to_rgb(p, q, h);
            let b = self.hue_to_rgb(p, q, h - 1.0 / 3.0);
            
            (r, g, b)
        }
    }
    
    /// Helper function for HSL to RGB conversion
    fn hue_to_rgb(&self, p: f32, q: f32, t: f32) -> f32 {
        let t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 1.0 / 2.0 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
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
    fn test_pixel_correction() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);
        
        // Test basic brightness adjustment
        color_node.set_brightness(0.2);
        let (r, g, b) = color_node.apply_pixel_correction(0.5, 0.5, 0.5);
        assert!(r > 0.5 && g > 0.5 && b > 0.5); // Should be brighter
        
        // Test contrast adjustment
        color_node.set_brightness(0.0);
        color_node.set_contrast(2.0);
        let (r, g, b) = color_node.apply_pixel_correction(0.25, 0.75, 0.5);
        assert!(r < 0.25); // Should be darker (below 0.5)
        assert!(g > 0.75); // Should be brighter (above 0.5)
        assert_eq!(b, 0.5); // Should stay the same (exactly 0.5)
    }
    
    #[test]
    fn test_white_balance() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let color_node = ColorCorrectionNode::new(node);
        
        let (r, g, b) = color_node.apply_white_balance(0.5, 0.5, 0.5);
        // With default temperature (6500K), should be roughly equal
        assert!((r - g).abs() < 0.1);
        assert!((g - b).abs() < 0.1);
        
        // Test with warm temperature
        let mut warm_node = ColorCorrectionNode::new(node);
        warm_node.set_temperature(3200.0);
        let (r, g, b) = warm_node.apply_white_balance(0.5, 0.5, 0.5);
        assert!(r > b); // Warm light should have more red than blue
    }
    
    #[test]
    fn test_gamma_correction() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);
        
        // Test gamma > 1.0 (darkens midtones)
        color_node.set_gamma(2.0);
        let (r, g, b) = color_node.apply_gamma_correction(0.5, 0.5, 0.5);
        assert!(r < 0.5 && g < 0.5 && b < 0.5); // Should be darker
        
        // Test gamma < 1.0 (brightens midtones)
        color_node.set_gamma(0.5);
        let (r, g, b) = color_node.apply_gamma_correction(0.5, 0.5, 0.5);
        assert!(r > 0.5 && g > 0.5 && b > 0.5); // Should be brighter
    }
    
    #[test]
    fn test_hue_saturation() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);
        
        // Test saturation increase
        color_node.set_saturation(2.0);
        let (r, g, b) = color_node.apply_hue_saturation(0.5, 0.25, 0.75);
        // Saturation should make colors more vivid (difference should increase)
        assert!(r > 0.5 && b < 0.75); // More saturated
        
        // Test hue shift
        color_node.set_saturation(1.0);
        color_node.set_hue(180.0); // 180 degree hue shift
        let (r, g, b) = color_node.apply_hue_saturation(1.0, 0.0, 0.0); // Pure red
        assert!(r < 0.5 && b > 0.5); // Should shift towards cyan
    }
    
    #[test]
    fn test_rgb_hsl_conversion() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let color_node = ColorCorrectionNode::new(node);
        
        // Test pure colors
        let (h, s, l) = color_node.rgb_to_hsl(1.0, 0.0, 0.0); // Red
        assert!((h - 0.0).abs() < 0.1); // Red hue
        assert_eq!(s, 1.0); // Full saturation
        assert_eq!(l, 0.5); // Normal lightness
        
        let (h, s, l) = color_node.rgb_to_hsl(0.0, 1.0, 0.0); // Green
        assert!((h - 1.0/3.0).abs() < 0.1); // Green hue
        
        let (h, s, l) = color_node.rgb_to_hsl(0.0, 0.0, 1.0); // Blue
        assert!((h - 2.0/3.0).abs() < 0.1); // Blue hue
        
        // Test gray (no saturation)
        let (h, s, l) = color_node.rgb_to_hsl(0.5, 0.5, 0.5);
        assert_eq!(s, 0.0); // No saturation
        assert_eq!(l, 0.5); // Normal lightness
        
        // Test round-trip conversion
        let (r, g, b) = (0.7, 0.3, 0.9);
        let (h, s, l) = color_node.rgb_to_hsl(r, g, b);
        let (r2, g2, b2) = color_node.hsl_to_rgb(h, s, l);
        assert!((r - r2).abs() < 0.01);
        assert!((g - g2).abs() < 0.01);
        assert!((b - b2).abs() < 0.01);
    }
    
    #[test]
    fn test_lift_gamma_gain() {
        let node = ColorCorrectionNode::create_standard("Test".to_string());
        let mut color_node = ColorCorrectionNode::new(node);
        
        // Test lift (shadows)
        color_node.set_lift(0.2);
        let (r, g, b) = color_node.apply_lift_gamma_gain(0.1, 0.1, 0.1);
        assert!(r > 0.1 && g > 0.1 && b > 0.1); // Should be lifted
        
        // Test gain (highlights)
        color_node.set_lift(0.0);
        color_node.set_gain(1.5);
        let (r, g, b) = color_node.apply_lift_gamma_gain(0.8, 0.8, 0.8);
        assert!(r > 0.8 && g > 0.8 && b > 0.8); // Should be boosted
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
    
    #[test]
    fn test_corrected_image_metadata() {
        let corrected = CorrectedImage {
            original_id: Uuid::new_v4(),
            corrected_id: Uuid::new_v4(),
            brightness: 0.5,
            contrast: 1.2,
            saturation: 1.1,
            gamma: 1.0,
            temperature: 6500.0,
            tint: 0.0,
            hue: 0.0,
            lift: 0.0,
            gamma_gain: 1.0,
            gain: 1.0,
        };
        
        assert_eq!(corrected.brightness, 0.5);
        assert_eq!(corrected.contrast, 1.2);
        assert_eq!(corrected.saturation, 1.1);
        assert_eq!(corrected.temperature, 6500.0);
    }
}
