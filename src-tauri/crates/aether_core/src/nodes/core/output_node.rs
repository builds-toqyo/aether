use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, PinDataType, ParameterValue};
use uuid::Uuid;
use log::debug;


#[derive(Debug)]
pub struct CoreOutputNode {
    node: Node,
}

impl CoreOutputNode {

    pub fn new(node: Node) -> Self {
        Self { node }
    }


    pub fn get_final_output(&self, context: &ExecutionContext) -> Option<ParameterValue> {
        if let Some(input_pin) = self.node.inputs.first() {
            if let Some(connection_id) = &input_pin.connection {

                context.get_input(&input_pin.id).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }


    pub fn has_input_connection(&self) -> bool {
        self.node.inputs.first()
            .and_then(|pin| pin.connection)
            .is_some()
    }


    pub fn get_input_connection(&self) -> Option<Uuid> {
        self.node.inputs.first()
            .and_then(|pin| pin.connection)
    }
}

impl NodeExecutor for CoreOutputNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }


        if let Some(input_pin) = self.node.inputs.first() {
            if let Some(connection_id) = &input_pin.connection {
                debug!("CoreOutputNode: Getting input from connection {}", connection_id);


                let output_value = context.get_input(&input_pin.id)
                    .cloned()
                    .unwrap_or(ParameterValue::None);


                if let Some(output_pin) = self.node.outputs.first() {
                    context.set_output(output_pin.id, output_value);
                    debug!("CoreOutputNode: Set final output");
                }
            } else {
                debug!("CoreOutputNode: No input connection");
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

    fn can_execute(&self, _context: &ExecutionContext) -> bool {
        self.node.enabled && self.has_input_connection()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{InputPin, OutputPin};

    #[test]
    fn test_core_output_node_creation() {
        let mut node = Node::new(NodeType::Output, "Test Output".to_string());


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

        let output_node = CoreOutputNode::new(node);

        assert_eq!(output_node.node_type(), NodeType::Output);
        assert!(!output_node.has_input_connection());
        assert_eq!(output_node.get_input_connection(), None);
    }

    #[test]
    fn test_input_connection_operations() {
        let mut node = Node::new(NodeType::Output, "Test".to_string());


        let connection_id = Uuid::new_v4();
        let input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: Some(connection_id),
        };
        node.add_input(input_pin);


        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);

        let output_node = CoreOutputNode::new(node);

        assert!(output_node.has_input_connection());
        assert_eq!(output_node.get_input_connection(), Some(connection_id));
    }

    #[test]
    fn test_get_final_output() {
        let mut node = Node::new(NodeType::Output, "Test".to_string());


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


        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);

        let output_node = CoreOutputNode::new(node);


        let context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };

        assert_eq!(output_node.get_final_output(&context), None);


        let mut context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };

        let test_value = ParameterValue::Image(Uuid::new_v4());
        context.inputs.insert(input_pin_id, test_value.clone());

        assert_eq!(output_node.get_final_output(&context), Some(test_value));
    }

    #[test]
    fn test_node_executor_interface() {
        let mut node = Node::new(NodeType::Output, "Test Output".to_string());


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

        let mut output_node = CoreOutputNode::new(node);


        assert_eq!(output_node.node_type(), NodeType::Output);


        assert_eq!(output_node.get_inputs().len(), 1);
        assert_eq!(output_node.get_outputs().len(), 1);


        let context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };
        assert!(!output_node.can_execute(&context));


        let mut context = ExecutionContext {
            frame: 1,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        };

        let result = output_node.execute(&mut context);
        assert!(result.is_ok());


        let test_value = ParameterValue::Image(Uuid::new_v4());
        context.inputs.insert(input_pin_id, test_value);

        let result = output_node.execute(&mut context);
        assert!(result.is_ok());


        assert_eq!(context.outputs.get(&output_pin_id), Some(&test_value));
    }
}
