use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use log::info;
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_editing_services as ges;
use gstreamer_editing_services::prelude::TimelineExt;
use gstreamer_pbutils::prelude::EncodingProfileBuilder;
use glib::{filename_to_uri, ControlFlow};
use crate::engine::editing::types::EditingError;

fn get_hardware_encoder_for_export() -> Option<&'static str> {
    #[cfg(target_os = "linux")]
    {
        if gst::ElementFactory::find("vaapiencode_h264").is_some() {
            info!("Using VAAPI H.264 hardware encoder (Linux)");
            return Some("vaapiencode_h264");
        }
        if gst::ElementFactory::find("vaapiencode").is_some() {
            info!("Using VAAPI hardware encoder (Linux)");
            return Some("vaapiencode");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if gst::ElementFactory::find("vtenc_h264").is_some() {
            info!("Using VideoToolbox H.264 hardware encoder (macOS)");
            return Some("vtenc_h264");
        }
        if gst::ElementFactory::find("videotoolboxenc").is_some() {
            info!("Using VideoToolbox hardware encoder (macOS)");
            return Some("videotoolboxenc");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if gst::ElementFactory::find("d3d11h264enc").is_some() {
            info!("Using D3D11 H.264 hardware encoder (Windows)");
            return Some("d3d11h264enc");
        }
        if gst::ElementFactory::find("d3d11enc").is_some() {
            info!("Using D3D11 hardware encoder (Windows)");
            return Some("d3d11enc");
        }
    }

    if gst::ElementFactory::find("nvh264enc").is_some() {
        info!("Using NVIDIA H.264 hardware encoder");
        return Some("nvh264enc");
    }
    if gst::ElementFactory::find("nvenc").is_some() {
        info!("Using NVIDIA NVENC hardware encoder");
        return Some("nvenc");
    }

    if gst::ElementFactory::find("amfh264enc").is_some() {
        info!("Using AMD AMF H.264 hardware encoder");
        return Some("amfh264enc");
    }
    if gst::ElementFactory::find("amfenc").is_some() {
        info!("Using AMD AMF hardware encoder");
        return Some("amfenc");
    }

    info!("No hardware encoder found, using software encoder");
    None
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub output_path: PathBuf,
    pub container: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub video_bitrate: u32,
    pub audio_bitrate: u32,
    pub frame_rate: f64,
    pub width: u32,
    pub height: u32,
    pub hardware_acceleration: bool,
    pub start_time: i64,
    pub end_time: i64,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            output_path: PathBuf::new(),
            container: "mkv".to_string(),
            video_codec: "libx264".to_string(),
            audio_codec: "flac".to_string(),
            video_bitrate: 0,
            audio_bitrate: 0,
            frame_rate: 30.0,
            width: 0,
            height: 0,
            hardware_acceleration: false,
            start_time: 0,
            end_time: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportProgress {
    pub position: i64,
    pub duration: i64,
    pub percent: f64,
    pub complete: bool,
    pub error: Option<String>,
}

pub struct IntermediateExporter {
    timeline: ges::Timeline,
    options: ExportOptions,
    pipeline: Option<gst::Pipeline>,
    progress: Arc<Mutex<ExportProgress>>,
    progress_callback: Option<Arc<Mutex<dyn Fn(ExportProgress) + Send + 'static>>>,
}

impl IntermediateExporter {
    pub fn new(timeline: ges::Timeline, options: ExportOptions) -> Result<Self, EditingError> {
        let progress = Arc::new(Mutex::new(ExportProgress {
            position: 0,
            duration: 0,
            percent: 0.0,
            complete: false,
            error: None,
        }));

        Ok(Self {
            timeline,
            options,
            pipeline: None,
            progress,
            progress_callback: None,
        })
    }

    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(ExportProgress) + Send + 'static,
    {
        self.progress_callback = Some(Arc::new(Mutex::new(callback)));
    }

    pub fn start_export(&mut self) -> Result<(), EditingError> {
        let _output_uri = filename_to_uri(&self.options.output_path, None)?;

        let profile = self.create_encoding_profile()?;

        let pipeline = gst::Pipeline::new();

        let filesink = gst::ElementFactory::make("filesink")
            .name("export_sink")
            .property("location", &self.options.output_path.to_string_lossy().to_string())
            .build()
            .map_err(|_| EditingError::ExportError("Failed to create filesink".to_string()))?;


        let encodebin = gst::ElementFactory::make("encodebin")
            .name("encoder")
            .property("profile", &profile)
            .build()
            .map_err(|_| EditingError::ExportError("Failed to create encodebin".to_string()))?;


        pipeline.add_many(&[&encodebin, &filesink])?;
        gst::Element::link_many(&[&encodebin, &filesink])?;

        let ges_pipeline = ges::Pipeline::new();
        let src_pad = ges_pipeline.static_pad("video_0").unwrap();
        let sink_pad = encodebin.static_pad("video_0").unwrap();
        src_pad.link(&sink_pad).map_err(|e| EditingError::ExportError(format!("Failed to link video pads: {}", e)))?;

        let src_pad = ges_pipeline.static_pad("audio_0").unwrap();
        let sink_pad = encodebin.static_pad("audio_0").unwrap();
        src_pad.link(&sink_pad).map_err(|e| EditingError::ExportError(format!("Failed to link audio pads: {}", e)))?;

        let progress = self.progress.clone();
        let callback = self.progress_callback.clone();

        let bus = pipeline.bus().unwrap();
        let pipeline_for_bus = pipeline.clone();
        let _watch_id = bus.add_watch(move |_, msg| {
            match msg.view() {
                gst::MessageView::Eos(..) => {

                    let mut progress = progress.lock().unwrap();
                    progress.complete = true;
                    progress.percent = 100.0;

                    if let Some(callback) = &callback {
                        callback.lock().unwrap()(progress.clone());
                    }
                },
                gst::MessageView::Error(err) => {
                    let mut progress = progress.lock().unwrap();
                    progress.error = Some(format!("{}: {}", err.error(), err.debug().unwrap_or_default()));

                    if let Some(callback) = &callback {
                        callback.lock().unwrap()(progress.clone());
                    }
                },
                gst::MessageView::StateChanged(state_changed) => {

                    if state_changed.src().map(|s| s == pipeline_for_bus.upcast_ref::<glib::Object>()).unwrap_or(false) {
                        if state_changed.current() == gst::State::Playing {

                        }
                    }
                },
                _ => (),
            }

            ControlFlow::Continue
        })
        .expect("Failed to add bus watch");

        let progress = self.progress.clone();
        let callback = self.progress_callback.clone();
        let timeline_duration = self.timeline.duration().nseconds() as i64;

        let pipeline_for_timer = pipeline.clone();
        let _timeout_id = glib::timeout_add_seconds(1, move || {
            if let Some(position) = pipeline_for_timer.query_position::<gst::ClockTime>() {
                let mut progress_guard = progress.lock().unwrap();
                progress_guard.position = position.nseconds() as i64;
                progress_guard.duration = timeline_duration;

                if timeline_duration > 0 {
                    progress_guard.percent = (progress_guard.position as f64 / timeline_duration as f64) * 100.0;
                }

                if let Some(callback) = &callback {
                    callback.lock().unwrap()(progress_guard.clone());
                }
            }

            ControlFlow::Continue
        });

        pipeline.set_state(gst::State::Playing)?;

        self.pipeline = Some(pipeline);

        Ok(())
    }

    fn create_encoding_profile(&self) -> Result<gst_pbutils::EncodingContainerProfile, EditingError> {
        let container_caps = gst::Caps::new_empty_simple(
            Self::container_to_caps(&self.options.container)
        );

        let video_codec = if self.options.hardware_acceleration {
            if let Some(hw_encoder) = get_hardware_encoder_for_export() {
                hw_encoder
            } else {
                info!("Hardware acceleration requested but no hardware encoder found, using software encoder");
                &self.options.video_codec
            }
        } else {
            &self.options.video_codec
        };

        let video_caps = gst::Caps::new_empty_simple(
            Self::codec_to_video_caps(video_codec)
        );
        let audio_caps = gst::Caps::new_empty_simple(
            Self::codec_to_audio_caps(&self.options.audio_codec)
        );

        let video_profile = gst_pbutils::EncodingVideoProfile::builder(&video_caps)
            .presence(1)
            .build();

        let audio_profile = gst_pbutils::EncodingAudioProfile::builder(&audio_caps)
            .presence(1)
            .build();

        let container_profile = gst_pbutils::EncodingContainerProfile::builder(&container_caps)
            .name("export")
            .description("Aether export profile")
            .add_profile(video_profile)
            .add_profile(audio_profile)
            .build();

        Ok(container_profile)
    }

    fn container_to_caps(container: &str) -> &str {
        match container {
            "mkv" | "matroska" => "video/x-matroska",
            "mp4" | "mov" | "quicktime" => "video/quicktime",
            "webm" => "video/webm",
            "avi" => "video/x-avi",
            "ogg" => "application/ogg",
            _ => "video/x-matroska",
        }
    }

    fn codec_to_video_caps(codec: &str) -> &str {
        match codec {
            // Software encoders
            "libx264" | "x264" | "h264" | "avc" => "video/x-h264",
            "libx265" | "x265" | "h265" | "hevc" => "video/x-h265",
            "vp8" => "video/x-vp8",
            "vp9" => "video/x-vp9",
            "av1" => "video/x-av1",
            "prores" => "video/x-prores",
            // Hardware encoders
            "nvh264enc" | "nvenc" | "vaapiencode_h264" | "vtenc_h264" | "d3d11h264enc" | "amfh264enc" => "video/x-h264",
            "nvh265enc" | "nvenc_hevc" | "vaapiencode_h265" | "vtenc_hevc" | "d3d11h265enc" | "amfh265enc" => "video/x-h265",
            _ => "video/x-h264",
        }
    }

    fn codec_to_audio_caps(codec: &str) -> &str {
        match codec {
            "flac" => "audio/x-flac",
            "aac" | "libfdk_aac" => "audio/mpeg",
            "mp3" | "libmp3lame" => "audio/mpeg",
            "opus" | "libopus" => "audio/x-opus",
            "vorbis" => "audio/x-vorbis",
            "wav" | "pcm" => "audio/x-raw",
            _ => "audio/x-flac",
        }
    }

    pub fn cancel_export(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null)?;

            let mut progress = self.progress.lock().unwrap();
            progress.complete = true;
            progress.error = Some("Export cancelled".to_string());

            if let Some(callback) = &self.progress_callback {
                callback.lock().unwrap()(progress.clone());
            }
        }

        self.pipeline = None;

        Ok(())
    }

    pub fn get_progress(&self) -> ExportProgress {
        self.progress.lock().unwrap().clone()
    }
}

impl Drop for IntermediateExporter {
    fn drop(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }
}
