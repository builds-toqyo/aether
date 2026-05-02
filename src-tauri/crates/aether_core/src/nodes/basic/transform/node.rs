use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use crate::nodes::basic::transform::{TransformParams, TransformOperations};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;
use log::debug;

/// Transform node for position, scale, rotation transforms
pub struct TransformNode {
    node: Node,
    params: TransformParams,
    operations: TransformOperations,
}

impl TransformNode {
    /// Create a new transform node
    pub fn new(node: Node) -> Self {
        let params = TransformParams::default();
        let operations = TransformOperations::new(params.clone());
        
        Self {
            node,
            params,
            operations,
        }
    }
    
    /// Set position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.params.set_position(x, y);
        self.operations.update_params(self.params.clone());
    }
    
    /// Get position
    pub fn get_position(&self) -> (f32, f32) {
        self.params.get_position()
    }
    
    /// Set scale
    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.params.set_scale(x, y);
        self.operations.update_params(self.params.clone());
    }
    
    /// Get scale
    pub fn get_scale(&self) -> (f32, f32) {
        self.params.get_scale()
    }
    
    /// Set uniform scale
    pub fn set_uniform_scale(&mut self, scale: f32) {
        self.params.set_uniform_scale(scale);
        self.operations.update_params(self.params.clone());
    }
    
    /// Get uniform scale
    pub fn get_uniform_scale(&self) -> f32 {
        self.params.get_uniform_scale()
    }
    
    /// Set rotation in degrees
    pub fn set_rotation(&mut self, rotation: f32) {
        self.params.set_rotation(rotation);
        self.operations.update_params(self.params.clone());
    }
    
    /// Get rotation in degrees
    pub fn get_rotation(&self) -> f32 {
        self.params.get_rotation()
    }
    
    /// Get rotation in radians
    pub fn get_rotation_rad(&self) -> f32 {
        self.params.get_rotation_rad()
    }
    
    /// Set anchor point
    pub fn set_anchor(&mut self, x: f32, y: f32) {
        self.params.set_anchor(x, y);
        self.operations.update_params(self.params.clone());
    }
    
    /// Get anchor point
    pub fn get_anchor(&self) -> (f32, f32) {
        self.params.get_anchor()
    }
    
    /// Set uniform scale flag
    pub fn set_uniform_scale_flag(&mut self, uniform: bool) {
        self.params.set_uniform_scale_flag(uniform);
        self.operations.update_params(self.params.clone());
    }
    
    /// Check if scale is uniform
    pub fn is_uniform_scale(&self) -> bool {
        self.params.is_uniform_scale()
    }
    
    /// Set all parameters at once
    pub fn set_params(&mut self, params: TransformParams) {
        self.params = params.clone();
        self.params.validate();
        self.operations.update_params(self.params.clone());
    }
    
    /// Get all parameters
    pub fn get_params(&self) -> &TransformParams {
        &self.params
    }
    
    /// Reset to identity transform
    pub fn reset(&mut self) {
        self.params.reset();
        self.operations.update_params(self.params.clone());
    }
    
    /// Check if transform is active
    pub fn is_active(&self) -> bool {
        self.params.is_active()
    }
    
    /// Transform a point
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        self.operations.transform_point(x, y)
    }
    
    /// Transform a vector
    pub fn transform_vector(&self, x: f32, y: f32) -> (f32, f32) {
        self.operations.transform_vector(x, y)
    }
    
    /// Get bounding box
    pub fn get_bounding_box(&self, width: f32, height: f32) -> ((f32, f32), (f32, f32)) {
        self.operations.get_bounding_box(width, height)
    }
    
    /// Create a standard transform node
    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::Transform, name);
        
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
        let position_x_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "position_x".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(position_x_param);
        
        let position_y_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "position_y".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(position_y_param);
        
        let scale_x_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "scale_x".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(ParameterValue::Float(0.001)),
            max_value: Some(ParameterValue::Float(1000.0)),
        };
        node.add_parameter(scale_x_param);
        
        let scale_y_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "scale_y".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(ParameterValue::Float(0.001)),
            max_value: Some(ParameterValue::Float(1000.0)),
        };
        node.add_parameter(scale_y_param);
        
        let rotation_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "rotation".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(rotation_param);
        
        let anchor_x_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "anchor_x".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(anchor_x_param);
        
        let anchor_y_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "anchor_y".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(anchor_y_param);
        
        let uniform_scale_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "uniform_scale".to_string(),
            data_type: PinDataType::Boolean,
            value: ParameterValue::Boolean(true),
            default_value: ParameterValue::Boolean(true),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(uniform_scale_param);
        
        node
    }
}

impl NodeExecutor for TransformNode {
    fn execute(&mut self, context: &ExecutionContext) -> NodeResult {
        // Get input value
        let input_value = self.node.get_input_value("input", context);
        
        // Apply transform using operations
        let output_value = self.operations.apply_transform(input_value);
        
        // Set output value
        self.node.set_output_value("output", output_value);
        
        Ok(())
    }
    
    fn get_node(&self) -> &Node {
        &self.node
    }
    
    fn get_node_mut(&mut self) -> &mut Node {
        &mut self.node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transform_node_creation() {
        let node = TransformNode::create_standard("Test Transform".to_string());
        let transform_node = TransformNode::new(node);
        
        assert_eq!(transform_node.get_position(), (0.0, 0.0));
        assert_eq!(transform_node.get_scale(), (1.0, 1.0));
        assert_eq!(transform_node.get_rotation(), 0.0);
        assert_eq!(transform_node.get_anchor(), (0.0, 0.0));
        assert!(transform_node.is_uniform_scale());
    }
    
    #[test]
    fn test_parameter_setting() {
        let node = TransformNode::create_standard("Test".to_string());
        let mut transform_node = TransformNode::new(node);
        
        transform_node.set_position(100.0, 200.0);
        assert_eq!(transform_node.get_position(), (100.0, 200.0));
        
        transform_node.set_scale(2.0, 3.0);
        assert_eq!(transform_node.get_scale(), (2.0, 3.0));
        
        transform_node.set_uniform_scale(1.5);
        assert_eq!(transform_node.get_scale(), (1.5, 1.5));
        
        transform_node.set_rotation(45.0);
        assert_eq!(transform_node.get_rotation(), 45.0);
        
        transform_node.set_anchor(50.0, 75.0);
        assert_eq!(transform_node.get_anchor(), (50.0, 75.0));
        
        transform_node.set_uniform_scale_flag(false);
        assert!(!transform_node.is_uniform_scale());
    }
    
    #[test]
    fn test_rotation_wrapping() {
        let node = TransformNode::create_standard("Test".to_string());
        let mut transform_node = TransformNode::new(node);
        
        // Test rotation wrapping
        transform_node.set_rotation(450.0); // Should wrap to 90.0
        assert_eq!(transform_node.get_rotation(), 90.0);
        
        transform_node.set_rotation(-90.0); // Should wrap to 270.0
        assert_eq!(transform_node.get_rotation(), 270.0);
        
        transform_node.set_rotation(720.0); // Should wrap to 0.0
        assert_eq!(transform_node.get_rotation(), 0.0);
    }
    
    #[test]
    fn test_is_active() {
        let node = TransformNode::create_standard("Test".to_string());
        let mut transform_node = TransformNode::new(node);
        
        // Should not be active with identity transform
        assert!(!transform_node.is_active());
        
        // Should be active with any non-default parameter
        transform_node.set_position(10.0, 0.0);
        assert!(transform_node.is_active());
        
        // Reset and test other parameters
        transform_node.reset();
        assert!(!transform_node.is_active());
        
        transform_node.set_scale(2.0, 2.0);
        assert!(transform_node.is_active());
        
        transform_node.reset();
        transform_node.set_rotation(45.0);
        assert!(transform_node.is_active());
    }
    
    #[test]
    fn test_uniform_scale_flag() {
        let node = TransformNode::create_standard("Test".to_string());
        let mut transform_node = TransformNode::new(node);
        
        // Set non-uniform scale
        transform_node.set_scale(2.0, 3.0);
        assert_eq!(transform_node.get_scale(), (2.0, 3.0));
        
        // Enable uniform scale flag
        transform_node.set_uniform_scale_flag(true);
        assert!(transform_node.is_uniform_scale());
        assert_eq!(transform_node.get_scale(), (2.5, 2.5)); // Average of 2.0 and 3.0
        
        // Set uniform scale
        transform_node.set_uniform_scale(1.5);
        assert_eq!(transform_node.get_scale(), (1.5, 1.5));
    }
    
    #[test]
    fn test_reset() {
        let node = TransformNode::create_standard("Test".to_string());
        let mut transform_node = TransformNode::new(node);
        
        // Change all parameters
        transform_node.set_position(100.0, 200.0);
        transform_node.set_scale(2.0, 3.0);
        transform_node.set_rotation(45.0);
        transform_node.set_anchor(50.0, 75.0);
        transform_node.set_uniform_scale_flag(false);
        
        // Reset
        transform_node.reset();
        
        // Check defaults
        assert_eq!(transform_node.get_position(), (0.0, 0.0));
        assert_eq!(transform_node.get_scale(), (1.0, 1.0));
        assert_eq!(transform_node.get_rotation(), 0.0);
        assert_eq!(transform_node.get_anchor(), (0.0, 0.0));
        assert!(transform_node.is_uniform_scale());
    }
    
    #[test]
    fn test_transform_point() {
        let node = TransformNode::create_standard("Test".to_string());
        let mut transform_node = TransformNode::new(node);
        
        // Test translation
        transform_node.set_position(10.0, 20.0);
        let result = transform_node.transform_point(0.0, 0.0);
        assert_eq!(result, (10.0, 20.0));
        
        // Test scaling
        transform_node.reset();
        transform_node.set_scale(2.0, 3.0);
        let result = transform_node.transform_point(10.0, 10.0);
        assert_eq!(result, (20.0, 30.0));
        
        // Test rotation
        transform_node.reset();
        transform_node.set_rotation(90.0);
        let result = transform_node.transform_point(1.0, 0.0);
        assert!((result.0 - 0.0).abs() < 0.01);
        assert!((result.1 - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_standard_node_creation() {
        let node = TransformNode::create_standard("Test Transform".to_string());
        
        assert_eq!(node.node_type, NodeType::Transform);
        assert_eq!(node.name, "Test Transform");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert!(node.inputs[0].required);
        
        // Check parameter names
        let param_names: Vec<String> = node.parameters.iter()
            .map(|p| p.name.clone())
            .collect();
        assert!(param_names.contains(&"position_x".to_string()));
        assert!(param_names.contains(&"position_y".to_string()));
        assert!(param_names.contains(&"scale_x".to_string()));
        assert!(param_names.contains(&"scale_y".to_string()));
        assert!(param_names.contains(&"rotation".to_string()));
        assert!(param_names.contains(&"anchor_x".to_string()));
        assert!(param_names.contains(&"anchor_y".to_string()));
        assert!(param_names.contains(&"uniform_scale".to_string()));
    }
}
