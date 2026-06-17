use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue};
use uuid::Uuid;
use log::debug;


#[derive(Debug)]
pub struct CoreTransformNode {
    node: Node,
}

impl CoreTransformNode {

    pub fn new(node: Node) -> Self {
        Self { node }
    }


    fn get_transform_params(&self) -> (f32, f32, f32, f32, f32) {

        let mut pos_x = 0.0;
        let mut pos_y = 0.0;
        let mut scale = 1.0;
        let mut rotation = 0.0;
        let mut anchor_x = 0.0;
        let mut _anchor_y = 0.0;

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

        if let Some(param) = self.node.parameters.get("anchor_x") {
            if let ParameterValue::Float(val) = param.value {
                anchor_x = val;
            }
        }

        if let Some(param) = self.node.parameters.get("anchor_y") {
            if let ParameterValue::Float(val) = param.value {
                _anchor_y = val;
            }
        }

        (pos_x, pos_y, scale, rotation, anchor_x)
    }


    fn apply_transform(&self, input_value: ParameterValue) -> ParameterValue {
        let (pos_x, pos_y, scale, rotation, anchor_x) = self.get_transform_params();

        debug!("Applying transform: pos=({:.2}, {:.2}), scale={:.2}, rotation={:.2}°, anchor_x={:.2}",
            pos_x, pos_y, scale, rotation, anchor_x);


        input_value
    }
}

impl NodeExecutor for CoreTransformNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }


        if let Some(input_pin) = self.node.inputs.first() {
            let input_value = context.get_input(&input_pin.id)
                .cloned()
                .unwrap_or(ParameterValue::None);


            let transformed_value = self.apply_transform(input_value);


            if let Some(output_pin) = self.node.outputs.first() {
                context.set_output(output_pin.id, transformed_value);
            }
        }

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Transform
    }

    fn get_inputs(&self) -> Vec<Uuid> {
        self.node.inputs.iter().map(|pin| pin.id).collect()
    }

    fn get_outputs(&self) -> Vec<Uuid> {
        self.node.outputs.iter().map(|pin| pin.id).collect()
    }

    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.node.enabled &&
        self.node.inputs.first()
            .and_then(|pin| context.get_input(&pin.id))
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{InputPin, OutputPin};

    #[test]
    fn test_core_transform_node_creation() {
        let mut node = Node::new(NodeType::Transform, "Test Transform".to_string());


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

        let transform_node = CoreTransformNode::new(node);

        assert_eq!(transform_node.node_type(), NodeType::Transform);
        assert!(!transform_node.is_transform_active());
    }

    #[test]
    fn test_transform_params() {
        let mut node = Node::new(NodeType::Transform, "Test".to_string());


        let pos_x_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "position_x".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(100.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(pos_x_param);

        let pos_y_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "position_y".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(50.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(pos_y_param);

        let scale_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "scale".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(2.0),
            default_value: ParameterValue::Float(1.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(scale_param);

        let rotation_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "rotation".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(45.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(rotation_param);

        let transform_node = CoreTransformNode::new(node);

        let (pos_x, pos_y, scale, rotation, anchor_x) = transform_node.get_transform_params();

        assert_eq!(pos_x, 100.0);
        assert_eq!(pos_y, 50.0);
        assert_eq!(scale, 2.0);
        assert_eq!(rotation, 45.0);
        assert_eq!(anchor_x, 0.0);

        assert!(transform_node.is_transform_active());
    }

    #[test]
    fn test_apply_transform() {
        let node = Node::new(NodeType::Transform, "Test".to_string());
        let transform_node = CoreTransformNode::new(node);

        let test_value = ParameterValue::Image(Uuid::new_v4());
        let result = transform_node.apply_transform(test_value.clone());


        assert_eq!(result, test_value);
    }

    #[test]
    fn test_node_executor_interface() {
        let mut node = Node::new(NodeType::Transform, "Test Transform".to_string());


        let input_pin_id = Uuid::new_v4();
        let input_pin = InputPin {
            id: input_pin_id,
            name: "input".to_string(),
            data_type: PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node.add_input(input_pin);


        let output_pin_id = Uuid::new_v4();
        let output_pin = OutputPin {
            id: output_pin_id,
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);

        let mut transform_node = CoreTransformNode::new(node);


        assert_eq!(transform_node.node_type(), NodeType::Transform);


        assert_eq!(transform_node.get_inputs().len(), 1);
        assert_eq!(transform_node.get_outputs().len(), 1);


        let context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };
        assert!(!transform_node.can_execute(&context));


        let mut context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };

        let test_value = ParameterValue::Image(Uuid::new_v4());
        context.inputs.insert(input_pin_id, test_value.clone());

        assert!(transform_node.can_execute(&context));


        let result = transform_node.execute(&mut context);
        assert!(result.is_ok());


        assert_eq!(context.outputs.get(&output_pin_id), Some(&test_value));
    }

    #[test]
    fn test_is_transform_active() {
        let node = Node::new(NodeType::Transform, "Test".to_string());
        let transform_node = CoreTransformNode::new(node);


        assert!(!transform_node.is_transform_active());


        let mut node = Node::new(NodeType::Transform, "Test".to_string());

        let pos_x_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "position_x".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(10.0),
            default_value: ParameterValue::Float(0.0),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(pos_x_param);

        let transform_node = CoreTransformNode::new(node);
        assert!(transform_node.is_transform_active());
    }
}
