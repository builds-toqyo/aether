use std::collections::HashMap;
use uuid::Uuid;
use wgpu::Device;
use anyhow::{Result, anyhow};
use log::{debug, info, warn};


pub struct GpuCpuSynchronization {
    pending_operations: Vec<PendingOperation>,
    last_sync_time: std::time::Instant,
    operation_counter: u64,
}

impl GpuCpuSynchronization {

    pub fn new() -> Self {
        info!("Creating GPU-CPU synchronization manager");

        Self {
            pending_operations: Vec::new(),
            last_sync_time: std::time::Instant::now(),
            operation_counter: 0,
        }
    }


    pub fn track_operation(&mut self, operation_type: String) -> u64 {
        let operation_id = self.operation_counter;
        self.operation_counter += 1;

        let operation = PendingOperation {
            id: Uuid::new_v4(),
            operation_id,
            operation_type,
            submitted_at: std::time::Instant::now(),
        };

        self.pending_operations.push(operation);
        debug!("Tracked GPU operation {}: {}", operation_id, operation.operation_type);

        operation_id
    }


    pub fn wait_for_gpu(&mut self, device: &Device) -> Result<()> {
        debug!("Waiting for GPU to complete {} operations", self.pending_operations.len());


        device.poll(wgpu::Maintain::Wait);


            if self.pending_operations.is_empty() {
                self.last_sync_time = std::time::Instant::now();
                debug!("GPU operations completed within timeout");
                return Ok(true);
            }


            if start_time.elapsed() > timeout {
                warn!("GPU operations did not complete within timeout");
                return Ok(false);
            }


            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }


    pub fn complete_operation(&mut self, operation_id: u64) -> Result<()> {
        let initial_count = self.pending_operations.len();

        self.pending_operations.retain(|op| op.operation_id != operation_id);

        if self.pending_operations.len() < initial_count {
            debug!("Completed GPU operation: {}", operation_id);
        } else {
            warn!("Operation {} not found in pending operations", operation_id);
        }

        Ok(())
    }


    pub fn get_status(&self) -> SyncStatus {
        SyncStatus {
            pending_operations: self.pending_operations.len(),
            last_sync_time: self.last_sync_time,
            is_synced: self.pending_operations.is_empty(),
        }
    }


    pub fn get_pending_operations(&self) -> Vec<&PendingOperation> {
        self.pending_operations.iter().collect()
    }


    pub fn force_clear(&mut self) {
        let count = self.pending_operations.len();
        self.pending_operations.clear();
        self.last_sync_time = std::time::Instant::now();
        warn!("Force cleared {} pending GPU operations", count);
    }


    pub fn get_operation_stats(&self) -> OperationStats {
        let operation_types: HashMap<String, usize> = self.pending_operations
            .iter()
            .map(|op| (op.operation_type.clone(), 0))
            .collect();

        let mut type_counts = HashMap::new();
        for op in &self.pending_operations {
            *type_counts.entry(op.operation_type.clone()).or_insert(0) += 1;
        }

        let oldest_operation = self.pending_operations
            .iter()
            .map(|op| op.submitted_at)
            .min();

        OperationStats {
            pending_count: self.pending_operations.len(),
            operation_types: type_counts,
            oldest_operation_age: oldest_operation
                .map(|time| time.elapsed())
                .unwrap_or_default(),
            last_sync_time: self.last_sync_time,
        }
    }
}


#[derive(Debug, Clone)]
pub struct PendingOperation {
    pub id: Uuid,
    pub operation_id: u64,
    pub operation_type: String,
    pub submitted_at: std::time::Instant,
}


#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub pending_operations: usize,
    pub last_sync_time: std::time::Instant,
    pub is_synced: bool,
}


#[derive(Debug, Clone)]
pub struct OperationStats {
    pub pending_count: usize,
    pub operation_types: std::collections::HashMap<String, usize>,
    pub oldest_operation_age: std::time::Duration,
    pub last_sync_time: std::time::Instant,
}

impl OperationStats {

    pub fn format_stats(&self) -> String {
        let mut type_info = String::new();
        for (op_type, count) in &self.operation_types {
            if !type_info.is_empty() {
                type_info.push_str(", ");
            }
            type_info.push_str(&format!("{}: {}", op_type, count));
        }

        format!(
            "Pending: {} operations ({}) | Oldest: {:?} | Last sync: {:?}",
            self.pending_count,
            type_info,
            self.oldest_operation_age,
            self.last_sync_time.elapsed()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_synchronization_initialization() {
        let sync = GpuCpuSynchronization::new();

        assert_eq!(sync.pending_operations.len(), 0);
        assert_eq!(sync.operation_counter, 0);
        assert!(!sync.last_sync_time.elapsed().is_zero());
    }

    #[test]
    fn test_gpu_operation_tracking() {
        let mut sync = GpuCpuSynchronization::new();


        let op1_id = sync.track_operation("texture_upload".to_string());
        let op2_id = sync.track_operation("buffer_copy".to_string());

        assert_eq!(op1_id, 0);
        assert_eq!(op2_id, 1);
        assert_eq!(sync.pending_operations.len(), 2);
        assert_eq!(sync.operation_counter, 2);


        let operations = sync.get_pending_operations();
        assert_eq!(operations.len(), 2);
        assert_eq!(operations[0].operation_type, "texture_upload");
        assert_eq!(operations[1].operation_type, "buffer_copy");
    }

    #[test]
    fn test_gpu_operation_completion() {
        let mut sync = GpuCpuSynchronization::new();


        let op1_id = sync.track_operation("texture_upload".to_string());
        let op2_id = sync.track_operation("buffer_copy".to_string());

        assert_eq!(sync.pending_operations.len(), 2);


        sync.complete_operation(op1_id).unwrap();
        assert_eq!(sync.pending_operations.len(), 1);


        sync.complete_operation(op2_id).unwrap();
        assert_eq!(sync.pending_operations.len(), 0);


        let result = sync.complete_operation(999);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gpu_sync_status() {
        let mut sync = GpuCpuSynchronization::new();


        let status = sync.get_status();
        assert_eq!(status.pending_operations, 0);
        assert!(status.is_synced);


        sync.track_operation("test_operation".to_string());

        let status = sync.get_status();
        assert_eq!(status.pending_operations, 1);
        assert!(!status.is_synced);


        sync.complete_operation(0).unwrap();

        let status = sync.get_status();
        assert_eq!(status.pending_operations, 0);
        assert!(status.is_synced);
    }

    #[test]
    fn test_gpu_force_clear() {
        let mut sync = GpuCpuSynchronization::new();


        sync.track_operation("op1".to_string());
        sync.track_operation("op2".to_string());
        sync.track_operation("op3".to_string());

        assert_eq!(sync.pending_operations.len(), 3);


        sync.force_clear();

        assert_eq!(sync.pending_operations.len(), 0);
        assert!(!sync.last_sync_time.elapsed().is_zero());
    }

    #[test]
    fn test_operation_stats() {
        let mut sync = GpuCpuSynchronization::new();


        sync.track_operation("texture_upload".to_string());
        sync.track_operation("texture_upload".to_string());
        sync.track_operation("buffer_copy".to_string());

        let stats = sync.get_operation_stats();
        assert_eq!(stats.pending_count, 3);
        assert_eq!(stats.operation_types.get("texture_upload"), Some(&2));
        assert_eq!(stats.operation_types.get("buffer_copy"), Some(&1));

        let formatted = stats.format_stats();
        assert!(formatted.contains("Pending: 3 operations"));
        assert!(formatted.contains("texture_upload: 2"));
        assert!(formatted.contains("buffer_copy: 1"));
    }
}
