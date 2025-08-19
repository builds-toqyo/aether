use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;

use crate::engine::timeline::{Timeline, Clip, ClipType, TimelineError};
use crate::engine::renderer::{Renderer, Frame, RendererError};
use crate::engine::video_decoder::{VideoDecoder, VideoDecoderConfig, VideoFrame, VideoDecoderError};
use crate::engine::VideoFormat;

#[derive(Debug)]
pub enum TimelineRendererError {
    TimelineError(TimelineError),
    RendererError(RendererError),
    DecoderError(VideoDecoderError),
    CompositionError(String),
    ResourceError(String),
}

impl fmt::Display for TimelineRendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimelineRendererError::TimelineError(e) => write!(f, "Timeline error: {}", e),
            TimelineRendererError::RendererError(e) => write!(f, "Renderer error: {}", e),
            TimelineRendererError::DecoderError(e) => write!(f, "Decoder error: {}", e),
            TimelineRendererError::CompositionError(msg) => write!(f, "Composition error: {}", msg),
            TimelineRendererError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
        }
    }
}

impl Error for TimelineRendererError {}

impl From<TimelineError> for TimelineRendererError {
    fn from(error: TimelineError) -> Self {
        TimelineRendererError::TimelineError(error)
    }
}

impl From<RendererError> for TimelineRendererError {
    fn from(error: RendererError) -> Self {
        TimelineRendererError::RendererError(error)
    }
}

impl From<VideoDecoderError> for TimelineRendererError {
    fn from(error: VideoDecoderError) -> Self {
        TimelineRendererError::DecoderError(error)
    }
}

pub struct TimelineRendererConfig {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub background_color: [u8; 4], // RGBA
    pub cache_size: usize,         // Number of frames to cache
}

impl Default for TimelineRendererConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30.0,
            background_color: [0, 0, 0, 255], // Black background
            cache_size: 30,                   // Cache 1 second of video at 30fps
        }
    }
}

pub struct ClipRenderer {
    decoder: VideoDecoder,
    clip_id: String,
    source_path: String,
    in_point: f64,
    out_point: f64,
    last_decoded_frame: Option<VideoFrame>,
}

impl ClipRenderer {
    pub fn new(clip_id: String, source_path: String, in_point: f64, out_point: f64) -> Result<Self, TimelineRendererError> {
        let mut config = VideoDecoderConfig::default();
        config.target_format = VideoFormat::RGBA;
        
        let decoder = VideoDecoder::new(config);
        
        Ok(Self {
            decoder,
            clip_id,
            source_path,
            in_point,
            out_point,
            last_decoded_frame: None,
        })
    }
    
    pub fn initialize(&mut self) -> Result<(), TimelineRendererError> {
        self.decoder.open(&self.source_path)?;
        Ok(())
    }
    
    pub fn seek_to_time(&mut self, timeline_time: f64, clip_start_time: f64) -> Result<(), TimelineRendererError> {
        let source_time = self.in_point + (timeline_time - clip_start_time);
        
        self.decoder.seek(source_time)?;
        Ok(())
    }
    
    pub fn decode_frame(&mut self) -> Result<&VideoFrame, TimelineRendererError> {
        let frame = self.decoder.decode_video_frame()?;
        self.last_decoded_frame = Some(frame);
        
        self.last_decoded_frame.as_ref().ok_or_else(|| {
            TimelineRendererError::ResourceError("Failed to decode frame".to_string())
        })
    }
    
    pub fn close(&mut self) -> Result<(), TimelineRendererError> {
        self.decoder.close()?;
        Ok(())
    }
}

pub struct TimelineRenderer {
    config: TimelineRendererConfig,
    timeline: Arc<Mutex<Timeline>>,
    renderer: Renderer,
    clip_renderers: HashMap<String, ClipRenderer>,
    frame_cache: HashMap<f64, Frame>, // Cache frames by timestamp
    is_initialized: bool,
}

impl TimelineRenderer {
    pub fn new(config: TimelineRendererConfig, timeline: Arc<Mutex<Timeline>>) -> Result<Self, TimelineRendererError> {
        let renderer_config = crate::engine::renderer::RendererConfig {
            width: config.width,
            height: config.height,
            fps: config.fps as u32,
        };
        
        let renderer = Renderer::new(renderer_config);
        
        Ok(Self {
            config,
            timeline,
            renderer,
            clip_renderers: HashMap::new(),
            frame_cache: HashMap::new(),
            is_initialized: false,
        })
    }
    
    pub fn initialize(&mut self) -> Result<(), TimelineRendererError> {
        self.renderer.initialize()?;
        
        let timeline = self.timeline.lock().unwrap();
        
        for (track_id, track) in timeline.tracks() {
            for clip in &track.clips {
                if clip.clip_type == ClipType::Video {
                    if let Some(source_path) = &clip.source_path {
                        let in_point = clip.properties.get("in_point")
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        
                        let out_point = in_point + clip.duration;
                        
                        let mut clip_renderer = ClipRenderer::new(
                            clip.id.clone(),
                            source_path.clone(),
                            in_point,
                            out_point,
                        )?;
                        
                        clip_renderer.initialize()?;
                        self.clip_renderers.insert(clip.id.clone(), clip_renderer);
                    }
                }
            }
        }
        
        self.is_initialized = true;
        Ok(())
    }
    
    pub fn render_frame(&mut self, time: f64) -> Result<&Frame, TimelineRendererError> {
        if !self.is_initialized {
            return Err(TimelineRendererError::ResourceError("Renderer not initialized".to_string()));
        }
        
        if let Some(frame) = self.frame_cache.get(&time) {
            return Ok(frame);
        }
        
        let timeline = self.timeline.lock().unwrap();
        let active_clips = timeline.active_clips();
        
        let mut frame_data = vec![
            self.config.background_color[0], // R
            self.config.background_color[1], // G
            self.config.background_color[2], // B
            self.config.background_color[3], // A
        ];
        
        frame_data.resize((self.config.width * self.config.height * 4) as usize, 0);
        
        // Render each active clip
        for (track_id, clips) in active_clips {
            for clip in clips {
                if clip.clip_type == ClipType::Video {
                    if let Some(clip_renderer) = self.clip_renderers.get_mut(&clip.id) {
                        // Seek to the correct time in the clip
                        clip_renderer.seek_to_time(time, clip.start_time)?;
                        
                        // Decode a frame
                        let video_frame = clip_renderer.decode_frame()?;
                        
                        // Composite the frame onto our output frame
                        self.composite_frame(&mut frame_data, video_frame)?;
                    }
                }
            }
        }
        
        // Render the final frame
        let frame = self.renderer.render(&frame_data, time)?;
        
        // Add to cache (if cache is full, remove oldest entry)
        if self.frame_cache.len() >= self.config.cache_size {
            if let Some(oldest_time) = self.frame_cache.keys().min_by(|a, b| a.partial_cmp(b).unwrap()).cloned() {
                self.frame_cache.remove(&oldest_time);
            }
        }
        
        // We can't actually add to cache here because frame is borrowed from renderer
        // In a real implementation, we'd need to clone the frame or use a different approach
        
        Ok(frame)
    }
    
    fn composite_frame(&self, output: &mut [u8], input: &VideoFrame) -> Result<(), TimelineRendererError> {
        // This is a simplified compositing function
        // In a real implementation, we'd need to handle scaling, positioning, alpha blending, etc.
        
        let out_width = self.config.width as usize;
        let out_height = self.config.height as usize;
        let in_width = input.width as usize;
        let in_height = input.height as usize;
        
        // Simple center positioning
        let x_offset = if out_width > in_width { (out_width - in_width) / 2 } else { 0 };
        let y_offset = if out_height > in_height { (out_height - in_height) / 2 } else { 0 };
        
        // Simple alpha blending
        for y in 0..std::cmp::min(in_height, out_height) {
            for x in 0..std::cmp::min(in_width, out_width) {
                let in_pos = (y * in_width + x) * 4;
                let out_pos = ((y + y_offset) * out_width + (x + x_offset)) * 4;
                
                if out_pos + 3 < output.len() && in_pos + 3 < input.data.len() {
                    // Simple alpha blending
                    let alpha = input.data[in_pos + 3] as f32 / 255.0;
                    
                    output[out_pos] = ((1.0 - alpha) * output[out_pos] as f32 + alpha * input.data[in_pos] as f32) as u8;
                    output[out_pos + 1] = ((1.0 - alpha) * output[out_pos + 1] as f32 + alpha * input.data[in_pos + 1] as f32) as u8;
                    output[out_pos + 2] = ((1.0 - alpha) * output[out_pos + 2] as f32 + alpha * input.data[in_pos + 2] as f32) as u8;
                    output[out_pos + 3] = 255; // Full opacity for output
                }
            }
        }
        
        Ok(())
    }
    
    pub fn update_timeline(&mut self, timeline: Arc<Mutex<Timeline>>) -> Result<(), TimelineRendererError> {
        self.timeline = timeline;
        
        // Clear cache as timeline has changed
        self.frame_cache.clear();
        
        // Close existing clip renderers
        for (_, renderer) in &mut self.clip_renderers {
            renderer.close()?;
        }
        
        self.clip_renderers.clear();
        
        // Re-initialize with new timeline
        self.initialize()?;
        
        Ok(())
    }
    
    pub fn cleanup(&mut self) -> Result<(), TimelineRendererError> {
        // Close all clip renderers
        for (_, renderer) in &mut self.clip_renderers {
            renderer.close()?;
        }
        
        self.clip_renderers.clear();
        self.frame_cache.clear();
        
        // Clean up renderer
        self.renderer.cleanup()?;
        self.is_initialized = false;
        
        Ok(())
    }
}

impl Drop for TimelineRenderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

pub fn create_default_timeline_renderer(timeline: Arc<Mutex<Timeline>>) -> Result<TimelineRenderer, TimelineRendererError> {
    let config = TimelineRendererConfig::default();
    let mut renderer = TimelineRenderer::new(config, timeline)?;
    renderer.initialize()?;
    Ok(renderer)
}
