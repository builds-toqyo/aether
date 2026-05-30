

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet, VecDeque};


    #[derive(Debug, Clone)]
    pub struct ExecutionNode {
        pub id: String,
        pub name: String,
        pub dependencies: Vec<String>,
        pub executed: bool,
        pub execution_order: Option<usize>,
    }

    impl ExecutionNode {
        pub fn new(id: &str, name: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                dependencies: Vec::new(),
                executed: false,
                execution_order: None,
            }
        }

        pub fn add_dependency(&mut self, dep_id: &str) {
            self.dependencies.push(dep_id.to_string());
        }
    }


    pub struct ExecutionGraph {
        pub nodes: HashMap<String, ExecutionNode>,
    }

    impl ExecutionGraph {
        pub fn new() -> Self {
            Self {
                nodes: HashMap::new(),
            }
        }

        pub fn add_node(&mut self, node: ExecutionNode) {
            self.nodes.insert(node.id.clone(), node);
        }


        pub fn topological_sort(&self) -> Result<Vec<String>, String> {
            let mut in_degree: HashMap<&str, usize> = HashMap::new();
            let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();


            for node_id in self.nodes.keys() {
                in_degree.insert(node_id.as_str(), 0);
                adj.insert(node_id.as_str(), Vec::new());
            }


            for (node_id, node) in &self.nodes {
                for dep_id in &node.dependencies {
                    if let Some(neighbors) = adj.get_mut(dep_id.as_str()) {
                        neighbors.push(node_id.as_str());
                    }
                    if let Some(degree) = in_degree.get_mut(node_id.as_str()) {
                        *degree += 1;
                    }
                }
            }


            let mut queue: VecDeque<&str> = VecDeque::new();
            for (node_id, &degree) in &in_degree {
                if degree == 0 {
                    queue.push_back(*node_id);
                }
            }

            let mut result: Vec<String> = Vec::new();

            while let Some(node_id) = queue.pop_front() {
                result.push(node_id.to_string());

                if let Some(neighbors) = adj.get(node_id) {
                    for &neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(neighbor) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(neighbor);
                            }
                        }
                    }
                }
            }

            if result.len() != self.nodes.len() {
                return Err("Cycle detected in graph".to_string());
            }

            Ok(result)
        }


        pub fn execute(&mut self) -> Result<Vec<String>, String> {
            let order = self.topological_sort()?;

            for (i, node_id) in order.iter().enumerate() {
                if let Some(node) = self.nodes.get_mut(node_id) {

                    for dep_id in &node.dependencies.clone() {
                        if let Some(dep) = self.nodes.get(dep_id) {
                            if !dep.executed {
                                return Err(format!(
                                    "Dependency '{}' not executed before '{}'",
                                    dep_id, node_id
                                ));
                            }
                        }
                    }
                    node.executed = true;
                    node.execution_order = Some(i);
                }
            }

            Ok(order)
        }


        pub fn get_execution_order(&self, node_id: &str) -> Option<usize> {
            self.nodes.get(node_id).and_then(|n| n.execution_order)
        }


        pub fn is_executed(&self, node_id: &str) -> bool {
            self.nodes.get(node_id).map(|n| n.executed).unwrap_or(false)
        }
    }

    #[test]
    fn test_linear_execution_order() {
        let mut graph = ExecutionGraph::new();


        let node_a = ExecutionNode::new("A", "Node A");
        let mut node_b = ExecutionNode::new("B", "Node B");
        node_b.add_dependency("A");
        let mut node_c = ExecutionNode::new("C", "Node C");
        node_c.add_dependency("B");

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);

        let order = graph.execute().unwrap();

        assert_eq!(order, vec!["A", "B", "C"]);
        assert!(graph.is_executed("A"));
        assert!(graph.is_executed("B"));
        assert!(graph.is_executed("C"));
        assert!(graph.get_execution_order("A") < graph.get_execution_order("B"));
        assert!(graph.get_execution_order("B") < graph.get_execution_order("C"));
    }

    #[test]
    fn test_parallel_execution_order() {
        let mut graph = ExecutionGraph::new();


        let node_a = ExecutionNode::new("A", "Node A");
        let node_b = ExecutionNode::new("B", "Node B");
        let mut node_c = ExecutionNode::new("C", "Node C");
        node_c.add_dependency("A");
        node_c.add_dependency("B");

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);

        let order = graph.execute().unwrap();


        assert_eq!(order.last().unwrap(), "C");

        assert!(graph.get_execution_order("A") < graph.get_execution_order("C"));
        assert!(graph.get_execution_order("B") < graph.get_execution_order("C"));
    }

    #[test]
    fn test_diamond_execution_order() {
        let mut graph = ExecutionGraph::new();


        let node_a = ExecutionNode::new("A", "Node A");
        let mut node_b = ExecutionNode::new("B", "Node B");
        node_b.add_dependency("A");
        let mut node_c = ExecutionNode::new("C", "Node C");
        node_c.add_dependency("A");
        let mut node_d = ExecutionNode::new("D", "Node D");
        node_d.add_dependency("B");
        node_d.add_dependency("C");

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);
        graph.add_node(node_d);

        let order = graph.execute().unwrap();


        assert_eq!(order.first().unwrap(), "A");
        assert_eq!(order.last().unwrap(), "D");


        let a_order = graph.get_execution_order("A").unwrap();
        let b_order = graph.get_execution_order("B").unwrap();
        let c_order = graph.get_execution_order("C").unwrap();
        let d_order = graph.get_execution_order("D").unwrap();

        assert!(a_order < b_order);
        assert!(a_order < c_order);
        assert!(b_order < d_order);
        assert!(c_order < d_order);
    }

    #[test]
    fn test_complex_graph_execution() {
        let mut graph = ExecutionGraph::new();


        let node_a = ExecutionNode::new("A", "Input");

        let mut node_b = ExecutionNode::new("B", "Process 1");
        node_b.add_dependency("A");

        let mut node_c = ExecutionNode::new("C", "Process 2");
        node_c.add_dependency("A");

        let mut node_d = ExecutionNode::new("D", "Transform 1");
        node_d.add_dependency("B");

        let mut node_e = ExecutionNode::new("E", "Transform 2");
        node_e.add_dependency("C");
        node_e.add_dependency("B");

        let mut node_f = ExecutionNode::new("F", "Output");
        node_f.add_dependency("D");
        node_f.add_dependency("E");

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);
        graph.add_node(node_d);
        graph.add_node(node_e);
        graph.add_node(node_f);

        let order = graph.execute().unwrap();

        assert_eq!(order.len(), 6);
        assert_eq!(order.first().unwrap(), "A");
        assert_eq!(order.last().unwrap(), "F");


        let get_order = |id: &str| graph.get_execution_order(id).unwrap();

        assert!(get_order("A") < get_order("B"));
        assert!(get_order("A") < get_order("C"));
        assert!(get_order("B") < get_order("D"));
        assert!(get_order("B") < get_order("E"));
        assert!(get_order("C") < get_order("E"));
        assert!(get_order("D") < get_order("F"));
        assert!(get_order("E") < get_order("F"));
    }

    #[test]
    fn test_single_node_execution() {
        let mut graph = ExecutionGraph::new();

        let node = ExecutionNode::new("A", "Single Node");
        graph.add_node(node);

        let order = graph.execute().unwrap();

        assert_eq!(order, vec!["A"]);
        assert!(graph.is_executed("A"));
        assert_eq!(graph.get_execution_order("A"), Some(0));
    }

    #[test]
    fn test_independent_nodes_execution() {
        let mut graph = ExecutionGraph::new();


        graph.add_node(ExecutionNode::new("A", "Node A"));
        graph.add_node(ExecutionNode::new("B", "Node B"));
        graph.add_node(ExecutionNode::new("C", "Node C"));

        let order = graph.execute().unwrap();

        assert_eq!(order.len(), 3);
        assert!(graph.is_executed("A"));
        assert!(graph.is_executed("B"));
        assert!(graph.is_executed("C"));
    }

    #[test]
    fn test_execution_with_missing_dependency() {
        let mut graph = ExecutionGraph::new();

        let mut node = ExecutionNode::new("A", "Node A");
        node.add_dependency("nonexistent");
        graph.add_node(node);


        let result = graph.topological_sort();
        assert!(result.is_ok());
    }

    #[test]
    fn test_deep_chain_execution() {
        let mut graph = ExecutionGraph::new();


        for i in 0..10 {
            let mut node = ExecutionNode::new(&format!("N{}", i), &format!("Node {}", i));
            if i > 0 {
                node.add_dependency(&format!("N{}", i - 1));
            }
            graph.add_node(node);
        }

        let order = graph.execute().unwrap();

        assert_eq!(order.len(), 10);
        for i in 0..10 {
            assert_eq!(order[i], format!("N{}", i));
        }
    }

    #[test]
    fn test_wide_graph_execution() {
        let mut graph = ExecutionGraph::new();


        let input = ExecutionNode::new("input", "Input");
        graph.add_node(input);

        for i in 0..5 {
            let mut node = ExecutionNode::new(&format!("output{}", i), &format!("Output {}", i));
            node.add_dependency("input");
            graph.add_node(node);
        }

        let order = graph.execute().unwrap();

        assert_eq!(order.len(), 6);
        assert_eq!(order.first().unwrap(), "input");


        let input_order = graph.get_execution_order("input").unwrap();
        for i in 0..5 {
            let output_order = graph.get_execution_order(&format!("output{}", i)).unwrap();
            assert!(input_order < output_order);
        }
    }
}
