pub mod core;
pub mod validation;
pub mod execution_order;
pub mod basic;

pub use core::*;
pub use validation::*;
pub use execution_order::*;
pub use basic::*;

use aether_types::{Node, Graph, Connection, NodeType, PinDataType, ParameterValue};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(Uuid),

    #[error("Invalid connection: {0}")]
    InvalidConnection(String),

    #[error("Pin not found: {0}")]
    PinNotFound(Uuid),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Required input not connected: {0}")]
    RequiredInputNotConnected(String),

    #[error("Node execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Parameter not found: {0}")]
    ParameterNotFound(String),

    #[error("Invalid parameter value: {0}")]
    InvalidParameterValue(String),
}

pub type NodeResult<T> = Result<T, NodeError>;

pub trait NodeExecutor {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()>;
    fn node_type(&self) -> NodeType;

    fn validate(&self) -> NodeResult<()> {
        Ok(())
    }

    fn get_inputs(&self) -> Vec<Uuid>;

    fn get_outputs(&self) -> Vec<Uuid>;

    fn can_execute(&self, context: &ExecutionContext) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct ExecutionContext {
    pub frame: u64,
    pub time: f64,
    pub frame_rate: f32,
    pub resolution: (u32, u32),
    pub inputs: HashMap<Uuid, ParameterValue>,
    pub outputs: HashMap<Uuid, ParameterValue>,
    pub global_parameters: HashMap<String, ParameterValue>,
    pub gpu_context: Option<GpuContext>,
}

#[derive(Debug, Clone)]
pub struct GpuContext {
    pub device: u64,
    pub command_queue: u64,
    pub available_memory: u64,
}

impl ExecutionContext {
    pub fn new(frame: u64, time: f64, frame_rate: f32, resolution: (u32, u32)) -> Self {
        Self {
            frame,
            time,
            frame_rate,
            resolution,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            global_parameters: HashMap::new(),
            gpu_context: None,
        }
    }

    pub fn get_input(&self, pin_id: &Uuid) -> Option<&ParameterValue> {
        self.inputs.get(pin_id)
    }

    pub fn set_output(&mut self, pin_id: Uuid, value: ParameterValue) {
        self.outputs.insert(pin_id, value);
    }

    pub fn get_global_parameter(&self, name: &str) -> Option<&ParameterValue> {
        self.global_parameters.get(name)
    }

    pub fn set_global_parameter(&mut self, name: String, value: ParameterValue) {
        self.global_parameters.insert(name, value);
    }
}

pub struct NodeRegistry {
    node_types: HashMap<NodeType, Box<dyn NodeExecutor + Send + Sync>>,
    factories: HashMap<NodeType, fn() -> Box<dyn NodeExecutor + Send + Sync>>,
}

impl std::fmt::Debug for NodeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRegistry")
            .field("node_types", &self.node_types.len())
            .field("factories", &self.factories.len())
            .finish()
    }
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            node_types: HashMap::new(),
            factories: HashMap::new(),
        }
    }

    pub fn register_node_type<F>(&mut self, node_type: NodeType, factory: F)
    where
        F: Fn() -> Box<dyn NodeExecutor + Send + Sync> + 'static,
    {
        self.factories.insert(node_type, factory);
    }

    pub fn create_node(&self, node_type: &NodeType) -> NodeResult<Box<dyn NodeExecutor + Send + Sync>> {
        let factory = self.factories.get(node_type)
            .ok_or_else(|| NodeError::ExecutionFailed(format!("Unknown node type: {:?}", node_type)))?;

        Ok(factory())
    }

    pub fn get_node_types(&self) -> Vec<NodeType> {
        self.factories.keys().cloned().collect()
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NodeManager {
    registry: NodeRegistry,
    nodes: HashMap<Uuid, Box<dyn NodeExecutor + Send + Sync>>,
    node_metadata: HashMap<Uuid, Node>,
}

impl std::fmt::Debug for NodeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeManager")
            .field("registry", &self.registry)
            .field("nodes", &self.nodes.len())
            .field("node_metadata", &self.node_metadata.len())
            .finish()
    }
}

impl NodeManager {
    pub fn new() -> Self {
        Self {
            registry: NodeRegistry::new(),
            nodes: HashMap::new(),
            node_metadata: HashMap::new(),
        }
    }

    pub fn registry(&mut self) -> &mut NodeRegistry {
        &mut self.registry
    }

    pub fn add_node(&mut self, node: Node) -> NodeResult<()> {
        let executor = self.registry.create_node(&node.node_type)?;
        self.nodes.insert(node.id, executor);
        self.node_metadata.insert(node.id, node);
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &Uuid) -> NodeResult<Box<dyn NodeExecutor + Send + Sync>> {

        let executor = self.nodes.remove(node_id)
            .ok_or_else(|| NodeError::NodeNotFound(*node_id))?;


        self.node_metadata.remove(node_id);

        Ok(executor)
    }

    pub fn get_node(&self, node_id: &Uuid) -> Option<&dyn NodeExecutor> {
        self.nodes.get(node_id).map(|executor| {
            let executor: &(dyn NodeExecutor + Send + Sync) = executor.as_ref();
            executor as &dyn NodeExecutor
        })
    }

    pub fn get_node_mut(&mut self, node_id: &Uuid) -> Option<&mut dyn NodeExecutor> {
        self.nodes.get_mut(node_id).map(|executor| {
            let executor: &mut (dyn NodeExecutor + Send + Sync) = executor.as_mut();
            executor as &mut dyn NodeExecutor
        })
    }

    pub fn get_node_metadata(&self, node_id: &Uuid) -> Option<&Node> {
        self.node_metadata.get(node_id)
    }


    pub fn get_node_metadata_mut(&mut self, node_id: &Uuid) -> Option<&mut Node> {
        self.node_metadata.get_mut(node_id)
    }


    pub fn get_nodes(&self) -> impl Iterator<Item = (&Uuid, &dyn NodeExecutor)> {
        self.nodes.iter().map(|(id, executor)| (id, executor.as_ref() as &dyn NodeExecutor))
    }


    pub fn execute_node(&mut self, node_id: &Uuid, context: &mut ExecutionContext) -> NodeResult<()> {
        let executor = self.nodes.get_mut(node_id)
            .ok_or_else(|| NodeError::NodeNotFound(*node_id))?;

        if !executor.can_execute(context) {
            return Err(NodeError::ExecutionFailed("Node cannot execute".to_string()));
        }

        executor.execute(context)
    }


    pub fn validate_all(&self) -> NodeResult<()> {
        for (node_id, executor) in &self.nodes {
            if let Err(e) = executor.validate() {
                return Err(NodeError::ExecutionFailed(format!("Node {} validation failed: {}", node_id, e)));
            }
        }
        Ok(())
    }


    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for NodeManager {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug)]
struct DummyNode;

impl NodeExecutor for DummyNode {
    fn execute(&mut self, _context: &mut ExecutionContext) -> NodeResult<()> {
        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Custom("dummy".to_string())
    }

    fn get_inputs(&self) -> Vec<Uuid> {
        vec![]
    }

    fn get_outputs(&self) -> Vec<Uuid> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{Node, NodeType};

    #[test]
    fn test_execution_context_creation() {
        let context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert_eq!(context.frame, 0);
        assert_eq!(context.time, 0.0);
        assert_eq!(context.frame_rate, 30.0);
        assert_eq!(context.resolution, (1920, 1080));
    }

    #[test]
    fn test_execution_context_inputs_outputs() {
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        let pin_id = Uuid::new_v4();


        assert!(context.get_input(&pin_id).is_none());


        context.set_output(pin_id, ParameterValue::Float(1.0));
        assert!(context.outputs.contains_key(&pin_id));
    }

    #[test]
    fn test_node_registry() {
        let mut registry = NodeRegistry::new();

        assert!(registry.create_node(&NodeType::Input).is_err());

        registry.register_node_type(NodeType::Input, || Box::new(DummyNode));

        let node = registry.create_node(&NodeType::Input);
        assert!(node.is_ok());

        let types = registry.get_node_types();
        assert!(types.contains(&NodeType::Input));
    }

    #[test]
    fn test_node_manager() {
        let mut manager = NodeManager::new();

        assert_eq!(manager.node_count(), 0);
        assert!(manager.get_node(&Uuid::new_v4()).is_none());

        assert!(manager.validate_all().is_ok());
    }
}
