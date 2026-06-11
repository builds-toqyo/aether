use crate::nodes::{NodeError, NodeResult};
use crate::nodes::validation::TypeChecker;
use aether_types::{Node, PinDataType, ParameterValue};
use log::debug;

pub struct NodeValidator;

impl NodeValidator {
    pub fn validate_node(node: &Node) -> NodeResult<()> {
        debug!("Validating node: {} ({})", node.name, node.id);

        Self::validate_input_pins(node)?;

        Self::validate_output_pins(node)?;

        Self::validate_parameters(node)?;

        Self::validate_node_name(node)?;

        debug!("Node validation passed: {}", node.name);

        Ok(())
    }

    pub fn validate_all_nodes(graph: &aether_types::Graph) -> NodeResult<()> {
        debug!("Validating all {} nodes", graph.nodes.len());

        for node in graph.get_nodes() {
            Self::validate_node(node)?;
        }

        debug!("All nodes validated successfully");

        Ok(())
    }

    fn validate_input_pins(node: &Node) -> NodeResult<()> {
        let mut input_pin_names = std::collections::HashSet::new();

        for (index, input_pin) in node.inputs.iter().enumerate() {

            if input_pin_names.contains(&input_pin.name) {
                return Err(NodeError::InvalidConnection(
                    format!("Duplicate input pin name '{}' in node '{}'", input_pin.name, node.name)
                ));
            }
            input_pin_names.insert(&input_pin.name);


            if input_pin.required && input_pin.connection.is_none() {
                return Err(NodeError::RequiredInputNotConnected(input_pin.name.clone()));
            }


            if input_pin.name.is_empty() {
                return Err(NodeError::InvalidConnection(
                    format!("Input pin {} in node '{}' has empty name", index, node.name)
                ));
            }
        }

        Ok(())
    }

    fn validate_output_pins(node: &Node) -> NodeResult<()> {
        let mut output_pin_names = std::collections::HashSet::new();

        for (index, output_pin) in node.outputs.iter().enumerate() {

            if output_pin_names.contains(&output_pin.name) {
                return Err(NodeError::InvalidConnection(
                    format!("Duplicate output pin name '{}' in node '{}'", output_pin.name, node.name)
                ));
            }
            output_pin_names.insert(&output_pin.name);


            if output_pin.name.is_empty() {
                return Err(NodeError::InvalidConnection(
                    format!("Output pin {} in node '{}' has empty name", index, node.name)
                ));
            }
        }

        Ok(())
    }

    fn validate_parameters(node: &Node) -> NodeResult<()> {
        let mut parameter_names = std::collections::HashSet::new();

        for (name, param) in &node.parameters {

            if parameter_names.contains(name) {
                return Err(NodeError::InvalidConnection(
                    format!("Duplicate parameter name '{}' in node '{}'", name, node.name)
                ));
            }
            parameter_names.insert(name);


            Self::validate_parameter(name, param)?;
        }

        Ok(())
    }

    fn validate_parameter(name: &str, param: &aether_types::Parameter) -> NodeResult<()> {
        if name.is_empty() {
            return Err(NodeError::InvalidConnection(
                "Parameter has empty name".to_string()
            ));
        }


        Self::validate_parameter_type(name, param)?;


        Self::validate_parameter_bounds(name, param)?;

        Ok(())
    }

    fn validate_parameter_type(name: &str, param: &aether_types::Parameter) -> NodeResult<()> {
        match (&param.data_type, &param.value) {
            (PinDataType::Float, ParameterValue::Float(_)) => Ok(()),
            (PinDataType::Integer, ParameterValue::Integer(_)) => Ok(()),
            (PinDataType::Boolean, ParameterValue::Boolean(_)) => Ok(()),
            (PinDataType::String, ParameterValue::String(_)) => Ok(()),
            (PinDataType::Vector2, ParameterValue::Vector2(_, _)) => Ok(()),
            (PinDataType::Vector3, ParameterValue::Vector3(_, _, _)) => Ok(()),
            (PinDataType::Vector4, ParameterValue::Vector4(_, _, _, _)) => Ok(()),
            (PinDataType::Color, ParameterValue::Color(_, _, _, _)) => Ok(()),
            (PinDataType::Array(_), ParameterValue::Array(_)) => Ok(()),
            (PinDataType::Image, ParameterValue::Image(_)) => Ok(()),
            (PinDataType::Image, ParameterValue::None) => Ok(()),
            _ => Err(NodeError::InvalidParameterValue(
                format!("Parameter '{}' has invalid value for type {:?}", name, param.data_type)
            )),
        }
    }

    fn validate_parameter_bounds(name: &str, param: &aether_types::Parameter) -> NodeResult<()> {
        match (&param.min_value, &param.max_value, &param.value) {
            (Some(min), Some(max), value) => {

                if !TypeChecker::are_values_compatible(&ParameterValue::Float(*min), &ParameterValue::Float(*max)) {
                    return Err(NodeError::InvalidParameterValue(
                        format!("Parameter '{}' min and max values are incompatible", name)
                    ));
                }


                if !TypeChecker::is_value_in_bounds(value, &ParameterValue::Float(*min), &ParameterValue::Float(*max))? {
                    return Err(NodeError::InvalidParameterValue(
                        format!("Parameter '{}' value is out of bounds", name)
                    ));
                }
            }
            (Some(min), None, value) => {

                if !TypeChecker::is_value_ge(value, &ParameterValue::Float(*min))? {
                    return Err(NodeError::InvalidParameterValue(
                        format!("Parameter '{}' value is below minimum", name)
                    ));
                }
            }
            (None, Some(max), value) => {

                if !TypeChecker::is_value_le(value, &ParameterValue::Float(*max))? {
                    return Err(NodeError::InvalidParameterValue(
                        format!("Parameter '{}' value is above maximum", name)
                    ));
                }
            }
            (None, None, _) => {

            }
        }

        Ok(())
    }


    fn validate_node_name(node: &Node) -> NodeResult<()> {
        if node.name.is_empty() {
            return Err(NodeError::InvalidConnection(
                format!("Node {} has empty name", node.id)
            ));
        }


        if node.name.contains(|c: char| c.is_control()) {
            return Err(NodeError::InvalidConnection(
                format!("Node '{}' name contains invalid characters", node.name)
            ));
        }

        Ok(())
    }


    pub fn can_execute_node(node: &Node, context: &crate::nodes::ExecutionContext) -> NodeResult<bool> {

        if !node.enabled {
            return Ok(false);
        }

        for input_pin in &node.inputs {
            if input_pin.required && input_pin.connection.is_none() {
                return Ok(false);
            }


            if let Some(_connection_id) = &input_pin.connection {
                if context.get_input(&input_pin.id).is_none() {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    pub fn get_validation_issues(node: &Node) -> Vec<String> {
        let mut issues = Vec::new();


        for input_pin in &node.inputs {
            if input_pin.required && input_pin.connection.is_none() {
                issues.push(format!("Required input '{}' is not connected", input_pin.name));
            }

            if input_pin.name.is_empty() {
                issues.push("Input pin has empty name".to_string());
            }
        }


        for output_pin in &node.outputs {
            if output_pin.name.is_empty() {
                issues.push("Output pin has empty name".to_string());
            }
        }


        for (name, param) in &node.parameters {
            if name.is_empty() {
                issues.push("Parameter has empty name".to_string());
            }

            if let Err(_) = Self::validate_parameter_type(name, param) {
                issues.push(format!("Parameter '{}' has invalid value type", name));
            }

            if let Err(_) = Self::validate_parameter_bounds(name, param) {
                issues.push(format!("Parameter '{}' has invalid bounds", name));
            }
        }


        if node.name.is_empty() {
            issues.push("Node has empty name".to_string());
        }

        issues
    }


    pub fn get_node_stats(node: &Node) -> NodeStats {
        let required_inputs = node.inputs.iter().filter(|pin| pin.required).count();
        let connected_inputs = node.inputs.iter().filter(|pin| pin.connection.is_some()).count();
        let connected_required_inputs = node.inputs.iter()
            .filter(|pin| pin.required && pin.connection.is_some())
            .count();

        NodeStats {
            total_inputs: node.inputs.len(),
            required_inputs,
            connected_inputs,
            connected_required_inputs,
            total_outputs: node.outputs.len(),
            total_parameters: node.parameters.len(),
            enabled: node.enabled,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeStats {

    pub total_inputs: usize,

    pub required_inputs: usize,

    pub connected_inputs: usize,

    pub connected_required_inputs: usize,

    pub total_outputs: usize,

    pub total_parameters: usize,

    pub enabled: bool,
}

impl NodeStats {

    pub fn is_ready(&self) -> bool {
        self.enabled && self.connected_required_inputs == self.required_inputs
    }


    pub fn connection_ratio(&self) -> f64 {
        if self.total_inputs == 0 {
            1.0
        } else {
            self.connected_inputs as f64 / self.total_inputs as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{Node, NodeType, InputPin, OutputPin, ParameterValue, PinDataType};

    #[test]
    fn test_validate_node() {
        let mut node = Node::new(NodeType::Input, "Test Node".to_string());

        let input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: PinDataType::Image,
            required: false,
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


        assert!(NodeValidator::validate_node(&node).is_ok());
    }

    #[test]
    fn test_validate_node_required_input_not_connected() {
        let mut node = Node::new(NodeType::Input, "Test Node".to_string());

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

        let result = NodeValidator::validate_node(&node);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NodeError::RequiredInputNotConnected(_)));
    }

    #[test]
    fn test_validate_node_empty_name() {
        let node = Node::new(NodeType::Input, "".to_string());

        let result = NodeValidator::validate_node(&node);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NodeError::InvalidConnection(_)));
    }

    #[test]
    fn test_validate_parameter() {
        let param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "test_param".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(1.0),
            default_value: ParameterValue::Float(0.0),
            min_value: Some(ParameterValue::Float(0.0)),
            max_value: Some(ParameterValue::Float(2.0)),
        };


        assert!(NodeValidator::validate_parameter("test_param", &param).is_ok());
    }

    #[test]
    fn test_validate_parameter_out_of_bounds() {
        let param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "test_param".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(5.0),
            default_value: ParameterValue::Float(0.0),
            min_value: Some(ParameterValue::Float(0.0)),
            max_value: Some(ParameterValue::Float(2.0)),
        };

        let result = NodeValidator::validate_parameter("test_param", &param);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NodeError::InvalidParameterValue(_)));
    }

    #[test]
    fn test_get_validation_issues() {
        let mut node = Node::new(NodeType::Input, "Test Node".to_string());


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

        let issues = NodeValidator::get_validation_issues(&node);


        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("Required input"));
    }

    #[test]
    fn test_node_stats() {
        let mut node = Node::new(NodeType::Input, "Test Node".to_string());


        let input_pin1 = InputPin {
            id: Uuid::new_v4(),
            name: "input1".to_string(),
            data_type: PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: Some(Uuid::new_v4()),
        };
        node.add_input(input_pin1);

        let input_pin2 = InputPin {
            id: Uuid::new_v4(),
            name: "input2".to_string(),
            data_type: PinDataType::Image,
            required: false,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node.add_input(input_pin2);

        let stats = NodeValidator::get_node_stats(&node);

        assert_eq!(stats.total_inputs, 2);
        assert_eq!(stats.required_inputs, 1);
        assert_eq!(stats.connected_inputs, 1);
        assert_eq!(stats.connected_required_inputs, 1);
        assert!(stats.is_ready());
        assert_eq!(stats.connection_ratio(), 0.5);
    }
}
