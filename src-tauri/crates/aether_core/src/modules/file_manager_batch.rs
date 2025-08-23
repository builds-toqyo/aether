use anyhow::{anyhow, Result};
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::file_manager::{FileManager, MediaInfo, ThumbnailOptions};

/// Status of a batch operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchStatus {
    /// Operation is queued but not started
    Queued,
    /// Operation is in progress
    InProgress,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
    /// Operation was cancelled
    Cancelled,
}

/// Result of a batch operation
#[derive(Debug, Clone)]
pub struct BatchResult<T> {
    /// Status of the operation
    pub status: BatchStatus,
    /// Result of the operation if completed
    pub result: Option<T>,
    /// Error message if failed
    pub error: Option<String>,
    /// Progress percentage (0-100)
    pub progress: u8,
}

impl<T> Default for BatchResult<T> {
    fn default() -> Self {
        Self {
            status: BatchStatus::Queued,
            result: None,
            error: None,
            progress: 0,
        }
    }
}

/// Batch operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchOperationType {
    /// Analyze media files
    Analyze,
    /// Generate thumbnails
    Thumbnail,
    /// Extract frames from videos
    ExtractFrames,
    /// Convert media files
    Convert,
}

/// Batch operation configuration
#[derive(Debug, Clone)]
pub struct BatchOperation {
    /// Operation type
    pub operation_type: BatchOperationType,
    /// Input files or directories
    pub inputs: Vec<PathBuf>,
    /// Output directory
    pub output_dir: Option<PathBuf>,
    /// Operation-specific options
    pub options: BatchOperationOptions,
}

/// Options for batch operations
#[derive(Debug, Clone)]
pub enum BatchOperationOptions {
    /// No specific options
    None,
    /// Thumbnail generation options
    Thumbnail(ThumbnailOptions),
    /// Frame extraction options
    ExtractFrames {
        /// Frames per second
        fps: f64,
    },
    /// Conversion options
    Convert {
        /// Target format
        format: String,
        /// Quality (0-100)
        quality: u8,
        /// Whether to preserve original aspect ratio
        preserve_aspect_ratio: bool,
        /// Target width (if any)
        width: Option<u32>,
        /// Target height (if any)
        height: Option<u32>,
    },
}

/// Batch processor for file operations
pub struct BatchProcessor {
    /// File manager instance
    file_manager: Arc<FileManager>,
    /// Batch operations queue
    operations: Arc<Mutex<Vec<(u64, BatchOperation)>>>,
    /// Batch operation results
    results: Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>,
    /// Next operation ID
    next_id: Arc<Mutex<u64>>,
    /// Whether the processor is running
    running: Arc<Mutex<bool>>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(file_manager: FileManager) -> Self {
        Self {
            file_manager: Arc::new(file_manager),
            operations: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(1)),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the batch processor
    pub fn start(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        // Clone Arc references for the worker thread
        let operations = self.operations.clone();
        let results = self.results.clone();
        let file_manager = self.file_manager.clone();
        let running_flag = self.running.clone();
        
        // Start worker thread
        thread::spawn(move || {
            Self::worker_thread(operations, results, file_manager, running_flag);
        });
        
        Ok(())
    }
    
    /// Stop the batch processor
    pub fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        *running = false;
        Ok(())
    }
    
    /// Add a batch operation to the queue
    pub fn add_operation(&self, operation: BatchOperation) -> Result<u64> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        // Add operation to queue
        self.operations.lock().unwrap().push((id, operation));
        
        // Initialize result
        self.results.lock().unwrap().push((id, BatchResult::default()));
        
        // Start processor if not already running
        self.start()?;
        
        Ok(id)
    }
    
    /// Get the status of a batch operation
    pub fn get_status(&self, id: u64) -> Result<BatchResult<Vec<PathBuf>>> {
        let results = self.results.lock().unwrap();
        
        for (op_id, result) in results.iter() {
            if *op_id == id {
                return Ok(result.clone());
            }
        }
        
        Err(anyhow!("Operation not found: {}", id))
    }
    
    /// Cancel a batch operation
    pub fn cancel_operation(&self, id: u64) -> Result<()> {
        let mut operations = self.operations.lock().unwrap();
        let mut results = self.results.lock().unwrap();
        
        // Remove from queue if not started
        operations.retain(|(op_id, _)| *op_id != id);
        
        // Mark as cancelled if in results
        for (op_id, result) in results.iter_mut() {
            if *op_id == id && result.status != BatchStatus::Completed && result.status != BatchStatus::Failed {
                result.status = BatchStatus::Cancelled;
                return Ok(());
            }
        }
        
        Err(anyhow!("Operation not found or already completed: {}", id))
    }
    
    /// Clear completed operations
    pub fn clear_completed(&self) -> Result<()> {
        let mut results = self.results.lock().unwrap();
        
        results.retain(|(_, result)| {
            result.status != BatchStatus::Completed && 
            result.status != BatchStatus::Failed && 
            result.status != BatchStatus::Cancelled
        });
        
        Ok(())
    }
    
    /// Worker thread for processing batch operations
    fn worker_thread(
        operations: Arc<Mutex<Vec<(u64, BatchOperation)>>>,
        results: Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>,
        file_manager: Arc<FileManager>,
        running: Arc<Mutex<bool>>
    ) {
        while *running.lock().unwrap() {
            // Get next operation
            let operation_opt = {
                let mut operations = operations.lock().unwrap();
                if operations.is_empty() {
                    None
                } else {
                    Some(operations.remove(0))
                }
            };
            
            if let Some((id, operation)) = operation_opt {
                // Update status to in progress
                {
                    let mut results = results.lock().unwrap();
                    for (op_id, result) in results.iter_mut() {
                        if *op_id == id {
                            result.status = BatchStatus::InProgress;
                            break;
                        }
                    }
                }
                
                // Process operation
                let operation_result = match operation.operation_type {
                    BatchOperationType::Analyze => {
                        Self::process_analyze(&file_manager, &operation, id, &results)
                    },
                    BatchOperationType::Thumbnail => {
                        Self::process_thumbnail(&file_manager, &operation, id, &results)
                    },
                    BatchOperationType::ExtractFrames => {
                        Self::process_extract_frames(&file_manager, &operation, id, &results)
                    },
                    BatchOperationType::Convert => {
                        Self::process_convert(&file_manager, &operation, id, &results)
                    },
                };
                
                // Update result
                {
                    let mut results = results.lock().unwrap();
                    for (op_id, result) in results.iter_mut() {
                        if *op_id == id {
                            match operation_result {
                                Ok(output_paths) => {
                                    result.status = BatchStatus::Completed;
                                    result.result = Some(output_paths);
                                    result.progress = 100;
                                },
                                Err(e) => {
                                    result.status = BatchStatus::Failed;
                                    result.error = Some(e.to_string());
                                },
                            }
                            break;
                        }
                    }
                }
            } else {
                // No operations, sleep for a bit
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    
    /// Process analyze operation
    fn process_analyze(
        file_manager: &FileManager,
        operation: &BatchOperation,
        id: u64,
        results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {
        let mut processed_files = Vec::new();
        let total_files = operation.inputs.len();
        
        for (i, path) in operation.inputs.iter().enumerate() {
            // Check if cancelled
            {
                let results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter() {
                    if *op_id == id && result.status == BatchStatus::Cancelled {
                        return Err(anyhow!("Operation cancelled"));
                    }
                }
            }
            
            // Update progress
            {
                let mut results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter_mut() {
                    if *op_id == id {
                        result.progress = ((i as f32 / total_files as f32) * 100.0) as u8;
                        break;
                    }
                }
            }
            
            // Process file
            if path.is_file() {
                let _ = file_manager.get_media_info(path)?;
                processed_files.push(path.to_path_buf());
            } else if path.is_dir() {
                // Process all files in directory
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        let _ = file_manager.get_media_info(&entry_path)?;
                        processed_files.push(entry_path);
                    }
                }
            }
        }
        
        Ok(processed_files)
    }
    
    /// Process thumbnail operation
    fn process_thumbnail(
        file_manager: &FileManager,
        operation: &BatchOperation,
        id: u64,
        results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {
        let mut thumbnail_paths = Vec::new();
        let total_files = operation.inputs.len();
        
        // Get thumbnail options
        let options = match &operation.options {
            BatchOperationOptions::Thumbnail(opts) => Some(opts.clone()),
            _ => None,
        };
        
        for (i, path) in operation.inputs.iter().enumerate() {
            // Check if cancelled
            {
                let results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter() {
                    if *op_id == id && result.status == BatchStatus::Cancelled {
                        return Err(anyhow!("Operation cancelled"));
                    }
                }
            }
            
            // Update progress
            {
                let mut results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter_mut() {
                    if *op_id == id {
                        result.progress = ((i as f32 / total_files as f32) * 100.0) as u8;
                        break;
                    }
                }
            }
            
            // Process file
            if path.is_file() {
                let thumbnail_path = file_manager.generate_thumbnail(path, options.clone())?;
                thumbnail_paths.push(thumbnail_path);
            } else if path.is_dir() {
                // Process all files in directory
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        let thumbnail_path = file_manager.generate_thumbnail(&entry_path, options.clone())?;
                        thumbnail_paths.push(thumbnail_path);
                    }
                }
            }
        }
        
        Ok(thumbnail_paths)
    }
    
    /// Process extract frames operation
    fn process_extract_frames(
        file_manager: &FileManager,
        operation: &BatchOperation,
        id: u64,
        results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {
        let mut frame_paths = Vec::new();
        let total_files = operation.inputs.len();
        
        // Get fps
        let fps = match &operation.options {
            BatchOperationOptions::ExtractFrames { fps } => *fps,
            _ => 1.0, // Default to 1 fps
        };
        
        // Get output directory
        let output_dir = operation.output_dir.clone()
            .ok_or_else(|| anyhow!("Output directory required for frame extraction"))?;
        
        for (i, path) in operation.inputs.iter().enumerate() {
            // Check if cancelled
            {
                let results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter() {
                    if *op_id == id && result.status == BatchStatus::Cancelled {
                        return Err(anyhow!("Operation cancelled"));
                    }
                }
            }
            
            // Update progress
            {
                let mut results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter_mut() {
                    if *op_id == id {
                        result.progress = ((i as f32 / total_files as f32) * 100.0) as u8;
                        break;
                    }
                }
            }
            
            // Process file
            if path.is_file() {
                // Create subdirectory for this file
                let file_name = path.file_stem().unwrap_or_default().to_string_lossy();
                let file_output_dir = output_dir.join(file_name.to_string());
                std::fs::create_dir_all(&file_output_dir)?;
                
                // Extract frames
                let frames = file_manager.extract_frames(path, &file_output_dir, fps)?;
                frame_paths.extend(frames);
            }
        }
        
        Ok(frame_paths)
    }
    
    /// Process convert operation
    fn process_convert(
        file_manager: &FileManager,
        operation: &BatchOperation,
        id: u64,
        results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {
        // This is a placeholder for media conversion functionality
        // In a real implementation, this would use GStreamer to convert media files
        
        Err(anyhow!("Media conversion not implemented yet"))
    }
}
