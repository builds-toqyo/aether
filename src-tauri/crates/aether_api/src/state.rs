use std::collections::HashMap;
use std::sync::Mutex;
use aether_types::{Graph, ParameterValue};
use uuid::Uuid;
use log::info;

/// Application state shared across Tauri commands
pub struct AppState {
    pub graph: Mutex<Graph>,
    pub execution_results: Mutex<HashMap<Uuid, ParameterValue>>,
    pub node_execution_order: Mutex<Vec<Uuid>>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        info!("Initializing application state");
        
        Self {
            graph: Mutex::new(Graph::new()),
            execution_results: Mutex::new(HashMap::new()),
            node_execution_order: Mutex::new(Vec::new()),
        }
    }
    
    /// Get a snapshot of the current graph
    pub fn get_graph_snapshot(&self) -> Result<Graph, String> {
        self.graph.lock()
            .map(|graph| graph.clone())
            .map_err(|e| format!("Failed to lock graph: {}", e))
    }
    
    /// Clear all execution results
    pub fn clear_execution_results(&self) -> Result<(), String> {
        let mut results = self.execution_results.lock()
            .map_err(|e| format!("Failed to lock execution results: {}", e))?;
        results.clear();
        Ok(())
    }
    
    /// Store execution result for a specific pin
    pub fn store_execution_result(&self, pin_id: Uuid, value: ParameterValue) -> Result<(), String> {
        let mut results = self.execution_results.lock()
            .map_err(|e| format!("Failed to lock execution results: {}", e))?;
        results.insert(pin_id, value);
        Ok(())
    }
    
    /// Get execution result for a specific pin
    pub fn get_execution_result(&self, pin_id: Uuid) -> Result<Option<ParameterValue>, String> {
        let results = self.execution_results.lock()
            .map_err(|e| format!("Failed to lock execution results: {}", e))?;
        Ok(results.get(&pin_id).cloned())
    }
    
    /// Update node execution order
    pub fn update_execution_order(&self, order: Vec<Uuid>) -> Result<(), String> {
        let mut node_order = self.node_execution_order.lock()
            .map_err(|e| format!("Failed to lock execution order: {}", e))?;
        *node_order = order;
        Ok(())
    }
    
    /// Get current execution order
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
        
        // Should be able to lock graph initially
        assert!(state.graph.lock().is_ok());
        assert!(state.execution_results.lock().is_ok());
        assert!(state.node_execution_order.lock().is_ok());
    }
    
    #[test]
    fn test_execution_results_storage() {
        let state = AppState::new();
        let pin_id = Uuid::new_v4();
        let value = ParameterValue::Float(3.14);
        
        // Store result
        assert!(state.store_execution_result(pin_id, value.clone()).is_ok());
        
        // Retrieve result
        let retrieved = state.get_execution_result(pin_id).unwrap();
        assert_eq!(retrieved, Some(value));
        
        // Clear results
        assert!(state.clear_execution_results().is_ok());
        
        // Should be empty now
        let retrieved = state.get_execution_result(pin_id).unwrap();
        assert_eq!(retrieved, None);
    }
    
    #[test]
    fn test_execution_order() {
        let state = AppState::new();
        let order = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        
        // Update order
        assert!(state.update_execution_order(order.clone()).is_ok());
        
        // Retrieve order
        let retrieved = state.get_execution_order().unwrap();
        assert_eq!(retrieved, order);
    }
}
