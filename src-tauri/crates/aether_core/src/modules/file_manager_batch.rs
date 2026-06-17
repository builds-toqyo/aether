use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::file_manager::{FileManager, ThumbnailOptions};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchStatus {

    Queued,

    InProgress,

    Completed,

    Failed,

    Cancelled,
}


#[derive(Debug, Clone)]
pub struct BatchResult<T> {

    pub status: BatchStatus,

    pub result: Option<T>,

    pub error: Option<String>,

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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchOperationType {

    Analyze,

    Thumbnail,

    ExtractFrames,

    Convert,
}


#[derive(Debug, Clone)]
pub struct BatchOperation {

    pub operation_type: BatchOperationType,

    pub inputs: Vec<PathBuf>,

    pub output_dir: Option<PathBuf>,

    pub options: BatchOperationOptions,
}


#[derive(Debug, Clone)]
pub enum BatchOperationOptions {

    None,

    Thumbnail(ThumbnailOptions),

    ExtractFrames {

        fps: f64,
    },

    Convert {

        format: String,

        quality: u8,

        preserve_aspect_ratio: bool,

        width: Option<u32>,

        height: Option<u32>,
    },
}


pub struct BatchProcessor {

    file_manager: Arc<FileManager>,

    operations: Arc<Mutex<Vec<(u64, BatchOperation)>>>,

    results: Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>,

    next_id: Arc<Mutex<u64>>,

    running: Arc<Mutex<bool>>,
}

impl BatchProcessor {

    pub fn new(file_manager: FileManager) -> Self {
        Self {
            file_manager: Arc::new(file_manager),
            operations: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(1)),
            running: Arc::new(Mutex::new(false)),
        }
    }


    pub fn start(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }

        *running = true;


        let operations = self.operations.clone();
        let results = self.results.clone();
        let file_manager = self.file_manager.clone();
        let running_flag = self.running.clone();


        thread::spawn(move || {
            Self::worker_thread(operations, results, file_manager, running_flag);
        });

        Ok(())
    }


    pub fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        *running = false;
        Ok(())
    }


    pub fn add_operation(&self, operation: BatchOperation) -> Result<u64> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };


        self.operations.lock().unwrap().push((id, operation));


        self.results.lock().unwrap().push((id, BatchResult::default()));


        self.start()?;

        Ok(id)
    }


    pub fn get_status(&self, id: u64) -> Result<BatchResult<Vec<PathBuf>>> {
        let results = self.results.lock().unwrap();

        for (op_id, result) in results.iter() {
            if *op_id == id {
                return Ok(result.clone());
            }
        }

        Err(anyhow!("Operation not found: {}", id))
    }


    pub fn cancel_operation(&self, id: u64) -> Result<()> {
        let mut operations = self.operations.lock().unwrap();
        let mut results = self.results.lock().unwrap();


        operations.retain(|(op_id, _)| *op_id != id);


        for (op_id, result) in results.iter_mut() {
            if *op_id == id && result.status != BatchStatus::Completed && result.status != BatchStatus::Failed {
                result.status = BatchStatus::Cancelled;
                return Ok(());
            }
        }

        Err(anyhow!("Operation not found or already completed: {}", id))
    }


    pub fn clear_completed(&self) -> Result<()> {
        let mut results = self.results.lock().unwrap();

        results.retain(|(_, result)| {
            result.status != BatchStatus::Completed &&
            result.status != BatchStatus::Failed &&
            result.status != BatchStatus::Cancelled
        });

        Ok(())
    }


    fn worker_thread(
        operations: Arc<Mutex<Vec<(u64, BatchOperation)>>>,
        results: Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>,
        file_manager: Arc<FileManager>,
        running: Arc<Mutex<bool>>
    ) {
        while *running.lock().unwrap() {

            let operation_opt = {
                let mut operations = operations.lock().unwrap();
                if operations.is_empty() {
                    None
                } else {
                    Some(operations.remove(0))
                }
            };

            if let Some((id, operation)) = operation_opt {

                {
                    let mut results = results.lock().unwrap();
                    for (op_id, result) in results.iter_mut() {
                        if *op_id == id {
                            result.status = BatchStatus::InProgress;
                            break;
                        }
                    }
                }


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

                thread::sleep(Duration::from_millis(100));
            }
        }
    }


    fn process_analyze(
        file_manager: &FileManager,
        operation: &BatchOperation,
        id: u64,
        results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {
        let mut processed_files = Vec::new();
        let total_files = operation.inputs.len();

        for (i, path) in operation.inputs.iter().enumerate() {

            {
                let results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter() {
                    if *op_id == id && result.status == BatchStatus::Cancelled {
                        return Err(anyhow!("Operation cancelled"));
                    }
                }
            }


            {
                let mut results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter_mut() {
                    if *op_id == id {
                        result.progress = ((i as f32 / total_files as f32) * 100.0) as u8;
                        break;
                    }
                }
            }


            if path.is_file() {
                let _ = file_manager.get_media_info(path)?;
                processed_files.push(path.to_path_buf());
            } else if path.is_dir() {

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


    fn process_thumbnail(
        file_manager: &FileManager,
        operation: &BatchOperation,
        id: u64,
        results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {
        let mut thumbnail_paths = Vec::new();
        let total_files = operation.inputs.len();


        let options = match &operation.options {
            BatchOperationOptions::Thumbnail(opts) => Some(opts.clone()),
            _ => None,
        };

        for (i, path) in operation.inputs.iter().enumerate() {

            {
                let results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter() {
                    if *op_id == id && result.status == BatchStatus::Cancelled {
                        return Err(anyhow!("Operation cancelled"));
                    }
                }
            }


            {
                let mut results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter_mut() {
                    if *op_id == id {
                        result.progress = ((i as f32 / total_files as f32) * 100.0) as u8;
                        break;
                    }
                }
            }


            if path.is_file() {
                let thumbnail_path = file_manager.generate_thumbnail(path, options.clone())?;
                thumbnail_paths.push(thumbnail_path);
            } else if path.is_dir() {

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


    fn process_extract_frames(
        file_manager: &FileManager,
        operation: &BatchOperation,
        id: u64,
        results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {
        let mut frame_paths = Vec::new();
        let total_files = operation.inputs.len();


        let fps = match &operation.options {
            BatchOperationOptions::ExtractFrames { fps } => *fps,
            _ => 1.0,
        };


        let output_dir = operation.output_dir.clone()
            .ok_or_else(|| anyhow!("Output directory required for frame extraction"))?;

        for (i, path) in operation.inputs.iter().enumerate() {

            {
                let results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter() {
                    if *op_id == id && result.status == BatchStatus::Cancelled {
                        return Err(anyhow!("Operation cancelled"));
                    }
                }
            }


            {
                let mut results_lock = results.lock().unwrap();
                for (op_id, result) in results_lock.iter_mut() {
                    if *op_id == id {
                        result.progress = ((i as f32 / total_files as f32) * 100.0) as u8;
                        break;
                    }
                }
            }


            if path.is_file() {

                let file_name = path.file_stem().unwrap_or_default().to_string_lossy();
                let file_output_dir = output_dir.join(file_name.to_string());
                std::fs::create_dir_all(&file_output_dir)?;


                let frames = file_manager.extract_frames(path, &file_output_dir, fps)?;
                frame_paths.extend(frames);
            }
        }

        Ok(frame_paths)
    }


    fn process_convert(
        _file_manager: &FileManager,
        _operation: &BatchOperation,
        _id: u64,
        _results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {


        Err(anyhow!("Media conversion not implemented yet"))
    }
}
