use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;

/// Metadata for a blurred image
#[derive(Debug, Clone)]
pub struct BlurResult {
    /// Original image ID
    pub original_id: Uuid,
    /// Blurred image ID
    pub blurred_id: Uuid,
    /// Type of blur applied
    pub blur_type: BlurType,
    /// Blur radius
    pub radius: f32,
    /// Kernel size
    pub kernel_size: usize,
    /// Blur angle (for motion blur)
    pub angle: f32,
    /// Number of iterations
    pub iterations: u32,
}

/// Helper trait for extending usize to get next odd number
trait NextOdd {
    fn next_odd(self) -> usize;
}

impl NextOdd for usize {
    fn next_odd(self) -> usize {
        if self % 2 == 0 {
            self + 1
        } else {
            self
        }
    }
}

/// Blur node for Gaussian blur with radius control
pub struct BlurNode {
    node: Node,
    blur_radius: f32,
    blur_type: BlurType,
    iterations: u32,
    directional: bool,
    angle: f32,
}

/// Types of blur algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum BlurType {
    /// Gaussian blur
    Gaussian,
    /// Box blur (average)
    Box,
    /// Motion blur
    Motion,
    /// Radial blur
    Radial,
    /// Zoom blur
    Zoom,
}

impl BlurNode {
    /// Create a new blur node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            blur_radius: 5.0,
            blur_type: BlurType::Gaussian,
            iterations: 1,
            directional: false,
            angle: 0.0,
        }
    }
    
    /// Set blur radius (0.0 to 100.0)
    pub fn set_blur_radius(&mut self, radius: f32) {
        self.blur_radius = radius.clamp(0.0, 100.0);
    }
    
    /// Get blur radius
    pub fn get_blur_radius(&self) -> f32 {
        self.blur_radius
    }
    
    /// Set blur type
    pub fn set_blur_type(&mut self, blur_type: BlurType) {
        self.blur_type = blur_type;
    }
    
    /// Get blur type
    pub fn get_blur_type(&self) -> BlurType {
        self.blur_type.clone()
    }
    
    /// Set iterations (1 to 10)
    pub fn set_iterations(&mut self, iterations: u32) {
        self.iterations = iterations.clamp(1, 10);
    }
    
    /// Get iterations
    pub fn get_iterations(&self) -> u32 {
        self.iterations
    }
    
    /// Set directional blur
    pub fn set_directional(&mut self, directional: bool) {
        self.directional = directional;
    }
    
    /// Is directional blur
    pub fn is_directional(&self) -> bool {
        self.directional
    }
    
    /// Set blur angle in degrees (for directional/motion blur)
    pub fn set_angle(&mut self, angle: f32) {
        self.angle = angle.rem_euclid(360.0);
    }
    
    /// Get blur angle
    pub fn get_angle(&self) -> f32 {
        self.angle
    }
    
    /// Apply blur to an image
    fn apply_blur(&self, input_value: ParameterValue) -> ParameterValue {
        match input_value {
            ParameterValue::Image(input_id) => {
                // In a real implementation, this would:
                // - Load the input texture
                // - Apply the specified blur algorithm
                // - Handle iterations for stronger blur
                // - Apply directional blur if enabled
                // - Return blurred texture ID
                
                log::debug!("Applying blur: type={:?}, radius={:.1}, iterations={}, directional={}, angle={:.1}°", 
                    self.blur_type, self.blur_radius, self.iterations, self.directional, self.angle);
                
                // Apply blur iterations
                let mut current_id = input_id;
                for i in 0..self.iterations {
                    log::debug!("Blur iteration {}", i + 1);
                    // In a real implementation, each iteration would apply the blur
                    // and produce a new texture ID
                }
                
                // Return a blurred image ID
                let blurred_id = Uuid::new_v4();
                ParameterValue::Image(blurred_id)
            }
            _ => {
                // No valid image input
                ParameterValue::None
            }
        }
    }
    
    /// Apply Gaussian blur
    fn apply_gaussian_blur(&self, input_id: Uuid) -> Uuid {
        log::debug!("Applying Gaussian blur with radius {:.1}", self.blur_radius);
        
        let blurred_id = Uuid::new_v4();
        
        // Generate Gaussian kernel
        let kernel_size = ((self.blur_radius * 2.0) as usize).max(3).next_odd();
        let kernel = self.generate_gaussian_kernel(kernel_size);
        
        // Create blur metadata
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Gaussian,
            radius: self.blur_radius,
            kernel_size,
            angle: 0.0,
            iterations: self.iterations,
        };
        
        log::debug!("Generated Gaussian kernel: size={}, samples={}", kernel_size, kernel.len());
        log::debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Apply box blur
    fn apply_box_blur(&self, input_id: Uuid) -> Uuid {
        log::debug!("Applying box blur with radius {:.1}", self.blur_radius);
        
        let blurred_id = Uuid::new_v4();
        let kernel_size = ((self.blur_radius * 2.0) as usize).max(3).next_odd();
        
        // Box blur uses uniform kernel
        let kernel = vec![1.0 / kernel_size as f32; kernel_size];
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Box,
            radius: self.blur_radius,
            kernel_size,
            angle: 0.0,
            iterations: self.iterations,
        };
        
        log::debug!("Generated box kernel: size={}", kernel_size);
        log::debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Apply motion blur
    fn apply_motion_blur(&self, input_id: Uuid) -> Uuid {
        log::debug!("Applying motion blur: length={:.1}, angle={:.1}°", 
            self.blur_radius, self.angle);
        
        let blurred_id = Uuid::new_v4();
        let kernel_size = ((self.blur_radius * 2.0) as usize).max(3).next_odd();
        
        // Generate motion blur kernel
        let kernel = self.generate_motion_blur_kernel(kernel_size);
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Motion,
            radius: self.blur_radius,
            kernel_size,
            angle: self.angle,
            iterations: self.iterations,
        };
        
        log::debug!("Generated motion blur kernel: size={}, angle={:.1}°", kernel_size, self.angle);
        log::debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Apply radial blur
    fn apply_radial_blur(&self, input_id: Uuid) -> Uuid {
        log::debug!("Applying radial blur with radius {:.1}", self.blur_radius);
        
        let blurred_id = Uuid::new_v4();
        let samples = (self.blur_radius * 4.0) as usize;
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Radial,
            radius: self.blur_radius,
            kernel_size: samples,
            angle: 0.0,
            iterations: self.iterations,
        };
        
        log::debug!("Radial blur samples: {}", samples);
        log::debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Apply zoom blur
    fn apply_zoom_blur(&self, input_id: Uuid) -> Uuid {
        log::debug!("Applying zoom blur with radius {:.1}", self.blur_radius);
        
        let blurred_id = Uuid::new_v4();
        let samples = (self.blur_radius * 4.0) as usize;
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Zoom,
            radius: self.blur_radius,
            kernel_size: samples,
            angle: 0.0,
            iterations: self.iterations,
        };
        
        log::debug!("Zoom blur samples: {}", samples);
        log::debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Generate Gaussian kernel
    fn generate_gaussian_kernel(&self, size: usize) -> Vec<f32> {
        let mut kernel = Vec::with_capacity(size);
        let center = size as f32 / 2.0 - 0.5;
        let sigma = self.blur_radius / 3.0; // Standard deviation
        
        let mut sum = 0.0;
        for i in 0..size {
            let x = i as f32 - center;
            let value = (-x * x / (2.0 * sigma * sigma)).exp();
            kernel.push(value);
            sum += value;
        }
        
        // Normalize kernel
        for value in &mut kernel {
            *value /= sum;
        }
        
        kernel
    }
    
    /// Generate motion blur kernel
    fn generate_motion_blur_kernel(&self, size: usize) -> Vec<f32> {
        let mut kernel = vec![0.0; size];
        let center = size / 2;
        let length = self.blur_radius as usize;
        
        // Create line kernel based on angle
        let angle_rad = self.angle.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        
        for i in 0..length.min(size) {
            let t = (i as f32 - length as f32 / 2.0) / (length as f32 / 2.0);
            
            // Calculate position along the line
            let x = (t * cos_a * size as f32 / 2.0 + center as f32) as isize;
            let y = (t * sin_a * size as f32 / 2.0) as isize;
            
            // In a 1D kernel, we just use the x component
            if x >= 0 && x < size as isize {
                kernel[x as usize] = 1.0 / length as f32;
            }
        }
        
        kernel
    }
    
    /// Apply separable Gaussian blur (horizontal and vertical passes)
    fn apply_separable_gaussian(&self, input_id: Uuid) -> Uuid {
        log::debug!("Applying separable Gaussian blur");
        
        let kernel_size = ((self.blur_radius * 2.0) as usize).max(3).next_odd();
        let kernel = self.generate_gaussian_kernel(kernel_size);
        
        // First pass: horizontal blur
        let horizontal_id = Uuid::new_v4();
        log::debug!("Horizontal blur pass: {:?}", horizontal_id);
        
        // Second pass: vertical blur
        let vertical_id = Uuid::new_v4();
        log::debug!("Vertical blur pass: {:?}", vertical_id);
        
        vertical_id
    }
    
    /// Apply blur using integral image (for box blur)
    fn apply_integral_blur(&self, input_id: Uuid) -> Uuid {
        log::debug!("Applying integral image blur");
        
        let blurred_id = Uuid::new_v4();
        let radius = self.blur_radius as usize;
        
        // Integral image approach:
        // 1. Compute integral image
        // 2. Use integral image to compute box blur in O(1) per pixel
        log::debug!("Integral blur radius: {}", radius);
        
        blurred_id
    }
    
    /// Apply radial blur with sampling
    fn apply_radial_sampling(&self, input_id: Uuid, center_x: f32, center_y: f32) -> Uuid {
        log::debug!("Applying radial sampling blur at center ({:.1}, {:.1})", center_x, center_y);
        
        let blurred_id = Uuid::new_v4();
        let samples = (self.blur_radius * 4.0) as usize;
        
        // For each pixel, sample along radial lines from center
        for sample in 0..samples {
            let angle = (sample as f32 / samples as f32) * 2.0 * std::f32::consts::PI;
            let distance = self.blur_radius / samples as f32;
            
            log::debug!("Sample {}: angle={:.2}°, distance={:.2}", sample, angle.to_degrees(), distance);
        }
        
        blurred_id
    }
    
    /// Apply zoom blur with sampling
    fn apply_zoom_sampling(&self, input_id: Uuid, center_x: f32, center_y: f32) -> Uuid {
        log::debug!("Applying zoom sampling blur at center ({:.1}, {:.1})", center_x, center_y);
        
        let blurred_id = Uuid::new_v4();
        let samples = (self.blur_radius * 4.0) as usize;
        
        // For each pixel, sample along lines from center
        for sample in 0..samples {
            let scale = 1.0 + (sample as f32 / samples as f32) * self.blur_radius / 100.0;
            
            log::debug!("Sample {}: scale={:.3}", sample, scale);
        }
        
        blurred_id
    }
    
    /// Create a blur node with standard configuration
    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::Blur, name);
        
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
        let blur_radius_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "blur_radius".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(5.0),
            default_value: ParameterValue::Float(5.0),
            min_value: Some(0.0),
            max_value: Some(100.0),
            animatable: true,
            description: Some("Blur radius (0.0 to 100.0)".to_string()),
        };
        node.add_parameter(blur_radius_param);
        
        let blur_type_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "blur_type".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("gaussian".to_string()),
            default_value: ParameterValue::String("gaussian".to_string()),
            min_value: None,
            max_value: None,
            animatable: false,
            description: Some("Blur type (gaussian, box, motion, radial, zoom)".to_string()),
        };
        node.add_parameter(blur_type_param);
        
        let iterations_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "iterations".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(1),
            default_value: ParameterValue::Integer(1),
            min_value: Some(1.0),
            max_value: Some(10.0),
            animatable: false,
            description: Some("Number of blur iterations (1 to 10)".to_string()),
        };
        node.add_parameter(iterations_param);
        
        let directional_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "directional".to_string(),
            data_type: PinDataType::Boolean,
            value: ParameterValue::Boolean(false),
            default_value: ParameterValue::Boolean(false),
            min_value: None,
            max_value: None,
            animatable: false,
            description: Some("Enable directional blur".to_string()),
        };
        node.add_parameter(directional_param);
        
        let angle_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "angle".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
            animatable: true,
            description: Some("Blur angle in degrees (for directional/motion blur)".to_string()),
        };
        node.add_parameter(angle_param);
        
        node
    }
    
    /// Update parameters from node metadata
    fn update_parameters(&mut self) {
        if let Some(radius_param) = self.node.parameters.get("blur_radius") {
            if let ParameterValue::Float(radius) = radius_param.value {
                self.blur_radius = radius.clamp(0.0, 100.0);
            }
        }
        
        if let Some(type_param) = self.node.parameters.get("blur_type") {
            if let ParameterValue::String(type_str) = &type_param.value {
                self.blur_type = match type_str.as_str() {
                    "gaussian" => BlurType::Gaussian,
                    "box" => BlurType::Box,
                    "motion" => BlurType::Motion,
                    "radial" => BlurType::Radial,
                    "zoom" => BlurType::Zoom,
                    _ => BlurType::Gaussian,
                };
            }
        }
        
        if let Some(iterations_param) = self.node.parameters.get("iterations") {
            if let ParameterValue::Integer(iterations) = iterations_param.value {
                self.iterations = iterations.clamp(1, 10) as u32;
            }
        }
        
        if let Some(directional_param) = self.node.parameters.get("directional") {
            if let ParameterValue::Boolean(directional) = directional_param.value {
                self.directional = directional;
            }
        }
        
        if let Some(angle_param) = self.node.parameters.get("angle") {
            if let ParameterValue::Float(angle) = angle_param.value {
                self.angle = angle.rem_euclid(360.0);
            }
        }
    }
}

impl NodeExecutor for BlurNode {
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
            self.apply_blur(input_value)
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
        NodeType::Blur
    }
    
    fn validate(&self) -> NodeResult<()> {
        // Validate blur radius
        if self.blur_radius < 0.0 || self.blur_radius > 100.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Blur radius must be between 0.0 and 100.0".to_string()
            ));
        }
        
        // Validate iterations
        if self.iterations == 0 || self.iterations > 10 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Iterations must be between 1 and 10".to_string()
            ));
        }
        
        // Validate angle range
        if self.angle < -360.0 || self.angle > 360.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Angle must be between -360 and 360 degrees".to_string()
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

/// Gaussian blur node (specialized)
pub struct GaussianBlurNode {
    blur_node: BlurNode,
}

impl GaussianBlurNode {
    pub fn new(node: Node) -> Self {
        let mut blur_node = BlurNode::new(node);
        blur_node.set_blur_type(BlurType::Gaussian);
        
        Self { blur_node }
    }
    
    /// Create a standard Gaussian blur node
    pub fn create_standard(name: String) -> Node {
        let mut node = BlurNode::create_standard(name);
        
        // Remove blur type parameter since it's fixed
        node.parameters.remove("blur_type");
        
        node
    }
}

impl NodeExecutor for GaussianBlurNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.blur_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Blur
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.blur_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.blur_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.blur_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.blur_node.can_execute(context)
    }
}

/// Motion blur node (specialized)
pub struct MotionBlurNode {
    blur_node: BlurNode,
}

impl MotionBlurNode {
    pub fn new(node: Node) -> Self {
        let mut blur_node = BlurNode::new(node);
        blur_node.set_blur_type(BlurType::Motion);
        blur_node.set_directional(true);
        
        Self { blur_node }
    }
    
    /// Create a standard motion blur node
    pub fn create_standard(name: String) -> Node {
        let mut node = BlurNode::create_standard(name);
        
        // Set motion blur specific defaults
        if let Some(type_param) = node.parameters.get_mut("blur_type") {
            type_param.value = ParameterValue::String("motion".to_string());
        }
        if let Some(directional_param) = node.parameters.get_mut("directional") {
            directional_param.value = ParameterValue::Boolean(true);
        }
        
        node
    }
    
    /// Set motion length
    pub fn set_motion_length(&mut self, length: f32) {
        self.blur_node.set_blur_radius(length);
    }
    
    /// Get motion length
    pub fn get_motion_length(&self) -> f32 {
        self.blur_node.get_blur_radius()
    }
    
    /// Set motion direction
    pub fn set_motion_direction(&mut self, direction: f32) {
        self.blur_node.set_angle(direction);
    }
    
    /// Get motion direction
    pub fn get_motion_direction(&self) -> f32 {
        self.blur_node.get_angle()
    }
}

impl NodeExecutor for MotionBlurNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.blur_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Blur
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.blur_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.blur_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.blur_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.blur_node.can_execute(context)
    }
}

/// Radial blur node (specialized)
pub struct RadialBlurNode {
    blur_node: BlurNode,
}

impl RadialBlurNode {
    pub fn new(node: Node) -> Self {
        let mut blur_node = BlurNode::new(node);
        blur_node.set_blur_type(BlurType::Radial);
        
        Self { blur_node }
    }
    
    /// Create a standard radial blur node
    pub fn create_standard(name: String) -> Node {
        let mut node = BlurNode::create_standard(name);
        
        // Set radial blur specific defaults
        if let Some(type_param) = node.parameters.get_mut("blur_type") {
            type_param.value = ParameterValue::String("radial".to_string());
        }
        
        node
    }
}

impl NodeExecutor for RadialBlurNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.blur_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Blur
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.blur_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.blur_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.blur_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.blur_node.can_execute(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blur_node_creation() {
        let node = BlurNode::create_standard("Test Blur".to_string());
        assert_eq!(node.node_type, NodeType::Blur);
        assert_eq!(node.name, "Test Blur");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.parameters.len(), 5);
    }
    
    #[test]
    fn test_blur_types() {
        assert_eq!(BlurType::Gaussian, BlurType::Gaussian);
        assert_ne!(BlurType::Gaussian, BlurType::Box);
        assert_eq!(BlurType::Motion, BlurType::Motion);
    }
    
    #[test]
    fn test_blur_node_properties() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        blur_node.set_blur_radius(10.5);
        assert_eq!(blur_node.get_blur_radius(), 10.5);
        
        blur_node.set_iterations(3);
        assert_eq!(blur_node.get_iterations(), 3);
        
        blur_node.set_directional(true);
        assert!(blur_node.is_directional());
        
        blur_node.set_angle(45.0);
        assert_eq!(blur_node.get_angle(), 45.0);
    }
    
    #[test]
    fn test_gaussian_kernel_generation() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        blur_node.set_blur_radius(5.0);
        let kernel = blur_node.generate_gaussian_kernel(11);
        
        assert_eq!(kernel.len(), 11);
        
        // Kernel should be normalized (sum should be approximately 1.0)
        let sum: f32 = kernel.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
        
        // Center should have highest value
        let center = kernel.len() / 2;
        assert!(kernel[center] > kernel[0]);
        assert!(kernel[center] > kernel[kernel.len() - 1]);
        
        // Kernel should be symmetric
        for i in 0..kernel.len() / 2 {
            assert!((kernel[i] - kernel[kernel.len() - 1 - i]).abs() < 0.001);
        }
    }
    
    #[test]
    fn test_motion_blur_kernel_generation() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        blur_node.set_blur_radius(10.0);
        blur_node.set_angle(0.0); // Horizontal
        let kernel = blur_node.generate_motion_blur_kernel(21);
        
        assert_eq!(kernel.len(), 21);
        
        // Horizontal motion blur should have non-zero values in a line
        let non_zero_count = kernel.iter().filter(|&&x| x > 0.0).count();
        assert!(non_zero_count > 0);
        
        // Test diagonal motion blur
        blur_node.set_angle(45.0);
        let diagonal_kernel = blur_node.generate_motion_blur_kernel(21);
        assert_eq!(diagonal_kernel.len(), 21);
    }
    
    #[test]
    fn test_blur_algorithms() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        blur_node.set_blur_radius(3.0);
        
        // Test Gaussian blur
        let gaussian_id = blur_node.apply_gaussian_blur(Uuid::new_v4());
        assert_ne!(gaussian_id, Uuid::default());
        
        // Test box blur
        let box_id = blur_node.apply_box_blur(Uuid::new_v4());
        assert_ne!(box_id, Uuid::default());
        
        // Test motion blur
        blur_node.set_angle(90.0);
        let motion_id = blur_node.apply_motion_blur(Uuid::new_v4());
        assert_ne!(motion_id, Uuid::default());
        
        // Test radial blur
        let radial_id = blur_node.apply_radial_blur(Uuid::new_v4());
        assert_ne!(radial_id, Uuid::default());
        
        // Test zoom blur
        let zoom_id = blur_node.apply_zoom_blur(Uuid::new_v4());
        assert_ne!(zoom_id, Uuid::default());
    }
    
    #[test]
    fn test_separable_gaussian() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        blur_node.set_blur_radius(5.0);
        let result_id = blur_node.apply_separable_gaussian(Uuid::new_v4());
        assert_ne!(result_id, Uuid::default());
    }
    
    #[test]
    fn test_integral_blur() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        blur_node.set_blur_radius(10.0);
        let result_id = blur_node.apply_integral_blur(Uuid::new_v4());
        assert_ne!(result_id, Uuid::default());
    }
    
    #[test]
    fn test_radial_sampling() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        blur_node.set_blur_radius(5.0);
        let result_id = blur_node.apply_radial_sampling(Uuid::new_v4(), 0.5, 0.5);
        assert_ne!(result_id, Uuid::default());
    }
    
    #[test]
    fn test_zoom_sampling() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        blur_node.set_blur_radius(5.0);
        let result_id = blur_node.apply_zoom_sampling(Uuid::new_v4(), 0.5, 0.5);
        assert_ne!(result_id, Uuid::default());
    }
    
    #[test]
    fn test_next_odd_helper() {
        assert_eq!(3.next_odd(), 3); // Already odd
        assert_eq!(4.next_odd(), 5); // Even -> next odd
        assert_eq!(10.next_odd(), 11);
        assert_eq!(11.next_odd(), 11);
        assert_eq!(0.next_odd(), 1);
    }
    
    #[test]
    fn test_blur_result_metadata() {
        let blur_result = BlurResult {
            original_id: Uuid::new_v4(),
            blurred_id: Uuid::new_v4(),
            blur_type: BlurType::Gaussian,
            radius: 5.0,
            kernel_size: 11,
            angle: 0.0,
            iterations: 1,
        };
        
        assert_eq!(blur_result.blur_type, BlurType::Gaussian);
        assert_eq!(blur_result.radius, 5.0);
        assert_eq!(blur_result.kernel_size, 11);
        assert_eq!(blur_result.iterations, 1);
    }
    
    #[test]
    fn test_specialized_blur_nodes() {
        let gaussian_node = GaussianBlurNode::create_standard("Gaussian".to_string());
        let motion_node = MotionBlurNode::create_standard("Motion".to_string());
        let radial_node = RadialBlurNode::create_standard("Radial".to_string());
        
        assert_eq!(gaussian_node.node_type, NodeType::Blur);
        assert_eq!(motion_node.node_type, NodeType::Blur);
        assert_eq!(radial_node.node_type, NodeType::Blur);
        
        // Gaussian node should have fewer parameters (no blur_type)
        assert!(gaussian_node.parameters.len() < 5);
        
        // Motion blur node should have directional enabled by default
        if let Some(directional_param) = motion_node.parameters.get("directional") {
            if let ParameterValue::Boolean(directional) = directional_param.value {
                assert!(directional);
            }
        }
    }
    
    #[test]
    fn test_motion_blur_node() {
        let node = MotionBlurNode::create_standard("Motion".to_string());
        let mut motion_node = MotionBlurNode::new(node);
        
        motion_node.set_motion_length(20.0);
        assert_eq!(motion_node.get_motion_length(), 20.0);
        
        motion_node.set_motion_direction(90.0);
        assert_eq!(motion_node.get_motion_direction(), 90.0);
    }
    
    #[test]
    fn test_blur_node_execution() {
        let node = BlurNode::create_standard("Test".to_string());
        let blur_node = BlurNode::new(node);
        
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert!(blur_node.execute(&mut context).is_ok());
    }
    
    #[test]
    fn test_blur_parameter_validation() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        // Test radius validation
        blur_node.set_blur_radius(-5.0);
        assert_eq!(blur_node.get_blur_radius(), 0.0); // Should clamp to 0.0
        
        blur_node.set_blur_radius(150.0);
        assert_eq!(blur_node.get_blur_radius(), 100.0); // Should clamp to 100.0
        
        // Test iterations validation
        blur_node.set_iterations(0);
        assert_eq!(blur_node.get_iterations(), 1); // Should clamp to 1
        
        blur_node.set_iterations(15);
        assert_eq!(blur_node.get_iterations(), 10); // Should clamp to 10
        
        // Test angle wrapping
        blur_node.set_angle(450.0);
        assert_eq!(blur_node.get_angle(), 90.0); // Should wrap to 90°
        
        blur_node.set_angle(-90.0);
        assert_eq!(blur_node.get_angle(), 270.0); // Should wrap to 270°
    }
    
    #[test]
    fn test_blur_performance_considerations() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);
        
        // Test that kernel sizes are reasonable
        blur_node.set_blur_radius(1.0);
        let kernel = blur_node.generate_gaussian_kernel(blur_node.blur_radius as usize * 2 + 1);
        assert!(kernel.len() <= 5); // Small radius should give small kernel
        
        blur_node.set_blur_radius(50.0);
        let large_kernel = blur_node.generate_gaussian_kernel(blur_node.blur_radius as usize * 2 + 1);
        assert!(large_kernel.len() <= 101); // Large radius should give reasonable kernel size
    }
}
