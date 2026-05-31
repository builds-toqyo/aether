use std::path::Path;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;
use std::slice;

use ffmpeg_next as ffmpeg;
use ffmpeg::format::{context::Input, input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context as SwsContext, flag::Flags};
use ffmpeg::util::frame::video::Video;
use ffmpeg::util::frame::{self, Frame};
use ffmpeg::util::format;
use ffmpeg::util::error::Error as FFmpegError;
use ffmpeg::util::log as ffmpeg_log;
use ffmpeg::{decoder, encoder};
use log::{debug, error, info, warn};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VideoDecoderError {
    #[error("Initialization error: {0}")]
    InitializationError(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("FFmpeg error: {0}")]
    FFmpegError(String),

    #[error("FFmpeg error: {0}")]
    FFmpegLibError(#[from] FFmpegError),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoFormat {
    RGB24,
    RGBA32,
    YUV420P,
    YUV422P,
    YUV444P,
    NV12,
    Custom(format::Pixel),
}

impl VideoFormat {

    pub fn to_ffmpeg_format(&self) -> format::Pixel {
        match self {
            VideoFormat::RGB24 => format::Pixel::RGB24,
            VideoFormat::RGBA32 => format::Pixel::RGBA,
            VideoFormat::YUV420P => format::Pixel::YUV420P,
            VideoFormat::YUV422P => format::Pixel::YUV422P,
            VideoFormat::YUV444P => format::Pixel::YUV444P,
            VideoFormat::NV12 => format::Pixel::NV12,
            VideoFormat::Custom(fmt) => *fmt,
        }
    }


    pub fn from_ffmpeg_format(format: format::Pixel) -> Self {
        match format {
            format::Pixel::RGB24 => VideoFormat::RGB24,
            format::Pixel::RGBA => VideoFormat::RGBA32,
            format::Pixel::YUV420P => VideoFormat::YUV420P,
            format::Pixel::YUV422P => VideoFormat::YUV422P,
            format::Pixel::YUV444P => VideoFormat::YUV444P,
            format::Pixel::NV12 => VideoFormat::NV12,
            other => VideoFormat::Custom(other),
        }
    }


    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            VideoFormat::RGB24 => 3,
            VideoFormat::RGBA32 => 4,
            VideoFormat::YUV420P => 1,
            VideoFormat::YUV422P => 2,
            VideoFormat::YUV444P => 3,
            VideoFormat::NV12 => 1,
            VideoFormat::Custom(_) => 1,
        }
    }
}


#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: VideoFormat,
    pub stride: u32,
    pub timestamp: f64,
    pub duration: f64,
    pub key_frame: bool,
}

impl VideoFrame {
    pub fn new(width: u32, height: u32, format: VideoFormat, timestamp: f64, duration: f64) -> Self {
        let bytes_per_pixel = format.bytes_per_pixel();
        let stride = width as u32 * bytes_per_pixel as u32;
        let buffer_size = (stride as usize) * (height as usize);

        Self {
            buffer: vec![0; buffer_size],
            width,
            height,
            format,
            stride,
            timestamp,
            duration,
            key_frame: false,
        }
    }

    pub fn with_buffer(mut self, buffer: Vec<u8>) -> Self {
        self.buffer = buffer;
        self
    }

    pub fn is_key_frame(&self) -> bool {
        self.key_frame
    }
}


#[derive(Debug, Clone)]
pub struct VideoStreamInfo {
    pub index: i32,
    pub width: u32,
    pub height: u32,
    pub format: VideoFormat,
    pub frame_rate: f64,
    pub duration: f64,
    pub bit_rate: u64,
    pub frames: i64,
}


#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    pub index: i32,
    pub sample_rate: u32,
    pub channels: u32,
    pub duration: f64,
    pub bit_rate: u64,
}


#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub path: String,
    pub format_name: String,
    pub duration: f64,
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
    pub metadata: HashMap<String, String>,
}


pub struct VideoDecoderConfig {
    pub hardware_acceleration: bool,
    pub output_format: VideoFormat,
    pub thread_count: u32,
}

impl Default for VideoDecoderConfig {
    fn default() -> Self {
        Self {
            hardware_acceleration: true,
            output_format: VideoFormat::RGB24,
            thread_count: 2,
        }
    }
}


pub struct VideoDecoder {
    config: VideoDecoderConfig,
    is_initialized: bool,
    media_info: Option<MediaInfo>,
    current_video_stream: i32,
    current_audio_stream: i32,
    current_position: f64,

    format_context: Option<Input>,
    video_codec_context: Option<decoder::Video>,
    audio_codec_context: Option<decoder::Audio>,
    sws_context: Option<SwsContext>,
    state: Arc<Mutex<DecoderState>>,
}


struct DecoderState {
    is_decoding: bool,
    is_seeking: bool,
    last_decoded_frame_pts: i64,
    error_count: u32,
}

impl VideoDecoder {

    pub fn new(config: VideoDecoderConfig) -> Self {
        let state = DecoderState {
            is_decoding: false,
            is_seeking: false,
            last_decoded_frame_pts: 0,
            error_count: 0,
        };

        Self {
            config,
            is_initialized: false,
            media_info: None,
            current_video_stream: -1,
            current_audio_stream: -1,
            current_position: 0.0,
            format_context: None,
            video_codec_context: None,
            audio_codec_context: None,
            sws_context: None,
            state: Arc::new(Mutex::new(state)),
        }
    }


    fn init_ffmpeg() -> Result<(), VideoDecoderError> {

        ffmpeg::init().map_err(|e| VideoDecoderError::InitializationError(format!("Failed to initialize FFmpeg: {}", e)))?;


        ffmpeg_log::set_level(ffmpeg_log::Level::Info);

        Ok(())
    }


    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> Result<&MediaInfo, VideoDecoderError> {

        Self::init_ffmpeg()?;


        if self.is_initialized {
            self.close()?;
        }

        let path_str = path.as_ref().to_string_lossy().to_string();
        debug!("Opening media file: {}", path_str);


        let input_ctx = input(&path_str)
            .map_err(|e| VideoDecoderError::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open input file: {}", e)
            )))?;


        self.format_context = Some(input_ctx);


        let format_ctx = self.format_context.as_mut().unwrap();


        let mut video_stream_index = -1;
        let mut audio_stream_index = -1;
        let mut video_streams = Vec::new();
        let mut audio_streams = Vec::new();


        for (stream_index, stream) in format_ctx.streams().enumerate() {
            let codec_params = stream.parameters();
            let stream_idx = stream_index as i32;

            match codec_params.medium() {
                Type::Video => {

                    if video_stream_index < 0 {
                        video_stream_index = stream_idx;
                    }


                    let decoder = ffmpeg::codec::context::Context::from_parameters(codec_params)
                        .map_err(|e| VideoDecoderError::FFmpegLibError(e))?
                        .decoder()
                        .video()
                        .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

                    let width = decoder.width();
                    let height = decoder.height();
                    let pixel_format = decoder.format();


                    let frame_rate = stream.avg_frame_rate()
                        .map(|(num, den)| {
                            if num == 0 || den == 0 {
                                30.0
                            } else {
                                num as f64 / den as f64
                            }
                        })
                        .unwrap_or(30.0);


                    let duration = match stream.duration() {
                        Some(d) => {
                            d.seconds() as f64
                        },
                        None => 0.0, // TODO: Get duration from format context when API is available
                    };


                    let video_info = VideoStreamInfo {
                        index: stream_idx,
                        width: width as u32,
                        height: height as u32,
                        format: VideoFormat::from_ffmpeg_format(pixel_format.into()),
                        frame_rate,
                        duration,
                        bit_rate: decoder.bit_rate() as u64,
                        frames: stream.frames() as i64,
                    };

                    video_streams.push(video_info);
                },
                Type::Audio => {

                    if audio_stream_index < 0 {
                        audio_stream_index = stream_idx;
                    }


                    let decoder = ffmpeg::codec::context::Context::from_parameters(codec_params)
                        .map_err(|e| VideoDecoderError::FFmpegLibError(e))?
                        .decoder()
                        .audio()
                        .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;


                    let duration = match stream.duration() {
                        Some(d) => {
                            d.seconds() as f64
                        },
                        None => 0.0, // TODO: Get duration from format context when API is available
                    };


                    let audio_info = AudioStreamInfo {
                        index: stream_idx,
                        sample_rate: decoder.rate(),
                        channels: decoder.channels() as u32,
                        duration,
                        bit_rate: decoder.bit_rate() as u64,
                    };

                    audio_streams.push(audio_info);
                },
                _ => {}
            }
        }


        if video_stream_index >= 0 {
            let stream = format_ctx.stream(video_stream_index as usize)
                .ok_or_else(|| VideoDecoderError::DecodingError("Video stream not found".to_string()))?;
            let codec_params = stream.parameters();


            let decoder_id = codec_params.id();
            let decoder = ffmpeg::codec::decoder::find(decoder_id)
                .ok_or_else(|| VideoDecoderError::DecodingError(
                    format!("Failed to find decoder for codec id: {:?}", decoder_id)
                ))?;


            let mut codec_ctx = ffmpeg::codec::context::Context::new();
            codec_ctx.set_parameters(codec_params)
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;


            // Get decoder properties before opening
            let decoder = codec_ctx.decoder().video()
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
            
            let src_format = decoder.format();
            let dst_format = self.config.output_format.to_ffmpeg_format();
            let width = decoder.width();
            let height = decoder.height();

            // Now open the decoder for actual use
            let video_decoder = decoder.open()
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

            if src_format != dst_format {

                let sws_ctx = SwsContext::get_context(
                    width as i32, height as i32, src_format,
                    width as i32, height as i32, dst_format,
                    Flags::BILINEAR,
                ).map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

                self.sws_context = Some(sws_ctx);
            }

            self.video_codec_context = Some(video_decoder);
            self.current_video_stream = video_stream_index;
        }


        if audio_stream_index >= 0 {
            let stream = format_ctx.stream(audio_stream_index as usize)
                .ok_or_else(|| VideoDecoderError::DecodingError("Audio stream not found".to_string()))?;
            let codec_params = stream.parameters();


            let decoder_id = codec_params.id();
            let decoder = ffmpeg::codec::decoder::find(decoder_id)
                .ok_or_else(|| VideoDecoderError::DecodingError(
                    format!("Failed to find decoder for codec id: {:?}", decoder_id)
                ))?;


            let mut codec_ctx = ffmpeg::codec::context::Context::new();
            codec_ctx.set_parameters(codec_params)
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;


            let audio_decoder = codec_ctx.decoder().audio()
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?
                .open()
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

            self.audio_codec_context = Some(audio_decoder);
            self.current_audio_stream = audio_stream_index;
        }


        // TODO: Get metadata when API is available
        let metadata = HashMap::new();


        let format_name = "unknown"; // TODO: Get format name when API is available
        let duration = 0.0; // TODO: Get duration from format context when API is available

        let media_info = MediaInfo {
            path: path_str,
            format_name,
            duration,
            video_streams,
            audio_streams,
            metadata,
        };

        self.media_info = Some(media_info);
        self.current_position = 0.0;
        self.is_initialized = true;


        self.media_info.as_ref().ok_or(VideoDecoderError::InitializationError(
            "Failed to initialize media info".to_string()
        ))
    }


    pub fn decode_video_frame(&mut self) -> Result<VideoFrame, VideoDecoderError> {
        if !self.is_initialized {
            return Err(VideoDecoderError::InitializationError("Decoder not initialized".to_string()));
        }


        if self.current_video_stream < 0 || self.video_codec_context.is_none() {
            return Err(VideoDecoderError::DecodingError("No valid video stream selected".to_string()));
        }

        let format_ctx = self.format_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Format context not initialized".to_string()))?;

        let video_ctx = self.video_codec_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Video codec context not initialized".to_string()))?;

        let video_stream_index = self.current_video_stream as usize;


        let mut decoded_frame = frame::Video::empty();


        {
            let mut state = self.state.lock().unwrap();
            state.is_decoding = true;
        }


        struct DecodeGuard<'a> {
            state: &'a Arc<Mutex<DecoderState>>,
        }

        impl<'a> Drop for DecodeGuard<'a> {
            fn drop(&mut self) {
                let mut state = self.state.lock().unwrap();
                state.is_decoding = false;
            }
        }

        let _guard = DecodeGuard { state: &self.state };


        let mut frame_decoded = false;

        for (stream, packet) in format_ctx.packets() {
            if stream.index() == video_stream_index {
                video_ctx.send_packet(&packet)
                    .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

                // Try to receive frame - it's ok if we need more packets
                while video_ctx.receive_frame(&mut decoded_frame).is_ok() {
                    frame_decoded = true;

                    let time_base = stream.time_base();
                    let pts = decoded_frame.pts().unwrap_or(0);
                    self.current_position = pts as f64 * time_base.0 as f64 / time_base.1 as f64;

                    let mut state = self.state.lock().unwrap();
                    state.last_decoded_frame_pts = pts;

                    if frame_decoded {
                        break;
                    }
                }

                if frame_decoded {
                    break;
                }
            }
        }

        if !frame_decoded {
            return Err(VideoDecoderError::DecodingError("End of stream reached".to_string()));
        }


        // decoded_frame is already a frame::Video
        let width = decoded_frame.width() as u32;
        let height = decoded_frame.height() as u32;
        let src_format = decoded_frame.format();
        let dst_format = self.config.output_format.to_ffmpeg_format();


        let mut output_frame = frame::Video::empty();
        let mut buffer: Vec<u8>;
        let stride: u32;


        if src_format != dst_format {
            let sws_ctx = match &mut self.sws_context {
                Some(ctx) => ctx,
                None => {
                    let width = decoded_frame.width();
                    let height = decoded_frame.height();

                    let sws_ctx = SwsContext::get_context(
                        width as i32, height as i32, src_format,
                        width as i32, height as i32, dst_format,
                        Flags::BILINEAR,
                    ).map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

                    self.sws_context = Some(sws_ctx);
                    self.sws_context.as_ref().unwrap()
                }
            };
        }

        Ok(output_frame)
    }

    pub fn select_video_stream(&mut self, stream_index: i32) -> Result<(), VideoDecoderError> {
        if self.current_video_stream == stream_index {
            return Ok(());
        }

        let format_ctx = self.format_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Format context not initialized".to_string()))?;

        self.video_codec_context = None;
        self.sws_context = None;

        let stream = format_ctx.stream(stream_index as usize).unwrap();
        let codec_params = stream.parameters();

        let decoder_id = codec_params.id();
        let decoder = ffmpeg::codec::decoder::find(decoder_id)
            .ok_or_else(|| VideoDecoderError::DecodingError(
                format!("Failed to find decoder for codec id: {:?}", decoder_id)
            ))?;

        let mut codec_ctx = ffmpeg::codec::context::Context::new();
        codec_ctx.set_parameters(codec_params)
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

        // Get decoder properties before opening
        let decoder = codec_ctx.decoder().video()
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
        
        let src_format = decoder.format();
        let dst_format = self.config.output_format.to_ffmpeg_format();
        let width = decoder.width();
        let height = decoder.height();

        // Now open the decoder for actual use
        let video_decoder = decoder.open()
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

        if src_format != dst_format {

            let sws_ctx = SwsContext::get_context(
                width as i32, height as i32, src_format,
                width as i32, height as i32, dst_format,
                Flags::BILINEAR,
            ).map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

            self.sws_context = Some(sws_ctx);
        }

        self.video_codec_context = Some(video_decoder);
        self.current_video_stream = stream_index;

        Ok(())
    }


    pub fn select_audio_stream(&mut self, stream_index: i32) -> Result<(), VideoDecoderError> {
        if !self.is_initialized {
            return Err(VideoDecoderError::InitializationError("Decoder not initialized".to_string()));
        }

        let media_info = self.media_info.as_ref().ok_or(VideoDecoderError::InitializationError(
            "No media info available".to_string()
        ))?;

        if stream_index < 0 || stream_index as usize >= media_info.audio_streams.len() {
            return Err(VideoDecoderError::DecodingError(
                format!("Invalid audio stream index: {}", stream_index)
            ));
        }


        if self.current_audio_stream == stream_index {
            return Ok(());
        }


        let format_ctx = self.format_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Format context not initialized".to_string()))?;


        self.audio_codec_context = None;


        let stream = format_ctx.stream(stream_index as usize).unwrap();
        let codec_params = stream.parameters();


        let decoder_id = codec_params.id();
        let decoder = ffmpeg::codec::decoder::find(decoder_id)
            .ok_or_else(|| VideoDecoderError::DecodingError(
                format!("Failed to find decoder for codec id: {:?}", decoder_id)
            ))?;


        let mut codec_ctx = ffmpeg::codec::context::Context::new();
        codec_ctx.set_parameters(codec_params)
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;


        let audio_decoder = codec_ctx.decoder().audio()
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?
            .open()
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;

        self.audio_codec_context = Some(audio_decoder);
        self.current_audio_stream = stream_index;

        Ok(())
    }


    pub fn get_position(&self) -> f64 {
        self.current_position
    }


    pub fn close(&mut self) -> Result<(), VideoDecoderError> {
        if !self.is_initialized {
            return Ok(());
        }


        {
            let mut state = self.state.lock().unwrap();
            while state.is_decoding || state.is_seeking {


                state.is_decoding = false;
                state.is_seeking = false;
            }
        }


        self.sws_context = None;


        self.video_codec_context = None;
        self.audio_codec_context = None;


        self.format_context = None;


        self.is_initialized = false;
        self.current_position = 0.0;
        self.current_video_stream = -1;
        self.current_audio_stream = -1;
        self.media_info = None;


        let mut state = self.state.lock().unwrap();
        state.last_decoded_frame_pts = 0;
        state.error_count = 0;

        Ok(())
    }


    pub fn get_media_info(&self) -> Option<&MediaInfo> {
        self.media_info.as_ref()
    }

    pub fn seek(&mut self, time: f64) -> Result<(), VideoDecoderError> {
        if !self.is_initialized {
            return Err(VideoDecoderError::NotInitialized);
        }

        // Convert time to stream time base
        if let (Some(format_ctx), Some(video_stream_index)) = (&self.format_context, &self.current_video_stream) {
            if let Some(stream) = format_ctx.streams().nth(*video_stream_index as usize) {
                let time_base = stream.time_base();
                let target_pts = (time * time_base.1 as f64 / time_base.0 as f64) as i64;
                
                // Seek to the target position
                format_ctx.seek(target_pts, ..)?;
                self.current_position = time;
                
                // Reset frame cache
                let mut state = self.state.lock().unwrap();
                state.last_decoded_frame_pts = target_pts;
                
                return Ok(());
            }
        }
        
        Err(VideoDecoderError::DecodingError("No video stream available for seeking".to_string()))
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {

        let _ = self.close();
    }
}


pub fn create_default_decoder() -> VideoDecoder {
    VideoDecoder::new(VideoDecoderConfig::default())
}


pub fn get_media_info<P: AsRef<Path>>(path: P) -> Result<MediaInfo, VideoDecoderError> {
    let mut decoder = create_default_decoder();
    let info = decoder.open(path)?;
    Ok(info.clone())
}
