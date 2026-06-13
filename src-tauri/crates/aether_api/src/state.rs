use std::collections::HashMap;
use std::sync::Mutex;
use aether_types::{Graph, ParameterValue};
use uuid::Uuid;
use log::info;
use crate::commands::editing::MediaInfo;
use crate::commands::rendering::RenderingState;
use crate::commands::multicam::MulticamRegistry;
use crate::commands::plugin::PluginRegistry;
use crate::engine_proxy::EditingEngineProxy;
use crate::project_registry::ProjectRegistry;

pub struct AppState {
    pub graph: Mutex<Graph>,
    pub execution_results: Mutex<HashMap<Uuid, ParameterValue>>,
    pub node_execution_order: Mutex<Vec<Uuid>>,
    pub rendering_state: Mutex<RenderingState>,
    pub editing_engine: Mutex<EditingEngineProxy>,
    pub media_registry: Mutex<HashMap<String, MediaInfo>>,
    pub project_registry: Mutex<ProjectRegistry>,
    pub multicam_registry: Mutex<MulticamRegistry>,
    pub plugin_registry: Mutex<PluginRegistry>,
}

impl AppState {

    pub fn new() -> Self {
        info!("Initializing application state");

        Self {
            graph: Mutex::new(Graph::new("default".to_string())),
            execution_results: Mutex::new(HashMap::new()),
            node_execution_order: Mutex::new(Vec::new()),
            rendering_state: Mutex::new(RenderingState::default()),
            editing_engine: Mutex::new(EditingEngineProxy::new()),
            media_registry: Mutex::new(HashMap::new()),
            project_registry: Mutex::new(ProjectRegistry::new().expect("Failed to initialize project registry")),
            multicam_registry: Mutex::new(MulticamRegistry::new()),
            plugin_registry: Mutex::new(PluginRegistry::new()),
        }
    }


    pub fn get_graph_snapshot(&self) -> Result<Graph, String> {
        self.graph.lock()
            .map(|graph| graph.clone())
            .map_err(|e| format!("Failed to lock graph: {}", e))
    }

    pub fn clear_execution_results(&self) -> Result<(), String> {
        let mut results = self.execution_results.lock()
            .map_err(|e| format!("Failed to lock execution results: {}", e))?;
        results.clear();
        Ok(())
    }


    pub fn store_execution_result(&self, pin_id: Uuid, value: ParameterValue) -> Result<(), String> {
        let mut results = self.execution_results.lock()
            .map_err(|e| format!("Failed to lock execution results: {}", e))?;
        results.insert(pin_id, value);
        Ok(())
    }


    pub fn get_execution_result(&self, pin_id: Uuid) -> Result<Option<ParameterValue>, String> {
        let results = self.execution_results.lock()
            .map_err(|e| format!("Failed to lock execution results: {}", e))?;
        Ok(results.get(&pin_id).cloned())
    }


    pub fn update_execution_order(&self, order: Vec<Uuid>) -> Result<(), String> {
        let mut node_order = self.node_execution_order.lock()
            .map_err(|e| format!("Failed to lock execution order: {}", e))?;
        *node_order = order;
        Ok(())
    }


    pub fn get_execution_order(&self) -> Result<Vec<Uuid>, String> {
        let node_order = self.node_execution_order.lock()
            .map_err(|e| format!("Failed to lock execution order: {}", e))?;
        Ok(node_order.clone())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();


        assert!(state.graph.lock().is_ok());
        assert!(state.execution_results.lock().is_ok());
        assert!(state.node_execution_order.lock().is_ok());
    }

    #[test]
    fn test_execution_results_storage() {
        let state = AppState::new();
        let pin_id = Uuid::new_v4();
        let value = ParameterValue::Float(3.14);


        assert!(state.store_execution_result(pin_id, value.clone()).is_ok());


        let retrieved = state.get_execution_result(pin_id).unwrap();
        assert_eq!(retrieved, Some(value));


        assert!(state.clear_execution_results().is_ok());


        let retrieved = state.get_execution_result(pin_id).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_execution_order() {
        let state = AppState::new();
        let order = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];


        assert!(state.update_execution_order(order.clone()).is_ok());


        let retrieved = state.get_execution_order().unwrap();
        assert_eq!(retrieved, order);
    }
}
