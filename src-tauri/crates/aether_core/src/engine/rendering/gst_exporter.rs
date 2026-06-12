use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::{Context, Result};
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_editing_services as ges;
use gstreamer_editing_services::prelude::*;
use glib::{filename_to_uri, ControlFlow, MainLoop, SourceId};
use crate::engine::editing::types::EditingError;
use crate::engine::rendering::formats::{VideoFormat, AudioFormat, ContainerFormat};
use crate::engine::rendering::encoder::EncoderPreset;

pub type ExportCallback = Arc<dyn Fn(ExportProgress) + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct ExportOptions {
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

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
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
            project_path: None,
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

    bus_watch_id: Option<gst::bus::BusWatchGuard>,

    timeout_id: Option<SourceId>,

    cancel_flag: Arc<Mutex<bool>>,
}

impl GstExporter {
    pub fn new(options: ExportOptions) -> Result<Self, EditingError> {
        gst::init().map_err(|e| EditingError::ExportError(format!("Failed to initialize GStreamer: {}", e)))?;
        ges::init().map_err(|e| EditingError::ExportError(format!("Failed to initialize GES: {}", e)))?;

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

        let pipeline = ges::Pipeline::new();

        let timeline = ges::Timeline::new();

        if let Some(project_path) = &self.options.project_path {
            let uri = filename_to_uri(project_path, None)
                .map_err(|e| EditingError::ExportError(format!("Failed to convert project path to URI: {}", e)))?;
            timeline.load_from_uri(&uri)
                .map_err(|e| EditingError::ExportError(format!("Failed to load timeline from {}: {}", project_path.display(), e)))?;
        }

        pipeline.set_timeline(&timeline)
            .map_err(|e| EditingError::ExportError(format!("Failed to set timeline on pipeline: {}", e)))?;

        let duration = timeline.duration();
        let duration_nanos = duration.nseconds();
        let duration_seconds = duration_nanos as f64 / gst::ClockTime::SECOND.nseconds() as f64;
        let total_frames = (duration_seconds * self.options.frame_rate) as u64;

        {
            let mut progress = self.progress.lock().unwrap();
            progress.total_frames = total_frames;
            progress.total_duration = duration_seconds;

            if let Some(callback) = &self.progress_callback {
                callback(progress.clone());
            }
        }

        let profile = self.create_encoding_profile()
            .context("Failed to create encoding profile")?;

        let output_uri = filename_to_uri(self.options.output_path.as_path(), None)
            .context("Failed to convert output path to URI")?;

        let encodebin = gst::ElementFactory::make("encodebin")
            .name("encoder")
            .property("profile", &profile)
            .build()
            .map_err(|_| EditingError::ExportError("Failed to create encodebin".to_string()))?;
        let filesink = gst::ElementFactory::make("filesink")
            .name("export_sink")
            .property("location", &output_uri)
            .build()
            .map_err(|_| EditingError::ExportError("Failed to create filesink".to_string()))?;

        pipeline.add_many(&[&encodebin, &filesink])
            .map_err(|e| EditingError::ExportError(format!("Failed to add elements: {}", e)))?;
        gst::Element::link_many(&[&encodebin, &filesink])
            .map_err(|e| EditingError::ExportError(format!("Failed to link encodebin to filesink: {}", e)))?;

        let v_src_pad = pipeline.static_pad("video_0")
            .ok_or_else(|| EditingError::ExportError("Pipeline has no video src pad".to_string()))?;
        let a_src_pad = pipeline.static_pad("audio_0")
            .ok_or_else(|| EditingError::ExportError("Pipeline has no audio src pad".to_string()))?;
        let v_sink_pad = encodebin.request_pad_simple("video_%u")
            .ok_or_else(|| EditingError::ExportError("Failed to request video sink pad".to_string()))?;
        let a_sink_pad = encodebin.request_pad_simple("audio_%u")
            .ok_or_else(|| EditingError::ExportError("Failed to request audio sink pad".to_string()))?;

        v_src_pad.link(&v_sink_pad)
            .map_err(|e| EditingError::ExportError(format!("Failed to link video to encodebin: {}", e)))?;
        a_src_pad.link(&a_sink_pad)
            .map_err(|e| EditingError::ExportError(format!("Failed to link audio to encodebin: {}", e)))?;

        let bus = pipeline.bus().expect("Pipeline without bus");

        let main_loop = MainLoop::new(None, false);
        let main_loop_clone = main_loop.clone();

        let progress_clone = self.progress.clone();
        let callback_clone = self.progress_callback.clone();
        let cancel_flag = self.cancel_flag.clone();

        let bus_watch_id = bus.clone().add_watch(move |_, msg| {
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
                bus.post(message).expect("Failed to post cancellation message");
            }

            ControlFlow::Continue
        }).context("Failed to add bus watch")?;

        let pipeline_clone = pipeline.clone();
        let progress_clone = self.progress.clone();
        let callback_clone = self.progress_callback.clone();

        let timeout_id = glib::source::timeout_add_seconds_local(1, move || {
            if let Some(position) = pipeline_clone.query_position::<gst::ClockTime>() {
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

            ControlFlow::Continue
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

        let video_profile = gst_pbutils::EncodingVideoProfile::builder(&video_caps)
            .presence(1)
            .build();

        let audio_caps = gst::Caps::builder(self.options.audio_format.to_mime_type())
            .build();

        let audio_profile = gst_pbutils::EncodingAudioProfile::builder(&audio_caps)
            .presence(1)
            .build();

        let container_caps = gst::Caps::builder(self.options.container_format.to_mime_type())
            .build();

        let container_profile = gst_pbutils::EncodingContainerProfile::builder(&container_caps)
            .name("container")
            .description("Container profile")
            .add_profile(video_profile)
            .add_profile(audio_profile)
            .build();

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
                bus.post(message).expect("Failed to post cancellation message");
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
        if let Some(_watch_id) = self.bus_watch_id.take() {
            // BusWatchGuard removed on drop
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
            ContainerFormat::Webm => "video/webm",
            ContainerFormat::Mov => "video/quicktime",
            ContainerFormat::Avi => "video/x-msvideo",
            ContainerFormat::Flv => "video/x-flv",
            ContainerFormat::Wmv => "video/x-ms-wmv",
            ContainerFormat::Mpg => "video/mpeg",
            ContainerFormat::Ts => "video/mpegts",
            ContainerFormat::Mxf => "application/mxf",
            ContainerFormat::Gif => "image/gif",
            ContainerFormat::PngSequence => "image/png",
            ContainerFormat::JpegSequence => "image/jpeg",
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
            VideoFormat::Mjpeg => "video/x-mjpeg",
            VideoFormat::Mpeg2 => "video/x-mpeg2video",
            VideoFormat::Mpeg4 => "video/x-mpeg4",
            VideoFormat::Theora => "video/x-theora",
            VideoFormat::Raw => "video/x-raw",
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
            AudioFormat::Ac3 => "audio/x-ac3",
            AudioFormat::Eac3 => "audio/x-eac3",
            AudioFormat::Wma => "audio/x-wma",
        }
    }
}
