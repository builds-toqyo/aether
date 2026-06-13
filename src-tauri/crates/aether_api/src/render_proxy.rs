use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use log::{error, info};

use aether_core::engine::rendering::gst_exporter::{
    GstExporter, ExportOptions as GstExportOptions, ExportProgress,
};
use aether_core::engine::rendering::formats::{VideoFormat, AudioFormat, ContainerFormat};
use aether_core::engine::rendering::encoder::EncoderPreset;
use aether_core::engine::editing::types::EditingError;

#[derive(Debug, Clone)]
pub struct GstExportConfig {
    pub output_path: PathBuf,
    pub container_format: ContainerFormat,
    pub video_format: VideoFormat,
    pub audio_format: AudioFormat,
    pub video_bitrate: u32,
    pub audio_bitrate: u32,
    pub frame_rate: f64,
    pub width: u32,
    pub height: u32,
    pub encoder_preset: EncoderPreset,
    pub crf: u8,
    pub hardware_acceleration: bool,
    pub threads: u8,
    pub project_path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum RenderCommand {
    StartExport { resp: Sender<Result<(), EditingError>> },
    CancelExport { resp: Sender<Result<(), EditingError>> },
    GetProgress { resp: Sender<ExportProgress> },
    IsComplete { resp: Sender<bool> },
    HasError { resp: Sender<bool> },
    GetError { resp: Sender<Option<String>> },
    Pause { resp: Sender<Result<(), EditingError>> },
    Resume { resp: Sender<Result<(), EditingError>> },
    IsPaused { resp: Sender<bool> },
}

pub struct GstExporterProxy {
    sender: Sender<RenderCommand>,
}

impl GstExporterProxy {
    pub fn new(config: GstExportConfig) -> Result<Self, EditingError> {
        let (sender, receiver) = channel::<RenderCommand>();
        std::thread::spawn(move || run_gst_exporter_thread(receiver, config));
        Ok(Self { sender })
    }

    pub fn start_export(&self) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(RenderCommand::StartExport { resp: tx })
            .map_err(|_| EditingError::ExportError("Render thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::ExportError("Render thread response lost".to_string()))?
    }

    pub fn cancel_export(&self) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(RenderCommand::CancelExport { resp: tx })
            .map_err(|_| EditingError::ExportError("Render thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::ExportError("Render thread response lost".to_string()))?
    }

    pub fn get_progress(&self) -> ExportProgress {
        let (tx, rx) = channel();
        let _ = self.sender.send(RenderCommand::GetProgress { resp: tx });
        rx.recv().unwrap_or(ExportProgress {
            current_frame: 0,
            total_frames: 0,
            current_time: 0.0,
            total_duration: 0.0,
            percent: 0.0,
            complete: false,
            error: Some("Render thread disconnected".to_string()),
        })
    }

    pub fn is_complete(&self) -> bool {
        let (tx, rx) = channel();
        let _ = self.sender.send(RenderCommand::IsComplete { resp: tx });
        rx.recv().unwrap_or(false)
    }

    pub fn has_error(&self) -> bool {
        let (tx, rx) = channel();
        let _ = self.sender.send(RenderCommand::HasError { resp: tx });
        rx.recv().unwrap_or(false)
    }

    pub fn get_error(&self) -> Option<String> {
        let (tx, rx) = channel();
        let _ = self.sender.send(RenderCommand::GetError { resp: tx });
        rx.recv().unwrap_or(None)
    }

    pub fn pause(&self) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(RenderCommand::Pause { resp: tx })
            .map_err(|_| EditingError::ExportError("Render thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::ExportError("Render thread response lost".to_string()))?
    }

    pub fn resume(&self) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(RenderCommand::Resume { resp: tx })
            .map_err(|_| EditingError::ExportError("Render thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::ExportError("Render thread response lost".to_string()))?
    }

    pub fn is_paused(&self) -> bool {
        let (tx, rx) = channel();
        let _ = self.sender.send(RenderCommand::IsPaused { resp: tx });
        rx.recv().unwrap_or(false)
    }
}

fn run_gst_exporter_thread(receiver: std::sync::mpsc::Receiver<RenderCommand>, config: GstExportConfig) {
    info!("Starting GstExporter background thread");
    let options = GstExportOptions {
        output_path: config.output_path,
        container_format: config.container_format,
        video_format: config.video_format,
        audio_format: config.audio_format,
        video_bitrate: config.video_bitrate,
        audio_bitrate: config.audio_bitrate,
        frame_rate: config.frame_rate,
        width: config.width,
        height: config.height,
        encoder_preset: config.encoder_preset,
        crf: config.crf,
        hardware_acceleration: config.hardware_acceleration,
        threads: config.threads,
        project_path: config.project_path.clone(),
    };
    match GstExporter::new(options) {
        Ok(mut exporter) => {
            info!("GstExporter initialized successfully");
            let mut paused = false;
            for cmd in receiver {
                match cmd {
                    RenderCommand::StartExport { resp } => {
                        let result = exporter.start_export();
                        let _ = resp.send(result);
                    }
                    RenderCommand::CancelExport { resp } => {
                        let result = exporter.cancel_export();
                        let _ = resp.send(result);
                    }
                    RenderCommand::GetProgress { resp } => {
                        let _ = resp.send(exporter.get_progress());
                    }
                    RenderCommand::IsComplete { resp } => {
                        let _ = resp.send(exporter.is_complete());
                    }
                    RenderCommand::HasError { resp } => {
                        let _ = resp.send(exporter.has_error());
                    }
                    RenderCommand::GetError { resp } => {
                        let _ = resp.send(exporter.get_error());
                    }
                    RenderCommand::Pause { resp } => {
                        let result = exporter.pause();
                        let _ = resp.send(result);
                    }
                    RenderCommand::Resume { resp } => {
                        let result = exporter.resume();
                        let _ = resp.send(result);
                    }
                    RenderCommand::IsPaused { resp } => {
                        let _ = resp.send(exporter.is_paused());
                    }
                }
            }
            info!("GstExporter command channel closed; shutting down");
        }
        Err(err) => {
            error!("Failed to initialize GstExporter: {}. Render thread exiting.", err);
            for cmd in receiver {
                match cmd {
                    RenderCommand::StartExport { resp } => {
                        let _ = resp.send(Err(EditingError::ExportError(err.to_string())));
                    }
                    RenderCommand::CancelExport { resp } => {
                        let _ = resp.send(Err(EditingError::ExportError(err.to_string())));
                    }
                    RenderCommand::GetProgress { resp } => {
                        let _ = resp.send(ExportProgress {
                            current_frame: 0,
                            total_frames: 0,
                            current_time: 0.0,
                            total_duration: 0.0,
                            percent: 0.0,
                            complete: true,
                            error: Some(err.to_string()),
                        });
                    }
                    RenderCommand::IsComplete { resp } => {
                        let _ = resp.send(true);
                    }
                    RenderCommand::HasError { resp } => {
                        let _ = resp.send(true);
                    }
                    RenderCommand::GetError { resp } => {
                        let _ = resp.send(Some(err.to_string()));
                    }
                    RenderCommand::Pause { resp } => {
                        let _ = resp.send(Err(EditingError::ExportError(err.to_string())));
                    }
                    RenderCommand::Resume { resp } => {
                        let _ = resp.send(Err(EditingError::ExportError(err.to_string())));
                    }
                    RenderCommand::IsPaused { resp } => {
                        let _ = resp.send(false);
                    }
                }
            }
        }
    }
}
