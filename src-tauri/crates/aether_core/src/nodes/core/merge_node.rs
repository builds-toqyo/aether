use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, PinDataType, ParameterValue};
use uuid::Uuid;
use log::debug;


#[derive(Debug)]
pub struct CoreMergeNode {
    node: Node,
}

impl CoreMergeNode {

    pub fn new(node: Node) -> Self {
        Self { node }
    }


    fn get_blend_mode(&self) -> String {
        if let Some(param) = self.node.parameters.get("blend_mode") {
            if let ParameterValue::String(mode) = &param.value {
                return mode.clone();
            }
        }
        "normal".to_string()
    }


    fn get_opacity(&self) -> f32 {
        if let Some(param) = self.node.parameters.get("opacity") {
            if let ParameterValue::Float(opacity) = param.value {
                return opacity.clamp(0.0, 1.0);
            }
        }
        1.0
    }


    fn apply_blend(&self, inputs: &[ParameterValue]) -> ParameterValue {
        let blend_mode = self.get_blend_mode();
        let opacity = self.get_opacity();

        debug!("Applying blend: {} inputs, mode={}, opacity={:.2}",
            inputs.len(), blend_mode, opacity);


        for input in inputs {
            if !matches!(input, ParameterValue::None) {
                return input.clone();
            }
        }

        ParameterValue::None
    }


    fn get_input_values(&self, context: &ExecutionContext) -> Vec<ParameterValue> {
        self.node.inputs.iter()
            .map(|pin| context.get_input(&pin.id).cloned().unwrap_or(ParameterValue::None))
            .collect()
    }
}

impl NodeExecutor for CoreMergeNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }


        let inputs = self.get_input_values(context);


        let blended_value = self.apply_blend(&inputs);


        if let Some(output_pin) = self.node.outputs.first() {
            context.set_output(output_pin.id, blended_value);
        }

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Merge
    }

    fn get_inputs(&self) -> Vec<Uuid> {
        self.node.inputs.iter().map(|pin| pin.id).collect()
    }

    fn get_outputs(&self) -> Vec<Uuid> {
        self.node.outputs.iter().map(|pin| pin.id).collect()
    }

    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.node.enabled &&
        self.node.inputs.iter().any(|pin| context.get_input(&pin.id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{InputPin, OutputPin};

    #[test]
    fn test_core_merge_node_creation() {
        let mut node = Node::new(NodeType::Merge, "Test Merge".to_string());


        for i in 0..2 {
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

        let merge_node = CoreMergeNode::new(node);

        assert_eq!(merge_node.node_type(), NodeType::Merge);
        assert_eq!(merge_node.get_blend_mode(), "normal");
        assert_eq!(merge_node.get_opacity(), 1.0);
    }

    #[test]
    fn test_blend_parameters() {
        let mut node = Node::new(NodeType::Merge, "Test Merge".to_string());


        let blend_mode_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "blend_mode".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("multiply".to_string()),
            default_value: ParameterValue::String("normal".to_string()),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(blend_mode_param);


        let opacity_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "opacity".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.5),
            default_value: ParameterValue::Float(1.0),
            min_value: Some(ParameterValue::Float(0.0)),
            max_value: Some(ParameterValue::Float(1.0)),
        };
        node.add_parameter(opacity_param);

        let merge_node = CoreMergeNode::new(node);

        assert_eq!(merge_node.get_blend_mode(), "multiply");
        assert_eq!(merge_node.get_opacity(), 0.5);
    }

    #[test]
    fn test_apply_blend() {
        let node = Node::new(NodeType::Merge, "Test Merge".to_string());
        let merge_node = CoreMergeNode::new(node);


        let empty_inputs: Vec<ParameterValue> = vec![];
        let result = merge_node.apply_blend(&empty_inputs);
        assert!(matches!(result, ParameterValue::None));


        let none_inputs: Vec<ParameterValue> = vec![ParameterValue::None, ParameterValue::None];
        let result = merge_node.apply_blend(&none_inputs);
        assert!(matches!(result, ParameterValue::None));


        let test_image = ParameterValue::Image(Uuid::new_v4());
        let valid_inputs: Vec<ParameterValue> = vec![test_image.clone(), ParameterValue::None];
        let result = merge_node.apply_blend(&valid_inputs);
        assert_eq!(result, test_image);


        let test_image1 = ParameterValue::Image(Uuid::new_v4());
        let test_image2 = ParameterValue::Image(Uuid::new_v4());
        let multi_inputs: Vec<ParameterValue> = vec![test_image1.clone(), test_image2.clone()];
        let result = merge_node.apply_blend(&multi_inputs);
        assert_eq!(result, test_image1);
    }

    #[test]
    fn test_get_input_values() {
        let mut node = Node::new(NodeType::Merge, "Test Merge".to_string());


        let input_pin_ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        for (i, &pin_id) in input_pin_ids.iter().enumerate() {
            let input_pin = InputPin {
                id: pin_id,
                name: format!("input_{}", i),
                data_type: PinDataType::Image,
                required: i == 0,
                default_value: ParameterValue::None,
                current_value: ParameterValue::None,
                connection: None,
            };
            node.add_input(input_pin);
        }

        let merge_node = CoreMergeNode::new(node);


        let context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };

        let input_values = merge_node.get_input_values(&context);
        assert_eq!(input_values.len(), 3);
        assert!(input_values.iter().all(|v| matches!(v, ParameterValue::None)));


        let mut context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };

        let test_value = ParameterValue::Image(Uuid::new_v4());
        context.inputs.insert(input_pin_ids[1], test_value.clone());

        let input_values = merge_node.get_input_values(&context);
        assert_eq!(input_values.len(), 3);
        assert_eq!(input_values[0], ParameterValue::None);
        assert_eq!(input_values[1], test_value);
        assert_eq!(input_values[2], ParameterValue::None);
    }

    #[test]
    fn test_node_executor_interface() {
        let mut node = Node::new(NodeType::Merge, "Test Merge".to_string());


        let input_pin_ids: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        for (i, &pin_id) in input_pin_ids.iter().enumerate() {
            let input_pin = InputPin {
                id: pin_id,
                name: format!("input_{}", i),
                data_type: PinDataType::Image,
                required: i == 0,
                default_value: ParameterValue::None,
                current_value: ParameterValue::None,
                connection: None,
            };
            node.add_input(input_pin);
        }


        let output_pin_id = Uuid::new_v4();
        let output_pin = OutputPin {
            id: output_pin_id,
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);

        let mut merge_node = CoreMergeNode::new(node);


        assert_eq!(merge_node.node_type(), NodeType::Merge);


        assert_eq!(merge_node.get_inputs().len(), 2);
        assert_eq!(merge_node.get_outputs().len(), 1);


        let context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };
        assert!(!merge_node.can_execute(&context));


        let mut context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };

        let test_value = ParameterValue::Image(Uuid::new_v4());
        context.inputs.insert(input_pin_ids[0], test_value.clone());

        assert!(merge_node.can_execute(&context));


        let result = merge_node.execute(&mut context);
        assert!(result.is_ok());


        assert_eq!(context.outputs.get(&output_pin_id), Some(&test_value));
    }
}
