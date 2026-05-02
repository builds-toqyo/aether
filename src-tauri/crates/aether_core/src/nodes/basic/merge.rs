use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin, BlendMode};
use uuid::Uuid;

/// Merge node for blending multiple inputs
pub struct MergeNode {
    node: Node,
    blend_mode: BlendMode,
    opacity: f32,
}

impl MergeNode {
    /// Create a new merge node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
        }
    }
    
    /// Set the blend mode
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }
    
    /// Get the blend mode
    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode.clone()
    }
    
    /// Set the opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
    
    /// Get the opacity
    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }
    
    /// Apply blend operation between two images
    fn apply_blend(&self, input1: ParameterValue, input2: ParameterValue) -> ParameterValue {
        match (&input1, &input2) {
            (ParameterValue::Image(id1), ParameterValue::Image(id2)) => {
                // In a real implementation, this would:
                // - Load the two image textures
                // - Apply the blend operation using GPU or CPU
                // - Return the blended result as a new texture ID
                
                log::debug!("Blending images: {:?} + {:?} with mode={:?}, opacity={}", 
                    id1, id2, self.blend_mode, self.opacity);
                
                // Return a new blended image ID
                let blended_id = Uuid::new_v4();
                ParameterValue::Image(blended_id)
            }
            (ParameterValue::Image(id), ParameterValue::None) => {
                // Pass through the image if second input is none
                input1
            }
            (ParameterValue::None, ParameterValue::Image(id)) => {
                // Pass through the image if first input is none
                input2
            }
            _ => {
                // No valid image inputs
                ParameterValue::None
            }
        }
    }
    
    /// Apply blend operation to multiple inputs
    fn apply_multi_blend(&self, inputs: &[ParameterValue]) -> ParameterValue {
        if inputs.is_empty() {
            return ParameterValue::None;
        }
        
        if inputs.len() == 1 {
            return inputs[0].clone();
        }
        
        // Start with the first input
        let mut result = inputs[0].clone();
        
        // Blend with each subsequent input
        for i in 1..inputs.len() {
            result = self.apply_blend(result, inputs[i].clone());
        }
        
        result
    }
    
    /// Create a merge node with standard configuration
    pub fn create_standard(name: String, input_count: usize) -> Node {
        let mut node = Node::new(NodeType::Merge, name);
        
        // Add input pins
        for i in 0..input_count {
            let input_pin = InputPin {
                id: Uuid::new_v4(),
                name: format!("input_{}", i),
                data_type: PinDataType::Image,
                required: i == 0, // First input is required
                default_value: ParameterValue::None,
                current_value: ParameterValue::None,
                connection: None,
            };
            node.add_input(input_pin);
        }
        
        // Add output pin
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);
        
        // Add parameters
        let blend_mode_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "blend_mode".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("normal".to_string()),
            default_value: ParameterValue::String("normal".to_string()),
            min_value: None,
            max_value: None,
            animatable: true,
            description: Some("Blend mode (normal, add, subtract, multiply, screen, overlay, soft_light, hard_light, color_dodge, color_burn, darken, lighten, difference, exclusion, hue, saturation, color, luminosity)".to_string()),
        };
        node.add_parameter(blend_mode_param);
        
        let opacity_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "opacity".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(0.0),
            max_value: Some(1.0),
            animatable: true,
            description: Some("Blend opacity (0.0 to 1.0)".to_string()),
        };
        node.add_parameter(opacity_param);
        
        node
    }
    
    /// Update parameters from node metadata
    fn update_parameters(&mut self) {
        if let Some(blend_mode_param) = self.node.parameters.get("blend_mode") {
            if let ParameterValue::String(mode_str) = &blend_mode_param.value {
                self.blend_mode = match mode_str.as_str() {
                    "add" => BlendMode::Add,
                    "subtract" => BlendMode::Subtract,
                    "multiply" => BlendMode::Multiply,
                    "screen" => BlendMode::Screen,
                    "overlay" => BlendMode::Overlay,
                    "soft_light" => BlendMode::SoftLight,
                    "hard_light" => BlendMode::HardLight,
                    "color_dodge" => BlendMode::ColorDodge,
                    "color_burn" => BlendMode::ColorBurn,
                    "darken" => BlendMode::Darken,
                    "lighten" => BlendMode::Lighten,
                    "difference" => BlendMode::Difference,
                    "exclusion" => BlendMode::Exclusion,
                    "hue" => BlendMode::Hue,
                    "saturation" => BlendMode::Saturation,
                    "color" => BlendMode::Color,
                    "luminosity" => BlendMode::Luminosity,
                    _ => BlendMode::Normal,
                };
            }
        }
        
        if let Some(opacity_param) = self.node.parameters.get("opacity") {
            if let ParameterValue::Float(opacity) = opacity_param.value {
                self.opacity = opacity.clamp(0.0, 1.0);
            }
        }
    }
}

impl NodeExecutor for MergeNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Collect all input values
        let mut input_values = Vec::new();
        for input_pin in &self.node.inputs {
            if let Some(input_value) = context.get_input(&input_pin.id) {
                input_values.push(input_value.clone());
            } else if !input_pin.required {
                // Optional input not connected, use default
                input_values.push(ParameterValue::None);
            }
        }
        
        // Apply blend operation
        let result = self.apply_multi_blend(&input_values);
        
        // Set output
        if let Some(output_pin) = self.node.outputs.first() {
            context.set_output(output_pin.id, result);
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Merge
    }
    
    fn validate(&self) -> NodeResult<()> {
        // Validate opacity range
        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                "Opacity must be between 0.0 and 1.0".to_string()
            ));
        }
        
        // Validate that at least one input is connected
        let connected_inputs = self.node.inputs.iter()
            .filter(|pin| pin.connection.is_some())
            .count();
        
        if connected_inputs == 0 {
            return Err(crate::nodes::NodeError::RequiredInputNotConnected(
                "At least one input must be connected".to_string()
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
        self.node.inputs.iter().any(|pin| context.get_input(&pin.id).is_some())
    }
}

/// Two-input merge node (most common case)
pub struct TwoInputMergeNode {
    merge_node: MergeNode,
}

impl TwoInputMergeNode {
    pub fn new(node: Node) -> Self {
        Self {
            merge_node: MergeNode::new(node),
        }
    }
    
    /// Create a standard two-input merge node
    pub fn create_standard(name: String) -> Node {
        MergeNode::create_standard(name, 2)
    }
}

impl NodeExecutor for TwoInputMergeNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.merge_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Merge
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.merge_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.merge_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.merge_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.merge_node.can_execute(context)
    }
}

/// Multi-input merge node for complex compositing
pub struct MultiInputMergeNode {
    merge_node: MergeNode,
    max_inputs: usize,
}

impl MultiInputMergeNode {
    pub fn new(node: Node, max_inputs: usize) -> Self {
        Self {
            merge_node: MergeNode::new(node),
            max_inputs,
        }
    }
    
    /// Create a standard multi-input merge node
    pub fn create_standard(name: String, max_inputs: usize) -> Node {
        MergeNode::create_standard(name, max_inputs)
    }
    
    /// Get the maximum number of inputs
    pub fn max_inputs(&self) -> usize {
        self.max_inputs
    }
    
    /// Get the number of connected inputs
    pub fn connected_inputs(&self) -> usize {
        self.merge_node.node.inputs.iter()
            .filter(|pin| pin.connection.is_some())
            .count()
    }
}

impl NodeExecutor for MultiInputMergeNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.merge_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Merge
    }
    
    fn validate(&self) -> NodeResult<()> {
        // Validate that we don't exceed max inputs
        if self.connected_inputs() > self.max_inputs {
            return Err(crate::nodes::NodeError::InvalidParameterValue(
                format!("Too many inputs connected (max: {})", self.max_inputs)
            ));
        }
        
        self.merge_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.merge_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.merge_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.merge_node.can_execute(context)
    }
}

/// Additive blend node (specialized for additive blending)
pub struct AdditiveMergeNode {
    merge_node: MergeNode,
}

impl AdditiveMergeNode {
    pub fn new(node: Node) -> Self {
        let mut merge_node = MergeNode::new(node);
        merge_node.set_blend_mode(BlendMode::Add);
        
        Self { merge_node }
    }
    
    /// Create a standard additive merge node
    pub fn create_standard(name: String) -> Node {
        MergeNode::create_standard(name, 2)
    }
}

impl NodeExecutor for AdditiveMergeNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.merge_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Merge
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.merge_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.merge_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.merge_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.merge_node.can_execute(context)
    }
}

/// Screen blend node (specialized for screen blending)
pub struct ScreenMergeNode {
    merge_node: MergeNode,
}

impl ScreenMergeNode {
    pub fn new(node: Node) -> Self {
        let mut merge_node = MergeNode::new(node);
        merge_node.set_blend_mode(BlendMode::Screen);
        
        Self { merge_node }
    }
    
    /// Create a standard screen merge node
    pub fn create_standard(name: String) -> Node {
        MergeNode::create_standard(name, 2)
    }
}

impl NodeExecutor for ScreenMergeNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.merge_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Merge
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.merge_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.merge_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.merge_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.merge_node.can_execute(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merge_node_creation() {
        let node = MergeNode::create_standard("Test Merge".to_string(), 2);
        assert_eq!(node.node_type, NodeType::Merge);
        assert_eq!(node.name, "Test Merge");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.parameters.len(), 2);
    }
    
    #[test]
    fn test_blend_modes() {
        assert_eq!(BlendMode::Normal, BlendMode::Normal);
        assert_ne!(BlendMode::Normal, BlendMode::Add);
        assert_eq!(BlendMode::Multiply, BlendMode::Multiply);
    }
    
    #[test]
    fn test_merge_node_execution() {
        let node = MergeNode::create_standard("Test".to_string(), 2);
        let merge_node = MergeNode::new(node);
        
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert!(merge_node.execute(&mut context).is_ok());
    }
    
    #[test]
    fn test_two_input_merge_node() {
        let node = TwoInputMergeNode::create_standard("Two Input".to_string());
        assert_eq!(node.inputs.len(), 2);
        
        let two_input_node = TwoInputMergeNode::new(node);
        assert_eq!(two_input_node.node_type(), NodeType::Merge);
    }
    
    #[test]
    fn test_multi_input_merge_node() {
        let node = MultiInputMergeNode::create_standard("Multi Input".to_string(), 5);
        assert_eq!(node.inputs.len(), 5);
        
        let multi_node = MultiInputMergeNode::new(node, 5);
        assert_eq!(multi_node.max_inputs(), 5);
    }
    
    #[test]
    fn test_specialized_merge_nodes() {
        let add_node = AdditiveMergeNode::create_standard("Add".to_string());
        let screen_node = ScreenMergeNode::create_standard("Screen".to_string());
        
        assert_eq!(add_node.node_type, NodeType::Merge);
        assert_eq!(screen_node.node_type, NodeType::Merge);
    }
}
