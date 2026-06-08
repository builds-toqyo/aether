use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PinDataType {
    Image,
    Float,
    Vector2,
    Vector3,
    Vector4,
    Color,
    Integer,
    Boolean,
    String,
    Array(Box<PinDataType>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputPin {
    pub id: Uuid,
    pub name: String,
    pub data_type: PinDataType,
    pub required: bool,
    pub default_value: ParameterValue,
    pub current_value: ParameterValue,
    pub connection: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPin {
    pub id: Uuid,
    pub name: String,
    pub data_type: PinDataType,
    pub value: ParameterValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub id: Uuid,
    pub name: String,
    pub data_type: PinDataType,
    pub value: ParameterValue,
    pub default_value: ParameterValue,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub animatable: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    None,
    Image(Uuid),
    Float(f32),
    Vector2(f32, f32),
    Vector3(f32, f32, f32),
    Vector4(f32, f32, f32, f32),
    Color(f32, f32, f32, f32),
    Integer(i32),
    Boolean(bool),
    String(String),
    Array(Vec<ParameterValue>),
    Binary(Vec<u8>),
}

impl Default for ParameterValue {
    fn default() -> Self {
        ParameterValue::None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    Input,
    Output,
    Merge,
    Transform,
    ColorCorrection,
    Blur,
    Sharpen,
    Crop,
    Scale,
    Rotate,
    Text,
    Shape,
    Mask,
    Key,
    Generator,
    Filter,
    Adjustment,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub node_type: NodeType,
    pub name: String,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub enabled: bool,
    pub selected: bool,
    pub inputs: Vec<InputPin>,
    pub outputs: Vec<OutputPin>,
    pub parameters: HashMap<String, Parameter>,
    pub metadata: HashMap<String, String>,
}

impl Node {
    pub fn new(node_type: NodeType, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type,
            name,
            position: (0.0, 0.0),
            size: (200.0, 150.0),
            enabled: true,
            selected: false,
            inputs: Vec::new(),
            outputs: Vec::new(),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_input(&mut self, pin: InputPin) {
        self.inputs.push(pin);
    }

    pub fn add_output(&mut self, pin: OutputPin) {
        self.outputs.push(pin);
    }

    pub fn add_parameter(&mut self, param: Parameter) {
        self.parameters.insert(param.name.clone(), param);
    }

    pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }

    pub fn get_parameter_mut(&mut self, name: &str) -> Option<&mut Parameter> {
        self.parameters.get_mut(name)
    }

    pub fn get_input_pin(&self, id: &Uuid) -> Option<&InputPin> {
        self.inputs.iter().find(|pin| &pin.id == id)
    }

    pub fn get_input_pin_mut(&mut self, id: &Uuid) -> Option<&mut InputPin> {
        self.inputs.iter_mut().find(|pin| &pin.id == id)
    }

    pub fn get_output_pin(&self, id: &Uuid) -> Option<&OutputPin> {
        self.outputs.iter().find(|pin| &pin.id == id)
    }

    pub fn get_output_pin_mut(&mut self, id: &Uuid) -> Option<&mut OutputPin> {
        self.outputs.iter_mut().find(|pin| &pin.id == id)
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.size = (width, height);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn set_output_value(&mut self, name: &str, value: ParameterValue) {
        if let Some(pin) = self.outputs.iter_mut().find(|pin| pin.name == name) {
            pin.value = value;
        }
    }

    pub fn get_input_value(&self, name: &str) -> Option<ParameterValue> {
        self.inputs.iter().find(|pin| pin.name == name).map(|pin| pin.current_value.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub output_node_id: Uuid,
    pub output_pin_id: Uuid,
    pub input_node_id: Uuid,
    pub input_pin_id: Uuid,
    pub enabled: bool,
}

impl Connection {
    pub fn new(
        output_node_id: Uuid,
        output_pin_id: Uuid,
        input_node_id: Uuid,
        input_pin_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            output_node_id,
            output_pin_id,
            input_node_id,
            input_pin_id,
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub id: Uuid,
    pub name: String,
    pub nodes: HashMap<Uuid, Node>,
    pub connections: HashMap<Uuid, Connection>,
    pub metadata: HashMap<String, String>,
    pub enabled: bool,
}

impl Graph {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            nodes: HashMap::new(),
            connections: HashMap::new(),
            metadata: HashMap::new(),
            enabled: true,
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn remove_node(&mut self, node_id: &Uuid) -> Option<Node> {
        self.connections.retain(|_, connection| {
            connection.output_node_id != *node_id && connection.input_node_id != *node_id
        });

        self.nodes.remove(node_id)
    }

    pub fn get_node(&self, node_id: &Uuid) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &Uuid) -> Option<&mut Node> {
        self.nodes.get_mut(node_id)
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.insert(connection.id, connection);
    }

    pub fn remove_connection(&mut self, connection_id: &Uuid) -> Option<Connection> {
        self.connections.remove(connection_id)
    }

    pub fn get_connection(&self, connection_id: &Uuid) -> Option<&Connection> {
        self.connections.get(connection_id)
    }

    pub fn get_connection_mut(&mut self, connection_id: &Uuid) -> Option<&mut Connection> {
        self.connections.get_mut(connection_id)
    }

    pub fn get_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn get_connections(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }

    pub fn get_nodes_by_type(&self, node_type: NodeType) -> impl Iterator<Item = &Node> {
        self.nodes.values().filter(move |node| node.node_type == node_type)
    }

    pub fn get_input_nodes(&self) -> impl Iterator<Item = &Node> {
        self.get_nodes_by_type(NodeType::Input)
    }

    pub fn get_output_nodes(&self) -> impl Iterator<Item = &Node> {
        self.get_nodes_by_type(NodeType::Output)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.connections.clear();
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for connection in self.get_connections() {
            if !self.nodes.contains_key(&connection.output_node_id) {
                errors.push(format!(
                    "Connection {} references non-existent output node {}",
                    connection.id, connection.output_node_id
                ));
            }
            if !self.nodes.contains_key(&connection.input_node_id) {
                errors.push(format!(
                    "Connection {} references non-existent input node {}",
                    connection.id, connection.input_node_id
                ));
            }
        }

        for node in self.get_nodes() {
            for input_pin in &node.inputs {
                if input_pin.required && input_pin.connection.is_none() {
                    errors.push(format!(
                        "Node {} has required input pin '{}' that is not connected",
                        node.name, input_pin.name
                    ));
                }
            }
        }

        errors
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Add,
    Subtract,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    ColorDodge,
    ColorBurn,
    Darken,
    Lighten,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyType {
    ChromaKey,
    LumaKey,
    DifferenceKey,
    ColorKey,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeneratorType {
    Noise,
    Gradient,
    Solid,
    Checkerboard,
    Grid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Ellipse,
    Triangle,
    Star,
    Polygon,
    Path,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterType {
    GaussianBlur,
    BoxBlur,
    MotionBlur,
    RadialBlur,
    Sharpen,
    UnsharpMask,
    Emboss,
    EdgeDetect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdjustmentType {
    Brightness,
    Contrast,
    Saturation,
    Hue,
    Gamma,
    Exposure,
    Levels,
    Curves,
    ColorBalance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(NodeType::Input, "Test Input".to_string());
        assert_eq!(node.node_type, NodeType::Input);
        assert_eq!(node.name, "Test Input");
        assert!(node.enabled);
        assert!(!node.selected);
    }

    #[test]
    fn test_graph_creation() {
        let graph = Graph::new("Test Graph".to_string());
        assert_eq!(graph.name, "Test Graph");
        assert!(graph.enabled);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.connection_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = Graph::new("Test Graph".to_string());
        let node = Node::new(NodeType::Input, "Test Input".to_string());
        let node_id = node.id;

        graph.add_node(node);
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(&node_id).is_some());
    }

    #[test]
    fn test_add_connection() {
        let mut graph = Graph::new("Test Graph".to_string());

        let output_node = Node::new(NodeType::Input, "Output".to_string());
        let input_node = Node::new(NodeType::Output, "Input".to_string());

        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };

        let input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };

        let output_node_id = output_node.id;
        let input_node_id = input_node.id;
        let output_pin_id = output_pin.id;
        let input_pin_id = input_pin.id;

        graph.add_node(output_node);
        graph.add_node(input_node);

        let connection = Connection::new(output_node_id, output_pin_id, input_node_id, input_pin_id);
        graph.add_connection(connection);

        assert_eq!(graph.connection_count(), 1);
    }

    #[test]
    fn test_remove_node() {
        let mut graph = Graph::new("Test Graph".to_string());
        let node = Node::new(NodeType::Input, "Test Input".to_string());
        let node_id = node.id;

        graph.add_node(node);
        assert_eq!(graph.node_count(), 1);

        let removed_node = graph.remove_node(&node_id);
        assert!(removed_node.is_some());
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_validate_empty_graph() {
        let graph = Graph::new("Test Graph".to_string());
        let errors = graph.validate();
        assert!(errors.is_empty());
    }
}
