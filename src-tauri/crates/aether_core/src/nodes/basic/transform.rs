use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;

/// Transform node for position, scale, rotation transforms
pub struct TransformNode {
    node: Node,
    position: (f32, f32),
    scale: (f32, f32),
    rotation: f32,
    anchor: (f32, f32),
    uniform_scale: bool,
}

impl TransformNode {
    /// Create a new transform node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            position: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation: 0.0,
            anchor: (0.0, 0.0),
            uniform_scale: true,
        }
    }
    
    /// Set position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }
    
    /// Get position
    pub fn get_position(&self) -> (f32, f32) {
        self.position
    }
    
    /// Set scale
    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale = (x, y);
    }
    
    /// Get scale
    pub fn get_scale(&self) -> (f32, f32) {
        self.scale
    }
    
    /// Set uniform scale
    pub fn set_uniform_scale(&mut self, scale: f32) {
        self.scale = (scale, scale);
    }
    
    /// Set rotation in degrees
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }
    
    /// Get rotation in degrees
    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }
    
    /// Set anchor point
    pub fn set_anchor(&mut self, x: f32, y: f32) {
        self.anchor = (x, y);
    }
    
    /// Get anchor point
    pub fn get_anchor(&self) -> (f32, f32) {
        self.anchor
    }
    
    /// Set uniform scale flag
    pub fn set_uniform_scale_flag(&mut self, uniform: bool) {
        self.uniform_scale = uniform;
    }
    
    /// Get uniform scale flag
    pub fn is_uniform_scale(&self) -> bool {
        self.uniform_scale
    }
    
    /// Apply transform to an image
    fn apply_transform(&self, input_value: ParameterValue) -> ParameterValue {
        match input_value {
            ParameterValue::Image(input_id) => {
                // In a real implementation, this would:
                // - Load the input texture
                // - Create transformation matrix from position, scale, rotation, anchor
                // - Apply GPU or CPU transformation
                // - Return transformed texture ID
                
                log::debug!("Applying transform: pos=({:.2}, {:.2}), scale=({:.2}, {:.2}), rot={:.2}°, anchor=({:.2}, {:.2})", 
                    self.position.0, self.position.1, 
                    self.scale.0, self.scale.1, 
                    self.rotation, 
                    self.anchor.0, self.anchor.1);
                
                // Return a transformed image ID
                let transformed_id = Uuid::new_v4();
                ParameterValue::Image(transformed_id)
            }
            _ => {
                // No valid image input
                ParameterValue::None
            }
        }
    }
    
    /// Create transformation matrix
    fn create_transform_matrix(&self) -> [[f32; 4]; 4] {
        // Convert rotation to radians
        let rotation_rad = self.rotation.to_radians();
        let cos_r = rotation_rad.cos();
        let sin_r = rotation_rad.sin();
        
        // Create transformation matrix (3D homogeneous coordinates)
        // Translation * Rotation * Scale
        let mut matrix = [[0.0; 4]; 4];
        
        // Scale
        matrix[0][0] = self.scale.0;
        matrix[1][1] = self.scale.1;
        matrix[2][2] = 1.0;
        matrix[3][3] = 1.0;
        
        // Rotation
        let rot_matrix = [
            [cos_r, -sin_r, 0.0, 0.0],
            [sin_r, cos_r, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        
        // Combine scale and rotation
        let mut combined = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    combined[i][j] += matrix[i][k] * rot_matrix[k][j];
                }
            }
        }
        
        // Translation
        combined[0][3] = self.position.0;
        combined[1][3] = self.position.1;
        
        combined
    }
    
    /// Create a transform node with standard configuration
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
            animatable: true,
            description: Some("X position".to_string()),
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
            animatable: true,
            description: Some("Y position".to_string()),
        };
        node.add_parameter(position_y_param);
        
        let scale_x_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "scale_x".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(0.0),
            max_value: None,
            animatable: true,
            description: Some("X scale".to_string()),
        };
        node.add_parameter(scale_x_param);
        
        let scale_y_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "scale_y".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(0.0),
            max_value: None,
            animatable: true,
            description: Some("Y scale".to_string()),
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
            animatable: true,
            description: Some("Rotation in degrees".to_string()),
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
            animatable: true,
            description: Some("X anchor point".to_string()),
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
            animatable: true,
            description: Some("Y anchor point".to_string()),
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
            animatable: false,
            description: Some("Lock X and Y scale together".to_string()),
        };
        node.add_parameter(uniform_scale_param);
        
        node
    }
    
    /// Update parameters from node metadata
    fn update_parameters(&mut self) {
        if let Some(pos_x_param) = self.node.parameters.get("position_x") {
            if let ParameterValue::Float(x) = pos_x_param.value {
                self.position.0 = x;
            }
        }
        
        if let Some(pos_y_param) = self.node.parameters.get("position_y") {
            if let ParameterValue::Float(y) = pos_y_param.value {
                self.position.1 = y;
            }
        }
        
        if let Some(scale_x_param) = self.node.parameters.get("scale_x") {
            if let ParameterValue::Float(x) = scale_x_param.value {
                self.scale.0 = x;
            }
        }
        
        if let Some(scale_y_param) = self.node.parameters.get("scale_y") {
            if let ParameterValue::Float(y) = scale_y_param.value {
                self.scale.1 = y;
            }
        }
        
        if let Some(rotation_param) = self.node.parameters.get("rotation") {
            if let ParameterValue::Float(rotation) = rotation_param.value {
                self.rotation = rotation;
            }
        }
        
        if let Some(anchor_x_param) = self.node.parameters.get("anchor_x") {
            if let ParameterValue::Float(x) = anchor_x_param.value {
                self.anchor.0 = x;
            }
        }
        
        if let Some(anchor_y_param) = self.node.parameters.get("anchor_y") {
            if let ParameterValue::Float(y) = anchor_y_param.value {
                self.anchor.1 = y;
            }
        }
        
        if let Some(uniform_param) = self.node.parameters.get("uniform_scale") {
            if let ParameterValue::Boolean(uniform) = uniform_param.value {
                self.uniform_scale = uniform;
            }
        }
        
        // Handle uniform scale
        if self.uniform_scale {
            let uniform_scale = self.scale.0;
            self.scale.1 = uniform_scale;
        }
    }
}

impl NodeExecutor for TransformNode {
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
            self.apply_transform(input_value)
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
        NodeType::Transform
    }
    
    fn validate(&self) -> NodeResult<()> {
        // Validate scale values
        if self.scale.0 <= 0.0 || self.scale.1 <= 0.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Scale values must be greater than 0".to_string()
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

/// 2D Transform node specialized for 2D transformations
pub struct Transform2DNode {
    transform_node: TransformNode,
}

impl Transform2DNode {
    pub fn new(node: Node) -> Self {
        Self {
            transform_node: TransformNode::new(node),
        }
    }
    
    /// Create a standard 2D transform node
    pub fn create_standard(name: String) -> Node {
        TransformNode::create_standard(name)
    }
    
    /// Get the transformation matrix
    pub fn get_transform_matrix(&self) -> [[f32; 4]; 4] {
        self.transform_node.create_transform_matrix()
    }
}

impl NodeExecutor for Transform2DNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.transform_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Transform
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.transform_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.transform_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.transform_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.transform_node.can_execute(context)
    }
}

/// Position-only transform node
pub struct PositionNode {
    transform_node: TransformNode,
}

impl PositionNode {
    pub fn new(node: Node) -> Self {
        let mut transform_node = TransformNode::new(node);
        transform_node.set_scale(1.0, 1.0);
        transform_node.set_rotation(0.0);
        
        Self { transform_node }
    }
    
    /// Create a standard position node
    pub fn create_standard(name: String) -> Node {
        let mut node = TransformNode::create_standard(name);
        
        // Remove scale and rotation parameters for position-only node
        node.parameters.retain(|name, _| !["scale_x", "scale_y", "rotation"].contains(&name.as_str()));
        
        node
    }
}

impl NodeExecutor for PositionNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.transform_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Transform
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.transform_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.transform_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.transform_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.transform_node.can_execute(context)
    }
}

/// Scale-only transform node
pub struct ScaleNode {
    transform_node: TransformNode,
}

impl ScaleNode {
    pub fn new(node: Node) -> Self {
        let mut transform_node = TransformNode::new(node);
        transform_node.set_position(0.0, 0.0);
        transform_node.set_rotation(0.0);
        
        Self { transform_node }
    }
    
    /// Create a standard scale node
    pub fn create_standard(name: String) -> Node {
        let mut node = TransformNode::create_standard(name);
        
        // Remove position and rotation parameters for scale-only node
        node.parameters.retain(|name, _| !["position_x", "position_y", "rotation"].contains(&name.as_str()));
        
        node
    }
}

impl NodeExecutor for ScaleNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.transform_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Transform
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.transform_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.transform_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.transform_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.transform_node.can_execute(context)
    }
}

/// Rotation-only transform node
pub struct RotationNode {
    transform_node: TransformNode,
}

impl RotationNode {
    pub fn new(node: Node) -> Self {
        let mut transform_node = TransformNode::new(node);
        transform_node.set_position(0.0, 0.0);
        transform_node.set_scale(1.0, 1.0);
        
        Self { transform_node }
    }
    
    /// Create a standard rotation node
    pub fn create_standard(name: String) -> Node {
        let mut node = TransformNode::create_standard(name);
        
        // Remove position and scale parameters for rotation-only node
        node.parameters.retain(|name, _| !["position_x", "position_y", "scale_x", "scale_y"].contains(&name.as_str()));
        
        node
    }
}

impl NodeExecutor for RotationNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.transform_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Transform
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.transform_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.transform_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.transform_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.transform_node.can_execute(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transform_node_creation() {
        let node = TransformNode::create_standard("Test Transform".to_string());
        assert_eq!(node.node_type, NodeType::Transform);
        assert_eq!(node.name, "Test Transform");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.parameters.len(), 8);
    }
    
    #[test]
    fn test_transform_node_properties() {
        let node = TransformNode::create_standard("Test".to_string());
        let mut transform_node = TransformNode::new(node);
        
        transform_node.set_position(100.0, 200.0);
        assert_eq!(transform_node.get_position(), (100.0, 200.0));
        
        transform_node.set_scale(2.0, 3.0);
        assert_eq!(transform_node.get_scale(), (2.0, 3.0));
        
        transform_node.set_rotation(45.0);
        assert_eq!(transform_node.get_rotation(), 45.0);
        
        transform_node.set_uniform_scale_flag(true);
        assert!(transform_node.is_uniform_scale());
    }
    
    #[test]
    fn test_transform_matrix() {
        let node = TransformNode::create_standard("Test".to_string());
        let transform_node = TransformNode::new(node);
        
        let matrix = transform_node.create_transform_matrix();
        // Should be identity matrix with default values
        assert_eq!(matrix[0][0], 1.0); // Scale X
        assert_eq!(matrix[1][1], 1.0); // Scale Y
        assert_eq!(matrix[0][3], 0.0); // Position X
        assert_eq!(matrix[1][3], 0.0); // Position Y
    }
    
    #[test]
    fn test_specialized_transform_nodes() {
        let pos_node = PositionNode::create_standard("Position".to_string());
        let scale_node = ScaleNode::create_standard("Scale".to_string());
        let rot_node = RotationNode::create_standard("Rotation".to_string());
        
        assert_eq!(pos_node.node_type, NodeType::Transform);
        assert_eq!(scale_node.node_type, NodeType::Transform);
        assert_eq!(rot_node.node_type, NodeType::Transform);
        
        // Position node should have fewer parameters
        assert!(pos_node.parameters.len() < 8);
        // Scale node should have fewer parameters
        assert!(scale_node.parameters.len() < 8);
        // Rotation node should have fewer parameters
        assert!(rot_node.parameters.len() < 8);
    }
    
    #[test]
    fn test_transform_node_execution() {
        let node = TransformNode::create_standard("Test".to_string());
        let transform_node = TransformNode::new(node);
        
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert!(transform_node.execute(&mut context).is_ok());
    }
}
