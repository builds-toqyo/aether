use anyhow::{anyhow, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::path::Path;

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
            ConversionFormat::MP4 => ".mp4",
            ConversionFormat::WebM => ".webm",
            ConversionFormat::MOV => ".mov",
            ConversionFormat::MP3 => ".mp3",
            ConversionFormat::WAV => ".wav",
            ConversionFormat::FLAC => ".flac",
            ConversionFormat::JPEG => ".jpeg",
            ConversionFormat::PNG => ".png",
            ConversionFormat::WebP => ".webp",
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
        gst::init()?;

        Ok(Self {
            initialized: true,
        })
    }

    pub fn convert_video<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
        options: VideoConversionOptions,
        _progress_callback: impl Fn(f64) + Send + 'static,
    ) -> Result<()> {
        if !self.initialized {
            return Err(anyhow!("GStreamer not initialized"));
        }

        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let muxer_name = match options.format {
            ConversionFormat::MP4 | ConversionFormat::MOV => "mp4mux",
            ConversionFormat::WebM => "webmmux",
            _ => "mp4mux",
        };
        let video_enc_name = match options.format {
            ConversionFormat::MP4 | ConversionFormat::MOV => "x264enc",
            ConversionFormat::WebM => "vp9enc",
            _ => "x264enc",
        };
        let audio_enc_name = match options.format {
            ConversionFormat::MP4 | ConversionFormat::MOV => "avenc_aac",
            ConversionFormat::WebM => "opusenc",
            _ => "avenc_aac",
        };

        let pipeline = gst::Pipeline::new();
        let filesrc = gst::ElementFactory::make("filesrc")
            .property("location", input_path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesrc: {}", e))?;
        let decodebin = gst::ElementFactory::make("decodebin")
            .build().map_err(|e| anyhow!("decodebin: {}", e))?;
        let muxer = gst::ElementFactory::make(muxer_name)
            .build().map_err(|e| anyhow!("muxer {}: {}", muxer_name, e))?;
        let filesink = gst::ElementFactory::make("filesink")
            .property("location", output_path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesink: {}", e))?;

        pipeline.add_many([&filesrc, &decodebin, &muxer, &filesink])
            .map_err(|e| anyhow!("add: {}", e))?;
        filesrc.link(&decodebin).map_err(|e| anyhow!("link filesrc-decodebin: {}", e))?;
        muxer.link(&filesink).map_err(|e| anyhow!("link muxer-filesink: {}", e))?;

        let video_enc = gst::ElementFactory::make(video_enc_name)
            .build().map_err(|e| anyhow!("video_enc: {}", e))?;
        let audio_enc = gst::ElementFactory::make(audio_enc_name)
            .build().map_err(|e| anyhow!("audio_enc: {}", e))?;
        let v_queue = gst::ElementFactory::make("queue").build().map_err(|e| anyhow!("v_queue: {}", e))?;
        let a_queue = gst::ElementFactory::make("queue").build().map_err(|e| anyhow!("a_queue: {}", e))?;
        let v_convert = gst::ElementFactory::make("videoconvert").build().map_err(|e| anyhow!("v_convert: {}", e))?;
        let a_convert = gst::ElementFactory::make("audioconvert").build().map_err(|e| anyhow!("a_convert: {}", e))?;

        pipeline.add_many([&v_queue, &v_convert, &video_enc, &a_queue, &a_convert, &audio_enc])
            .map_err(|e| anyhow!("add enc: {}", e))?;

        gst::Element::link_many([&v_queue, &v_convert, &video_enc]).ok();
        gst::Element::link_many([&a_queue, &a_convert, &audio_enc]).ok();

        decodebin.connect_pad_added(move |_db, src_pad| {
            let caps = src_pad.current_caps();
            let name = caps.and_then(|c| c.structure(0).map(|s| s.name().to_string()));
            if let Some(name) = name {
                if name.starts_with("video/") {
                    if let Some(sink) = v_queue.static_pad("sink") {
                        if !sink.is_linked() {
                            let _ = src_pad.link(&sink);
                            let src = video_enc.static_pad("src");
                            let mux_sink = muxer.request_pad_simple("video_%u");
                            if let (Some(src), Some(mux_sink)) = (src, mux_sink) {
                                let _ = src.link(&mux_sink);
                            }
                        }
                    }
                } else if name.starts_with("audio/") {
                    if let Some(sink) = a_queue.static_pad("sink") {
                        if !sink.is_linked() {
                            let _ = src_pad.link(&sink);
                            let src = audio_enc.static_pad("src");
                            let mux_sink = muxer.request_pad_simple("audio_%u");
                            if let (Some(src), Some(mux_sink)) = (src, mux_sink) {
                                let _ = src.link(&mux_sink);
                            }
                        }
                    }
                }
            }
        });

        let bus = pipeline.bus().ok_or_else(|| anyhow!("no bus"))?;
        pipeline.set_state(gst::State::Playing).map_err(|e| anyhow!("play: {}", e))?;

        let msg = bus.timed_pop_filtered(gst::ClockTime::from_seconds(300),
            &[gst::MessageType::Error, gst::MessageType::Eos]);
        pipeline.set_state(gst::State::Null).map_err(|e| anyhow!("stop: {}", e))?;

        match msg {
            Some(msg) => match msg.view() {
                gst::MessageView::Error(err) => Err(anyhow!("err: {}", err.error())),
                gst::MessageView::Eos(_) => Ok(()),
                _ => Err(anyhow!("unexpected")),
            },
            None => Err(anyhow!("timeout")),
        }
    }

    /// Convert an audio file
    pub fn convert_audio<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
        options: AudioConversionOptions,
        _progress_callback: impl Fn(f64) + Send + 'static,
    ) -> Result<()> {
        if !self.initialized {
            return Err(anyhow!("GStreamer not initialized"));
        }

        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();


        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let enc_name = match options.format {
            ConversionFormat::MP3 => "lamemp3enc",
            ConversionFormat::WAV => "wavenc",
            ConversionFormat::FLAC => "flacenc",
            _ => "lamemp3enc",
        };

        let pipeline = gst::Pipeline::new();
        let filesrc = gst::ElementFactory::make("filesrc")
            .property("location", input_path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesrc: {}", e))?;
        let decodebin = gst::ElementFactory::make("decodebin")
            .build().map_err(|e| anyhow!("decodebin: {}", e))?;
        let queue = gst::ElementFactory::make("queue")
            .build().map_err(|e| anyhow!("queue: {}", e))?;
        let audioconvert = gst::ElementFactory::make("audioconvert")
            .build().map_err(|e| anyhow!("audioconvert: {}", e))?;
        let encoder = gst::ElementFactory::make(enc_name)
            .build().map_err(|e| anyhow!("encoder {}: {}", enc_name, e))?;
        let filesink = gst::ElementFactory::make("filesink")
            .property("location", output_path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesink: {}", e))?;

        pipeline.add_many([&filesrc, &decodebin, &queue, &audioconvert, &encoder, &filesink])
            .map_err(|e| anyhow!("add: {}", e))?;
        filesrc.link(&decodebin).map_err(|e| anyhow!("link: {}", e))?;
        gst::Element::link_many([&queue, &audioconvert, &encoder, &filesink])
            .map_err(|e| anyhow!("downstream: {}", e))?;

        decodebin.connect_pad_added(move |_, src_pad| {
            if let Some(sink) = queue.static_pad("sink") {
                if !sink.is_linked() {
                    if let Some(caps) = src_pad.current_caps() {
                        if let Some(s) = caps.structure(0) {
                            if s.name().starts_with("audio/") {
                                let _ = src_pad.link(&sink);
                            }
                        }
                    }
                }
            }
        });

        let bus = pipeline.bus().ok_or_else(|| anyhow!("no bus"))?;
        pipeline.set_state(gst::State::Playing).map_err(|e| anyhow!("play: {}", e))?;

        let msg = bus.timed_pop_filtered(gst::ClockTime::from_seconds(300),
            &[gst::MessageType::Error, gst::MessageType::Eos]);
        pipeline.set_state(gst::State::Null).map_err(|e| anyhow!("stop: {}", e))?;

        match msg {
            Some(msg) => match msg.view() {
                gst::MessageView::Error(err) => Err(anyhow!("err: {}", err.error())),
                gst::MessageView::Eos(_) => Ok(()),
                _ => Err(anyhow!("unexpected")),
            },
            None => Err(anyhow!("timeout")),
        }
    }

    fn build_video_pipeline_string(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &VideoConversionOptions,
    ) -> Result<String> {
        let _input_uri = format!("file://{}", input_path.to_string_lossy());
        let _output_uri = format!("file://{}", output_path.to_string_lossy());


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
        let _input_uri = format!("file://{}", input_path.to_string_lossy());
        let _output_uri = format!("file://{}", output_path.to_string_lossy());


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


        let _container_format = match options.format {
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
