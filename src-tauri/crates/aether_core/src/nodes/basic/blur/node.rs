use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use crate::nodes::basic::blur::{BlurParams, BlurAlgorithms, BlurType};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;


pub struct BlurNode {
    node: Node,
    params: BlurParams,
    algorithms: BlurAlgorithms,
}

impl BlurNode {

    pub fn new(node: Node) -> Self {
        let params = BlurParams::default();
        let algorithms = BlurAlgorithms::new(params.clone());

        Self {
            node,
            params,
            algorithms,
        }
    }


    pub fn set_blur_radius(&mut self, radius: f32) {
        self.params.radius = radius.clamp(0.0, 100.0);
        self.algorithms.update_params(self.params.clone());
    }


    pub fn get_blur_radius(&self) -> f32 {
        self.params.radius
    }


    pub fn set_blur_type(&mut self, blur_type: BlurType) {
        self.params.blur_type = blur_type;
        self.algorithms.update_params(self.params.clone());
    }


    pub fn get_blur_type(&self) -> BlurType {
        self.params.blur_type.clone()
    }


    pub fn set_iterations(&mut self, iterations: u32) {
        self.params.iterations = iterations.clamp(1, 10);
        self.algorithms.update_params(self.params.clone());
    }


    pub fn get_iterations(&self) -> u32 {
        self.params.iterations
    }


    pub fn set_directional(&mut self, directional: bool) {
        self.params.directional = directional;
        self.algorithms.update_params(self.params.clone());
    }


    pub fn is_directional(&self) -> bool {
        self.params.directional
    }


    pub fn set_angle(&mut self, angle: f32) {
        self.params.angle = angle.rem_euclid(360.0);
        self.algorithms.update_params(self.params.clone());
    }


    pub fn get_angle(&self) -> f32 {
        self.params.angle
    }


    pub fn set_params(&mut self, params: BlurParams) {
        self.params = params.clone();
        self.params.validate();
        self.algorithms.update_params(self.params.clone());
    }


    pub fn get_params(&self) -> &BlurParams {
        &self.params
    }


    pub fn reset_params(&mut self) {
        self.params = BlurParams::default();
        self.algorithms.update_params(self.params.clone());
    }


    pub fn is_active(&self) -> bool {
        self.params.is_active()
    }


    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::Blur, name);


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


        let radius_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "radius".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(5.0),
            default_value: ParameterValue::Float(5.0),
            min_value: Some(0.0),
            max_value: Some(100.0),
            animatable: true,
            description: Some("Blur radius in pixels".to_string()),
        };
        node.add_parameter(radius_param);

        let type_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "blur_type".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("gaussian".to_string()),
            default_value: ParameterValue::String("gaussian".to_string()),
            min_value: None,
            max_value: None,
            animatable: false,
            description: Some("Type of blur (gaussian, box, motion)".to_string()),
        };
        node.add_parameter(type_param);

        let iterations_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "iterations".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(1),
            default_value: ParameterValue::Integer(1),
            min_value: Some(1.0),
            max_value: Some(10.0),
            animatable: false,
            description: Some("Number of blur iterations".to_string()),
        };
        node.add_parameter(iterations_param);

        let angle_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "angle".to_string(),
            data_type: PinDataType::Float,
            value: ParameterValue::Float(0.0),
            default_value: ParameterValue::Float(0.0),
            min_value: Some(0.0),
            max_value: Some(360.0),
            animatable: true,
            description: Some("Motion blur angle in degrees".to_string()),
        };
        node.add_parameter(angle_param);

        node
    }
}

impl NodeExecutor for BlurNode {
    fn execute(&mut self, _context: &mut ExecutionContext) -> NodeResult<()> {

        let input_value = self.node.get_input_value("input").unwrap_or(ParameterValue::None);


        let output_value = self.algorithms.apply_blur(input_value);


        self.node.set_output_value("output", output_value);

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Blur
    }

    fn get_inputs(&self) -> Vec<Uuid> {
        self.node.inputs.iter().map(|pin| pin.id).collect()
    }

    fn get_outputs(&self) -> Vec<Uuid> {
        self.node.outputs.iter().map(|pin| pin.id).collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blur_node_creation() {
        let node = BlurNode::create_standard("Test Blur".to_string());
        let blur_node = BlurNode::new(node);

        assert_eq!(blur_node.get_blur_radius(), 5.0);
        assert_eq!(blur_node.get_blur_type(), BlurType::Gaussian);
        assert_eq!(blur_node.get_iterations(), 1);
        assert!(!blur_node.is_directional());
        assert_eq!(blur_node.get_angle(), 0.0);
    }

    #[test]
    fn test_parameter_setting() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);

        blur_node.set_blur_radius(10.0);
        assert_eq!(blur_node.get_blur_radius(), 10.0);

        blur_node.set_blur_type(BlurType::Box);
        assert_eq!(blur_node.get_blur_type(), BlurType::Box);

        blur_node.set_iterations(3);
        assert_eq!(blur_node.get_iterations(), 3);

        blur_node.set_directional(true);
        assert!(blur_node.is_directional());

        blur_node.set_angle(45.0);
        assert_eq!(blur_node.get_angle(), 45.0);
    }

    #[test]
    fn test_parameter_clamping() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);

        blur_node.set_blur_radius(150.0);
        assert_eq!(blur_node.get_blur_radius(), 100.0);

        blur_node.set_blur_radius(-10.0);
        assert_eq!(blur_node.get_blur_radius(), 0.0);


        blur_node.set_iterations(15);
        assert_eq!(blur_node.get_iterations(), 10);

        blur_node.set_iterations(0);
        assert_eq!(blur_node.get_iterations(), 1);


        blur_node.set_angle(450.0);
        assert_eq!(blur_node.get_angle(), 90.0);

        blur_node.set_angle(-90.0);
        assert_eq!(blur_node.get_angle(), 270.0);
    }

    #[test]
    fn test_is_active() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);


        assert!(blur_node.is_active());


        blur_node.set_blur_radius(0.0);
        assert!(!blur_node.is_active());


        blur_node.set_blur_radius(1.0);
        assert!(blur_node.is_active());
    }

    #[test]
    fn test_reset_params() {
        let node = BlurNode::create_standard("Test".to_string());
        let mut blur_node = BlurNode::new(node);


        blur_node.set_blur_radius(20.0);
        blur_node.set_blur_type(BlurType::Motion);
        blur_node.set_iterations(5);
        blur_node.set_angle(90.0);


        blur_node.reset_params();


        assert_eq!(blur_node.get_blur_radius(), 5.0);
        assert_eq!(blur_node.get_blur_type(), BlurType::Gaussian);
        assert_eq!(blur_node.get_iterations(), 1);
        assert_eq!(blur_node.get_angle(), 0.0);
    }
}
