use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use crate::engine::editing::{
    EditingEngine,
    ExportOptions as GstExportOptions,
    ExportProgress as GstExportProgress
};
use crate::engine::rendering::{
    RenderingEngine,
    ExportOptions as FfmpegExportOptions,
    ExportProgress as FfmpegExportProgress
};
use crate::engine::editing::types::EditingError;


#[derive(Debug, Clone)]
pub struct ExportProgress {

    pub stage: ExportStage,


    pub percent: f64,


    pub stage_progress: Option<String>,


    pub complete: bool,


    pub error: Option<String>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStage {

    Preparing,


    IntermediateExport,


    FinalRendering,


    Cleanup,
}

impl ExportStage {

    pub fn display_name(&self) -> &'static str {
        match self {
            ExportStage::Preparing => __STRING_0__,
            ExportStage::IntermediateExport => __STRING_1__,
            ExportStage::FinalRendering => __STRING_2__,
            ExportStage::Cleanup => __STRING_3__,
        }
    }
}

/// Options for the integrated export process
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Output file path
    pub output_path: PathBuf,

    /// Whether to keep the intermediate file
    pub keep_intermediate: bool,

    /// Path for the intermediate file (if keep_intermediate is true)
    pub intermediate_path: Option<PathBuf>,

    /// GStreamer export options
    pub gst_options: GstExportOptions,

    /// FFmpeg export options
    pub ffmpeg_options: FfmpegExportOptions,
}

impl ExportOptions {
    /// Create new export options with default settings
    pub fn new<P: AsRef<Path>>(output_path: P) -> Self {
        let output_path = output_path.as_ref().to_path_buf();

        // Create a temporary path for the intermediate file
        let intermediate_path = std::env::temp_dir()
            .join(format!(__STRING_4__, chrono::Utc::now().timestamp()));

        // Create GStreamer export options
        let mut gst_options = GstExportOptions::default();
        gst_options.output_path = intermediate_path.clone();
        gst_options.container = 'static,
    {
        self.progress_callback = Some(Arc::new(Mutex::new(callback)));
    }


    pub fn start_export(&mut self) -> Result<(), EditingError> {

        self.update_progress(ExportStage::Preparing, 0.0, None);


        let timeline = self.editing_engine.lock().unwrap()
            .timeline().lock().unwrap()
            .get_ges_timeline()
            .ok_or(EditingError::NotInitialized)?
            .clone();

        let intermediate_exporter = self.editing_engine.lock().unwrap()
            .create_intermediate_export(self.options.gst_options.clone())?;

        self.intermediate_exporter = Some(intermediate_exporter);


        let progress = self.progress.clone();
        let callback = self.progress_callback.clone();

        if let Some(ref mut exporter) = self.intermediate_exporter {
            exporter.set_progress_callback(move |gst_progress: GstExportProgress| {
                let mut progress_guard = progress.lock().unwrap();
                progress_guard.stage = ExportStage::IntermediateExport;
                progress_guard.percent = gst_progress.percent;
                progress_guard.stage_progress = Some(format!(
                    __STRING_8__,
                    gst_progress.position as f64 / 1_000_000_000.0,
                    gst_progress.duration as f64 / 1_000_000_000.0,
                ));

                if gst_progress.complete {
                    progress_guard.stage = ExportStage::FinalRendering;
                    progress_guard.percent = 0.0;
                }

                if let Some(error) = gst_progress.error {
                    progress_guard.error = Some(error);
                    progress_guard.complete = true;
                }

                if let Some(callback) = &callback {
                    callback.lock().unwrap()(progress_guard.clone());
                }
            });


            exporter.start_export()?;
        }


    intermediate_exporter: Option<crate::engine::editing::IntermediateExporter>,


    final_exporter: Option<Arc<Mutex<crate::engine::rendering::Exporter>>>,
}

impl IntegratedExporter {

    pub fn new(
        editing_engine: Arc<Mutex<EditingEngine>>,
        rendering_engine: Arc<Mutex<RenderingEngine>>,
        options: ExportOptions,
    ) -> Result<Self, EditingError> {
        let progress = Arc::new(Mutex::new(ExportProgress {
            stage: ExportStage::Preparing,
            percent: 0.0,
            stage_progress: None,
            complete: false,
            error: None,
        }));

        Ok(Self {
            editing_engine,
            rendering_engine,
            options,
            progress,
            progress_callback: None,
            intermediate_exporter: None,
            final_exporter: None,
        })
    }


    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(ExportProgress) + Send + 'static,
    {
        self.progress_callback = Some(Arc::new(Mutex::new(callback)));
    }

    /// Start the export process
    pub fn start_export(&mut self) -> Result<(), EditingError> {
        // Update progress to preparing stage
        self.update_progress(ExportStage::Preparing, 0.0, None);

        // Create intermediate exporter
        let timeline = self.editing_engine.lock().unwrap()
            .timeline().lock().unwrap()
            .get_ges_timeline()
            .ok_or(EditingError::NotInitialized)?
            .clone();

        let intermediate_exporter = self.editing_engine.lock().unwrap()
            .create_intermediate_export(self.options.gst_options.clone())?;

        self.intermediate_exporter = Some(intermediate_exporter);

        // Set up progress callback for intermediate export
        let progress = self.progress.clone();
        let callback = self.progress_callback.clone();

        if let Some(ref mut exporter) = self.intermediate_exporter {
            exporter.set_progress_callback(move |gst_progress: GstExportProgress| {
                let mut progress_guard = progress.lock().unwrap();
                progress_guard.stage = ExportStage::IntermediateExport;
                progress_guard.percent = gst_progress.percent;
                progress_guard.stage_progress = Some(format!(
                    __STRING_8__,
                    gst_progress.position as f64 / 1_000_000_000.0,
                    gst_progress.duration as f64 / 1_000_000_000.0,
                ));

                if gst_progress.complete {
                    progress_guard.stage = ExportStage::FinalRendering;
                    progress_guard.percent = 0.0;
                }

                if let Some(error) = gst_progress.error {
                    progress_guard.error = Some(error);
                    progress_guard.complete = true;
                }

                if let Some(callback) = &callback {
                    callback.lock().unwrap()(progress_guard.clone());
                }
            });

            // Start the intermediate export
            exporter.start_export()?;
        }

        // Wait for intermediate export to complete
        // This would normally be handled by the callback system
        // For simplicity, we're not implementing the full async workflow here


        let final_exporter = self.rendering_engine.lock().unwrap()
            .create_export(self.options.ffmpeg_options.clone())?;

        self.final_exporter = Some(final_exporter.clone());


        let progress = self.progress.clone();
        let callback = self.progress_callback.clone();
        let keep_intermediate = self.options.keep_intermediate;
        let intermediate_path = self.options.intermediate_path.clone();

        final_exporter.lock().unwrap().set_progress_callback(move |ffmpeg_progress: FfmpegExportProgress| {
            let mut progress_guard = progress.lock().unwrap();
            progress_guard.stage = ExportStage::FinalRendering;
            progress_guard.percent = ffmpeg_progress.percent;
            progress_guard.stage_progress = Some(format!(
                "Frame: {} / {} ({:.2} / {:.2} seconds)",
                ffmpeg_progress.current_frame,
                ffmpeg_progress.total_frames,
                ffmpeg_progress.current_time,
                ffmpeg_progress.total_duration,
            ));

            if ffmpeg_progress.complete {
                if !keep_intermediate && intermediate_path.is_some() {
                    progress_guard.stage = ExportStage::Cleanup;
                    progress_guard.percent = 0.0;


                    if let Some(path) = &intermediate_path {
                        if let Err(e) = std::fs::remove_file(path) {
                            progress_guard.stage_progress = Some(format!("Failed to delete intermediate file: {}", e));
                        } else {
                            progress_guard.stage_progress = Some("Deleted intermediate file".to_string());
                        }
                    }
                }

                progress_guard.complete = true;
                progress_guard.percent = 100.0;
            }

            if let Some(error) = ffmpeg_progress.error.clone() {
                progress_guard.error = Some(error);
                progress_guard.complete = true;
            }

            if let Some(callback) = &callback {
                callback.lock().unwrap()(progress_guard.clone());
            }
        });


        final_exporter.lock().unwrap().start_export()?;

        Ok(())
    }


    fn update_progress(&self, stage: ExportStage, percent: f64, stage_progress: Option<String>) {
        let mut progress = self.progress.lock().unwrap();
        progress.stage = stage;
        progress.percent = percent;
        progress.stage_progress = stage_progress;

        if let Some(callback) = &self.progress_callback {
            callback.lock().unwrap()(progress.clone());
        }
    }


    pub fn cancel_export(&mut self) -> Result<(), EditingError> {

        if let Some(ref mut exporter) = self.intermediate_exporter {
            exporter.cancel_export()?;
        }


        if let Some(ref exporter) = self.final_exporter {
            exporter.lock().unwrap().cancel()?;
        }


        let mut progress = self.progress.lock().unwrap();
        progress.error = Some("Export cancelled".to_string());
        progress.complete = true;

        if let Some(callback) = &self.progress_callback {
            callback.lock().unwrap()(progress.clone());
        }

        Ok(())
    }


    pub fn get_progress(&self) -> ExportProgress {
        self.progress.lock().unwrap().clone()
    }
}


pub fn create_integrated_exporter(
    editing_engine: Arc<Mutex<EditingEngine>>,
    rendering_engine: Arc<Mutex<RenderingEngine>>,
    options: ExportOptions,
) -> Result<IntegratedExporter, EditingError> {
    IntegratedExporter::new(editing_engine, rendering_engine, options)
}
