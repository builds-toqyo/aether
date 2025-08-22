use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::{Context, Result};
use glib::{MainContext, MainLoop, SourceId};
use gst::prelude::*;
use gst_pbutils::prelude::*;
use crate::engine::editing::types::EditingError;
use crate::engine::rendering::formats::{VideoFormat, AudioFormat, ContainerFormat};
use crate::engine::rendering::encoder::EncoderPreset;

pub type ExportCallback = Arc<dyn Fn(ExportProgress) + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub timeline: ges::Timeline,
    
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
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            timeline: ges::Timeline::new(),
            output_path: PathBuf::new(),
            container_format: ContainerFormat::Mp4,
            video_format: VideoFormat::H264,
            audio_format: AudioFormat::Aac,
            video_bitrate: 2_000_000,
            audio_bitrate: 128_000,
            frame_rate: 30.0,
            width: 0,
            height: 0,
            encoder_preset: EncoderPreset::Medium,
            crf: 23,
            hardware_acceleration: false,
            threads: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportProgress {
    pub current_frame: u64,
    
    pub total_frames: u64,
    
    pub current_time: f64,
    
    pub total_duration: f64,
    
    pub percent: f64,
    
    pub complete: bool,
    
    pub error: Option<String>,
}

pub struct GstExporter {
    options: ExportOptions,
    
    pipeline: Option<ges::Pipeline>,
    
    main_loop: Option<MainLoop>,
    
    progress: Arc<Mutex<ExportProgress>>,
    
    progress_callback: Option<ExportCallback>,
    
    bus_watch_id: Option<SourceId>,
    
    timeout_id: Option<SourceId>,
    
    cancel_flag: Arc<Mutex<bool>>,
}

impl GstExporter {
    pub fn new(options: ExportOptions) -> Result<Self, EditingError> {
        if !gst::is_initialized() {
            gst::init().map_err(|e| EditingError::ExportError(format!("Failed to initialize GStreamer: {}", e)))?;
        }
        
        let progress = Arc::new(Mutex::new(ExportProgress {
            current_frame: 0,
            total_frames: 0,
            current_time: 0.0,
            total_duration: 0.0,
            percent: 0.0,
            complete: false,
            error: None,
        }));
        
        Ok(Self {
            options,
            pipeline: None,
            main_loop: None,
            progress,
            progress_callback: None,
            bus_watch_id: None,
            timeout_id: None,
            cancel_flag: Arc::new(Mutex::new(false)),
        })
    }
    
    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(ExportProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(callback));
    }
    
    pub fn start_export(&mut self) -> Result<(), EditingError> {
        *self.cancel_flag.lock().unwrap() = false;
        
        let pipeline = ges::Pipeline::new()
            .context("Failed to create GES pipeline")?;
        
        pipeline.set_timeline(&self.options.timeline)
            .context("Failed to set timeline on pipeline")?;
        
        let duration = self.options.timeline.duration();
        let total_frames = (duration as f64 / gst::ClockTime::SECOND.nseconds() as f64 * self.options.frame_rate) as u64;
        
        {
            let mut progress = self.progress.lock().unwrap();
            progress.total_frames = total_frames;
            progress.total_duration = duration as f64 / gst::ClockTime::SECOND.nseconds() as f64;
            
            if let Some(callback) = &self.progress_callback {
                callback(progress.clone());
            }
        }
        
        let profile = self.create_encoding_profile()
            .context("Failed to create encoding profile")?;
        
        let output_uri = gst::filename_to_uri(self.options.output_path.as_path())
            .context("Failed to convert output path to URI")?;
        
        pipeline.set_render_settings(&output_uri, &profile)
            .context("Failed to set render settings")?;
        
        pipeline.set_mode(ges::PipelineFlags::RENDER)
            .context("Failed to set pipeline mode to render")?;
        
        let bus = pipeline.bus().expect("Pipeline without bus");
        
        let main_loop = MainLoop::new(None, false);
        let main_loop_clone = main_loop.clone();
        
        let progress_clone = self.progress.clone();
        let callback_clone = self.progress_callback.clone();
        let cancel_flag = self.cancel_flag.clone();
        
        let bus_watch_id = bus.add_watch(move |_, msg| {
            match msg.view() {
                gst::MessageView::Eos(..) => {
                    let mut progress = progress_clone.lock().unwrap();
                    progress.complete = true;
                    progress.percent = 100.0;
                    progress.current_frame = progress.total_frames;
                    progress.current_time = progress.total_duration;
                    
                    if let Some(callback) = &callback_clone {
                        callback(progress.clone());
                    }
                    
                    main_loop_clone.quit();
                },
                gst::MessageView::Error(err) => {
                    let error_msg = format!("Export error: {} ({})", err.error(), err.debug().unwrap_or_default());
                    let mut progress = progress_clone.lock().unwrap();
                    progress.error = Some(error_msg);
                    progress.complete = true;
                    
                    if let Some(callback) = &callback_clone {
                        callback(progress.clone());
                    }
                    
                    main_loop_clone.quit();
                },
                gst::MessageView::Application(app) => {
                    let structure = app.structure();
                    if let Some(structure) = structure {
                        if structure.name() == "export-cancelled" {
                            let error_msg = "Export cancelled";
                            let mut progress = progress_clone.lock().unwrap();
                            progress.error = Some(error_msg.to_string());
                            progress.complete = true;
                            
                            if let Some(callback) = &callback_clone {
                                callback(progress.clone());
                            }
                            
                            // Quit the main loop
                            main_loop_clone.quit();
                        }
                    }
                },
                _ => (),
            }
            
            if *cancel_flag.lock().unwrap() {
                let structure = gst::Structure::builder("export-cancelled")
                    .build();
                let message = gst::message::Application::new(structure);
                bus.post(&message).expect("Failed to post cancellation message");
            }
            
            glib::Continue(true)
        }).context("Failed to add bus watch")?;
        
        let pipeline_weak = pipeline.downgrade();
        let progress_clone = self.progress.clone();
        let callback_clone = self.progress_callback.clone();
        
        let timeout_id = glib::timeout_add_seconds(1, move || {
            if let Some(pipeline) = pipeline_weak.upgrade() {
                if let Ok(position) = pipeline.query_position::<gst::ClockTime>() {
                    let position_seconds = position.nseconds() as f64 / gst::ClockTime::SECOND.nseconds() as f64;
                    let duration_seconds = progress_clone.lock().unwrap().total_duration;
                    
                    if duration_seconds > 0.0 {
                        let percent = (position_seconds / duration_seconds) * 100.0;
                        let current_frame = (position_seconds * progress_clone.lock().unwrap().total_frames as f64 / duration_seconds) as u64;
                        
                        let mut progress = progress_clone.lock().unwrap();
                        progress.current_time = position_seconds;
                        progress.current_frame = current_frame;
                        progress.percent = percent;
                        
                        if let Some(callback) = &callback_clone {
                            callback(progress.clone());
                        }
                    }
                }
                
                glib::Continue(true)
            } else {
                glib::Continue(false)
            }
        });
        
        self.pipeline = Some(pipeline);
        self.main_loop = Some(main_loop);
        self.bus_watch_id = Some(bus_watch_id);
        self.timeout_id = Some(timeout_id);
        
        self.pipeline.as_ref().unwrap().set_state(gst::State::Playing)
            .context("Failed to start pipeline")?;
        
        let main_loop_clone = self.main_loop.as_ref().unwrap().clone();
        std::thread::spawn(move || {
            main_loop_clone.run();
        });
        
        Ok(())
    }
    
    fn create_encoding_profile(&self) -> Result<gst_pbutils::EncodingProfile, EditingError> {
        let container_caps = gst::Caps::builder(self.options.container_format.to_mime_type())
            .build();
        
        let container_profile = gst_pbutils::EncodingContainerProfile::new(
            Some("container"),
            Some("Container profile"),
            &container_caps,
            None,
        ).context("Failed to create container profile")?;
        
        let video_caps = if self.options.hardware_acceleration {
            match self.options.video_format {
                VideoFormat::H264 => {
                    gst::Caps::builder("video/x-h264")
                        .field("profile", "high")
                        .build()
                },
                VideoFormat::H265 => {
                    gst::Caps::builder("video/x-h265")
                        .field("profile", "main")
                        .build()
                },
                _ => {
                    gst::Caps::builder("video/x-h264")
                        .field("profile", "high")
                        .build()
                }
            }
        } else {
            gst::Caps::builder(self.options.video_format.to_mime_type())
                .build()
        };
        
        let video_profile = gst_pbutils::EncodingVideoProfile::new(
            &video_caps,
            None,
            gst::Caps::builder("video/x-raw").build(),
            1, // Presence
        ).context("Failed to create video profile")?;
        
        if self.options.video_bitrate > 0 {
            video_profile.set_bitrate(self.options.video_bitrate as u32);
        }
        
        if self.options.width > 0 && self.options.height > 0 {
            let restriction = gst::Caps::builder("video/x-raw")
                .field("width", self.options.width as i32)
                .field("height", self.options.height as i32)
                .build();
            video_profile.set_restriction(Some(&restriction));
        }
        
        container_profile.add_profile(&video_profile.upcast())
            .context("Failed to add video profile to container")?;
        
        let audio_caps = gst::Caps::builder(self.options.audio_format.to_mime_type())
            .build();
        
        let audio_profile = gst_pbutils::EncodingAudioProfile::new(
            &audio_caps,
            None,
            gst::Caps::builder("audio/x-raw").build(),
            1, // Presence
        ).context("Failed to create audio profile")?;
        
        if self.options.audio_bitrate > 0 {
            audio_profile.set_bitrate(self.options.audio_bitrate as u32);
        }
        
        container_profile.add_profile(&audio_profile.upcast())
            .context("Failed to add audio profile to container")?;
        
        Ok(container_profile.upcast())
    }
    
    pub fn cancel_export(&mut self) -> Result<(), EditingError> {
        *self.cancel_flag.lock().unwrap() = true;
        
        std::thread::sleep(Duration::from_millis(100));
        
        if let Some(pipeline) = &self.pipeline {
            if let Some(bus) = pipeline.bus() {
                let structure = gst::Structure::builder("export-cancelled")
                    .build();
                let message = gst::message::Application::new(structure);
                bus.post(&message).expect("Failed to post cancellation message");
            }
        }
        
        Ok(())
    }
    
    pub fn get_progress(&self) -> ExportProgress {
        self.progress.lock().unwrap().clone()
    }
    
    pub fn is_complete(&self) -> bool {
        self.progress.lock().unwrap().complete
    }
    
    pub fn has_error(&self) -> bool {
        self.progress.lock().unwrap().error.is_some()
    }
    
    pub fn get_error(&self) -> Option<String> {
        self.progress.lock().unwrap().error.clone()
    }
}

impl Drop for GstExporter {
    fn drop(&mut self) {
        if let Some(watch_id) = self.bus_watch_id.take() {
            watch_id.remove();
        }
        
        if let Some(timeout_id) = self.timeout_id.take() {
            timeout_id.remove();
        }
        
        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
        
        if let Some(main_loop) = &self.main_loop {
            if main_loop.is_running() {
                main_loop.quit();
            }
        }
    }
}

trait ContainerFormatExt {
    fn to_mime_type(&self) -> &'static str;
}

impl ContainerFormatExt for ContainerFormat {
    fn to_mime_type(&self) -> &'static str {
        match self {
            ContainerFormat::Mp4 => "video/quicktime, variant=iso",
            ContainerFormat::Mkv => "video/x-matroska",
            ContainerFormat::WebM => "video/webm",
            ContainerFormat::Mov => "video/quicktime",
            ContainerFormat::Avi => "video/x-msvideo",
        }
    }
}

trait VideoFormatExt {
    fn to_mime_type(&self) -> &'static str;
}

impl VideoFormatExt for VideoFormat {
    fn to_mime_type(&self) -> &'static str {
        match self {
            VideoFormat::H264 => "video/x-h264",
            VideoFormat::H265 => "video/x-h265",
            VideoFormat::Vp8 => "video/x-vp8",
            VideoFormat::Vp9 => "video/x-vp9",
            VideoFormat::Av1 => "video/x-av1",
            VideoFormat::ProRes => "video/x-prores",
            VideoFormat::Dnxhd => "video/x-dnxhd",
        }
    }
}

trait AudioFormatExt {
    fn to_mime_type(&self) -> &'static str;
}

impl AudioFormatExt for AudioFormat {
    fn to_mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Aac => "audio/mpeg, mpegversion=4",
            AudioFormat::Mp3 => "audio/mpeg, mpegversion=1, layer=3",
            AudioFormat::Flac => "audio/x-flac",
            AudioFormat::Vorbis => "audio/x-vorbis",
            AudioFormat::Opus => "audio/x-opus",
            AudioFormat::Pcm => "audio/x-raw",
        }
    }
}
