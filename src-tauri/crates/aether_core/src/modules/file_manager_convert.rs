use anyhow::{anyhow, Result};
use glib::MainLoop;
use gstreamer as gst;
use gstreamer::prelude::*;
use log::{debug, error, info, warn};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionFormat {
    MP4,
    WebM,
    MOV,
    MP3,
    WAV,
    FLAC,
    JPEG,
    PNG,
    WebP,
}

impl ConversionFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ConversionFormat::MP4 => __STRING_0__,
            ConversionFormat::WebM => __STRING_1__,
            ConversionFormat::MOV => __STRING_2__,
            ConversionFormat::MP3 => __STRING_3__,
            ConversionFormat::WAV => __STRING_4__,
            ConversionFormat::FLAC => __STRING_5__,
            ConversionFormat::JPEG => __STRING_6__,
            ConversionFormat::PNG => __STRING_7__,
            ConversionFormat::WebP => __STRING_8__,
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            ConversionFormat::MP4 => "video/mp4",
            ConversionFormat::WebM => "video/webm",
            ConversionFormat::MOV => "video/quicktime",
            ConversionFormat::MP3 => "audio/mpeg",
            ConversionFormat::WAV => "audio/wav",
            ConversionFormat::FLAC => "audio/flac",
            ConversionFormat::JPEG => "image/jpeg",
            ConversionFormat::PNG => "image/png",
            ConversionFormat::WebP => "image/webp",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp4" => Some(ConversionFormat::MP4),
            "webm" => Some(ConversionFormat::WebM),
            "mov" => Some(ConversionFormat::MOV),
            "mp3" => Some(ConversionFormat::MP3),
            "wav" => Some(ConversionFormat::WAV),
            "flac" => Some(ConversionFormat::FLAC),
            "jpg" | "jpeg" => Some(ConversionFormat::JPEG),
            "png" => Some(ConversionFormat::PNG),
            "webp" => Some(ConversionFormat::WebP),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoConversionOptions {
    pub format: ConversionFormat,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub video_bitrate: Option<u32>,
    pub audio_bitrate: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub preserve_aspect_ratio: bool,
    pub frame_rate: Option<f64>,
    pub fastcopy: bool,
}

impl Default for VideoConversionOptions {
    fn default() -> Self {
        Self {
            format: ConversionFormat::MP4,
            video_codec: None,
            audio_codec: None,
            video_bitrate: None,
            audio_bitrate: None,
            width: None,
            height: None,
            preserve_aspect_ratio: true,
            frame_rate: None,
            fastcopy: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioConversionOptions {
    pub format: ConversionFormat,
    pub audio_codec: Option<String>,
    pub audio_bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub fastcopy: bool,
}

impl Default for AudioConversionOptions {
    fn default() -> Self {
        Self {
            format: ConversionFormat::MP3,
            audio_codec: None,
            audio_bitrate: None,
            sample_rate: None,
            channels: None,
            fastcopy: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageConversionOptions {
    pub format: ConversionFormat,
    pub quality: u8,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub preserve_aspect_ratio: bool,
}

impl Default for ImageConversionOptions {
    fn default() -> Self {
        Self {
            format: ConversionFormat::JPEG,
            quality: 90,
            width: None,
            height: None,
            preserve_aspect_ratio: true,
        }
    }
}

pub struct MediaConverter {
    initialized: bool,
}

impl MediaConverter {
    pub fn new() -> Result<Self> {
        if !gst::is_initialized() {
            gst::init()?;
        }

        Ok(Self {
            initialized: true,
        })
    }

    pub fn convert_video<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
        options: VideoConversionOptions,
        progress_callback: impl Fn(f64) + Send + 'static,
    ) -> Result<()> {
        if !self.initialized {
            return Err(anyhow!(__STRING_28__));
        }

        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pipeline_str = self.build_video_pipeline_string(input_path, output_path, &options)?;
        debug!(__STRING_29__, pipeline_str);

        let pipeline = gst::parse_launch(&pipeline_str)?;
        let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

        let progress = Arc::new(Mutex::new(0.0));
        let progress_for_callback = progress.clone();

        let bus = pipeline.bus().unwrap();
        let main_loop = MainLoop::new(None, false);
        let main_loop_clone = main_loop.clone();

        bus.add_watch(move |_, msg| {
            match msg.view() {
                gst::MessageView::Eos(..) => {
                    let mut progress = progress.lock().unwrap();
                    *progress = 100.0;
                    progress_callback(100.0);
                    main_loop_clone.quit();
                },
                gst::MessageView::Error(err) => {
                    error!(__STRING_30__, err.error(), err.debug().unwrap_or_default());
                    main_loop_clone.quit();
                },
                gst::MessageView::StateChanged(state_changed) => {
                    if state_changed.src().map(|s| s == pipeline.upcast_ref::<gst::Object>()).unwrap_or(false) {
                        debug!(__STRING_31__,
                               state_changed.old(),
                               state_changed.current());
                    }
                },
                gst::MessageView::Element(element) => {
                    let structure = element.structure();
                    if let Some(structure) = structure {
                        if structure.name() == __STRING_32__ {
                            if let Ok(percent) = structure.get::<f64>(__STRING_33__) {
                                let mut progress = progress.lock().unwrap();
                                *progress = percent;
                                progress_callback(percent);
                            }
                        }
                    }
                },
                _ => (),
            }

            glib::Continue(true)
        })?;

        // Start the pipeline
        pipeline.set_state(gst::State::Playing)?;

        // Run the main loop
        main_loop.run();

        // Clean up
        pipeline.set_state(gst::State::Null)?;

        // Check final progress
        let final_progress = *progress_for_callback.lock().unwrap();
        if final_progress < 100.0 {
            return Err(anyhow!(__STRING_34__));
        }

        Ok(())
    }

    /// Convert an audio file
    pub fn convert_audio<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
        options: AudioConversionOptions,
        progress_callback: impl Fn(f64) + Send + 'static,
    ) -> Result<()> {
        if !self.initialized {
            return Err(anyhow!("GStreamer not initialized"));
        }

        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();


        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }


        let pipeline_str = self.build_image_pipeline_string(input_path, output_path, &options)?;
        debug!("Image conversion pipeline: {}", pipeline_str);


        let pipeline = gst::parse_launch(&pipeline_str)?;
        let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();


        let bus = pipeline.bus().unwrap();
        let main_loop = MainLoop::new(None, false);
        let main_loop_clone = main_loop.clone();

        bus.add_watch(move |_, msg| {
            match msg.view() {
                gst::MessageView::Eos(..) => {
                    main_loop_clone.quit();
                },
                gst::MessageView::Error(err) => {
                    error!("Error from GStreamer pipeline: {} ({})", err.error(), err.debug().unwrap_or_default());
                    main_loop_clone.quit();
                },
                _ => (),
            }

            glib::Continue(true)
        })?;


        pipeline.set_state(gst::State::Playing)?;


        main_loop.run();


        pipeline.set_state(gst::State::Null)?;


        if !output_path.exists() {
            return Err(anyhow!("Conversion failed: output file not created"));
        }

        Ok(())
    }


    fn build_video_pipeline_string(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &VideoConversionOptions,
    ) -> Result<String> {
        let input_uri = format!("file://{}", input_path.to_string_lossy());
        let output_uri = format!("file://{}", output_path.to_string_lossy());


        let video_encoder = match options.video_codec.as_deref() {
            Some(codec) => codec.to_string(),
            None => match options.format {
                ConversionFormat::MP4 | ConversionFormat::MOV => "x264enc".to_string(),
                ConversionFormat::WebM => "vp9enc".to_string(),
                _ => return Err(anyhow!("Unsupported video format: {:?}", options.format)),
            },
        };


        let audio_encoder = match options.audio_codec.as_deref() {
            Some(codec) => codec.to_string(),
            None => match options.format {
                ConversionFormat::MP4 | ConversionFormat::MOV => "avenc_aac".to_string(),
                ConversionFormat::WebM => "opusenc".to_string(),
                _ => return Err(anyhow!("Unsupported audio format: {:?}", options.format)),
            },
        };


        let mut video_enc_options = String::new();

        if let Some(bitrate) = options.video_bitrate {
            video_enc_options.push_str(&format!(" bitrate={}", bitrate / 1000));
        }


        let mut video_scale_options = String::new();

        if options.width.is_some() || options.height.is_some() {
            video_scale_options.push_str(" ! videoscale");

            if options.preserve_aspect_ratio {
                video_scale_options.push_str(" ! videoscale method=lanczos");
            }

            video_scale_options.push_str(" ! video/x-raw");

            if let Some(width) = options.width {
                video_scale_options.push_str(&format!(", width={}", width));
            }

            if let Some(height) = options.height {
                video_scale_options.push_str(&format!(", height={}", height));
            }
        }


        let mut framerate_options = String::new();

        if let Some(fps) = options.frame_rate {
            framerate_options.push_str(&format!(" ! videorate ! video/x-raw, framerate={}/1", fps as i32));
        }


        let mut audio_enc_options = String::new();

        if let Some(bitrate) = options.audio_bitrate {
            audio_enc_options.push_str(&format!(" bitrate={}", bitrate / 1000));
        }


        let container_format = match options.format {
            ConversionFormat::MP4 => "mp4mux",
            ConversionFormat::WebM => "webmmux",
            ConversionFormat::MOV => "qtmux",
            _ => return Err(anyhow!("Unsupported video container format: {:?}", options.format)),
        };


        let pipeline = if options.fastcopy {

            format!(
                "filesrc location=\"{}\" ! decodebin name=demux \
                 demux.video_0 ! queue ! {} ! {} name=mux \
                 demux.audio_0 ! queue ! {} ! mux. \
                 mux. ! progressreport update-freq=1 ! filesink location=\"{}\"",
                input_path.to_string_lossy(),
                video_encoder, container_format,
                audio_encoder,
                output_path.to_string_lossy()
            )
        } else {

            format!(
                "filesrc location=\"{}\" ! decodebin name=demux \
                 demux.video_0 ! queue{}{} ! {} {} ! {} name=mux \
                 demux.audio_0 ! queue ! audioconvert ! {} {} ! mux. \
                 mux. ! progressreport update-freq=1 ! filesink location=\"{}\"",
                input_path.to_string_lossy(),
                video_scale_options, framerate_options,
                video_encoder, video_enc_options, container_format,
                audio_encoder, audio_enc_options,
                output_path.to_string_lossy()
            )
        };

        Ok(pipeline)
    }


    fn build_audio_pipeline_string(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &AudioConversionOptions,
    ) -> Result<String> {
        let input_uri = format!("file://{}", input_path.to_string_lossy());
        let output_uri = format!("file://{}", output_path.to_string_lossy());


        let audio_encoder = match options.audio_codec.as_deref() {
            Some(codec) => codec.to_string(),
            None => match options.format {
                ConversionFormat::MP3 => "lamemp3enc".to_string(),
                ConversionFormat::WAV => "wavenc".to_string(),
                ConversionFormat::FLAC => "flacenc".to_string(),
                _ => return Err(anyhow!("Unsupported audio format: {:?}", options.format)),
            },
        };


        let mut audio_enc_options = String::new();

        if let Some(bitrate) = options.audio_bitrate {
            audio_enc_options.push_str(&format!(" bitrate={}", bitrate / 1000));
        }


        let mut audio_convert_options = String::new();

        if options.sample_rate.is_some() || options.channels.is_some() {
            audio_convert_options.push_str(" ! audio/x-raw");

            if let Some(rate) = options.sample_rate {
                audio_convert_options.push_str(&format!(", rate={}", rate));
            }

            if let Some(channels) = options.channels {
                audio_convert_options.push_str(&format!(", channels={}", channels));
            }
        }


        let container_format = match options.format {
            ConversionFormat::MP3 => "",
            ConversionFormat::WAV => "",
            ConversionFormat::FLAC => "",
            _ => return Err(anyhow!("Unsupported audio container format: {:?}", options.format)),
        };


        let pipeline = if options.fastcopy {

            format!(
                "filesrc location=\"{}\" ! decodebin ! queue ! {} {} ! progressreport update-freq=1 ! filesink location=\"{}\"",
                input_path.to_string_lossy(),
                audio_encoder, audio_enc_options,
                output_path.to_string_lossy()
            )
        } else {

            format!(
                "filesrc location=\"{}\" ! decodebin ! queue ! audioconvert{} ! {} {} ! progressreport update-freq=1 ! filesink location=\"{}\"",
                input_path.to_string_lossy(),
                audio_convert_options,
                audio_encoder, audio_enc_options,
                output_path.to_string_lossy()
            )
        };

        Ok(pipeline)
    }


    fn build_image_pipeline_string(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &ImageConversionOptions,
    ) -> Result<String> {

        let (image_encoder, encoder_options) = match options.format {
            ConversionFormat::JPEG => ("jpegenc", format!(" quality={}", options.quality)),
            ConversionFormat::PNG => ("pngenc", format!(" compression-level={}", 9 - (options.quality / 11))),
            ConversionFormat::WebP => ("webpenc", format!(" quality={}", options.quality as f32 / 100.0)),
            _ => return Err(anyhow!("Unsupported image format: {:?}", options.format)),
        };


        let mut image_scale_options = String::new();

        if options.width.is_some() || options.height.is_some() {
            image_scale_options.push_str(" ! videoscale");

            if options.preserve_aspect_ratio {
                image_scale_options.push_str(" ! videoscale method=lanczos");
            }

            image_scale_options.push_str(" ! video/x-raw");

            if let Some(width) = options.width {
                image_scale_options.push_str(&format!(", width={}", width));
            }

            if let Some(height) = options.height {
                image_scale_options.push_str(&format!(", height={}", height));
            }
        }


        let pipeline = format!(
            "filesrc location=\"{}\" ! decodebin ! videoconvert{} ! {} {} ! filesink location=\"{}\"",
            input_path.to_string_lossy(),
            image_scale_options,
            image_encoder, encoder_options,
            output_path.to_string_lossy()
        );

        Ok(pipeline)
    }
}
