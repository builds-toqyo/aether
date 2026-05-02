use crate::nodes::{NodeExecutor, ExecutionContext, NodeError, NodeResult};
use aether_types::{Node, NodeType, PinDataType, ParameterValue};
use std::collections::HashMap;
use uuid::Uuid;

/// Core node implementations for basic functionality
pub struct CoreNodes;

impl CoreNodes {
    pub fn create_input_node(node: Node) -> Box<dyn NodeExecutor + Send + Sync> {
        Box::new(InputNode::new(node))
    }
    
    pub fn create_output_node(node: Node) -> Box<dyn NodeExecutor + Send + Sync> {
        Box::new(OutputNode::new(node))
    }
    
    pub fn create_transform_node(node: Node) -> Box<dyn NodeExecutor + Send + Sync> {
        Box::new(TransformNode::new(node))
    }
    
    pub fn create_merge_node(node: Node) -> Box<dyn NodeExecutor + Send + Sync> {
        Box::new(MergeNode::new(node))
    }
}

/// Input node - provides media source input
#[derive(Debug)]
pub struct InputNode {
    node: Node,
    media_path: Option<String>,
    frame_cache: HashMap<u64, ParameterValue>,
}

impl InputNode {
    pub fn new(node: Node) -> Self {
        Self {
            node,
            media_path: None,
            frame_cache: HashMap::new(),
        }
    }
    
    pub fn set_media_path(&mut self, path: String) {
        self.media_path = Some(path);
    }
    
    fn generate_frame(&self, frame: u64) -> ParameterValue {
        // Check cache first
        if let Some(cached_frame) = self.frame_cache.get(&frame) {
            return cached_frame.clone();
        }
        
        // Generate a dummy frame for now
        // In a real implementation, this would load from media file
        let frame_id = Uuid::new_v4();
        ParameterValue::Image(frame_id)
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
        // Validate that we have a media path or generate mode
        if self.media_path.is_none() {
            log::warn!("Input node has no media path set");
        }
        Ok(())
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        &[]
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        // Return output pin IDs
        &self.node.outputs.iter().map(|pin| pin.id).collect::<Vec<_>>()
    }
    
    fn can_execute(&self, _context: &ExecutionContext) -> bool {
        self.node.enabled
    }
}

/// Output node - final result output
#[derive(Debug)]
pub struct OutputNode {
    node: Node,
}

impl OutputNode {
    pub fn new(node: Node) -> Self {
        Self { node }
    }
}

impl NodeExecutor for OutputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Get input from connected node
        if let Some(input_pin) = self.node.inputs.first() {
            if let Some(connection_id) = &input_pin.connection {
                // In a real implementation, we'd get the value from the connected output
                // For now, just pass through a default value
                let output_value = context.get_input(&input_pin.id)
                    .cloned()
                    .unwrap_or(ParameterValue::None);
                
                // Set as final output
                context.set_output(self.node.outputs[0].id, output_value);
            }
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Output
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

/// Transform node - position, scale, rotation
#[derive(Debug)]
pub struct TransformNode {
    node: Node,
}

impl TransformNode {
    pub fn new(node: Node) -> Self {
        Self { node }
    }
    
    fn get_transform_params(&self) -> (f32, f32, f32, f32, f32) {
        // Get transform parameters (position x, y, scale, rotation)
        let mut pos_x = 0.0;
        let mut pos_y = 0.0;
        let mut scale = 1.0;
        let mut rotation = 0.0;
        
        if let Some(param) = self.node.parameters.get("position_x") {
            if let ParameterValue::Float(val) = param.value {
                pos_x = val;
            }
        }
        
        if let Some(param) = self.node.parameters.get("position_y") {
            if let ParameterValue::Float(val) = param.value {
                pos_y = val;
            }
        }
        
        if let Some(param) = self.node.parameters.get("scale") {
            if let ParameterValue::Float(val) = param.value {
                scale = val;
            }
        }
        
        if let Some(param) = self.node.parameters.get("rotation") {
            if let ParameterValue::Float(val) = param.value {
                rotation = val;
            }
        }
        
        (pos_x, pos_y, scale, rotation, 0.0) // anchor_x = 0.0 for now
    }
}

impl NodeExecutor for TransformNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Get input image
        let input_image = if let Some(input_pin) = self.node.inputs.first() {
            context.get_input(&input_pin.id).cloned()
        } else {
            None
        };
        
        if let Some(ParameterValue::Image(input_id)) = input_image {
            // Apply transform
            let (pos_x, pos_y, scale, rotation, _anchor) = self.get_transform_params();
            
            // In a real implementation, this would apply the transform using GPU
            // For now, just pass through with transform metadata
            let transformed_id = Uuid::new_v4();
            
            // Store transform metadata (in real implementation, this would be GPU data)
            log::debug!("Applying transform: pos=({}, {}), scale={}, rotation={}", 
                pos_x, pos_y, scale, rotation);
            
            // Set output
            if let Some(output_pin) = self.node.outputs.first() {
                context.set_output(output_pin.id, ParameterValue::Image(transformed_id));
            }
        } else {
            // No input, pass through default
            if let Some(output_pin) = self.node.outputs.first() {
                context.set_output(output_pin.id, ParameterValue::None);
            }
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Transform
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

/// Merge node - blend modes
#[derive(Debug)]
pub struct MergeNode {
    node: Node,
}

impl MergeNode {
    pub fn new(node: Node) -> Self {
        Self { node }
    }
    
    fn get_blend_mode(&self) -> aether_types::BlendMode {
        if let Some(param) = self.node.parameters.get("blend_mode") {
            if let ParameterValue::String(mode) = &param.value {
                match mode.as_str() {
                    "add" => aether_types::BlendMode::Add,
                    "multiply" => aether_types::BlendMode::Multiply,
                    "screen" => aether_types::BlendMode::Screen,
                    "overlay" => aether_types::BlendMode::Overlay,
                    _ => aether_types::BlendMode::Normal,
                }
            } else {
                aether_types::BlendMode::Normal
            }
        } else {
            aether_types::BlendMode::Normal
        }
    }
    
    fn get_opacity(&self) -> f32 {
        if let Some(param) = self.node.parameters.get("opacity") {
            if let ParameterValue::Float(val) = param.value {
                val.clamp(0.0, 1.0)
            } else {
                1.0
            }
        } else {
            1.0
        }
    }
}

impl NodeExecutor for MergeNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Get input images
        let input1 = if self.node.inputs.len() > 0 {
            context.get_input(&self.node.inputs[0].id).cloned()
        } else {
            None
        };
        
        let input2 = if self.node.inputs.len() > 1 {
            context.get_input(&self.node.inputs[1].id).cloned()
        } else {
            None
        };
        
        match (input1, input2) {
            (Some(ParameterValue::Image(input1_id)), Some(ParameterValue::Image(input2_id))) => {
                // Apply blend operation
                let blend_mode = self.get_blend_mode();
                let opacity = self.get_opacity();
                
                // In a real implementation, this would use GPU compositing
                let merged_id = Uuid::new_v4();
                
                log::debug!("Merging images: {:?} + {:?} with mode={:?}, opacity={}", 
                    input1_id, input2_id, blend_mode, opacity);
                
                // Set output
                if let Some(output_pin) = self.node.outputs.first() {
                    context.set_output(output_pin.id, ParameterValue::Image(merged_id));
                }
            }
            (Some(ParameterValue::Image(input_id)), None) => {
                // Pass through first input
                if let Some(output_pin) = self.node.outputs.first() {
                    context.set_output(output_pin.id, ParameterValue::Image(input_id));
                }
            }
            (None, Some(ParameterValue::Image(input_id))) => {
                // Pass through second input
                if let Some(output_pin) = self.node.outputs.first() {
                    context.set_output(output_pin.id, ParameterValue::Image(input_id));
                }
            }
            _ => {
                // No valid inputs
                if let Some(output_pin) = self.node.outputs.first() {
                    context.set_output(output_pin.id, ParameterValue::None);
                }
            }
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Merge
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        &self.node.inputs.iter().map(|pin| pin.id).collect::<Vec<_>>()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        &self.node.outputs.iter().map(|pin| pin.id).collect::<Vec<_>>()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.node.enabled && (
            self.node.inputs.iter().any(|pin| context.get_input(&pin.id).is_some())
        )
    }
}

/// Color correction node
#[derive(Debug)]
pub struct ColorCorrectionNode {
    node: Node,
}

impl ColorCorrectionNode {
    pub fn new(node: Node) -> Self {
        Self { node }
    }
    
    fn get_color_adjustments(&self) -> (f32, f32, f32, f32, f32) {
        let mut brightness = 0.0;
        let mut contrast = 1.0;
        let mut saturation = 1.0;
        let mut gamma = 1.0;
        let mut temperature = 6500.0;
        
        if let Some(param) = self.node.parameters.get("brightness") {
            if let ParameterValue::Float(val) = param.value {
                brightness = val;
            }
        }
        
        if let Some(param) = self.node.parameters.get("contrast") {
            if let ParameterValue::Float(val) = param.value {
                contrast = val;
            }
        }
        
        if let Some(param) = self.node.parameters.get("saturation") {
            if let ParameterValue::Float(val) = param.value {
                saturation = val;
            }
        }
        
        if let Some(param) = self.node.parameters.get("gamma") {
            if let ParameterValue::Float(val) = param.value {
                gamma = val;
            }
        }
        
        if let Some(param) = self.node.parameters.get("temperature") {
            if let ParameterValue::Float(val) = param.value {
                temperature = val;
            }
        }
        
        (brightness, contrast, saturation, gamma, temperature)
    }
}

impl NodeExecutor for ColorCorrectionNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Get input image
        let input_image = if let Some(input_pin) = self.node.inputs.first() {
            context.get_input(&input_pin.id).cloned()
        } else {
            None
        };
        
        if let Some(ParameterValue::Image(input_id)) = input_image {
            // Apply color corrections
            let (brightness, contrast, saturation, gamma, temperature) = self.get_color_adjustments();
            
            // In a real implementation, this would use GPU color grading
            let corrected_id = Uuid::new_v4();
            
            log::debug!("Color correction: brightness={}, contrast={}, saturation={}, gamma={}, temperature={}", 
                brightness, contrast, saturation, gamma, temperature);
            
            // Set output
            if let Some(output_pin) = self.node.outputs.first() {
                context.set_output(output_pin.id, ParameterValue::Image(corrected_id));
            }
        } else {
            // No input, pass through default
            if let Some(output_pin) = self.node.outputs.first() {
                context.set_output(output_pin.id, ParameterValue::None);
            }
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::ColorCorrection
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

/// Factory for creating core nodes
pub struct CoreNodeFactory;

impl CoreNodeFactory {
    /// Create a node based on its type
    pub fn create_node(node_type: NodeType, node: Node) -> NodeResult<Box<dyn NodeExecutor + Send + Sync>> {
        match node_type {
            NodeType::Input => Ok(CoreNodes::create_input_node(node)),
            NodeType::Output => Ok(CoreNodes::create_output_node(node)),
            NodeType::Transform => Ok(CoreNodes::create_transform_node(node)),
            NodeType::Merge => Ok(CoreNodes::create_merge_node(node)),
            NodeType::ColorCorrection => Ok(Box::new(ColorCorrectionNode::new(node))),
            _ => Err(NodeError::ExecutionFailed(format!("Unsupported node type: {:?}", node_type))),
        }
    }
    
    /// Register all core node types with a registry
    pub fn register_core_nodes(registry: &mut crate::nodes::NodeRegistry) {
        registry.register_node_type(NodeType::Input, || {
            let node = aether_types::Node::new(NodeType::Input, "Input".to_string());
            CoreNodes::create_input_node(node)
        });
        
        registry.register_node_type(NodeType::Output, || {
            let node = aether_types::Node::new(NodeType::Output, "Output".to_string());
            CoreNodes::create_output_node(node)
        });
        
        registry.register_node_type(NodeType::Transform, || {
            let node = aether_types::Node::new(NodeType::Transform, "Transform".to_string());
            CoreNodes::create_transform_node(node)
        });
        
        registry.register_node_type(NodeType::Merge, || {
            let node = aether_types::Node::new(NodeType::Merge, "Merge".to_string());
            CoreNodes::create_merge_node(node)
        });
        
        registry.register_node_type(NodeType::ColorCorrection, || {
            let node = aether_types::Node::new(NodeType::ColorCorrection, "Color Correction".to_string());
            Box::new(ColorCorrectionNode::new(node))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{Node, NodeType, InputPin, OutputPin, PinDataType, ParameterValue};
    
    #[test]
    fn test_input_node() {
        let mut node = Node::new(NodeType::Input, "Test Input".to_string());
        
        // Add output pin
        let output_pin = OutputPin {
            id: uuid::Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);
        
        let input_node = InputNode::new(node);
        
        // Test execution
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert!(input_node.execute(&mut context).is_ok());
        
        // Should have output
        assert!(!context.outputs.is_empty());
    }
    
    #[test]
    fn test_transform_node() {
        let mut node = Node::new(NodeType::Transform, "Test Transform".to_string());
        
        // Add input pin
        let input_pin = InputPin {
            id: uuid::Uuid::new_v4(),
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
            id: uuid::Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);
        
        let transform_node = TransformNode::new(node);
        
        // Test without input (should not execute)
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert!(!transform_node.can_execute(&context));
        
        // Test with input
        let input_id = uuid::Uuid::new_v4();
        context.inputs.insert(transform_node.node.inputs[0].id, ParameterValue::Image(input_id));
        assert!(transform_node.can_execute(&context));
    }
    
    #[test]
    fn test_core_node_factory() {
        let node = Node::new(NodeType::Input, "Test".to_string());
        
        // Test creating supported node types
        assert!(CoreNodeFactory::create_node(NodeType::Input, node.clone()).is_ok());
        assert!(CoreNodeFactory::create_node(NodeType::Output, node.clone()).is_ok());
        assert!(CoreNodeFactory::create_node(NodeType::Transform, node.clone()).is_ok());
        assert!(CoreNodeFactory::create_node(NodeType::Merge, node.clone()).is_ok());
        assert!(CoreNodeFactory::create_node(NodeType::ColorCorrection, node.clone()).is_ok());
        
        // Test unsupported node type
        assert!(CoreNodeFactory::create_node(NodeType::Blur, node).is_err());
    }
}
