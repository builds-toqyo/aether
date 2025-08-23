use std::path::Path;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;
use std::slice;

use ffmpeg_next as ffmpeg;
use ffmpeg::format::{context::Context, input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context as SwsContext, flag::Flags};
use ffmpeg::util::frame::video::Video;
use ffmpeg::util::frame::Frame;
use ffmpeg::util::format;
use ffmpeg::util::error::Error as FFmpegError;
use ffmpeg::util::log as ffmpeg_log;
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
    Custom(format::Pixel), // For custom pixel formats
}

impl VideoFormat {
    /// Convert to FFmpeg pixel format
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
    
    /// Create VideoFormat from FFmpeg pixel format
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
    
    /// Get bytes per pixel for this format
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            VideoFormat::RGB24 => 3,
            VideoFormat::RGBA32 => 4,
            VideoFormat::YUV420P => 1, // Note: This is approximate as YUV420P is planar
            VideoFormat::YUV422P => 2, // Note: This is approximate as YUV422P is planar
            VideoFormat::YUV444P => 3, // Note: This is approximate as YUV444P is planar
            VideoFormat::NV12 => 1,    // Note: This is approximate as NV12 is planar
            VideoFormat::Custom(_) => 1, // Default to 1 for unknown formats
        }
    }
}

/// Video frame structure
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: VideoFormat,
    pub stride: u32,      // Bytes per row
    pub timestamp: f64,   // In seconds
    pub duration: f64,    // Frame duration in seconds
    pub key_frame: bool,  // Whether this is a key frame
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

/// Video stream information
#[derive(Debug, Clone)]
pub struct VideoStreamInfo {
    pub index: i32,
    pub width: u32,
    pub height: u32,
    pub format: VideoFormat,
    pub frame_rate: f64,
    pub duration: f64, // In seconds
    pub bit_rate: u64, // In bits per second
    pub frames: i64,   // Total frames if known, -1 otherwise
}

/// Audio stream information
#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    pub index: i32,
    pub sample_rate: u32,
    pub channels: u32,
    pub duration: f64, // In seconds
    pub bit_rate: u64, // In bits per second
}

/// Media file information
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub path: String,
    pub format_name: String,
    pub duration: f64, // In seconds
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
    pub metadata: HashMap<String, String>,
}

/// Video decoder configuration
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

/// Main video decoder struct
pub struct VideoDecoder {
    config: VideoDecoderConfig,
    is_initialized: bool,
    media_info: Option<MediaInfo>,
    current_video_stream: i32,
    current_audio_stream: i32,
    current_position: f64, // In seconds
    // FFmpeg contexts
    format_context: Option<Context>,
    video_codec_context: Option<ffmpeg::codec::context::Context>,
    audio_codec_context: Option<ffmpeg::codec::context::Context>,
    sws_context: Option<SwsContext>,      // For video format conversion
    state: Arc<Mutex<DecoderState>>,
}

/// Internal decoder state
struct DecoderState {
    is_decoding: bool,
    is_seeking: bool,
    last_decoded_frame_pts: i64,
    error_count: u32,
}

impl VideoDecoder {
    /// Create a new video decoder with the given configuration
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
    
    /// Initialize FFmpeg libraries
    fn init_ffmpeg() -> Result<(), VideoDecoderError> {
        // Initialize FFmpeg
        ffmpeg::init().map_err(|e| VideoDecoderError::InitializationError(format!("Failed to initialize FFmpeg: {}", e)))?;
        
        // Set up logging
        ffmpeg_log::set_level(ffmpeg_log::Level::Info);
        
        Ok(())
    }
    
    /// Open a media file and prepare for decoding
    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> Result<&MediaInfo, VideoDecoderError> {
        // Initialize FFmpeg if not already done
        Self::init_ffmpeg()?;
        
        // Close any previously opened file
        if self.is_initialized {
            self.close()?;
        }
        
        let path_str = path.as_ref().to_string_lossy().to_string();
        debug!("Opening media file: {}", path_str);
        
        // Open the input file
        let input_ctx = input(&path_str)
            .map_err(|e| VideoDecoderError::IOError(std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("Failed to open input file: {}", e)
            )))?;
        
        // Store the format context
        self.format_context = Some(input_ctx);
        
        // Get format context for stream information
        let format_ctx = self.format_context.as_mut().unwrap();
        
        // Find the best video and audio streams
        let mut video_stream_index = -1;
        let mut audio_stream_index = -1;
        let mut video_streams = Vec::new();
        let mut audio_streams = Vec::new();
        
        // Collect stream information
        for (stream_index, stream) in format_ctx.streams().enumerate() {
            let codec_params = stream.codec().parameters();
            let stream_idx = stream_index as i32;
            
            match codec_params.medium() {
                Type::Video => {
                    // If this is the first video stream, select it by default
                    if video_stream_index < 0 {
                        video_stream_index = stream_idx;
                    }
                    
                    // Get video stream info
                    let codec = ffmpeg::codec::context::Context::new();
                    let decoder = codec.decoder().video()
                        .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
                    
                    let width = codec_params.width();
                    let height = codec_params.height();
                    let pixel_format = codec_params.format();
                    
                    // Calculate frame rate
                    let frame_rate = match stream.avg_frame_rate() {
                        (0, _) | (_, 0) => 30.0, // Default if not available
                        (num, den) => num as f64 / den as f64,
                    };
                    
                    // Calculate duration
                    let duration = match stream.duration() {
                        Some(d) => {
                            let tb = stream.time_base();
                            d as f64 * tb.0 as f64 / tb.1 as f64
                        },
                        None => format_ctx.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64,
                    };
                    
                    // Create video stream info
                    let video_info = VideoStreamInfo {
                        index: stream_idx,
                        width: width as u32,
                        height: height as u32,
                        format: VideoFormat::from_ffmpeg_format(pixel_format.into()),
                        frame_rate,
                        duration,
                        bit_rate: codec_params.bit_rate() as u64,
                        frames: stream.frames() as i64,
                    };
                    
                    video_streams.push(video_info);
                },
                Type::Audio => {
                    // If this is the first audio stream, select it by default
                    if audio_stream_index < 0 {
                        audio_stream_index = stream_idx;
                    }
                    
                    // Get audio stream info
                    let codec = ffmpeg::codec::context::Context::new();
                    let decoder = codec.decoder().audio()
                        .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
                    
                    // Calculate duration
                    let duration = match stream.duration() {
                        Some(d) => {
                            let tb = stream.time_base();
                            d as f64 * tb.0 as f64 / tb.1 as f64
                        },
                        None => format_ctx.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64,
                    };
                    
                    // Create audio stream info
                    let audio_info = AudioStreamInfo {
                        index: stream_idx,
                        sample_rate: codec_params.sample_rate() as u32,
                        channels: codec_params.channels() as u32,
                        duration,
                        bit_rate: codec_params.bit_rate() as u64,
                    };
                    
                    audio_streams.push(audio_info);
                },
                _ => {}
            }
        }
        
        // Set up video codec context if we found a video stream
        if video_stream_index >= 0 {
            let stream = format_ctx.stream(video_stream_index as usize).unwrap();
            let codec_params = stream.codec().parameters();
            
            // Find decoder for the stream
            let decoder_id = codec_params.id();
            let decoder = ffmpeg::codec::decoder::find(decoder_id)
                .ok_or_else(|| VideoDecoderError::DecodingError(
                    format!("Failed to find decoder for codec id: {:?}", decoder_id)
                ))?;
            
            // Create a codec context for the decoder
            let mut codec_ctx = ffmpeg::codec::context::Context::new();
            codec_ctx.set_parameters(codec_params)
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
            
            // Open the decoder
            let video_ctx = codec_ctx.decoder().open(decoder)
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
            
            self.video_codec_context = Some(video_ctx);
            self.current_video_stream = video_stream_index;
            
            // Set up scaling context if needed
            if let Some(video_ctx) = &self.video_codec_context {
                let video_ctx = video_ctx.decoder().video().unwrap();
                let src_format = video_ctx.format();
                let dst_format = self.config.output_format.to_ffmpeg_format();
                
                if src_format != dst_format {
                    let width = video_ctx.width();
                    let height = video_ctx.height();
                    
                    let sws_ctx = SwsContext::get(
                        width, height, src_format,
                        width, height, dst_format,
                        Flags::BILINEAR,
                    ).map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
                    
                    self.sws_context = Some(sws_ctx);
                }
            }
        }
        
        // Set up audio codec context if we found an audio stream
        if audio_stream_index >= 0 {
            let stream = format_ctx.stream(audio_stream_index as usize).unwrap();
            let codec_params = stream.codec().parameters();
            
            // Find decoder for the stream
            let decoder_id = codec_params.id();
            let decoder = ffmpeg::codec::decoder::find(decoder_id)
                .ok_or_else(|| VideoDecoderError::DecodingError(
                    format!("Failed to find decoder for codec id: {:?}", decoder_id)
                ))?;
            
            // Create a codec context for the decoder
            let mut codec_ctx = ffmpeg::codec::context::Context::new();
            codec_ctx.set_parameters(codec_params)
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
            
            // Open the decoder
            let audio_ctx = codec_ctx.decoder().open(decoder)
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
            
            self.audio_codec_context = Some(audio_ctx);
            self.current_audio_stream = audio_stream_index;
        }
        
        // Extract metadata
        let mut metadata = HashMap::new();
        for (k, v) in format_ctx.metadata().iter() {
            metadata.insert(k.to_string(), v.to_string());
        }
        
        // Create media info
        let format_name = format_ctx.format().name().to_string();
        let duration = format_ctx.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64;
        
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
        
        // Return reference to the media info
        self.media_info.as_ref().ok_or(VideoDecoderError::InitializationError(
            "Failed to initialize media info".to_string()
        ))
    }
    
    /// Decode the next video frame
    pub fn decode_video_frame(&mut self) -> Result<VideoFrame, VideoDecoderError> {
        if !self.is_initialized {
            return Err(VideoDecoderError::InitializationError("Decoder not initialized".to_string()));
        }
        
        // Check if we have a valid video stream
        if self.current_video_stream < 0 || self.video_codec_context.is_none() {
            return Err(VideoDecoderError::DecodingError("No valid video stream selected".to_string()));
        }
        
        let format_ctx = self.format_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Format context not initialized".to_string()))?;
        
        let video_ctx = self.video_codec_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Video codec context not initialized".to_string()))?;
        
        let video_stream_index = self.current_video_stream as usize;
        
        // Create a frame to hold the decoded data
        let mut decoded_frame = Frame::new();
        
        // Mark decoding state
        {
            let mut state = self.state.lock().unwrap();
            state.is_decoding = true;
        }
        
        // Cleanup state when we exit this function
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
        
        // Loop until we decode a frame or hit an error
        let mut frame_decoded = false;
        
        while !frame_decoded {
            // Read the next packet
            match format_ctx.packets().next() {
                Some((stream_index, packet)) => {
                    // Check if this packet belongs to the video stream
                    if stream_index == video_stream_index {
                        // Send the packet to the decoder
                        video_ctx.send_packet(&packet)
                            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
                        
                        // Try to receive a frame
                        match video_ctx.receive_frame(&mut decoded_frame) {
                            Ok(_) => {
                                frame_decoded = true;
                                
                                // Update position based on frame timestamp
                                let stream = format_ctx.stream(video_stream_index).unwrap();
                                let time_base = stream.time_base();
                                let pts = decoded_frame.pts().unwrap_or(0);
                                self.current_position = pts as f64 * time_base.0 as f64 / time_base.1 as f64;
                                
                                // Update internal state
                                let mut state = self.state.lock().unwrap();
                                state.last_decoded_frame_pts = pts;
                            },
                            Err(FFmpegError::Again) => {
                                // Need more packets, continue loop
                                continue;
                            },
                            Err(e) => {
                                return Err(VideoDecoderError::FFmpegLibError(e));
                            }
                        }
                    }
                },
                None => {
                    // End of file
                    return Err(VideoDecoderError::DecodingError("End of stream reached".to_string()));
                }
            }
        }
        
        // Get video information
        let video_frame = decoded_frame.video()
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
        
        let width = video_frame.width() as u32;
        let height = video_frame.height() as u32;
        let src_format = video_frame.format();
        let dst_format = self.config.output_format.to_ffmpeg_format();
        
        // Create output frame
        let mut output_frame = Frame::new();
        let mut buffer: Vec<u8>;
        let stride: u32;
        
        // Convert format if needed
        if src_format != dst_format {
            // Use scaling context for conversion
            let sws_ctx = match &mut self.sws_context {
                Some(ctx) => ctx,
                None => {
                    // Create a new scaling context if we don't have one
                    let ctx = SwsContext::get(
                        width as i32, height as i32, src_format,
                        width as i32, height as i32, dst_format,
                        Flags::BILINEAR,
                    ).map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
                    
                    self.sws_context = Some(ctx);
                    self.sws_context.as_mut().unwrap()
                }
            };
            
            // Prepare output frame
            unsafe {
                let dst_format = self.config.output_format.to_ffmpeg_format();
                ffmpeg::ffi::av_image_alloc(
                    output_frame.as_mut_ptr() as *mut *mut u8,
                    output_frame.linesize().as_mut_ptr() as *mut i32,
                    width as i32,
                    height as i32,
                    dst_format.into(),
                    1
                );
            }
            
            // Perform the conversion
            sws_ctx.run(&video_frame, &mut output_frame)
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
            
            // Get the converted frame data
            let output_video = output_frame.video()
                .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
            
            stride = output_video.stride(0) as u32;
            let buffer_size = stride as usize * height as usize;
            
            // Copy the data to our buffer
            buffer = vec![0u8; buffer_size];
            let src_data = output_video.data(0);
            buffer.copy_from_slice(unsafe { 
                std::slice::from_raw_parts(src_data, buffer_size) 
            });
        } else {
            // No conversion needed, use the frame directly
            stride = video_frame.stride(0) as u32;
            let buffer_size = stride as usize * height as usize;
            
            // Copy the data to our buffer
            buffer = vec![0u8; buffer_size];
            let src_data = video_frame.data(0);
            buffer.copy_from_slice(unsafe { 
                std::slice::from_raw_parts(src_data, buffer_size) 
            });
        }
        
        // Calculate frame duration
        let stream = format_ctx.stream(video_stream_index).unwrap();
        let frame_rate = match stream.avg_frame_rate() {
            (0, _) | (_, 0) => 30.0, // Default if not available
            (num, den) => num as f64 / den as f64,
        };
        let frame_duration = 1.0 / frame_rate;
        
        // Create and return the frame
        Ok(VideoFrame {
            width,
            height,
            format: self.config.output_format,
            buffer,
            stride,
            timestamp: self.current_position,
            duration: frame_duration,
            key_frame: decoded_frame.is_key(),
        })
    }
    
    /// Seek to a specific time position in the media
    pub fn seek(&mut self, time_sec: f64) -> Result<(), VideoDecoderError> {
        if !self.is_initialized {
            return Err(VideoDecoderError::InitializationError("Decoder not initialized".to_string()));
        }
        
        let media_info = self.media_info.as_ref().ok_or(VideoDecoderError::InitializationError(
            "No media info available".to_string()
        ))?;
        
        if time_sec < 0.0 || time_sec > media_info.duration {
            return Err(VideoDecoderError::DecodingError(
                format!("Seek time {} is outside media bounds (0 to {})", time_sec, media_info.duration)
            ));
        }
        
        // Lock the state for the seeking operation
        let mut state = self.state.lock().unwrap();
        state.is_seeking = true;
        drop(state); // Release the lock before FFmpeg operations
        
        // Get format context
        let format_ctx = self.format_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Format context not initialized".to_string()))?;
        
        // Convert time to stream timebase for the video stream
        let stream_index = self.current_video_stream as usize;
        let stream = format_ctx.stream(stream_index).unwrap();
        let time_base = stream.time_base();
        let timestamp = (time_sec * time_base.1 as f64 / time_base.0 as f64) as i64;
        
        // Perform the seek operation
        format_ctx.seek(timestamp, 0)
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
        
        // Flush codec buffers
        if let Some(video_ctx) = &mut self.video_codec_context {
            video_ctx.flush();
        }
        
        if let Some(audio_ctx) = &mut self.audio_codec_context {
            audio_ctx.flush();
        }
        
        // Update position
        self.current_position = time_sec;
        
        // Update state
        let mut state = self.state.lock().unwrap();
        state.is_seeking = false;
        
        Ok(())
    }
    
    /// Get information about the current media file
    pub fn get_media_info(&self) -> Option<&MediaInfo> {
        self.media_info.as_ref()
    }
    
    /// Select a specific video stream
    pub fn select_video_stream(&mut self, stream_index: i32) -> Result<(), VideoDecoderError> {
        if !self.is_initialized {
            return Err(VideoDecoderError::InitializationError("Decoder not initialized".to_string()));
        }
        
        let media_info = self.media_info.as_ref().ok_or(VideoDecoderError::InitializationError(
            "No media info available".to_string()
        ))?;
        
        if stream_index < 0 || stream_index as usize >= media_info.video_streams.len() {
            return Err(VideoDecoderError::DecodingError(
                format!("Invalid video stream index: {}", stream_index)
            ));
        }
        
        // If we're already using this stream, do nothing
        if self.current_video_stream == stream_index {
            return Ok(());
        }
        
        // Get format context
        let format_ctx = self.format_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Format context not initialized".to_string()))?;
        
        // Close the current video codec context if open
        self.video_codec_context = None;
        self.sws_context = None;
        
        // Get the stream
        let stream = format_ctx.stream(stream_index as usize).unwrap();
        let codec_params = stream.codec().parameters();
        
        // Find decoder for the stream
        let decoder_id = codec_params.id();
        let decoder = ffmpeg::codec::decoder::find(decoder_id)
            .ok_or_else(|| VideoDecoderError::DecodingError(
                format!("Failed to find decoder for codec id: {:?}", decoder_id)
            ))?;
        
        // Create a codec context for the decoder
        let mut codec_ctx = ffmpeg::codec::context::Context::new();
        codec_ctx.set_parameters(codec_params)
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
        
        // Open the decoder
        let video_ctx = codec_ctx.decoder().open(decoder)
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
        
        self.video_codec_context = Some(video_ctx);
        self.current_video_stream = stream_index;
        
        // Set up scaling context if needed
        if let Some(video_ctx) = &self.video_codec_context {
            let video_ctx = video_ctx.decoder().video().unwrap();
            let src_format = video_ctx.format();
            let dst_format = self.config.output_format.to_ffmpeg_format();
            
            if src_format != dst_format {
                let width = video_ctx.width();
                let height = video_ctx.height();
                
                let sws_ctx = SwsContext::get(
                    width, height, src_format,
                    width, height, dst_format,
                    Flags::BILINEAR,
                ).map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
                
                self.sws_context = Some(sws_ctx);
            }
        }
        
        Ok(())
    }
    
    /// Select a specific audio stream
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
        
        // If we're already using this stream, do nothing
        if self.current_audio_stream == stream_index {
            return Ok(());
        }
        
        // Get format context
        let format_ctx = self.format_context.as_mut()
            .ok_or_else(|| VideoDecoderError::DecodingError("Format context not initialized".to_string()))?;
        
        // Close the current audio codec context if open
        self.audio_codec_context = None;
        
        // Get the stream
        let stream = format_ctx.stream(stream_index as usize).unwrap();
        let codec_params = stream.codec().parameters();
        
        // Find decoder for the stream
        let decoder_id = codec_params.id();
        let decoder = ffmpeg::codec::decoder::find(decoder_id)
            .ok_or_else(|| VideoDecoderError::DecodingError(
                format!("Failed to find decoder for codec id: {:?}", decoder_id)
            ))?;
        
        // Create a codec context for the decoder
        let mut codec_ctx = ffmpeg::codec::context::Context::new();
        codec_ctx.set_parameters(codec_params)
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
        
        // Open the decoder
        let audio_ctx = codec_ctx.decoder().open(decoder)
            .map_err(|e| VideoDecoderError::FFmpegLibError(e))?;
        
        self.audio_codec_context = Some(audio_ctx);
        self.current_audio_stream = stream_index;
        
        Ok(())
    }
    
    /// Get the current playback position
    pub fn get_position(&self) -> f64 {
        self.current_position
    }
    
    /// Close the decoder and release resources
    pub fn close(&mut self) -> Result<(), VideoDecoderError> {
        if !self.is_initialized {
            return Ok(());
        }
        
        // Wait for any ongoing operations to complete
        {
            let mut state = self.state.lock().unwrap();
            while state.is_decoding || state.is_seeking {
                // In a real implementation, we would use a condition variable
                // For now, just set the flags to false
                state.is_decoding = false;
                state.is_seeking = false;
            }
        }
        
        // Clean up resources in reverse order of creation
        
        // Free scaling context
        self.sws_context = None;
        
        // Close codec contexts
        self.video_codec_context = None;
        self.audio_codec_context = None;
        
        // Close format context (this will also close associated streams)
        self.format_context = None;
        
        // Reset state
        self.is_initialized = false;
        self.current_position = 0.0;
        self.current_video_stream = -1;
        self.current_audio_stream = -1;
        self.media_info = None;
        
        // Reset internal state
        let mut state = self.state.lock().unwrap();
        state.last_decoded_frame_pts = 0;
        state.error_count = 0;
        
        Ok(())
    }
    
    /// Get information about the current media file
    pub fn get_media_info(&self) -> Option<&MediaInfo> {
        self.media_info.as_ref()
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        // Ensure resources are cleaned up when the decoder is dropped
        let _ = self.close();
    }
}

/// Factory function to create a video decoder with default configuration
pub fn create_default_decoder() -> VideoDecoder {
    VideoDecoder::new(VideoDecoderConfig::default())
}

/// Utility function to get information about a media file without fully opening it
pub fn get_media_info<P: AsRef<Path>>(path: P) -> Result<MediaInfo, VideoDecoderError> {
    let mut decoder = create_default_decoder();
    let info = decoder.open(path)?;
    Ok(info.clone())
}
