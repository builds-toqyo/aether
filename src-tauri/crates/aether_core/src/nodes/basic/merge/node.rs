use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use crate::nodes::basic::merge::BlendOperations;
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin, BlendMode};
use uuid::Uuid;
use log::debug;


pub struct MergeNode {
    node: Node,
    blend_ops: BlendOperations,
}

impl MergeNode {

    pub fn new(node: Node) -> Self {
        let blend_ops = BlendOperations::new(BlendMode::Normal, 1.0);

        Self {
            node,
            blend_ops,
        }
    }


    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_ops.set_blend_mode(mode);
    }


    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend_ops.get_blend_mode()
    }


    pub fn set_opacity(&mut self, opacity: f32) {
        self.blend_ops.set_opacity(opacity);
    }


    pub fn get_opacity(&self) -> f32 {
        self.blend_ops.get_opacity()
    }


    pub fn create_standard(name: String, input_count: usize) -> Node {
        let mut node = Node::new(NodeType::Merge, name);


        for i in 0..input_count {
            let input_pin = InputPin {
                id: Uuid::new_v4(),
                name: format!("input_{}", i),
                data_type: PinDataType::Image,
                required: i == 0,
                default_value: ParameterValue::None,
                current_value: ParameterValue::None,
                connection: None,
            };
            node.add_input(input_pin);
        }


        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);


        let blend_mode_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "blend_mode".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("normal".to_string()),
            default_value: ParameterValue::String("normal".to_string()),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(blend_mode_param);

        let opacity_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "opacity".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(ParameterValue::Float(0.0)),
            max_value: Some(ParameterValue::Float(1.0)),
        };
        node.add_parameter(opacity_param);

        node
    }


    fn process_multiple_inputs(&self, inputs: &[ParameterValue]) -> ParameterValue {
        if inputs.is_empty() {
            return ParameterValue::None;
        }

        if inputs.len() == 1 {
            return inputs[0].clone();
        }


        let mut result = inputs[0].clone();


        for i in 1..inputs.len() {
            result = self.blend_ops.apply_blend(result, inputs[i].clone());
        }

        result
    }
}

impl NodeExecutor for MergeNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {

        let mut inputs = Vec::new();
        let mut i = 0;

        while let Some(input_value) = self.node.get_input_value(&format!("input_{}", i), context) {
            inputs.push(input_value);
            i += 1;
        }

        debug!("Processing merge with {} inputs", inputs.len());


        let output_value = self.process_multiple_inputs(&inputs);


        self.node.set_output_value("output", output_value);

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Merge
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
    fn test_merge_node_creation() {
        let node = MergeNode::create_standard("Test Merge".to_string(), 2);
        let merge_node = MergeNode::new(node);

        assert_eq!(merge_node.get_blend_mode(), BlendMode::Normal);
        assert_eq!(merge_node.get_opacity(), 1.0);
    }

    #[test]
    fn test_parameter_setting() {
        let node = MergeNode::create_standard("Test".to_string(), 2);
        let mut merge_node = MergeNode::new(node);

        merge_node.set_blend_mode(BlendMode::Multiply);
        assert_eq!(merge_node.get_blend_mode(), BlendMode::Multiply);

        merge_node.set_opacity(0.5);
        assert_eq!(merge_node.get_opacity(), 0.5);
    }

    #[test]
    fn test_standard_node_creation() {
        let node = MergeNode::create_standard("Test".to_string(), 3);

        assert_eq!(node.node_type, NodeType::Merge);
        assert_eq!(node.name, "Test Merge");
        assert_eq!(node.inputs.len(), 3);
        assert_eq!(node.outputs.len(), 1);
        assert!(node.inputs[0].required);
        assert!(!node.inputs[1].required);
        assert!(!node.inputs[2].required);


        let param_names: Vec<String> = node.parameters.iter()
            .map(|p| p.name.clone())
            .collect();
        assert!(param_names.contains(&"blend_mode".to_string()));
        assert!(param_names.contains(&"opacity".to_string()));
    }

    #[test]
    fn test_opacity_clamping() {
        let node = MergeNode::create_standard("Test".to_string(), 2);
        let mut merge_node = MergeNode::new(node);


        merge_node.set_opacity(1.5);
        assert_eq!(merge_node.get_opacity(), 1.0);

        merge_node.set_opacity(-0.5);
        assert_eq!(merge_node.get_opacity(), 0.0);

        merge_node.set_opacity(0.7);
        assert_eq!(merge_node.get_opacity(), 0.7);
    }
}
