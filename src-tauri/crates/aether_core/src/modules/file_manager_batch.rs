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
        file_manager: &FileManager,
        operation: &BatchOperation,
        id: u64,
        results: &Arc<Mutex<Vec<(u64, BatchResult<Vec<PathBuf>>)>>>
    ) -> Result<Vec<PathBuf>> {
        use super::file_manager::MediaType;
        use super::file_manager_convert::{
            AudioConversionOptions, ConversionFormat, MediaConverter, VideoConversionOptions,
        };

        let (format_str, quality, preserve_aspect_ratio, width, height) = match &operation.options {
            BatchOperationOptions::Convert { format, quality, preserve_aspect_ratio, width, height } => {
                (format.clone(), *quality, *preserve_aspect_ratio, *width, *height)
            },
            _ => return Err(anyhow!("Convert operation requires Convert options")),
        };

        let target_format = ConversionFormat::from_extension(format_str.trim_start_matches('.'))
            .ok_or_else(|| anyhow!("Unsupported target format: {}", format_str))?;

        let output_dir = operation.output_dir.clone()
            .ok_or_else(|| anyhow!("Output directory required for conversion"))?;
        std::fs::create_dir_all(&output_dir)?;

        let converter = MediaConverter::new()?;
        let mut output_paths = Vec::new();
        let total_files = operation.inputs.len().max(1);

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

            if !path.is_file() {
                continue;
            }

            let media_info = file_manager.get_media_info(path)?;
            let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let output_path = output_dir.join(format!("{}{}", file_stem, target_format.extension()));

            match media_info.media_type {
                MediaType::Video => {
                    let options = VideoConversionOptions {
                        format: target_format,
                        width,
                        height,
                        preserve_aspect_ratio,
                        ..Default::default()
                    };
                    converter.convert_video(path, &output_path, options, |_| {})?;
                },
                MediaType::Audio => {
                    let options = AudioConversionOptions {
                        format: target_format,
                        ..Default::default()
                    };
                    converter.convert_audio(path, &output_path, options, |_| {})?;
                },
                MediaType::Image => {
                    Self::convert_image(path, &output_path, target_format, quality, width, height, preserve_aspect_ratio)?;
                },
                MediaType::Unknown => {
                    return Err(anyhow!("Cannot determine media type for: {:?}", path));
                },
            }

            output_paths.push(output_path);
        }

        Ok(output_paths)
    }


    fn convert_image(
        input_path: &std::path::Path,
        output_path: &std::path::Path,
        format: super::file_manager_convert::ConversionFormat,
        quality: u8,
        width: Option<u32>,
        height: Option<u32>,
        preserve_aspect_ratio: bool,
    ) -> Result<()> {
        use super::file_manager_convert::ConversionFormat;

        let img = image::open(input_path)?;

        let img = if width.is_some() || height.is_some() {
            let (src_w, src_h) = (img.width(), img.height());
            let (target_w, target_h) = if preserve_aspect_ratio {
                let w = width.unwrap_or(src_w);
                let h = height.unwrap_or(src_h);
                let scale = (w as f32 / src_w as f32).min(h as f32 / src_h as f32);
                (
                    (src_w as f32 * scale).round().max(1.0) as u32,
                    (src_h as f32 * scale).round().max(1.0) as u32,
                )
            } else {
                (width.unwrap_or(src_w), height.unwrap_or(src_h))
            };
            img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        match format {
            ConversionFormat::JPEG => {
                let mut file = std::fs::File::create(output_path)?;
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
                encoder.encode_image(&img)?;
            },
            _ => {
                img.save(output_path)?;
            },
        }

        Ok(())
    }
}
