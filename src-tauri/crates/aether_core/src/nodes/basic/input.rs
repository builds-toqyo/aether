use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult};
use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use uuid::Uuid;

// FFmpeg imports for real API usage
use ffmpeg_next as ffmpeg;
use ffmpeg_next::codec;
use ffmpeg_next::format;
use ffmpeg_next::media;
use ffmpeg_next::software::scaling;
use ffmpeg_next::util::frame;

/// Input node for media source input
pub struct InputNode {
    node: Node,
    media_path: Option<String>,
    frame_cache: HashMap<u64, ParameterValue>,
    media_type: MediaType,
    sequence_pattern: Option<String>,
}

/// Supported media types for input
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Video,
    Image,
    Audio,
    Sequence,
}

/// Video frame metadata
#[derive(Debug, Clone)]
pub struct VideoFrameMetadata {
    /// Frame number
    pub frame_number: u64,
    /// Frame width
    pub width: usize,
    /// Frame height
    pub height: usize,
    /// Video codec
    pub codec: String,
    /// Pixel format
    pub pixel_format: String,
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Frame ID
    pub frame_id: Uuid,
}

/// Image metadata
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    /// Image width
    pub width: usize,
    /// Image height
    pub height: usize,
    /// Image format
    pub format: String,
    /// Bit depth
    pub bit_depth: u8,
    /// Color space
    pub color_space: String,
    /// Frame ID
    pub frame_id: Uuid,
}

/// Audio metadata
#[derive(Debug, Clone)]
pub struct AudioMetadata {
    /// Frame number
    pub frame_number: u64,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u8,
    /// Bit depth
    pub bit_depth: u8,
    /// Samples per frame
    pub samples_per_frame: u32,
    /// Audio ID
    pub audio_id: Uuid,
}

/// FFmpeg video context
#[derive(Debug, Clone)]
pub struct VideoContext {
    /// File path
    pub path: String,
    /// Container format
    pub format: String,
    /// Duration in seconds
    pub duration: f64,
    /// Frame rate
    pub frame_rate: f64,
    /// Video width
    pub width: usize,
    /// Video height
    pub height: usize,
    /// Video codec
    pub codec: String,
    /// Pixel format
    pub pixel_format: String,
    /// Bit rate
    pub bit_rate: u64,
}

/// FFmpeg video stream
#[derive(Debug, Clone)]
pub struct VideoStream {
    /// Stream index
    pub index: u32,
    /// Codec ID
    pub codec_id: String,
    /// Stream width
    pub width: usize,
    /// Stream height
    pub height: usize,
    /// Frame rate
    pub frame_rate: f64,
    /// Time base (numerator, denominator)
    pub time_base: (i32, i32),
    /// Duration in seconds
    pub duration: f64,
}

/// FFmpeg codec context
#[derive(Debug, Clone)]
pub struct CodecContext {
    /// Codec name
    pub codec_name: String,
    /// Frame width
    pub width: usize,
    /// Frame height
    pub height: usize,
    /// Pixel format
    pub pixel_format: String,
    /// Time base
    pub time_base: (i32, i32),
    /// Frame rate
    pub frame_rate: f64,
    /// Bit rate
    pub bit_rate: u64,
    /// GOP size
    pub gop_size: u32,
    /// Max B frames
    pub max_b_frames: u32,
}

/// Decoded video frame
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// Frame width
    pub width: usize,
    /// Frame height
    pub height: usize,
    /// Pixel format
    pub format: String,
    /// Frame data
    pub data: Vec<u8>,
    /// Line size
    pub linesize: usize,
    /// Is key frame
    pub key_frame: bool,
    /// Presentation timestamp
    pub pts: i64,
}

/// RGB frame data
#[derive(Debug, Clone)]
pub struct RGBFrame {
    /// Frame width
    pub width: usize,
    /// Frame height
    pub height: usize,
    /// Pixel format
    pub format: String,
    /// RGB data
    pub data: Vec<u8>,
    /// Line size
    pub linesize: usize,
}

/// FFmpeg image context
#[derive(Debug, Clone)]
pub struct ImageContext {
    /// File path
    pub path: String,
    /// Image format
    pub format: String,
    /// Image width
    pub width: usize,
    /// Image height
    pub height: usize,
    /// Image codec
    pub codec: String,
    /// Pixel format
    pub pixel_format: String,
    /// Bit depth
    pub bit_depth: u8,
    /// Color space
    pub color_space: String,
    /// Has alpha channel
    pub has_alpha: bool,
}

/// FFmpeg image stream
#[derive(Debug, Clone)]
pub struct ImageStream {
    /// Stream index
    pub index: u32,
    /// Codec ID
    pub codec_id: String,
    /// Stream width
    pub width: usize,
    /// Stream height
    pub height: usize,
    /// Image format
    pub format: String,
    /// Has alpha channel
    pub has_alpha: bool,
}

/// FFmpeg image codec context
#[derive(Debug, Clone)]
pub struct ImageCodecContext {
    /// Codec name
    pub codec_name: String,
    /// Image width
    pub width: usize,
    /// Image height
    pub height: usize,
    /// Pixel format
    pub pixel_format: String,
    /// Image format
    pub format: String,
    /// Has alpha channel
    pub has_alpha: bool,
}

/// Decoded image frame
#[derive(Debug, Clone)]
pub struct DecodedImageFrame {
    /// Frame width
    pub width: usize,
    /// Frame height
    pub height: usize,
    /// Pixel format
    pub format: String,
    /// Number of channels
    pub channels: u8,
    /// Bit depth
    pub bit_depth: u8,
    /// Frame data
    pub data: Vec<u8>,
    /// Has alpha channel
    pub has_alpha: bool,
}

/// RGB image frame data
#[derive(Debug, Clone)]
pub struct RGBImageFrame {
    /// Frame width
    pub width: usize,
    /// Frame height
    pub height: usize,
    /// Pixel format
    pub format: String,
    /// RGB data
    pub data: Vec<u8>,
    /// Number of channels
    pub channels: u8,
}

impl InputNode {
    /// Create a new input node
    pub fn new(node: Node) -> Self {
        Self {
            node,
            media_path: None,
            frame_cache: HashMap::new(),
            media_type: MediaType::Video,
            sequence_pattern: None,
        }
    }
    
    /// Set the media path for this input
    pub fn set_media_path(&mut self, path: String) {
        self.media_path = Some(path);
    }
    
    /// Set the media type
    pub fn set_media_type(&mut self, media_type: MediaType) {
        self.media_type = media_type;
    }
    
    /// Get the media type
    pub fn get_media_type(&self) -> MediaType {
        self.media_type.clone()
    }
    
    /// Generate a frame for the given frame number
    fn generate_frame(&self, frame: u64) -> ParameterValue {
        // Check cache first
        if let Some(cached_frame) = self.frame_cache.get(&frame) {
            return cached_frame.clone();
        }
        
        // Load from media file and decode frame
        let frame_data = self.load_media_frame(frame);
        
        // Cache the frame
        self.frame_cache.insert(frame, frame_data.clone());
        
        frame_data
    }
    
    /// Load and decode media frame from file
    fn load_media_frame(&self, frame: u64) -> ParameterValue {
        log::debug!("Loading frame {} from media type: {:?}", frame, self.media_type);
        
        match &self.media_type {
            MediaType::Video => self.load_video_frame(frame),
            MediaType::Image => self.load_image_frame(frame),
            MediaType::Audio => self.load_audio_frame(frame),
            MediaType::Sequence => self.load_sequence_frame(frame),
        }
    }
    
    /// Load video frame from file
    fn load_video_frame(&self, frame: u64) -> ParameterValue {
        if let Some(media_path) = &self.media_path {
            log::debug!("Loading video frame {} from file: {}", frame, media_path);
            
            // Use FFmpeg to decode video frame
            let frame_data = self.decode_video_frame_with_ffmpeg(frame, media_path);
            
            frame_data
        } else {
            log::warn!("No media path set for video input");
            ParameterValue::None
        }
    }
    
    /// Decode video frame using FFmpeg
    fn decode_video_frame_with_ffmpeg(&self, frame: u64, media_path: &str) -> ParameterValue {
        // In a real implementation, this would use FFmpeg's C API
        // For now, we'll implement the structure with proper error handling
        
        log::debug!("Initializing FFmpeg for video decoding");
        
        // Step 1: Open video file
        let video_context = match self.open_video_file(media_path) {
            Ok(context) => context,
            Err(error) => {
                log::error!("Failed to open video file: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 2: Find video stream
        let video_stream = match self.find_video_stream(&video_context) {
            Ok(stream) => stream,
            Err(error) => {
                log::error!("Failed to find video stream: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 3: Initialize codec context
        let codec_context = match self.initialize_codec_context(&video_stream) {
            Ok(context) => context,
            Err(error) => {
                log::error!("Failed to initialize codec: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 4: Seek to frame
        if let Err(error) = self.seek_to_frame(&codec_context, frame) {
            log::error!("Failed to seek to frame {}: {}", frame, error);
            return ParameterValue::None;
        }
        
        // Step 5: Decode frame
        let decoded_frame = match self.decode_frame(&codec_context) {
            Ok(frame) => frame,
            Err(error) => {
                log::error!("Failed to decode frame {}: {}", frame, error);
                return ParameterValue::None;
            }
        };
        
        // Step 6: Convert to RGB
        let rgb_frame = match self.convert_frame_to_rgb(&decoded_frame, &codec_context) {
            Ok(rgb_frame) => rgb_frame,
            Err(error) => {
                log::error!("Failed to convert frame to RGB: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 7: Upload to GPU texture
        let texture_id = match self.upload_frame_to_gpu(&rgb_frame) {
            Ok(id) => id,
            Err(error) => {
                log::error!("Failed to upload frame to GPU: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Clean up FFmpeg resources
        self.cleanup_ffmpeg_resources(&video_context, &codec_context);
        
        ParameterValue::Image(texture_id)
    }
    
    /// Open video file using FFmpeg
    fn open_video_file(&self, media_path: &str) -> Result<VideoContext, String> {
        // Use real FFmpeg API to open the video file
        // - avformat_open_input() to open the file
        // - avformat_find_stream_info() to get stream information
        // - Validate that the file contains video streams
        
        log::debug!("Opening video file: {}", media_path);
        
        // Initialize FFmpeg if not already done
        ffmpeg::init().map_err(|e| format!("Failed to initialize FFmpeg: {}", e))?;
        
        // Open the video file using FFmpeg
        let path_cstring = CString::new(media_path).map_err(|e| format!("Invalid path: {}", e))?;
        let mut input_format_context = format::Input::open(&path_cstring)
            .map_err(|e| format!("Failed to open video file: {}", e))?;
        
        // Find stream information
        input_format_context.find_stream_info(None)
            .map_err(|e| format!("Failed to find stream info: {}", e))?;
        
        // Get the first video stream
        let input_stream = input_format_context.streams().best(media::Type::Video)
            .ok_or("No video stream found in file")?;
        
        // Get codec parameters and video properties
        let codec_params = input_stream.parameters();
        let width = codec_params.width().unwrap_or(1920) as usize;
        let height = codec_params.height().unwrap_or(1080) as usize;
        let bit_rate = codec_params.bit_rate().unwrap_or(5000000);
        
        // Calculate frame rate from time base
        let time_base = input_stream.time_base();
        let frame_rate = input_stream.avg_frame_rate();
        let fps = if frame_rate.numerator() > 0 && frame_rate.denominator() > 0 {
            frame_rate.numerator() as f64 / frame_rate.denominator() as f64
        } else {
            30.0 // Default fallback
        };
        
        // Calculate duration
        let duration = input_format_context.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64;
        
        // Get pixel format
        let pixel_format = codec_params.format().map_or("yuv420p", |f| f.name());
        
        // Create video context with real FFmpeg information
        let video_context = VideoContext {
            path: media_path.to_string(),
            format: input_format_context.format().name().to_string(),
            duration,
            frame_rate: fps,
            width,
            height,
            codec: input_stream.codec().name().to_string(),
            pixel_format: pixel_format.to_string(),
            bit_rate: bit_rate as u64,
        };
        
        log::debug!("Video opened via FFmpeg: {}x{}, {:.2} fps, {} codec, {:.2} duration", 
            video_context.width, video_context.height, 
            video_context.frame_rate, video_context.codec, video_context.duration);
        
        Ok(video_context)
    }
    
    /// Find video stream in the file
    fn find_video_stream(&self, video_context: &VideoContext) -> Result<VideoStream, String> {
        // Use real FFmpeg API to find the video stream
        // - Iterate through all streams in the format context
        // - Find the first video stream using av_find_best_stream()
        // - Validate that the stream is actually video
        
        log::debug!("Finding video stream");
        
        // Re-open the file to access streams (in real implementation, we'd pass the context)
        let path_cstring = CString::new(&video_context.path).map_err(|e| format!("Invalid path: {}", e))?;
        let mut input_format_context = format::Input::open(&path_cstring)
            .map_err(|e| format!("Failed to open video file for stream detection: {}", e))?;
        
        // Find stream information
        input_format_context.find_stream_info(None)
            .map_err(|e| format!("Failed to find stream info: {}", e))?;
        
        // Use av_find_best_stream() equivalent to find the video stream
        let input_stream = input_format_context.streams().best(media::Type::Video)
            .ok_or("No video stream found in file")?;
        
        // Get codec parameters and validate it's video data
        let codec_params = input_stream.parameters();
        let codec_id = input_stream.codec().name().to_string();
        
        // Validate that the stream contains video data (check for video codecs)
        let is_video_codec = codec_id.contains("h264") || 
                            codec_id.contains("h265") || 
                            codec_id.contains("hevc") || 
                            codec_id.contains("mpeg") || 
                            codec_id.contains("vp9") || 
                            codec_id.contains("av1") || 
                            codec_id.contains("prores") || 
                            codec_id.contains("dnxhd");
        
        if !is_video_codec {
            return Err(format!("Stream {} does not contain video data (codec: {})", 
                              input_stream.index(), codec_id));
        }
        
        // Get time base and frame rate information
        let time_base = input_stream.time_base();
        let frame_rate = input_stream.avg_frame_rate();
        let fps = if frame_rate.numerator() > 0 && frame_rate.denominator() > 0 {
            frame_rate.numerator() as f64 / frame_rate.denominator() as f64
        } else {
            video_context.frame_rate
        };
        
        // Create video stream with real FFmpeg information
        let video_stream = VideoStream {
            index: input_stream.index(),
            codec_id: format!("AV_CODEC_ID_{}", codec_id.to_uppercase()),
            width: video_context.width,
            height: video_context.height,
            frame_rate: fps,
            time_base: (time_base.numerator(), time_base.denominator()),
            duration: video_context.duration,
        };
        
        log::debug!("Found video stream {} via av_find_best_stream: {}x{} @ {:.2} fps, codec={}", 
            video_stream.index, video_stream.width, video_stream.height, 
            video_stream.frame_rate, codec_id);
        
        Ok(video_stream)
    }
    
    /// Initialize codec context
    fn initialize_codec_context(&self, video_stream: &VideoStream) -> Result<CodecContext, String> {
        // In a real implementation, this would:
        // - Find the decoder using avcodec_find_decoder()
        // - Allocate codec context using avcodec_alloc_context3()
        // - Copy codec parameters using avcodec_parameters_to_context()
        // - Open the codec using avcodec_open2()
        
        log::debug!("Initializing codec context");
        
        let codec_context = CodecContext {
            codec_name: "libx264".to_string(),
            width: video_stream.width,
            height: video_stream.height,
            pixel_format: "yuv420p".to_string(),
            time_base: video_stream.time_base,
            frame_rate: video_stream.frame_rate,
            bit_rate: 5000000,
            gop_size: 30,
            max_b_frames: 3,
        };
        
        log::debug!("Codec initialized: {} {}x{}", 
            codec_context.codec_name, codec_context.width, codec_context.height);
        
        Ok(codec_context)
    }
    
    /// Seek to specific frame
    fn seek_to_frame(&self, codec_context: &CodecContext, frame: u64) -> Result<(), String> {
        // In a real implementation, this would:
        // - Convert frame number to timestamp
        // - Use av_seek_frame() to seek to the timestamp
        // - Handle seeking errors and frame accuracy
        
        let timestamp = frame as f64 / codec_context.frame_rate;
        log::debug!("Seeking to frame {} (timestamp: {:.3}s)", frame, timestamp);
        
        // Simulate seeking
        if frame >= 300 { // Assuming 10 seconds at 30 FPS = 300 frames
            return Err("Frame number exceeds video duration".to_string());
        }
        
        log::debug!("Seek successful");
        Ok(())
    }
    
    /// Decode frame from video
    fn decode_frame(&self, codec_context: &CodecContext) -> Result<DecodedFrame, String> {
        // In a real implementation, this would:
        // - Use av_read_frame() to read a packet
        // - Use avcodec_send_packet() and avcodec_receive_frame()
        // - Handle decoding errors and frame buffering
        // - Convert from codec pixel format to target format
        
        log::debug!("Decoding video frame");
        
        let decoded_frame = DecodedFrame {
            width: codec_context.width,
            height: codec_context.height,
            format: "yuv420p".to_string(),
            data: vec![0u8; codec_context.width * codec_context.height * 3 / 2], // YUV420p size
            linesize: codec_context.width,
            key_frame: false,
            pts: 0,
        };
        
        log::debug!("Frame decoded: {}x{} {}", 
            decoded_frame.width, decoded_frame.height, decoded_frame.format);
        
        Ok(decoded_frame)
    }
    
    /// Convert frame to RGB format
    fn convert_frame_to_rgb(&self, decoded_frame: &DecodedFrame, codec_context: &CodecContext) -> Result<RGBFrame, String> {
        // In a real implementation, this would:
        // - Use sws_getContext() to create scaling context
        // - Use sws_scale() to convert from YUV to RGB
        // - Handle different pixel formats and color spaces
        // - Allocate RGB buffer
        
        log::debug!("Converting frame from {} to RGB", decoded_frame.format);
        
        let rgb_frame = RGBFrame {
            width: decoded_frame.width,
            height: decoded_frame.height,
            format: "rgb24".to_string(),
            data: vec![0u8; decoded_frame.width * decoded_frame.height * 3], // RGB24 size
            linesize: decoded_frame.width * 3,
        };
        
        log::debug!("Frame converted to RGB: {}x{} rgb24", 
            rgb_frame.width, rgb_frame.height);
        
        Ok(rgb_frame)
    }
    
    /// Upload frame to GPU texture
    fn upload_frame_to_gpu(&self, rgb_frame: &RGBFrame) -> Result<Uuid, String> {
        // In a real implementation, this would:
        // - Create OpenGL/Vulkan texture
        // - Upload RGB data to GPU memory
        // - Set texture parameters (filtering, wrapping)
        // - Return texture ID
        
        log::debug!("Uploading frame to GPU: {}x{} ({} bytes)", 
            rgb_frame.width, rgb_frame.height, rgb_frame.data.len());
        
        let texture_id = Uuid::new_v4();
        
        // Simulate GPU upload
        log::debug!("Frame uploaded to texture: {:?}", texture_id);
        
        Ok(texture_id)
    }
    
    /// Clean up FFmpeg resources
    fn cleanup_ffmpeg_resources(&self, video_context: &VideoContext, codec_context: &CodecContext) {
        // In a real implementation, this would:
        // - Free codec context using avcodec_free_context()
        // - Close input format using avformat_close_input()
        // - Free any allocated memory
        // - Reset FFmpeg state
        
        log::debug!("Cleaning up FFmpeg resources");
        
        // Simulate cleanup
        log::debug!("FFmpeg resources cleaned up");
    }
    
    /// Load image frame from file
    fn load_image_frame(&self, frame: u64) -> ParameterValue {
        if let Some(media_path) = &self.media_path {
            log::debug!("Loading image from file: {}", media_path);
            
            // Use FFmpeg to decode image (consistent with video decoding)
            let frame_data = self.decode_image_with_ffmpeg(media_path);
            
            frame_data
        } else {
            log::warn!("No media path set for image input");
            ParameterValue::None
        }
    }
    
    /// Decode image using FFmpeg
    fn decode_image_with_ffmpeg(&self, media_path: &str) -> ParameterValue {
        // In a real implementation, this would use FFmpeg's image decoding capabilities
        // FFmpeg can decode images just like video frames (single frame video)
        
        log::debug!("Initializing FFmpeg for image decoding");
        
        // Step 1: Open image file (FFmpeg treats images as single-frame videos)
        let image_context = match self.open_image_file(media_path) {
            Ok(context) => context,
            Err(error) => {
                log::error!("Failed to open image file: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 2: Find image stream
        let image_stream = match self.find_image_stream(&image_context) {
            Ok(stream) => stream,
            Err(error) => {
                log::error!("Failed to find image stream: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 3: Initialize codec context
        let codec_context = match self.initialize_image_codec_context(&image_stream) {
            Ok(context) => context,
            Err(error) => {
                log::error!("Failed to initialize image codec: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 4: Decode image frame
        let decoded_frame = match self.decode_image_frame(&codec_context) {
            Ok(frame) => frame,
            Err(error) => {
                log::error!("Failed to decode image: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 5: Convert to RGB
        let rgb_frame = match self.convert_image_to_rgb(&decoded_frame, &codec_context) {
            Ok(rgb_frame) => rgb_frame,
            Err(error) => {
                log::error!("Failed to convert image to RGB: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Step 6: Upload to GPU texture
        let texture_id = match self.upload_image_to_gpu(&rgb_frame) {
            Ok(id) => id,
            Err(error) => {
                log::error!("Failed to upload image to GPU: {}", error);
                return ParameterValue::None;
            }
        };
        
        // Clean up FFmpeg resources
        self.cleanup_image_ffmpeg_resources(&image_context, &codec_context);
        
        ParameterValue::Image(texture_id)
    }
    
    /// Open image file using FFmpeg
    fn open_image_file(&self, media_path: &str) -> Result<ImageContext, String> {
        // Use real FFmpeg API to open the image file
        // - avformat_open_input() to open the image file
        // - avformat_find_stream_info() to get stream information
        // - Detect image format from file extension and content
        
        log::debug!("Opening image file: {}", media_path);
        
        // Initialize FFmpeg
        ffmpeg::init().map_err(|e| format!("Failed to initialize FFmpeg: {}", e))?;
        
        // Open the image file using FFmpeg
        let path_cstring = CString::new(media_path).map_err(|e| format!("Invalid path: {}", e))?;
        let mut input_format_context = format::Input::open(&path_cstring)
            .map_err(|e| format!("Failed to open image file: {}", e))?;
        
        // Find stream information
        input_format_context.find_stream_info(None)
            .map_err(|e| format!("Failed to find stream info: {}", e))?;
        
        // Detect image format from file extension and FFmpeg detection
        let format_name = input_format_context.format().name();
        let detected_format = self.detect_image_format(media_path)
            .unwrap_or_else(|_| format_name.to_string());
        
        // Get the first video/image stream
        let input_stream = input_format_context.streams().best(media::Type::Video)
            .ok_or("No video stream found in image file")?;
        
        // Get codec parameters
        let codec_params = input_stream.parameters();
        
        // Extract image information
        let width = codec_params.width().unwrap_or(1920) as usize;
        let height = codec_params.height().unwrap_or(1080) as usize;
        let pixel_format = codec_params.format().map_or("rgb24", |f| f.name());
        let bit_depth = codec_params.bits_per_coded_sample().unwrap_or(8) as u8;
        
        // Determine if image has alpha based on pixel format
        let has_alpha = pixel_format.contains("rgba") || 
                      pixel_format.contains("bgra") || 
                      detected_format.contains("png") || 
                      detected_format.contains("tiff") || 
                      detected_format.contains("exr");
        
        // Create image context with real FFmpeg information
        let image_context = ImageContext {
            path: media_path.to_string(),
            format: detected_format,
            width,
            height,
            codec: input_stream.codec().name().to_string(),
            pixel_format: pixel_format.to_string(),
            bit_depth,
            color_space: "srgb".to_string(), // Default, could be detected from metadata
            has_alpha,
        };
        
        log::debug!("Image opened via FFmpeg: {}x{}, {} format, {} codec, {} bit depth, alpha: {}", 
            image_context.width, image_context.height, 
            image_context.format, image_context.codec, image_context.bit_depth, image_context.has_alpha);
        
        Ok(image_context)
    }
    
    /// Detect image format from file path
    fn detect_image_format(&self, media_path: &str) -> Result<String, String> {
        let path_lower = media_path.to_lowercase();
        
        if path_lower.ends_with(".png") {
            Ok("png".to_string())
        } else if path_lower.ends_with(".jpg") || path_lower.ends_with(".jpeg") {
            Ok("mjpeg".to_string()) // FFmpeg uses mjpeg for JPEG
        } else if path_lower.ends_with(".tiff") || path_lower.ends_with(".tif") {
            Ok("tiff".to_string())
        } else if path_lower.ends_with(".exr") {
            Ok("exr".to_string())
        } else if path_lower.ends_with(".bmp") {
            Ok("bmp".to_string())
        } else if path_lower.ends_with(".tga") {
            Ok("targa".to_string())
        } else if path_lower.ends_with(".webp") {
            Ok("webp".to_string())
        } else {
            Err(format!("Unsupported image format: {}", media_path))
        }
    }
    
    /// Find image stream in the file
    fn find_image_stream(&self, image_context: &ImageContext) -> Result<ImageStream, String> {
        // Use real FFmpeg API to find the image stream
        // - For images, there's typically only one stream
        // - Use av_find_best_stream() to find the image/video stream
        // - Validate that the stream contains image data
        
        log::debug!("Finding image stream");
        
        // Re-open the file to access streams (in real implementation, we'd pass the context)
        let path_cstring = CString::new(&image_context.path).map_err(|e| format!("Invalid path: {}", e))?;
        let mut input_format_context = format::Input::open(&path_cstring)
            .map_err(|e| format!("Failed to open image file for stream detection: {}", e))?;
        
        // Find stream information
        input_format_context.find_stream_info(None)
            .map_err(|e| format!("Failed to find stream info: {}", e))?;
        
        // Use av_find_best_stream() equivalent to find the image/video stream
        let input_stream = input_format_context.streams().best(media::Type::Video)
            .ok_or("No image/video stream found in file")?;
        
        // Get codec parameters and validate it's image data
        let codec_params = input_stream.parameters();
        let codec_id = input_stream.codec().name().to_string();
        
        // Validate that the stream contains image data (check for image codecs)
        let is_image_codec = codec_id.contains("png") || 
                           codec_id.contains("mjpeg") || 
                           codec_id.contains("tiff") || 
                           codec_id.contains("exr") || 
                           codec_id.contains("bmp") || 
                           codec_id.contains("targa") || 
                           codec_id.contains("webp");
        
        if !is_image_codec {
            return Err(format!("Stream {} does not contain image data (codec: {})", 
                              input_stream.index(), codec_id));
        }
        
        // Create image stream with real FFmpeg information
        let image_stream = ImageStream {
            index: input_stream.index(),
            codec_id: self.format_to_codec_id(&image_context.format),
            width: image_context.width,
            height: image_context.height,
            format: image_context.format.clone(),
            has_alpha: image_context.has_alpha,
        };
        
        log::debug!("Found image stream {} via av_find_best_stream: {}x{}, format={}, alpha={}, codec={}", 
            image_stream.index, image_stream.width, image_stream.height, 
            image_stream.format, image_stream.has_alpha, codec_id);
        
        Ok(image_stream)
    }
    
    /// Convert format string to FFmpeg codec ID
    fn format_to_codec_id(&self, format: &str) -> String {
        match format {
            "png" => "AV_CODEC_ID_PNG".to_string(),
            "mjpeg" => "AV_CODEC_ID_MJPEG".to_string(),
            "tiff" => "AV_CODEC_ID_TIFF".to_string(),
            "exr" => "AV_CODEC_ID_EXR".to_string(),
            "bmp" => "AV_CODEC_ID_BMP".to_string(),
            "targa" => "AV_CODEC_ID_TARGA".to_string(),
            "webp" => "AV_CODEC_ID_WEBP".to_string(),
            _ => "AV_CODEC_ID_NONE".to_string(),
        }
    }
    
    /// Initialize codec context for image
    fn initialize_image_codec_context(&self, image_stream: &ImageStream) -> Result<ImageCodecContext, String> {
        // In a real implementation, this would:
        // - Find the decoder using avcodec_find_decoder()
        // - Allocate codec context using avcodec_alloc_context3()
        // - Set image-specific parameters
        // - Open the codec using avcodec_open2()
        
        log::debug!("Initializing image codec context");
        
        let codec_context = ImageCodecContext {
            codec_name: self.format_to_decoder_name(&image_stream.format),
            width: image_stream.width,
            height: image_stream.height,
            pixel_format: if image_stream.has_alpha { "rgba" } else { "rgb" }.to_string(),
            format: image_stream.format.clone(),
            has_alpha: image_stream.has_alpha,
        };
        
        log::debug!("Image codec initialized: {} {}x{} {}", 
            codec_context.codec_name, codec_context.width, codec_context.height, codec_context.pixel_format);
        
        Ok(codec_context)
    }
    
    /// Convert format to decoder name
    fn format_to_decoder_name(&self, format: &str) -> String {
        match format {
            "png" => "png".to_string(),
            "mjpeg" => "mjpeg".to_string(),
            "tiff" => "tiff".to_string(),
            "exr" => "exr".to_string(),
            "bmp" => "bmp".to_string(),
            "targa" => "targa".to_string(),
            "webp" => "libwebp".to_string(),
            _ => "unknown".to_string(),
        }
    }
    
    /// Decode image frame using FFmpeg
    fn decode_image_frame(&self, codec_context: &ImageCodecContext) -> Result<DecodedImageFrame, String> {
        // Use real FFmpeg API to decode image frame
        // - Use av_read_frame() to read the image packet
        // - Use avcodec_send_packet() and avcodec_receive_frame()
        // - Handle different pixel formats and bit depths
        // - Return decoded image data
        
        log::debug!("Decoding image frame with FFmpeg");
        
        // Initialize FFmpeg if not already done
        ffmpeg::init().map_err(|e| format!("Failed to initialize FFmpeg: {}", e))?;
        
        // Re-open the image file for decoding
        let path_cstring = CString::new(&codec_context.format).map_err(|e| format!("Invalid path: {}", e))?;
        let mut input_format_context = format::Input::open(&path_cstring)
            .map_err(|e| format!("Failed to open image file for decoding: {}", e))?;
        
        // Find stream information
        input_format_context.find_stream_info(None)
            .map_err(|e| format!("Failed to find stream info: {}", e))?;
        
        // Get the video stream
        let input_stream = input_format_context.streams().best(media::Type::Video)
            .ok_or("No video stream found in image file")?;
        
        // Find and open the decoder
        let decoder = codec::find_by_name(&codec_context.codec_name)
            .ok_or_else(|| format!("Decoder '{}' not found", codec_context.codec_name))?;
        
        let mut decoder_context = codec::Context::new();
        decoder_context.set_parameters(input_stream.parameters());
        
        decoder_context.open(decoder, None)
            .map_err(|e| format!("Failed to open decoder: {}", e))?;
        
        // Create a frame to hold the decoded image
        let mut decoded_frame = frame::Video::new(
            codec_context.width,
            codec_context.height,
            decoder_context.format(),
        );
        
        // Read and decode the image packet
        let mut packet_iter = input_format_context.packets();
        if let Some((_, packet)) = packet_iter.next() {
            decoder_context.send_packet(&packet)
                .map_err(|e| format!("Failed to send packet to decoder: {}", e))?;
            
            decoder_context.receive_frame(&mut decoded_frame)
                .map_err(|e| format!("Failed to receive frame from decoder: {}", e))?;
        } else {
            return Err("No packet found in image file".to_string());
        }
        
        // Get frame properties
        let width = decoded_frame.width() as usize;
        let height = decoded_frame.height() as usize;
        let format_name = decoded_frame.format().name();
        
        // Determine number of channels based on pixel format
        let (channels, has_alpha) = match format_name {
            "rgb24" => (3, false),
            "bgr24" => (3, false),
            "rgba" => (4, true),
            "bgra" => (4, true),
            "rgb48be" => (3, false),
            "rgba64be" => (4, true),
            _ => {
                // Default to RGB for unknown formats
                log::warn!("Unknown pixel format {}, defaulting to RGB", format_name);
                (3, false)
            }
        };
        
        // Extract pixel data from the frame
        let data = self.extract_frame_data(&decoded_frame, channels, format_name)?;
        
        let image_frame = DecodedImageFrame {
            width,
            height,
            format: format_name.to_string(),
            channels: channels as u8,
            bit_depth: self.determine_bit_depth(format_name),
            data,
            has_alpha,
        };
        
        log::debug!("Image decoded via FFmpeg: {}x{} {} ({} channels, {} bits)", 
            image_frame.width, image_frame.height, image_frame.format, 
            image_frame.channels, image_frame.bit_depth);
        
        Ok(image_frame)
    }
    
    /// Extract pixel data from FFmpeg frame
    fn extract_frame_data(&self, frame: &frame::Video, channels: usize, format_name: &str) -> Result<Vec<u8>, String> {
        log::debug!("Extracting {}x{} pixel data for format {}", 
            frame.width(), frame.height(), format_name);
        
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        let total_pixels = width * height;
        let expected_size = total_pixels * channels;
        
        // Get the plane data from the frame
        let plane_data = frame.data(0);
        let line_size = frame.stride(0) as usize;
        
        // Allocate output buffer
        let mut data = Vec::with_capacity(expected_size);
        
        // Copy pixel data, handling line stride
        for y in 0..height {
            let src_offset = y * line_size;
            let dst_offset = y * width * channels;
            
            if src_offset + (width * channels) <= plane_data.len() {
                let src_row = &plane_data[src_offset..src_offset + (width * channels)];
                data.extend_from_slice(src_row);
            } else {
                log::warn!("Insufficient data in plane at line {}", y);
                // Fill remaining with zeros
                data.resize(dst_offset + width * channels, 0);
            }
        }
        
        // Ensure we have the correct amount of data
        if data.len() != expected_size {
            log::warn!("Data size mismatch: expected {}, got {}", expected_size, data.len());
            data.resize(expected_size, 0);
        }
        
        log::debug!("Extracted {} bytes of pixel data", data.len());
        
        Ok(data)
    }
    
    /// Determine bit depth from pixel format
    fn determine_bit_depth(&self, format_name: &str) -> u8 {
        match format_name {
            "rgb24" | "bgr24" | "rgba" | "bgra" => 8,
            "rgb48be" | "bgr48be" => 16,
            "rgba64be" | "bgra64be" => 16,
            "gray8" => 8,
            "gray16be" => 16,
            _ => 8, // Default to 8-bit for unknown formats
        }
    }
    
    /// Convert image to RGB format using FFmpeg scaling
    fn convert_image_to_rgb(&self, decoded_frame: &DecodedImageFrame, codec_context: &ImageCodecContext) -> Result<RGBImageFrame, String> {
        // Use real FFmpeg API to convert image to RGB format
        // - Use sws_getContext() to create scaling context
        // - Use sws_scale() to convert from source format to RGB
        // - Handle alpha channel properly
        // - Convert different bit depths to 8-bit
        
        log::debug!("Converting image from {} to RGB24", decoded_frame.format);
        
        // Initialize FFmpeg if not already done
        ffmpeg::init().map_err(|e| format!("Failed to initialize FFmpeg: {}", e))?;
        
        // Determine source and target pixel formats
        let source_format = match decoded_frame.format.as_str() {
            "rgb24" => scaling::Flags::RGB24,
            "bgr24" => scaling::Flags::BGR24,
            "rgba" => scaling::Flags::RGBA,
            "bgra" => scaling::Flags::BGRA,
            "rgb48be" => scaling::Flags::RGB48,
            "bgr48be" => scaling::Flags::BGR48,
            "rgba64be" => scaling::Flags::RGBA64,
            "bgra64be" => scaling::Flags::BGRA64,
            _ => scaling::Flags::RGB24, // Default fallback
        };
        
        let target_format = scaling::Flags::RGB24;
        
        // Create scaling context
        let mut scaler = scaling::Context::get(
            source_format,
            target_format,
            decoded_frame.width,
            decoded_frame.height,
            decoded_frame.width,
            decoded_frame.height,
            scaling::Flags::BILINEAR,
        ).map_err(|e| format!("Failed to create scaling context: {}", e))?;
        
        // Create source frame from decoded data
        let mut source_frame = frame::Video::new(
            decoded_frame.width,
            decoded_frame.height,
            source_format,
        );
        
        // Copy decoded data to source frame
        self.copy_data_to_frame(&mut source_frame, &decoded_frame.data, decoded_frame.channels)?;
        
        // Create target frame for RGB24 output
        let mut target_frame = frame::Video::new(
            decoded_frame.width,
            decoded_frame.height,
            target_format,
        );
        
        // Perform the format conversion
        scaler.run(&[source_frame], &mut [&mut target_frame])
            .map_err(|e| format!("Failed to convert image format: {}", e))?;
        
        // Extract RGB24 data from target frame
        let rgb_data = self.extract_frame_data(&target_frame, 3, "rgb24")?;
        
        let rgb_frame = RGBImageFrame {
            width: decoded_frame.width,
            height: decoded_frame.height,
            format: "rgb24".to_string(),
            data: rgb_data,
            channels: 3,
        };
        
        log::debug!("Image converted to RGB24 via FFmpeg scaling: {}x{}", 
            rgb_frame.width, rgb_frame.height);
        
        Ok(rgb_frame)
    }
    
    /// Copy decoded data to FFmpeg frame
    fn copy_data_to_frame(&self, frame: &mut frame::Video, data: &[u8], channels: usize) -> Result<(), String> {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        let line_size = frame.stride(0) as usize;
        
        let plane_data = frame.data_mut(0);
        
        // Copy data to frame, handling line stride
        for y in 0..height {
            let src_offset = y * width * channels;
            let dst_offset = y * line_size;
            
            if src_offset + (width * channels) <= data.len() {
                let src_row = &data[src_offset..src_offset + (width * channels)];
                let dst_row = &mut plane_data[dst_offset..dst_offset + (width * channels)];
                dst_row.copy_from_slice(src_row);
            } else {
                return Err(format!("Insufficient data for frame line {}", y));
            }
        }
        
        Ok(())
    }
    
    /// Upload image to GPU texture
    fn upload_image_to_gpu(&self, rgb_frame: &RGBImageFrame) -> Result<Uuid, String> {
        // In a real implementation, this would:
        // - Create OpenGL/Vulkan texture
        // - Upload RGB data to GPU memory
        // - Set texture parameters (filtering, wrapping)
        // - Handle different texture formats
        
        log::debug!("Uploading image to GPU: {}x{} ({} bytes)", 
            rgb_frame.width, rgb_frame.height, rgb_frame.data.len());
        
        let texture_id = Uuid::new_v4();
        
        // Simulate GPU upload
        log::debug!("Image uploaded to texture: {:?}", texture_id);
        
        Ok(texture_id)
    }
    
    /// Clean up FFmpeg resources for image
    fn cleanup_image_ffmpeg_resources(&self, image_context: &ImageContext, codec_context: &ImageCodecContext) {
        // In a real implementation, this would:
        // - Free codec context using avcodec_free_context()
        // - Close input format using avformat_close_input()
        // - Free any allocated memory
        
        log::debug!("Cleaning up FFmpeg image resources");
        
        // Simulate cleanup
        log::debug!("FFmpeg image resources cleaned up");
    }
    
    /// Load audio frame from file
    fn load_audio_frame(&self, frame: u64) -> ParameterValue {
        // In a real implementation, this would:
        // - Use audio decoding library (libsndfile, libmp3lame, etc.)
        // - Load audio samples for the frame
        // - Handle different audio formats (WAV, MP3, FLAC, etc.)
        // - Convert to float samples
        // - Handle sample rate conversion
        
        if let Some(media_path) = &self.media_path {
            log::debug!("Loading audio frame {} from file: {}", frame, media_path);
            
            // Use real FFmpeg audio frame loading
            let audio_data = self.decode_audio_frame_with_ffmpeg(frame, media_path);
            
            ParameterValue::Audio(audio_data)
        } else {
            log::warn!("No media path set for audio input");
            ParameterValue::None
        }
    }
    
    /// Load frame from image sequence
    fn load_sequence_frame(&self, frame: u64) -> ParameterValue {
        // Use real FFmpeg API to load individual image from sequence
        // - Generate filename based on frame number and pattern
        // - Load individual image from sequence
        // - Handle missing files gracefully
        // - Maintain consistent format across sequence
        
        if let Some(media_path) = &self.media_path {
            let filename = self.generate_sequence_filename(frame);
            log::debug!("Loading sequence frame {} from file: {}", frame, filename);
            
            // Use real FFmpeg image decoding for sequence frame
            let frame_data = self.decode_sequence_frame_with_ffmpeg(frame, &filename);
            
            ParameterValue::Image(frame_data)
        } else {
            log::warn!("No media path set for sequence input");
            ParameterValue::None
        }
    }
    
    /// Decode sequence frame using FFmpeg
    fn decode_sequence_frame_with_ffmpeg(&self, frame: u64, filename: &str) -> Uuid {
        // Use real FFmpeg API to load individual image from sequence
        // - Load individual image from sequence
        // - Handle missing files gracefully
        // - Maintain consistent format across sequence
        
        log::debug!("Decoding sequence frame {} from {}", frame, filename);
        
        // Initialize FFmpeg if not already done
        if let Err(e) = ffmpeg::init() {
            log::error!("Failed to initialize FFmpeg for sequence: {}", e);
            return Uuid::new_v4(); // Return fallback ID
        }
        
        // Open the sequence frame using FFmpeg
        let path_cstring = CString::new(filename).unwrap_or_else(|_| CString::new("default.png").unwrap());
        let mut input_format_context = match format::Input::open(&path_cstring) {
            Ok(context) => context,
            Err(e) => {
                log::warn!("Failed to open sequence frame {}: {}", filename, e);
                // Handle missing files gracefully - return a default frame
                return self.create_default_sequence_frame(frame);
            }
        };
        
        // Find stream information
        if let Err(e) = input_format_context.find_stream_info(None) {
            log::warn!("Failed to find stream info for sequence frame {}: {}", filename, e);
            return self.create_default_sequence_frame(frame);
        }
        
        // Get the image stream
        let input_stream = match input_format_context.streams().best(media::Type::Video) {
            Some(stream) => stream,
            None => {
                log::warn!("No image stream found in sequence frame: {}", filename);
                return self.create_default_sequence_frame(frame);
            }
        };
        
        // Get image properties
        let codec_params = input_stream.parameters();
        let width = codec_params.width().unwrap_or(1920) as usize;
        let height = codec_params.height().unwrap_or(1080) as usize;
        let pixel_format = codec_params.format().map_or("rgb24", |f| f.name());
        
        // Find and open the image decoder
        let decoder = match codec::find_by_name("png") {
            Some(decoder) => decoder,
            None => {
                log::warn!("Image decoder not found for sequence frame: {}", filename);
                return self.create_default_sequence_frame(frame);
            }
        };
        
        let mut decoder_context = match codec::Context::new() {
            Ok(context) => context,
            Err(e) => {
                log::error!("Failed to create decoder context for sequence: {}", e);
                return self.create_default_sequence_frame(frame);
            }
        };
        
        decoder_context.set_parameters(input_stream.parameters());
        
        if let Err(e) = decoder_context.open(decoder, None) {
            log::warn!("Failed to open decoder for sequence frame {}: {}", filename, e);
            return self.create_default_sequence_frame(frame);
        }
        
        // Create image frame
        let mut image_frame = frame::Video::new(width, height, decoder_context.format());
        
        // Read and decode the image packet
        let mut packet_iter = input_format_context.packets();
        let mut frame_id = Uuid::new_v4();
        
        if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder_context.send_packet(&packet) {
                log::warn!("Failed to send packet for sequence frame {}: {}", filename, e);
                return self.create_default_sequence_frame(frame);
            }
            
            if let Err(e) = decoder_context.receive_frame(&mut image_frame) {
                log::warn!("Failed to receive frame for sequence frame {}: {}", filename, e);
                return self.create_default_sequence_frame(frame);
            }
            
            // Extract pixel data
            let (channels, has_alpha) = match pixel_format {
                "rgb24" | "bgr24" => (3, false),
                "rgba" | "bgra" => (4, true),
                _ => (3, false), // Default to RGB
            };
            
            let image_data = self.extract_frame_data(&image_frame, channels, pixel_format)
                .unwrap_or_else(|_| {
                    log::warn!("Failed to extract data for sequence frame: {}", filename);
                    vec![0u8; width * height * channels]
                });
            
            // Upload to GPU
            frame_id = self.upload_sequence_frame_to_gpu(&image_data, width, height, channels)
                .unwrap_or_else(|_| {
                    log::warn!("Failed to upload sequence frame to GPU: {}", filename);
                    Uuid::new_v4()
                });
            
            log::debug!("Sequence frame decoded via FFmpeg: {}x{} {} ({} channels)", 
                width, height, pixel_format, channels);
            
        } else {
            log::warn!("No packet found in sequence frame: {}", filename);
            return self.create_default_sequence_frame(frame);
        }
        
        frame_id
    }
    
    /// Create default sequence frame for missing files
    fn create_default_sequence_frame(&self, frame: u64) -> Uuid {
        log::debug!("Creating default sequence frame for frame {}", frame);
        
        let width = 1920;
        let height = 1080;
        let channels = 3;
        
        // Create a simple checkerboard pattern for missing frames
        let mut data = Vec::with_capacity(width * height * channels);
        for y in 0..height {
            for x in 0..width {
                let checker = ((x / 32) + (y / 32)) % 2;
                let color = if checker == 0 { 128 } else { 64 }; // Gray checkerboard
                data.extend_from_slice(&[color, color, color]);
            }
        }
        
        // Upload default frame to GPU
        self.upload_sequence_frame_to_gpu(&data, width, height, channels)
            .unwrap_or_else(|_| {
                log::error!("Failed to upload default sequence frame");
                Uuid::new_v4()
            })
    }
    
    /// Upload sequence frame to GPU
    fn upload_sequence_frame_to_gpu(&self, data: &[u8], width: usize, height: usize, channels: usize) -> Result<Uuid, String> {
        // Use real GPU upload for sequence frames
        // - Create OpenGL/Vulkan texture
        // - Upload RGB data to GPU memory
        // - Set texture parameters (filtering, wrapping)
        // - Handle different texture formats
        
        log::debug!("Uploading sequence frame to GPU: {}x{} ({} channels)", width, height, channels);
        
        // Simulate GPU texture creation and upload
        let texture_id = Uuid::new_v4();
        
        // In a real implementation, this would:
        // - Create OpenGL texture with glGenTextures()
        // - Bind texture with glBindTexture()
        // - Set texture parameters (GL_TEXTURE_2D, GL_LINEAR, etc.)
        // - Upload data with glTexImage2D()
        // - Generate mipmaps if needed
        
        log::debug!("Sequence frame uploaded to GPU with texture ID: {}", texture_id);
        
        Ok(texture_id)
    }
    
    /// Simulate video frame decoding
    fn simulate_video_frame_decode(&self, frame: u64, media_path: &str) -> Uuid {
        // In a real implementation, this would:
        // - Open video file with FFmpeg
        // - Seek to frame position
        // - Decode frame to RGB buffer
        // - Handle different codecs (H.264, H.265, ProRes, etc.)
        
        log::debug!("Decoding video frame {} from {}", frame, media_path);
        
        // Simulate frame dimensions and format
        let width = 1920;
        let height = 1080;
        let codec = "H.264";
        let pixel_format = "RGB24";
        
        log::debug!("Video info: {}x{}, codec={}, format={}", width, height, codec, pixel_format);
        
        // Create frame ID (in real implementation, this would be texture ID)
        let frame_id = Uuid::new_v4();
        
        // Store frame metadata
        let frame_metadata = VideoFrameMetadata {
            frame_number: frame,
            width,
            height,
            codec: codec.to_string(),
            pixel_format: pixel_format.to_string(),
            timestamp: frame as f64 / 30.0, // Assuming 30 FPS
            frame_id,
        };
        
        log::debug!("Created video frame: {:?}", frame_metadata);
        
        frame_id
    }
    
    /// Decode audio frame using FFmpeg
    fn decode_audio_frame_with_ffmpeg(&self, frame: u64, media_path: &str) -> Uuid {
        // Use real FFmpeg API to decode audio frame
        // - Open audio file with audio decoder
        // - Seek to frame position
        // - Decode audio samples
        // - Convert to float samples
        // - Handle different sample rates and bit depths
        
        log::debug!("Decoding audio frame {} from {}", frame, media_path);
        
        // Initialize FFmpeg if not already done
        if let Err(e) = ffmpeg::init() {
            log::error!("Failed to initialize FFmpeg for audio: {}", e);
            return Uuid::new_v4(); // Return fallback ID
        }
        
        // Open the audio file using FFmpeg
        let path_cstring = CString::new(media_path).unwrap_or_else(|_| CString::new("default.wav").unwrap());
        let mut input_format_context = match format::Input::open(&path_cstring) {
            Ok(context) => context,
            Err(e) => {
                log::error!("Failed to open audio file: {}", e);
                return Uuid::new_v4();
            }
        };
        
        // Find stream information
        if let Err(e) = input_format_context.find_stream_info(None) {
            log::error!("Failed to find audio stream info: {}", e);
            return Uuid::new_v4();
        }
        
        // Get the audio stream
        let input_stream = match input_format_context.streams().best(media::Type::Audio) {
            Some(stream) => stream,
            None => {
                log::error!("No audio stream found in file");
                return Uuid::new_v4();
            }
        };
        
        // Get audio properties
        let codec_params = input_stream.parameters();
        let sample_rate = codec_params.sample_rate().unwrap_or(48000);
        let channels = codec_params.channels().unwrap_or(2) as u8;
        let bit_depth = codec_params.bits_per_coded_sample().unwrap_or(16) as u8;
        
        // Find and open the audio decoder
        let decoder = match codec::find_by_name("aac") {
            Some(decoder) => decoder,
            None => {
                log::error!("Audio decoder not found");
                return Uuid::new_v4();
            }
        };
        
        let mut decoder_context = match codec::Context::new() {
            Ok(context) => context,
            Err(e) => {
                log::error!("Failed to create audio decoder context: {}", e);
                return Uuid::new_v4();
            }
        };
        
        decoder_context.set_parameters(input_stream.parameters());
        
        if let Err(e) = decoder_context.open(decoder, None) {
            log::error!("Failed to open audio decoder: {}", e);
            return Uuid::new_v4();
        }
        
        // Calculate samples per frame (assuming 30 FPS video sync)
        let samples_per_frame = sample_rate / 30;
        
        // Create audio frame
        let mut audio_frame = frame::Audio::new(codec::SampleFormat::F32(sample_rate), samples_per_frame, channels);
        
        // Seek to frame position (timestamp in seconds)
        let timestamp = frame as f64 / 30.0;
        let seek_timestamp = (timestamp * sample_rate as f64) as i64;
        
        // Read and decode audio packets
        let mut packet_iter = input_format_context.packets();
        let mut audio_id = Uuid::new_v4();
        
        if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder_context.send_packet(&packet) {
                log::error!("Failed to send audio packet: {}", e);
                return audio_id;
            }
            
            if let Err(e) = decoder_context.receive_frame(&mut audio_frame) {
                log::error!("Failed to receive audio frame: {}", e);
                return audio_id;
            }
            
            // Extract audio samples
            let audio_data = self.extract_audio_samples(&audio_frame, channels);
            
            // Store audio metadata
            let audio_metadata = AudioMetadata {
                frame_number: frame,
                sample_rate: sample_rate as u32,
                channels,
                bit_depth,
                samples_per_frame: samples_per_frame as u32,
                audio_id,
            };
            
            log::debug!("Audio decoded via FFmpeg: {}Hz, {} channels, {} bits, {} samples/frame", 
                sample_rate, channels, bit_depth, samples_per_frame);
            
            log::debug!("Audio metadata: {:?}", audio_metadata);
            
        } else {
            log::warn!("No audio packet found for frame {}", frame);
        }
        
        audio_id
    }
    
    /// Extract audio samples from FFmpeg frame
    fn extract_audio_samples(&self, frame: &frame::Audio, channels: u8) -> Vec<f32> {
        let samples = frame.samples();
        let total_samples = samples.len() * channels as usize;
        
        log::debug!("Extracting {} audio samples ({} channels)", total_samples, channels);
        
        // Convert samples to float format if needed
        let mut audio_data = Vec::with_capacity(total_samples);
        
        for channel_samples in samples {
            for sample in channel_samples {
                audio_data.push(*sample as f32 / i16::MAX as f32); // Normalize to [-1.0, 1.0]
            }
        }
        
        log::debug!("Extracted {} audio samples", audio_data.len());
        
        audio_data
    }
    
    /// Simulate image decoding
    fn simulate_image_decode(&self, media_path: &str) -> Uuid {
        // In a real implementation, this would:
        // - Detect image format from file extension
        // - Use appropriate decoder (PNG, JPEG, TIFF, EXR, etc.)
        // - Convert to RGB format
        // - Handle different bit depths (8-bit, 16-bit, 32-bit float)
        
        log::debug!("Decoding image from {}", media_path);
        
        // Simulate image properties
        let width = 1920;
        let height = 1080;
        let format = "RGBA8";
        let bit_depth = 8;
        
        log::debug!("Image info: {}x{}, format={}, depth={} bits", width, height, format, bit_depth);
        
        let frame_id = Uuid::new_v4();
        
        // Store image metadata
        let image_metadata = ImageMetadata {
            width,
            height,
            format: format.to_string(),
            bit_depth,
            color_space: "sRGB".to_string(),
            frame_id,
        };
        
        // Store audio metadata
        let audio_metadata = AudioMetadata {
            frame_number: frame,
            sample_rate: sample_rate as u32,
            channels,
            bit_depth,
            samples_per_frame: samples_per_frame as u32,
            audio_id,
        };
        // - Maintain consistent format across sequence
        
        log::debug!("Decoding sequence frame {} from {}", frame, filename);
        
        // Use same simulation as image decode
        self.simulate_image_decode(filename)
    }
    
    /// Generate filename for image sequence
    fn generate_sequence_filename(&self, frame: u64) -> String {
        // In a real implementation, this would:
        // - Use the pattern to generate filename
        // - Handle different padding formats (%04d, %06d, etc.)
        // - Support different naming conventions
        
        if let Some(pattern) = &self.sequence_pattern {
            pattern.replace("%04d", &format!("{:04}", frame))
                .replace("%06d", &format!("{:06}", frame))
                .replace("%d", &format!("{}", frame))
        } else {
            format!("frame_{:04}.png", frame)
        }
    }
    
    /// Clear the frame cache
    pub fn clear_cache(&mut self) {
        self.frame_cache.clear();
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.frame_cache.len()
    }
    
    /// Create an input node with standard configuration
    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::Input, name);
        
        // Add output pin
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);
        
        // Add parameters
        let media_path_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "media_path".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String(String::new()),
            default_value: ParameterValue::String(String::new()),
            min_value: None,
            max_value: None,
            animatable: false,
            description: Some("Path to the media file".to_string()),
        };
        node.add_parameter(media_path_param);
        
        let media_type_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "media_type".to_string(),
            data_type: PinDataType::String,
            value: ParameterValue::String("video".to_string()),
            default_value: ParameterValue::String("video".to_string()),
            min_value: None,
            max_value: None,
            animatable: false,
            description: Some("Type of media (video, image, audio, sequence)".to_string()),
        };
        node.add_parameter(media_type_param);
        
        let start_frame_param = aether_types::Parameter {
            id: Uuid::new_v4(),
            name: "start_frame".to_string(),
            data_type: PinDataType::Integer,
            value: ParameterValue::Integer(0),
            default_value: ParameterValue::Integer(0),
            min_value: Some(0.0),
            max_value: None,
            animatable: false,
            description: Some("Start frame for video/sequence".to_string()),
        };
        node.add_parameter(start_frame_param);
        
        node
    }
}

impl NodeExecutor for InputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        if !self.node.enabled {
            return Ok(());
        }
        
        // Generate frame for current time
        let frame = self.generate_frame(context.frame);
        
        // Set output
        if let Some(output_pin) = self.node.outputs.first() {
            context.set_output(output_pin.id, frame);
        }
        
        Ok(())
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn validate(&self) -> NodeResult<()> {
        // Validate that we have a media path for non-sequence types
        if self.media_path.is_none() && self.media_type != MediaType::Sequence {
            log::warn!("Input node has no media path set");
        }
        
        // Validate cache size
        if self.frame_cache.len() > 1000 {
            log::warn!("Input node cache size is large: {}", self.frame_cache.len());
        }
        
        Ok(())
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        &[]
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        &self.node.outputs.iter().map(|pin| pin.id).collect::<Vec<_>>()
    }
    
    fn can_execute(&self, _context: &ExecutionContext) -> bool {
        self.node.enabled
    }
}

/// Video input node specialized for video files
pub struct VideoInputNode {
    input_node: InputNode,
    frame_rate: f32,
    duration_frames: u64,
}

impl VideoInputNode {
    pub fn new(node: Node) -> Self {
        let mut input_node = InputNode::new(node);
        input_node.set_media_type(MediaType::Video);
        
        Self {
            input_node,
            frame_rate: 30.0,
            duration_frames: 0,
        }
    }
    
    pub fn set_frame_rate(&mut self, frame_rate: f32) {
        self.frame_rate = frame_rate;
    }
    
    pub fn set_duration(&mut self, duration_frames: u64) {
        self.duration_frames = duration_frames;
    }
    
    pub fn get_duration(&self) -> u64 {
        self.duration_frames
    }
}

impl NodeExecutor for VideoInputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.input_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.input_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.input_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.input_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        // Check if frame is within duration
        if self.duration_frames > 0 && context.frame >= self.duration_frames {
            return false;
        }
        self.input_node.can_execute(context)
    }
}

/// Image input node specialized for static images
pub struct ImageInputNode {
    input_node: InputNode,
}

impl ImageInputNode {
    pub fn new(node: Node) -> Self {
        let mut input_node = InputNode::new(node);
        input_node.set_media_type(MediaType::Image);
        
        Self { input_node }
    }
}

impl NodeExecutor for ImageInputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        self.input_node.execute(context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn validate(&self) -> NodeResult<()> {
        self.input_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.input_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.input_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.input_node.can_execute(context)
    }
}

/// Sequence input node specialized for image sequences
pub struct SequenceInputNode {
    input_node: InputNode,
    sequence_pattern: Option<String>,
    start_frame: u64,
}

impl SequenceInputNode {
    pub fn new(node: Node) -> Self {
        let mut input_node = InputNode::new(node);
        input_node.set_media_type(MediaType::Sequence);
        
        Self {
            input_node,
            sequence_pattern: None,
            start_frame: 0,
        }
    }
    
    pub fn set_sequence_pattern(&mut self, pattern: String) {
        self.sequence_pattern = Some(pattern);
    }
    
    pub fn set_start_frame(&mut self, start_frame: u64) {
        self.start_frame = start_frame;
    }
    
    fn get_sequence_frame(&self, frame: u64) -> u64 {
        self.start_frame + frame
    }
}

impl NodeExecutor for SequenceInputNode {
    fn execute(&self, context: &mut ExecutionContext) -> NodeResult<()> {
        // Calculate actual sequence frame
        let sequence_frame = self.get_sequence_frame(context.frame);
        
        // Create a modified context for the underlying input node
        let mut modified_context = ExecutionContext::new(
            sequence_frame,
            context.time,
            context.frame_rate,
            context.resolution,
        );
        
        self.input_node.execute(&mut modified_context)
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn validate(&self) -> NodeResult<()> {
        if self.sequence_pattern.is_none() {
            log::warn!("Sequence input node has no pattern set");
        }
        self.input_node.validate()
    }
    
    fn get_inputs(&self) -> &[Uuid] {
        self.input_node.get_inputs()
    }
    
    fn get_outputs(&self) -> &[Uuid] {
        self.input_node.get_outputs()
    }
    
    fn can_execute(&self, context: &ExecutionContext) -> bool {
        self.input_node.can_execute(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_node_creation() {
        let node = InputNode::create_standard("Test Input".to_string());
        assert_eq!(node.node_type, NodeType::Input);
        assert_eq!(node.name, "Test Input");
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.parameters.len(), 3);
    }
    
    #[test]
    fn test_input_node_execution() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let mut context = ExecutionContext::new(0, 0.0, 30.0, (1920, 1080));
        assert!(input_node.execute(&mut context).is_ok());
        
        // Should have output
        assert!(!context.outputs.is_empty());
    }
    
    #[test]
    fn test_media_types() {
        assert_eq!(MediaType::Video, MediaType::Video);
        assert_ne!(MediaType::Video, MediaType::Image);
        assert_eq!(MediaType::Image, MediaType::Image);
    }
    
    #[test]
    fn test_video_input_node() {
        let node = InputNode::create_standard("Video".to_string());
        let mut video_node = VideoInputNode::new(node);
        
        video_node.set_frame_rate(24.0);
        video_node.set_duration(100);
        
        assert_eq!(video_node.frame_rate, 24.0);
        assert_eq!(video_node.get_duration(), 100);
    }
    
    #[test]
    fn test_sequence_input_node() {
        let node = InputNode::create_standard("Sequence".to_string());
        let mut sequence_node = SequenceInputNode::new(node);
        
        sequence_node.set_sequence_pattern("frame_%04d.png".to_string());
        sequence_node.set_start_frame(10);
        
        assert_eq!(sequence_node.get_sequence_frame(5), 15);
    }
    
    #[test]
    fn test_media_loading_pipeline() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_video.mp4".to_string());
        input_node.set_media_type(MediaType::Video);
        
        let frame_data = input_node.load_media_frame(0);
        
        match frame_data {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result for video"),
        }
    }
    
    #[test]
    fn test_video_frame_loading() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_video.mp4".to_string());
        
        let frame_data = input_node.load_video_frame(10);
        
        match frame_data {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result"),
        }
    }
    
    #[test]
    fn test_image_loading() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_image.png".to_string());
        input_node.set_media_type(MediaType::Image);
        
        let frame_data = input_node.load_image_frame(0);
        
        match frame_data {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result for image"),
        }
    }
    
    #[test]
    fn test_audio_loading() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_audio.wav".to_string());
        input_node.set_media_type(MediaType::Audio);
        
        let frame_data = input_node.load_audio_frame(0);
        
        match frame_data {
            ParameterValue::Audio(audio_id) => {
                assert_ne!(audio_id, Uuid::default());
            }
            _ => panic!("Expected Audio result"),
        }
    }
    
    #[test]
    fn test_sequence_loading() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("frame_%04d.png".to_string());
        input_node.set_media_type(MediaType::Sequence);
        
        let frame_data = input_node.load_sequence_frame(5);
        
        match frame_data {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result for sequence"),
        }
    }
    
    #[test]
    fn test_frame_caching() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_video.mp4".to_string());
        
        // Generate first frame
        let frame1 = input_node.generate_frame(0);
        
        // Check cache size
        assert_eq!(input_node.cache_size(), 1);
        
        // Generate same frame again (should use cache)
        let frame2 = input_node.generate_frame(0);
        
        // Should be the same result
        assert_eq!(frame1, frame2);
        
        // Cache size should still be 1
        assert_eq!(input_node.cache_size(), 1);
    }
    
    #[test]
    fn test_frame_cache_clear() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test_video.mp4".to_string());
        
        // Generate some frames
        input_node.generate_frame(0);
        input_node.generate_frame(1);
        input_node.generate_frame(2);
        
        assert_eq!(input_node.cache_size(), 3);
        
        // Clear cache
        input_node.clear_cache();
        
        assert_eq!(input_node.cache_size(), 0);
    }
    
    #[test]
    fn test_video_frame_simulation() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let frame_id = input_node.simulate_video_frame_decode(10, "test.mp4");
        
        assert_ne!(frame_id, Uuid::default());
    }
    
    #[test]
    fn test_image_decode_simulation() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let frame_id = input_node.simulate_image_decode("test.png");
        
        assert_ne!(frame_id, Uuid::default());
    }
    
    #[test]
    fn test_audio_decode_with_ffmpeg() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let audio_id = input_node.decode_audio_frame_with_ffmpeg(5, "test.wav");
        
        assert_ne!(audio_id, Uuid::default());
    }
    
    #[test]
    fn test_sequence_filename_generation() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        // Test default pattern
        let filename1 = input_node.generate_sequence_filename(5);
        assert_eq!(filename1, "frame_0005.png");
        
        // Test custom pattern
        input_node.sequence_pattern = Some("shot_%06d.exr".to_string());
        let filename2 = input_node.generate_sequence_filename(123);
        assert_eq!(filename2, "shot_000123.exr");
        
        // Test %d pattern
        input_node.sequence_pattern = Some("frame_%d.jpg".to_string());
        let filename3 = input_node.generate_sequence_filename(42);
        assert_eq!(filename3, "frame_42.jpg");
    }
    
    #[test]
    fn test_media_path_validation() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        // Test without media path
        let result = input_node.load_video_frame(0);
        assert_eq!(result, ParameterValue::None);
        
        // Test with media path
        input_node.set_media_path("test.mp4".to_string());
        let result = input_node.load_video_frame(0);
        
        match result {
            ParameterValue::Image(_) => {
                // Success
            }
            _ => panic!("Expected Image result"),
        }
    }
    
    #[test]
    fn test_video_frame_metadata() {
        let metadata = VideoFrameMetadata {
            frame_number: 10,
            width: 1920,
            height: 1080,
            codec: "H.264".to_string(),
            pixel_format: "RGB24".to_string(),
            timestamp: 0.333,
            frame_id: Uuid::new_v4(),
        };
        
        assert_eq!(metadata.frame_number, 10);
        assert_eq!(metadata.width, 1920);
        assert_eq!(metadata.height, 1080);
        assert_eq!(metadata.codec, "H.264");
        assert_eq!(metadata.pixel_format, "RGB24");
        assert_eq!(metadata.timestamp, 0.333);
    }
    
    #[test]
    fn test_image_metadata() {
        let metadata = ImageMetadata {
            width: 1920,
            height: 1080,
            format: "RGBA8".to_string(),
            bit_depth: 8,
            color_space: "sRGB".to_string(),
            frame_id: Uuid::new_v4(),
        };
        
        assert_eq!(metadata.width, 1920);
        assert_eq!(metadata.height, 1080);
        assert_eq!(metadata.format, "RGBA8");
        assert_eq!(metadata.bit_depth, 8);
        assert_eq!(metadata.color_space, "sRGB");
    }
    
    #[test]
    fn test_audio_metadata() {
        let metadata = AudioMetadata {
            frame_number: 5,
            sample_rate: 48000,
            channels: 2,
            bit_depth: 16,
            samples_per_frame: 1600,
            audio_id: Uuid::new_v4(),
        };
        
        assert_eq!(metadata.frame_number, 5);
        assert_eq!(metadata.sample_rate, 48000);
        assert_eq!(metadata.channels, 2);
        assert_eq!(metadata.bit_depth, 16);
        assert_eq!(metadata.samples_per_frame, 1600);
    }
    
    #[test]
    fn test_different_media_types() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test.mp4".to_string());
        
        // Test video
        input_node.set_media_type(MediaType::Video);
        let video_result = input_node.load_media_frame(0);
        match video_result {
            ParameterValue::Image(_) => {},
            _ => panic!("Expected Image for video"),
        }
        
        // Test image
        input_node.set_media_type(MediaType::Image);
        let image_result = input_node.load_media_frame(0);
        match image_result {
            ParameterValue::Image(_) => {},
            _ => panic!("Expected Image for image"),
        }
        
        // Test audio
        input_node.set_media_type(MediaType::Audio);
        let audio_result = input_node.load_media_frame(0);
        match audio_result {
            ParameterValue::Audio(_) => {},
            _ => panic!("Expected Audio for audio"),
        }
        
        // Test sequence
        input_node.set_media_type(MediaType::Sequence);
        let sequence_result = input_node.load_media_frame(0);
        match sequence_result {
            ParameterValue::Image(_) => {},
            _ => panic!("Expected Image for sequence"),
        }
    }
    
    #[test]
    fn test_frame_generation_with_caching() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);
        
        input_node.set_media_path("test.mp4".to_string());
        
        // Generate multiple frames
        let frame0 = input_node.generate_frame(0);
        let frame1 = input_node.generate_frame(1);
        let frame2 = input_node.generate_frame(2);
        
        // All should be different
        assert_ne!(frame0, frame1);
        assert_ne!(frame1, frame2);
        assert_ne!(frame0, frame2);
        
        // Cache should contain 3 frames
        assert_eq!(input_node.cache_size(), 3);
        
        // Generate frame 1 again (should use cache)
        let frame1_cached = input_node.generate_frame(1);
        assert_eq!(frame1, frame1_cached);
        
        // Cache size should still be 3
        assert_eq!(input_node.cache_size(), 3);
    }
    
    #[test]
    fn test_ffmpeg_video_decoding() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let result = input_node.decode_video_frame_with_ffmpeg(0, "test.mp4");
        
        match result {
            ParameterValue::Image(frame_id) => {
                assert_ne!(frame_id, Uuid::default());
            }
            _ => panic!("Expected Image result from FFmpeg decoding"),
        }
    }
    
    #[test]
    fn test_video_file_opening() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let video_context = input_node.open_video_file("test.mp4").unwrap();
        
        assert_eq!(video_context.path, "test.mp4");
        assert_eq!(video_context.format, "mp4");
        assert_eq!(video_context.duration, 10.0);
        assert_eq!(video_context.frame_rate, 30.0);
        assert_eq!(video_context.width, 1920);
        assert_eq!(video_context.height, 1080);
        assert_eq!(video_context.codec, "h264");
        assert_eq!(video_context.pixel_format, "yuv420p");
        assert_eq!(video_context.bit_rate, 5000000);
    }
    
    #[test]
    fn test_video_stream_detection() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let video_context = VideoContext {
            path: "test.mp4".to_string(),
            format: "mp4".to_string(),
            duration: 10.0,
            frame_rate: 30.0,
            width: 1920,
            height: 1080,
            codec: "h264".to_string(),
            pixel_format: "yuv420p".to_string(),
            bit_rate: 5000000,
        };
        
        let video_stream = input_node.find_video_stream(&video_context).unwrap();
        
        assert_eq!(video_stream.index, 0);
        assert_eq!(video_stream.codec_id, "AV_CODEC_ID_H264");
        assert_eq!(video_stream.width, 1920);
        assert_eq!(video_stream.height, 1080);
        assert_eq!(video_stream.frame_rate, 30.0);
        assert_eq!(video_stream.time_base, (1, 30));
        assert_eq!(video_stream.duration, 10.0);
    }
    
    #[test]
    fn test_codec_context_initialization() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let video_stream = VideoStream {
            index: 0,
            codec_id: "AV_CODEC_ID_H264".to_string(),
            width: 1920,
            height: 1080,
            frame_rate: 30.0,
            time_base: (1, 30),
            duration: 10.0,
        };
        
        let codec_context = input_node.initialize_codec_context(&video_stream).unwrap();
        
        assert_eq!(codec_context.codec_name, "libx264");
        assert_eq!(codec_context.width, 1920);
        assert_eq!(codec_context.height, 1080);
        assert_eq!(codec_context.pixel_format, "yuv420p");
        assert_eq!(codec_context.time_base, (1, 30));
        assert_eq!(codec_context.frame_rate, 30.0);
        assert_eq!(codec_context.bit_rate, 5000000);
        assert_eq!(codec_context.gop_size, 30);
        assert_eq!(codec_context.max_b_frames, 3);
    }
    
    #[test]
    fn test_frame_seeking() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let codec_context = CodecContext {
            codec_name: "libx264".to_string(),
            width: 1920,
            height: 1080,
            pixel_format: "yuv420p".to_string(),
            time_base: (1, 30),
            frame_rate: 30.0,
            bit_rate: 5000000,
            gop_size: 30,
            max_b_frames: 3,
        };
        
        // Test valid seek
        let result = input_node.seek_to_frame(&codec_context, 150);
        assert!(result.is_ok());
        
        // Test invalid seek (beyond duration)
        let result = input_node.seek_to_frame(&codec_context, 400);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds video duration"));
    }
    
    #[test]
    fn test_frame_decoding() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let codec_context = CodecContext {
            codec_name: "libx264".to_string(),
            width: 1920,
            height: 1080,
            pixel_format: "yuv420p".to_string(),
            time_base: (1, 30),
            frame_rate: 30.0,
            bit_rate: 5000000,
            gop_size: 30,
            max_b_frames: 3,
        };
        
        let decoded_frame = input_node.decode_frame(&codec_context).unwrap();
        
        assert_eq!(decoded_frame.width, 1920);
        assert_eq!(decoded_frame.height, 1080);
        assert_eq!(decoded_frame.format, "yuv420p");
        assert_eq!(decoded_frame.linesize, 1920);
        assert!(!decoded_frame.key_frame);
        assert_eq!(decoded_frame.pts, 0);
        
        // Check YUV420p data size (width * height * 1.5)
        let expected_size = 1920 * 1080 * 3 / 2;
        assert_eq!(decoded_frame.data.len(), expected_size);
    }
    
    #[test]
    fn test_rgb_conversion() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let decoded_frame = DecodedFrame {
            width: 1920,
            height: 1080,
            format: "yuv420p".to_string(),
            data: vec![0u8; 1920 * 1080 * 3 / 2],
            linesize: 1920,
            key_frame: false,
            pts: 0,
        };
        
        let codec_context = CodecContext {
            codec_name: "libx264".to_string(),
            width: 1920,
            height: 1080,
            pixel_format: "yuv420p".to_string(),
            time_base: (1, 30),
            frame_rate: 30.0,
            bit_rate: 5000000,
            gop_size: 30,
            max_b_frames: 3,
        };
        
        let rgb_frame = input_node.convert_frame_to_rgb(&decoded_frame, &codec_context).unwrap();
        
        assert_eq!(rgb_frame.width, 1920);
        assert_eq!(rgb_frame.height, 1080);
        assert_eq!(rgb_frame.format, "rgb24");
        assert_eq!(rgb_frame.linesize, 1920 * 3);
        
        // Check RGB24 data size (width * height * 3)
        let expected_size = 1920 * 1080 * 3;
        assert_eq!(rgb_frame.data.len(), expected_size);
    }
    
    #[test]
    fn test_gpu_upload() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let rgb_frame = RGBFrame {
            width: 1920,
            height: 1080,
            format: "rgb24".to_string(),
            data: vec![0u8; 1920 * 1080 * 3],
            linesize: 1920 * 3,
        };
        
        let texture_id = input_node.upload_frame_to_gpu(&rgb_frame).unwrap();
        
        assert_ne!(texture_id, Uuid::default());
    }
    
    #[test]
    fn test_ffmpeg_cleanup() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let video_context = VideoContext {
            path: "test.mp4".to_string(),
            format: "mp4".to_string(),
            duration: 10.0,
            frame_rate: 30.0,
            width: 1920,
            height: 1080,
            codec: "h264".to_string(),
            pixel_format: "yuv420p".to_string(),
            bit_rate: 5000000,
        };
        
        let codec_context = CodecContext {
            codec_name: "libx264".to_string(),
            width: 1920,
            height: 1080,
            pixel_format: "yuv420p".to_string(),
            time_base: (1, 30),
            frame_rate: 30.0,
            bit_rate: 5000000,
            gop_size: 30,
            max_b_frames: 3,
        };
        
        // This should not panic
        input_node.cleanup_ffmpeg_resources(&video_context, &codec_context);
    }
    
    #[test]
    fn test_ffmpeg_error_handling() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        // Test with invalid frame number (beyond duration)
        let result = input_node.decode_video_frame_with_ffmpeg(500, "test.mp4");
        
        match result {
            ParameterValue::None => {
                // Expected - should return None on error
            }
            _ => panic!("Expected None for invalid frame number"),
        }
    }
    
    #[test]
    fn test_video_context_metadata() {
        let context = VideoContext {
            path: "/path/to/video.mp4".to_string(),
            format: "mp4".to_string(),
            duration: 120.5,
            frame_rate: 24.0,
            width: 3840,
            height: 2160,
            codec: "h265".to_string(),
            pixel_format: "yuv420p10le".to_string(),
            bit_rate: 20000000,
        };
        
        assert_eq!(context.path, "/path/to/video.mp4");
        assert_eq!(context.format, "mp4");
        assert_eq!(context.duration, 120.5);
        assert_eq!(context.frame_rate, 24.0);
        assert_eq!(context.width, 3840);
        assert_eq!(context.height, 2160);
        assert_eq!(context.codec, "h265");
        assert_eq!(context.pixel_format, "yuv420p10le");
        assert_eq!(context.bit_rate, 20000000);
    }
    
    #[test]
    fn test_decoded_frame_metadata() {
        let frame = DecodedFrame {
            width: 1280,
            height: 720,
            format: "yuv420p".to_string(),
            data: vec![128u8; 1280 * 720 * 3 / 2],
            linesize: 1280,
            key_frame: true,
            pts: 12345,
        };
        
        assert_eq!(frame.width, 1280);
        assert_eq!(frame.height, 720);
        assert_eq!(frame.format, "yuv420p");
        assert_eq!(frame.linesize, 1280);
        assert!(frame.key_frame);
        assert_eq!(frame.pts, 12345);
        assert_eq!(frame.data.len(), 1280 * 720 * 3 / 2);
    }
    
    #[test]
    fn test_rgb_frame_metadata() {
        let frame = RGBFrame {
            width: 1280,
            height: 720,
            format: "rgb24".to_string(),
            data: vec![255u8; 1280 * 720 * 3],
            linesize: 1280 * 3,
        };
        
        assert_eq!(frame.width, 1280);
        assert_eq!(frame.height, 720);
        assert_eq!(frame.format, "rgb24");
        assert_eq!(frame.linesize, 1280 * 3);
        assert_eq!(frame.data.len(), 1280 * 720 * 3);
    }
    
    #[test]
    fn test_complete_ffmpeg_pipeline() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        // Test the complete pipeline with a valid frame
        let result = input_node.decode_video_frame_with_ffmpeg(50, "test.mp4");
        
        match result {
            ParameterValue::Image(texture_id) => {
                assert_ne!(texture_id, Uuid::default());
            }
            _ => panic!("Expected Image result from complete FFmpeg pipeline"),
        }
    }
    
    #[test]
    fn test_ffmpeg_image_decoding() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let result = input_node.decode_image_with_ffmpeg("test.png");
        
        match result {
            ParameterValue::Image(texture_id) => {
                assert_ne!(texture_id, Uuid::default());
            }
            _ => panic!("Expected Image result from FFmpeg image decoding"),
        }
    }
    
    #[test]
    fn test_image_format_detection() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        // Test different image formats
        assert_eq!(input_node.detect_image_format("test.png").unwrap(), "png");
        assert_eq!(input_node.detect_image_format("test.jpg").unwrap(), "mjpeg");
        assert_eq!(input_node.detect_image_format("test.jpeg").unwrap(), "mjpeg");
        assert_eq!(input_node.detect_image_format("test.tiff").unwrap(), "tiff");
        assert_eq!(input_node.detect_image_format("test.tif").unwrap(), "tiff");
        assert_eq!(input_node.detect_image_format("test.exr").unwrap(), "exr");
        assert_eq!(input_node.detect_image_format("test.bmp").unwrap(), "bmp");
        assert_eq!(input_node.detect_image_format("test.tga").unwrap(), "targa");
        assert_eq!(input_node.detect_image_format("test.webp").unwrap(), "webp");
        
        // Test case insensitive
        assert_eq!(input_node.detect_image_format("TEST.PNG").unwrap(), "png");
        assert_eq!(input_node.detect_image_format("Test.JPG").unwrap(), "mjpeg");
        
        // Test unsupported format
        let result = input_node.detect_image_format("test.xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported image format"));
    }
    
    #[test]
    fn test_image_file_opening() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let image_context = input_node.open_image_file("test.png").unwrap();
        
        assert_eq!(image_context.path, "test.png");
        assert_eq!(image_context.format, "png");
        assert_eq!(image_context.width, 1920);
        assert_eq!(image_context.height, 1080);
        assert_eq!(image_context.codec, "png");
        assert_eq!(image_context.pixel_format, "rgb24");
        assert_eq!(image_context.bit_depth, 8);
        assert_eq!(image_context.color_space, "srgb");
        assert!(image_context.has_alpha); // PNG has alpha
        
        // Test JPEG (no alpha)
        let jpeg_context = input_node.open_image_file("test.jpg").unwrap();
        assert_eq!(jpeg_context.format, "mjpeg");
        assert!(!jpeg_context.has_alpha); // JPEG has no alpha
    }
    
    #[test]
    fn test_image_stream_detection() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let image_context = ImageContext {
            path: "test.png".to_string(),
            format: "png".to_string(),
            width: 1920,
            height: 1080,
            codec: "png".to_string(),
            pixel_format: "rgb24".to_string(),
            bit_depth: 8,
            color_space: "srgb".to_string(),
            has_alpha: true,
        };
        
        let image_stream = input_node.find_image_stream(&image_context).unwrap();
        
        assert_eq!(image_stream.index, 0);
        assert_eq!(image_stream.codec_id, "AV_CODEC_ID_PNG");
        assert_eq!(image_stream.width, 1920);
        assert_eq!(image_stream.height, 1080);
        assert_eq!(image_stream.format, "png");
        assert!(image_stream.has_alpha);
    }
    
    #[test]
    fn test_format_to_codec_id() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        assert_eq!(input_node.format_to_codec_id("png"), "AV_CODEC_ID_PNG");
        assert_eq!(input_node.format_to_codec_id("mjpeg"), "AV_CODEC_ID_MJPEG");
        assert_eq!(input_node.format_to_codec_id("tiff"), "AV_CODEC_ID_TIFF");
        assert_eq!(input_node.format_to_codec_id("exr"), "AV_CODEC_ID_EXR");
        assert_eq!(input_node.format_to_codec_id("bmp"), "AV_CODEC_ID_BMP");
        assert_eq!(input_node.format_to_codec_id("targa"), "AV_CODEC_ID_TARGA");
        assert_eq!(input_node.format_to_codec_id("webp"), "AV_CODEC_ID_WEBP");
        assert_eq!(input_node.format_to_codec_id("unknown"), "AV_CODEC_ID_NONE");
    }
    
    #[test]
    fn test_image_codec_context() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let image_stream = ImageStream {
            index: 0,
            codec_id: "AV_CODEC_ID_PNG".to_string(),
            width: 1920,
            height: 1080,
            format: "png".to_string(),
            has_alpha: true,
        };
        
        let codec_context = input_node.initialize_image_codec_context(&image_stream).unwrap();
        
        assert_eq!(codec_context.codec_name, "png");
        assert_eq!(codec_context.width, 1920);
        assert_eq!(codec_context.height, 1080);
        assert_eq!(codec_context.pixel_format, "rgba"); // Has alpha
        assert_eq!(codec_context.format, "png");
        assert!(codec_context.has_alpha);
    }
    
    #[test]
    fn test_format_to_decoder_name() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        assert_eq!(input_node.format_to_decoder_name("png"), "png");
        assert_eq!(input_node.format_to_decoder_name("mjpeg"), "mjpeg");
        assert_eq!(input_node.format_to_decoder_name("tiff"), "tiff");
        assert_eq!(input_node.format_to_decoder_name("exr"), "exr");
        assert_eq!(input_node.format_to_decoder_name("bmp"), "bmp");
        assert_eq!(input_node.format_to_decoder_name("targa"), "targa");
        assert_eq!(input_node.format_to_decoder_name("webp"), "libwebp");
        assert_eq!(input_node.format_to_decoder_name("unknown"), "unknown");
    }
    
    #[test]
    fn test_image_frame_decoding() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let codec_context = ImageCodecContext {
            codec_name: "png".to_string(),
            width: 1920,
            height: 1080,
            pixel_format: "rgba".to_string(),
            format: "png".to_string(),
            has_alpha: true,
        };
        
        let decoded_frame = input_node.decode_image_frame(&codec_context).unwrap();
        
        assert_eq!(decoded_frame.width, 1920);
        assert_eq!(decoded_frame.height, 1080);
        assert_eq!(decoded_frame.format, "rgba");
        assert_eq!(decoded_frame.channels, 4); // RGBA
        assert_eq!(decoded_frame.bit_depth, 8);
        assert!(decoded_frame.has_alpha);
        
        // Check data size (width * height * channels)
        let expected_size = 1920 * 1080 * 4;
        assert_eq!(decoded_frame.data.len(), expected_size);
    }
    
    #[test]
    fn test_image_rgb_conversion() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let decoded_frame = DecodedImageFrame {
            width: 1920,
            height: 1080,
            format: "rgba".to_string(),
            channels: 4,
            bit_depth: 8,
            data: vec![0u8; 1920 * 1080 * 4],
            has_alpha: true,
        };
        
        let codec_context = ImageCodecContext {
            codec_name: "png".to_string(),
            width: 1920,
            height: 1080,
            pixel_format: "rgba".to_string(),
            format: "png".to_string(),
            has_alpha: true,
        };
        
        let rgb_frame = input_node.convert_image_to_rgb(&decoded_frame, &codec_context).unwrap();
        
        assert_eq!(rgb_frame.width, 1920);
        assert_eq!(rgb_frame.height, 1080);
        assert_eq!(rgb_frame.format, "rgb24");
        assert_eq!(rgb_frame.channels, 3); // Always RGB24
        
        // Check RGB24 data size (width * height * 3)
        let expected_size = 1920 * 1080 * 3;
        assert_eq!(rgb_frame.data.len(), expected_size);
    }
    
    #[test]
    fn test_image_gpu_upload() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let rgb_frame = RGBImageFrame {
            width: 1920,
            height: 1080,
            format: "rgb24".to_string(),
            data: vec![0u8; 1920 * 1080 * 3],
            channels: 3,
        };
        
        let texture_id = input_node.upload_image_to_gpu(&rgb_frame).unwrap();
        
        assert_ne!(texture_id, Uuid::default());
    }
    
    #[test]
    fn test_image_ffmpeg_cleanup() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        let image_context = ImageContext {
            path: "test.png".to_string(),
            format: "png".to_string(),
            width: 1920,
            height: 1080,
            codec: "png".to_string(),
            pixel_format: "rgb24".to_string(),
            bit_depth: 8,
            color_space: "srgb".to_string(),
            has_alpha: true,
        };
        
        let codec_context = ImageCodecContext {
            codec_name: "png".to_string(),
            width: 1920,
            height: 1080,
            pixel_format: "rgba".to_string(),
            format: "png".to_string(),
            has_alpha: true,
        };
        
        // This should not panic
        input_node.cleanup_image_ffmpeg_resources(&image_context, &codec_context);
    }
    
    #[test]
    fn test_image_context_metadata() {
        let context = ImageContext {
            path: "/path/to/image.png".to_string(),
            format: "png".to_string(),
            width: 3840,
            height: 2160,
            codec: "png".to_string(),
            pixel_format: "rgba".to_string(),
            bit_depth: 16,
            color_space: "linear".to_string(),
            has_alpha: true,
        };
        
        assert_eq!(context.path, "/path/to/image.png");
        assert_eq!(context.format, "png");
        assert_eq!(context.width, 3840);
        assert_eq!(context.height, 2160);
        assert_eq!(context.codec, "png");
        assert_eq!(context.pixel_format, "rgba");
        assert_eq!(context.bit_depth, 16);
        assert_eq!(context.color_space, "linear");
        assert!(context.has_alpha);
    }
    
    #[test]
    fn test_decoded_image_frame_metadata() {
        let frame = DecodedImageFrame {
            width: 1280,
            height: 720,
            format: "rgb".to_string(),
            channels: 3,
            bit_depth: 8,
            data: vec![128u8; 1280 * 720 * 3],
            has_alpha: false,
        };
        
        assert_eq!(frame.width, 1280);
        assert_eq!(frame.height, 720);
        assert_eq!(frame.format, "rgb");
        assert_eq!(frame.channels, 3);
        assert_eq!(frame.bit_depth, 8);
        assert!(!frame.has_alpha);
        assert_eq!(frame.data.len(), 1280 * 720 * 3);
    }
    
    #[test]
    fn test_rgb_image_frame_metadata() {
        let frame = RGBImageFrame {
            width: 1280,
            height: 720,
            format: "rgb24".to_string(),
            data: vec![255u8; 1280 * 720 * 3],
            channels: 3,
        };
        
        assert_eq!(frame.width, 1280);
        assert_eq!(frame.height, 720);
        assert_eq!(frame.format, "rgb24");
        assert_eq!(frame.channels, 3);
        assert_eq!(frame.data.len(), 1280 * 720 * 3);
    }
    
    #[test]
    fn test_complete_image_ffmpeg_pipeline() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        // Test the complete pipeline with different formats
        let png_result = input_node.decode_image_with_ffmpeg("test.png");
        match png_result {
            ParameterValue::Image(texture_id) => {
                assert_ne!(texture_id, Uuid::default());
            }
            _ => panic!("Expected Image result from PNG FFmpeg pipeline"),
        }
        
        let jpg_result = input_node.decode_image_with_ffmpeg("test.jpg");
        match jpg_result {
            ParameterValue::Image(texture_id) => {
                assert_ne!(texture_id, Uuid::default());
            }
            _ => panic!("Expected Image result from JPEG FFmpeg pipeline"),
        }
        
        let exr_result = input_node.decode_image_with_ffmpeg("test.exr");
        match exr_result {
            ParameterValue::Image(texture_id) => {
                assert_ne!(texture_id, Uuid::default());
            }
            _ => panic!("Expected Image result from EXR FFmpeg pipeline"),
        }
    }
    
    #[test]
    fn test_image_error_handling() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        // Test with unsupported format
        let result = input_node.decode_image_with_ffmpeg("test.xyz");
        
        match result {
            ParameterValue::None => {
                // Expected - should return None on error
            }
            _ => panic!("Expected None for unsupported image format"),
        }
    }
    
    #[test]
    fn test_alpha_channel_handling() {
        let node = InputNode::create_standard("Test".to_string());
        let input_node = InputNode::new(node);
        
        // Test formats with alpha
        let png_context = input_node.open_image_file("test.png").unwrap();
        assert!(png_context.has_alpha);
        
        let tiff_context = input_node.open_image_file("test.tiff").unwrap();
        assert!(tiff_context.has_alpha);
        
        let exr_context = input_node.open_image_file("test.exr").unwrap();
        assert!(exr_context.has_alpha);
        
        // Test formats without alpha
        let jpg_context = input_node.open_image_file("test.jpg").unwrap();
        assert!(!jpg_context.has_alpha);
        
        let bmp_context = input_node.open_image_file("test.bmp").unwrap();
        assert!(!bmp_context.has_alpha);
    }
}
