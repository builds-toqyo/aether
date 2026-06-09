

#[cfg(test)]
mod tests {
    use std::collections::HashMap;


    #[derive(Debug, Clone, PartialEq)]
    pub enum PortType {
        Image,
        Float,
        Color,
        Vector2,
        Vector3,
        Bool,
        String,
    }


    #[derive(Debug, Clone, PartialEq)]
    pub enum PortDirection {
        Input,
        Output,
    }


    #[derive(Debug, Clone)]
    pub struct TestPort {
        pub name: String,
        pub port_type: PortType,
        pub direction: PortDirection,
        pub connected_to: Option<(String, String)>,
    }


    #[derive(Debug, Clone)]
    pub struct TestNode {
        pub id: String,
        pub name: String,
        pub inputs: Vec<TestPort>,
        pub outputs: Vec<TestPort>,
    }

    impl TestNode {
        pub fn new(id: &str, name: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
            }
        }

        pub fn add_input(&mut self, name: &str, port_type: PortType) {
            self.inputs.push(TestPort {
                name: name.to_string(),
                port_type,
                direction: PortDirection::Input,
                connected_to: None,
            });
        }

        pub fn add_output(&mut self, name: &str, port_type: PortType) {
            self.outputs.push(TestPort {
                name: name.to_string(),
                port_type,
                direction: PortDirection::Output,
                connected_to: None,
            });
        }

        pub fn get_input(&self, name: &str) -> Option<&TestPort> {
            self.inputs.iter().find(|p| p.name == name)
        }

        pub fn get_output(&self, name: &str) -> Option<&TestPort> {
            self.outputs.iter().find(|p| p.name == name)
        }
    }


    pub struct TestGraph {
        pub nodes: HashMap<String, TestNode>,
        pub connections: Vec<(String, String, String, String)>,
    }

    impl TestGraph {
        pub fn new() -> Self {
            Self {
                nodes: HashMap::new(),
                connections: Vec::new(),
            }
        }

        pub fn add_node(&mut self, node: TestNode) {
            self.nodes.insert(node.id.clone(), node);
        }

        pub fn can_connect(&self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Result<(), String> {

            let from = self.nodes.get(from_node)
                .ok_or_else(|| format!("Source node '{}' not found", from_node))?;
            let to = self.nodes.get(to_node)
                .ok_or_else(|| format!("Target node '{}' not found", to_node))?;


            let output = from.get_output(from_port)
                .ok_or_else(|| format!("Output port '{}' not found on node '{}'", from_port, from_node))?;
            let input = to.get_input(to_port)
                .ok_or_else(|| format!("Input port '{}' not found on node '{}'", to_port, to_node))?;


            if output.port_type != input.port_type {
                return Err(format!(
                    "Type mismatch: {:?} cannot connect to {:?}",
                    output.port_type, input.port_type
                ));
            }


            if from_node == to_node {
                return Err("Cannot connect a node to itself".to_string());
            }


            if self.connections.iter().any(|(fn_, fp, tn, tp)| {
                fn_ == from_node && fp == from_port && tn == to_node && tp == to_port
            }) {
                return Err("Connection already exists".to_string());
            }

            Ok(())
        }

        pub fn connect(&mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Result<(), String> {
            self.can_connect(from_node, from_port, to_node, to_port)?;
            self.connections.push((
                from_node.to_string(),
                from_port.to_string(),
                to_node.to_string(),
                to_port.to_string(),
            ));
            Ok(())
        }

        pub fn disconnect(&mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> bool {
            let initial_len = self.connections.len();
            self.connections.retain(|(fn_, fp, tn, tp)| {
                !(fn_ == from_node && fp == from_port && tn == to_node && tp == to_port)
            });
            self.connections.len() < initial_len
        }

        pub fn has_cycle(&self) -> bool {

            let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
            for node_id in self.nodes.keys() {
                adj.insert(node_id.as_str(), Vec::new());
            }
            for (from, _, to, _) in &self.connections {
                adj.get_mut(from.as_str()).unwrap().push(to.as_str());
            }


            let mut visited: HashMap<&str, bool> = HashMap::new();
            let mut rec_stack: HashMap<&str, bool> = HashMap::new();

            for node_id in self.nodes.keys() {
                visited.insert(node_id.as_str(), false);
                rec_stack.insert(node_id.as_str(), false);
            }

            fn dfs<'a>(
                node: &'a str,
                adj: &HashMap<&'a str, Vec<&'a str>>,
                visited: &mut HashMap<&'a str, bool>,
                rec_stack: &mut HashMap<&'a str, bool>,
            ) -> bool {
                visited.insert(node, true);
                rec_stack.insert(node, true);

                if let Some(neighbors) = adj.get(node) {
                    for &neighbor in neighbors {
                        if !visited.get(neighbor).copied().unwrap_or(false) {
                            if dfs(neighbor, adj, visited, rec_stack) {
                                return true;
                            }
                        } else if rec_stack.get(neighbor).copied().unwrap_or(false) {
                            return true;
                        }
                    }
                }

                rec_stack.insert(node, false);
                false
            }

            for node_id in self.nodes.keys() {
                if !visited.get(node_id.as_str()).copied().unwrap_or(false) {
                    if dfs(node_id.as_str(), &adj, &mut visited, &mut rec_stack) {
                        return true;
                    }
                }
            }

            false
        }
    }

    #[test]
    fn test_valid_connection() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "Source");
        node1.add_output("output", PortType::Image);
        graph.add_node(node1);

        let mut node2 = TestNode::new("node2", "Destination");
        node2.add_input("input", PortType::Image);
        graph.add_node(node2);

        assert!(graph.connect("node1", "output", "node2", "input").is_ok());
        assert_eq!(graph.connections.len(), 1);
    }

    #[test]
    fn test_type_mismatch_connection() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "Source");
        node1.add_output("output", PortType::Image);
        graph.add_node(node1);

        let mut node2 = TestNode::new("node2", "Destination");
        node2.add_input("input", PortType::Float);
        graph.add_node(node2);

        let result = graph.connect("node1", "output", "node2", "input");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_self_connection_prevention() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "Node");
        node1.add_output("output", PortType::Image);
        node1.add_input("input", PortType::Image);
        graph.add_node(node1);

        let result = graph.connect("node1", "output", "node1", "input");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot connect a node to itself"));
    }

    #[test]
    fn test_duplicate_connection_prevention() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "Source");
        node1.add_output("output", PortType::Image);
        graph.add_node(node1);

        let mut node2 = TestNode::new("node2", "Destination");
        node2.add_input("input", PortType::Image);
        graph.add_node(node2);

        assert!(graph.connect("node1", "output", "node2", "input").is_ok());
        let result = graph.connect("node1", "output", "node2", "input");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_nonexistent_node_connection() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "Source");
        node1.add_output("output", PortType::Image);
        graph.add_node(node1);

        let result = graph.connect("node1", "output", "nonexistent", "input");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_nonexistent_port_connection() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "Source");
        node1.add_output("output", PortType::Image);
        graph.add_node(node1);

        let mut node2 = TestNode::new("node2", "Destination");
        node2.add_input("input", PortType::Image);
        graph.add_node(node2);

        let result = graph.connect("node1", "nonexistent", "node2", "input");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_disconnect() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "Source");
        node1.add_output("output", PortType::Image);
        graph.add_node(node1);

        let mut node2 = TestNode::new("node2", "Destination");
        node2.add_input("input", PortType::Image);
        graph.add_node(node2);

        graph.connect("node1", "output", "node2", "input").unwrap();
        assert_eq!(graph.connections.len(), 1);

        assert!(graph.disconnect("node1", "output", "node2", "input"));
        assert_eq!(graph.connections.len(), 0);
    }

    #[test]
    fn test_cycle_detection_no_cycle() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "A");
        node1.add_output("out", PortType::Image);
        graph.add_node(node1);

        let mut node2 = TestNode::new("node2", "B");
        node2.add_input("in", PortType::Image);
        node2.add_output("out", PortType::Image);
        graph.add_node(node2);

        let mut node3 = TestNode::new("node3", "C");
        node3.add_input("in", PortType::Image);
        graph.add_node(node3);

        graph.connect("node1", "out", "node2", "in").unwrap();
        graph.connect("node2", "out", "node3", "in").unwrap();

        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_multiple_port_types() {
        let mut graph = TestGraph::new();

        let mut node1 = TestNode::new("node1", "Multi-Output");
        node1.add_output("image", PortType::Image);
        node1.add_output("alpha", PortType::Float);
        node1.add_output("color", PortType::Color);
        graph.add_node(node1);

        let mut node2 = TestNode::new("node2", "Multi-Input");
        node2.add_input("image", PortType::Image);
        node2.add_input("opacity", PortType::Float);
        node2.add_input("tint", PortType::Color);
        graph.add_node(node2);

        assert!(graph.connect("node1", "image", "node2", "image").is_ok());
        assert!(graph.connect("node1", "alpha", "node2", "opacity").is_ok());
        assert!(graph.connect("node1", "color", "node2", "tint").is_ok());
        assert_eq!(graph.connections.len(), 3);
    }
}
