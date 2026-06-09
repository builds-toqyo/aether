use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};

// Wrapper type for f64 timestamps that implements Hash + Eq
#[derive(Debug, Clone, Copy, PartialEq)]
struct Timestamp(f64);

impl Eq for Timestamp {}

impl Hash for Timestamp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Use a fixed-point representation for hashing
        let scaled = (self.0 * 1000.0) as i64;
        scaled.hash(state);
    }
}

impl From<f64> for Timestamp {
    fn from(time: f64) -> Self {
        Timestamp(time)
    }
}

impl From<Timestamp> for f64 {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

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
        config.output_format = VideoFormat::RGBA32;

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
            TimelineRendererError::ResourceError("No decoded frame available".to_string())
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
    frame_cache: HashMap<Timestamp, Frame>, // Cache frames by timestamp
    is_initialized: bool,
}

impl TimelineRenderer {
    pub fn new(config: TimelineRendererConfig, timeline: Arc<Mutex<Timeline>>) -> Result<Self, TimelineRendererError> {
        let mut renderer_config = crate::engine::renderer::RendererConfig::default();
        renderer_config.width = config.width;
        renderer_config.height = config.height;
        renderer_config.frame_rate = config.fps as f64;

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
                        let in_point = clip.properties.get("in-point")
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
            return Err(TimelineRendererError::ResourceError("Timeline renderer not initialized".to_string()));
        }

        // TODO: fix frame_cache borrow checker issue
        // if let Some(frame) = self.frame_cache.get(&Timestamp::from(time)) {
        //     return Ok(frame);
        // }

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

                        // TODO: Implement frame compositing
                        // self.composite_frame(&mut frame_data, video_frame)?;
                    }
                }
            }
        }

        // Render the final frame
        let frame = self.renderer.render(&frame_data, time)?;

        // We can't actually add to cache here because frame is borrowed from renderer

        Ok(frame)
    }

    pub fn update_timeline(&mut self, timeline: Arc<Mutex<Timeline>>) -> Result<(), TimelineRendererError> {
        self.timeline = timeline;


        self.frame_cache.clear();


        for (_, renderer) in &mut self.clip_renderers {
            renderer.close()?;
        }

        self.clip_renderers.clear();


        self.initialize()?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<(), TimelineRendererError> {

        for (_, renderer) in &mut self.clip_renderers {
            renderer.close()?;
        }

        self.clip_renderers.clear();
        self.frame_cache.clear();


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
