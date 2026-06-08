use crate::nodes::{NodeError, NodeResult};
use crate::nodes::validation::{NodeValidator, ConnectionValidator};
use aether_types::Graph;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use log::debug;


pub struct GraphValidator;

impl GraphValidator {

    pub fn validate_graph(graph: &Graph) -> NodeResult<()> {
        debug!("Validating graph with {} nodes and {} connections",
            graph.nodes.len(), graph.connections.len());


        NodeValidator::validate_all_nodes(graph)?;


        ConnectionValidator::validate_all_connections(graph)?;


        Self::check_circular_dependencies(graph)?;


        Self::check_orphaned_nodes(graph)?;


        ConnectionValidator::check_duplicate_connections(graph)?;

        debug!("Graph validation completed successfully");

        Ok(())
    }


    fn check_circular_dependencies(graph: &Graph) -> NodeResult<()> {
        debug!("Checking for circular dependencies");

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();


        let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for connection in graph.get_connections() {
            if !connection.enabled {
                continue;
            }

            adjacency
                .entry(connection.input_node_id)
                .or_insert_with(Vec::new)
                .push(connection.output_node_id);
        }


        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                if Self::has_cycle_util(node_id, &adjacency, &mut visited, &mut rec_stack)? {
                    return Err(NodeError::CircularDependency);
                }
            }
        }

        debug!("No circular dependencies found");

        Ok(())
    }


    fn has_cycle_util(
        node_id: &Uuid,
        adjacency: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
    ) -> NodeResult<bool> {
        visited.insert(*node_id);
        rec_stack.insert(*node_id);

        if let Some(neighbors) = adjacency.get(node_id) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::has_cycle_util(neighbor, adjacency, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(neighbor) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(node_id);
        Ok(false)
    }


    fn check_orphaned_nodes(graph: &Graph) -> NodeResult<()> {
        debug!("Checking for orphaned nodes");

        let mut connected_nodes = HashSet::new();


        for connection in graph.get_connections() {
            if connection.enabled {
                connected_nodes.insert(connection.output_node_id);
                connected_nodes.insert(connection.input_node_id);
            }
        }


        Self::check_io_nodes(graph)?;

        debug!("Graph execution validation passed");

        Ok(())
    }


    fn check_io_nodes(graph: &Graph) -> NodeResult<()> {
        let mut has_input = false;
        let mut has_output = false;

        for node in graph.get_nodes() {
            if node.node_type == aether_types::NodeType::Input {
                has_input = true;
            }
            if node.node_type == aether_types::NodeType::Output {
                has_output = true;
            }

            if has_input && has_output {
                break;
            }
        }

        if !has_input {
            return Err(NodeError::InvalidConnection(
                "Graph must have at least one input node".to_string()
            ));
        }

        if !has_output {
            return Err(NodeError::InvalidConnection(
                "Graph must have at least one output node".to_string()
            ));
        }

        Ok(())
    }


    pub fn get_validation_stats(graph: &Graph) -> GraphValidationStats {
        let mut stats = GraphValidationStats::default();


        for node in graph.get_nodes() {
            match node.node_type {
                aether_types::NodeType::Input => stats.input_nodes += 1,
                aether_types::NodeType::Output => stats.output_nodes += 1,
                aether_types::NodeType::Transform => stats.transform_nodes += 1,
                aether_types::NodeType::Merge => stats.merge_nodes += 1,
                aether_types::NodeType::ColorCorrection => stats.color_correction_nodes += 1,
                aether_types::NodeType::Blur => stats.blur_nodes += 1,
                _ => {}
            }
        }


        stats.total_connections = graph.connections.len();
        stats.enabled_connections = graph.connections.iter().filter(|(_, c)| c.enabled).count();
        stats.disabled_connections = stats.total_connections - stats.enabled_connections;


        let mut connected_nodes = HashSet::new();
        for connection in graph.get_connections() {
            if connection.enabled {
                connected_nodes.insert(connection.output_node_id);
                connected_nodes.insert(connection.input_node_id);
            }
        }
        stats.connected_nodes = connected_nodes.len();


        stats.orphaned_nodes = stats.total_nodes() - stats.connected_nodes;


        stats.has_required_inputs_unconnected = Self::count_unconnected_required_inputs(graph);
        stats.has_circular_dependencies = Self::check_for_cycles(graph).is_err();

        stats
    }


    fn count_unconnected_required_inputs(graph: &Graph) -> usize {
        graph.get_nodes()
            .flat_map(|node| node.inputs.iter())
            .filter(|pin| pin.required && pin.connection.is_none())
            .count()
    }


    fn check_for_cycles(graph: &Graph) -> NodeResult<()> {
        Self::check_circular_dependencies(graph)
    }


    pub fn validate_for_realtime(graph: &Graph) -> NodeResult<()> {
        debug!("Validating graph for realtime execution");


        Self::validate_graph(graph)?;


        Self::check_realtime_issues(graph)?;

        debug!("Graph realtime validation passed");

        Ok(())
    }


    fn check_realtime_issues(graph: &Graph) -> NodeResult<()> {

        if graph.nodes.len() > 100 {
            log::warn!("Graph has {} nodes, which may affect realtime performance", graph.nodes.len());
        }


        let max_depth = Self::calculate_max_depth(graph)?;
        if max_depth > 20 {
            log::warn!("Graph has depth {}, which may affect realtime performance", max_depth);
        }


        for node in graph.get_nodes() {
            match node.node_type {
                aether_types::NodeType::Blur => {
                    log::debug!("Blur node detected: {} (may affect performance)", node.name);
                }
                aether_types::NodeType::ColorCorrection => {
                    log::debug!("Color correction node detected: {} (may affect performance)", node.name);
                }
                _ => {}
            }
        }

        Ok(())
    }


    fn calculate_max_depth(graph: &Graph) -> NodeResult<usize> {

        let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for connection in graph.get_connections() {
            if !connection.enabled {
                continue;
            }

            adjacency
                .entry(connection.input_node_id)
                .or_insert_with(Vec::new)
                .push(connection.output_node_id);
        }


        let mut max_depth = 0;
        let mut visited = HashSet::new();

        for node_id in graph.nodes.keys() {
            let depth = Self::calculate_depth_util(node_id, &adjacency, &mut visited, 0)?;
            max_depth = max_depth.max(depth);
        }

        Ok(max_depth)
    }


    fn calculate_depth_util(
        node_id: &Uuid,
        adjacency: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashSet<Uuid>,
        current_depth: usize,
    ) -> NodeResult<usize> {
        if visited.contains(node_id) {
            return Ok(current_depth);
        }

        visited.insert(*node_id);

        let mut max_child_depth = current_depth;

        if let Some(neighbors) = adjacency.get(node_id) {
            for neighbor in neighbors {
                let child_depth = Self::calculate_depth_util(neighbor, adjacency, visited, current_depth + 1)?;
                max_child_depth = max_child_depth.max(child_depth);
            }
        }

        visited.remove(node_id);
        Ok(max_child_depth)
    }
}


#[derive(Debug, Clone, Default)]
pub struct GraphValidationStats {

    pub input_nodes: usize,

    pub output_nodes: usize,

    pub transform_nodes: usize,

    pub merge_nodes: usize,

    pub color_correction_nodes: usize,

    pub blur_nodes: usize,

    pub total_connections: usize,

    pub enabled_connections: usize,

    pub disabled_connections: usize,

    pub connected_nodes: usize,

    pub orphaned_nodes: usize,

    pub has_required_inputs_unconnected: usize,

    pub has_circular_dependencies: bool,
}

impl GraphValidationStats {

    pub fn total_nodes(&self) -> usize {
        self.input_nodes + self.output_nodes + self.transform_nodes +
        self.merge_nodes + self.color_correction_nodes + self.blur_nodes
    }


    pub fn connection_ratio(&self) -> f64 {
        if self.total_nodes() == 0 {
            0.0
        } else {
            self.connected_nodes as f64 / self.total_nodes() as f64
        }
    }


    pub fn is_ready(&self) -> bool {
        self.has_required_inputs_unconnected == 0 &&
        !self.has_circular_dependencies &&
        self.input_nodes > 0 &&
        self.output_nodes > 0
    }


    pub fn get_summary(&self) -> String {
        format!(
            "Nodes: {} (I:{}, O:{}, T:{}, M:{}, CC:{}, B:{}), Connections: {}/{} (enabled/total), Ready: {}",
            self.total_nodes(),
            self.input_nodes,
            self.output_nodes,
            self.transform_nodes,
            self.merge_nodes,
            self.color_correction_nodes,
            self.blur_nodes,
            self.enabled_connections,
            self.total_connections,
            self.is_ready()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{Node, NodeType, InputPin, OutputPin, ParameterValue};

    fn create_test_graph() -> Graph {
        let mut graph = Graph::new();


        let input_id = Uuid::new_v4();
        let transform_id = Uuid::new_v4();
        let output_id = Uuid::new_v4();

        let mut input_node = Node::new(NodeType::Input, "Input".to_string());
        let mut transform_node = Node::new(NodeType::Transform, "Transform".to_string());
        let mut output_node = Node::new(NodeType::Output, "Output".to_string());


        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: aether_types::PinDataType::Image,
            value: ParameterValue::None,
        };
        input_node.add_output(output_pin);

        let input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: aether_types::PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        transform_node.add_input(input_pin);

        let transform_output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: aether_types::PinDataType::Image,
            value: ParameterValue::None,
        };
        transform_node.add_output(transform_output_pin);

        let output_input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: aether_types::PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        output_node.add_input(output_input_pin);


        graph.nodes.insert(input_id, input_node);
        graph.nodes.insert(transform_id, transform_node);
        graph.nodes.insert(output_id, output_node);


        let connection1 = Connection {
            id: Uuid::new_v4(),
            output_node_id: input_id,
            output_pin_id: graph.nodes[&input_id].outputs[0].id,
            input_node_id: transform_id,
            input_pin_id: graph.nodes[&transform_id].inputs[0].id,
            enabled: true,
        };

        let connection2 = Connection {
            id: Uuid::new_v4(),
            output_node_id: transform_id,
            output_pin_id: graph.nodes[&transform_id].outputs[0].id,
            input_node_id: output_id,
            input_pin_id: graph.nodes[&output_id].inputs[0].id,
            enabled: true,
        };

        graph.connections.push(connection1);
        graph.connections.push(connection2);

        graph
    }

    #[test]
    fn test_validate_graph() {
        let graph = create_test_graph();


        assert!(GraphValidator::validate_graph(&graph).is_ok());
    }

    #[test]
    fn test_validate_execution_readiness() {
        let graph = create_test_graph();


        assert!(GraphValidator::validate_execution_readiness(&graph).is_ok());
    }

    #[test]
    fn test_validate_graph_missing_output() {
        let mut graph = Graph::new();


        let input_id = Uuid::new_v4();
        let mut input_node = Node::new(NodeType::Input, "Input".to_string());
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: aether_types::PinDataType::Image,
            value: ParameterValue::None,
        };
        input_node.add_output(output_pin);
        graph.nodes.insert(input_id, input_node);


        let result = GraphValidator::validate_execution_readiness(&graph);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NodeError::InvalidConnection(_)));
    }

    #[test]
    fn test_get_validation_stats() {
        let graph = create_test_graph();

        let stats = GraphValidator::get_validation_stats(&graph);

        assert_eq!(stats.input_nodes, 1);
        assert_eq!(stats.output_nodes, 1);
        assert_eq!(stats.transform_nodes, 1);
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.enabled_connections, 2);
        assert_eq!(stats.connected_nodes, 3);
        assert_eq!(stats.orphaned_nodes, 0);
        assert!(stats.is_ready());
    }

    #[test]
    fn test_validate_for_realtime() {
        let graph = create_test_graph();


        assert!(GraphValidator::validate_for_realtime(&graph).is_ok());
    }
}
