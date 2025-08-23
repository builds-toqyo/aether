use anyhow::{anyhow, Result};
use gst::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub path: PathBuf,
    pub media_type: MediaType,
    pub size: u64,
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f64>,
    pub codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ThumbnailOptions {
    pub width: u32,
    pub height: u32,
    pub position: Option<f64>,
    pub quality: u8,
}

impl Default for ThumbnailOptions {
    fn default() -> Self {
        Self {
            width: 320,
            height: 180,
            position: Some(0.0),
            quality: 90,
        }
    }
}

pub struct FileManager {
    temp_dir: PathBuf,
    media_info_cache: Arc<Mutex<HashMap<PathBuf, MediaInfo>>>,
    thumbnail_cache: Arc<Mutex<HashMap<PathBuf, PathBuf>>>,
}

impl FileManager {
    pub fn new() -> Result<Self> {
        if !gst::is_initialized() {
            gst::init()?;
        }
        
        let temp_dir = std::env::temp_dir().join("aether");
        fs::create_dir_all(&temp_dir)?;
        
        Ok(Self {
            temp_dir,
            media_info_cache: Arc::new(Mutex::new(HashMap::new())),
            thumbnail_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub fn get_media_info(&self, path: &Path) -> Result<MediaInfo> {
        if let Some(info) = self.media_info_cache.lock().unwrap().get(path) {
            return Ok(info.clone());
        }
        
        if !path.exists() {
            return Err(anyhow!("File does not exist: {:?}", path));
        }
        
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        
        let media_type = self.determine_media_type(path);
        
        let mut info = MediaInfo {
            path: path.to_path_buf(),
            media_type,
            size,
            duration: None,
            width: None,
            height: None,
            frame_rate: None,
            codec: None,
            sample_rate: None,
            channels: None,
            metadata: HashMap::new(),
        };
        
        match media_type {
            MediaType::Video | MediaType::Audio => {
                self.extract_media_info_gstreamer(path, &mut info)?;
            },
            MediaType::Image => {
                self.extract_image_info(path, &mut info)?;
            },
            MediaType::Unknown => {
                // No additional info for unknown types
            },
        }
        
        self.media_info_cache.lock().unwrap().insert(path.to_path_buf(), info.clone());
        
        Ok(info)
    }
    
    /// Generate a thumbnail for a media file
    pub fn generate_thumbnail(&self, path: &Path, options: Option<ThumbnailOptions>) -> Result<PathBuf> {
        let options = options.unwrap_or_default();
        
        // Check cache first
        let cache_key = path.to_path_buf();
        if let Some(thumbnail_path) = self.thumbnail_cache.lock().unwrap().get(&cache_key) {
            if thumbnail_path.exists() {
                return Ok(thumbnail_path.clone());
            }
        }
        
        // Determine media type
        let media_type = self.determine_media_type(path);
        
        // Generate thumbnail based on media type
        let thumbnail_path = match media_type {
            MediaType::Video => self.generate_video_thumbnail(path, &options)?,
            MediaType::Image => self.generate_image_thumbnail(path, &options)?,
            MediaType::Audio => self.generate_audio_thumbnail(path, &options)?,
            MediaType::Unknown => return Err(anyhow!("Cannot generate thumbnail for unknown media type")),
        };
        
        // Cache the result
        self.thumbnail_cache.lock().unwrap().insert(cache_key, thumbnail_path.clone());
        
        Ok(thumbnail_path)
    }
    
    /// Copy a file with progress reporting
    pub fn copy_file<F>(&self, source: &Path, destination: &Path, progress_callback: F) -> Result<()>
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        // Check if source exists
        if !source.exists() {
            return Err(anyhow!("Source file does not exist: {:?}", source));
        }
        
        // Create destination directory if it doesn't exist
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Get file size
        let file_size = fs::metadata(source)?.len();
        
        // Open source file
        let mut source_file = File::open(source)?;
        
        // Create destination file
        let mut dest_file = File::create(destination)?;
        
        // Copy with progress reporting
        let mut buffer = [0; 65536]; // 64KB buffer
        let mut bytes_copied = 0;
        
        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            dest_file.write_all(&buffer[..bytes_read])?;
            bytes_copied += bytes_read as u64;
            
            // Report progress
            progress_callback(bytes_copied, file_size);
        }
        
        Ok(())
    }
    
    /// Extract frames from a video file
    pub fn extract_frames(&self, video_path: &Path, output_dir: &Path, fps: f64) -> Result<Vec<PathBuf>> {
        // Check if video exists
        if !video_path.exists() {
            return Err(anyhow!("Video file does not exist: {:?}", video_path));
        }
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir)?;
        
        // Create GStreamer pipeline for frame extraction
        let pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! videorate ! video/x-raw,framerate={}/1 ! \
             videoconvert ! jpegenc quality=90 ! multifilesink location=\"{}/frame-%04d.jpg\"",
            video_path.to_str().unwrap(),
            fps,
            output_dir.to_str().unwrap()
        );
        
        let pipeline = gst::parse_launch(&pipeline_str)?;
        let bus = pipeline.bus().unwrap();
        
        // Start the pipeline
        pipeline.set_state(gst::State::Playing)?;
        
        // Wait for EOS or error
        let mut frame_paths = Vec::new();
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            match msg.view() {
                gst::MessageView::Eos(..) => {
                    // End of stream
                    break;
                },
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!("Error extracting frames: {}", err.error()));
                },
                _ => (),
            }
        }
        
        // Clean up
        pipeline.set_state(gst::State::Null)?;
        
        // Collect frame paths
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "jpg") {
                frame_paths.push(path);
            }
        }
        
        // Sort frames by name
        frame_paths.sort();
        
        Ok(frame_paths)
    }
    
    /// Clean up temporary files
    pub fn cleanup(&self) -> Result<()> {
        // Clear caches
        self.media_info_cache.lock().unwrap().clear();
        self.thumbnail_cache.lock().unwrap().clear();
        
        // Remove temporary directory
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)?;
        }
        
        Ok(())
    }
    
    /// Determine media type based on file extension
    fn determine_media_type(&self, path: &Path) -> MediaType {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            
            // Video extensions
            if ["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"].contains(&ext.as_str()) {
                return MediaType::Video;
            }
            
            // Audio extensions
            if ["mp3", "wav", "ogg", "flac", "aac", "m4a"].contains(&ext.as_str()) {
                return MediaType::Audio;
            }
            
            // Image extensions
            if ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"].contains(&ext.as_str()) {
                return MediaType::Image;
            }
        }
        
        MediaType::Unknown
    }
    
    /// Extract media information using GStreamer
    fn extract_media_info_gstreamer(&self, path: &Path, info: &mut MediaInfo) -> Result<()> {
        // Create discoverer
        let timeout = 5 * gst::ClockTime::SECOND;
        let discoverer = gst_pbutils::Discoverer::new(timeout)
            .map_err(|_| anyhow!("Failed to create GStreamer discoverer"))?;
        
        // Discover media info
        let uri = format!("file://{}", path.to_str().unwrap());
        let discover_info = discoverer.discover_uri(&uri)
            .map_err(|err| anyhow!("Failed to discover media info: {}", err))?;
        
        // Extract duration
        let duration = discover_info.duration();
        if duration != gst::ClockTime::NONE {
            info.duration = Some(duration.seconds() as f64 + (duration.nanoseconds() as f64 / 1_000_000_000.0));
        }
        
        // Extract video information
        if let Some(video_info) = discover_info.video_streams().get(0) {
            info.width = Some(video_info.width());
            info.height = Some(video_info.height());
            
            // Extract frame rate
            let fps_num = video_info.framerate_num();
            let fps_denom = video_info.framerate_denom();
            if fps_denom > 0 {
                info.frame_rate = Some(fps_num as f64 / fps_denom as f64);
            }
            
            // Extract codec
            if let Some(caps) = video_info.caps() {
                if let Some(s) = caps.structure(0) {
                    info.codec = s.name().to_string().into();
                }
            }
        }
        
        // Extract audio information
        if let Some(audio_info) = discover_info.audio_streams().get(0) {
            info.sample_rate = Some(audio_info.sample_rate());
            info.channels = Some(audio_info.channels());
            
            // Extract codec
            if info.codec.is_none() {
                if let Some(caps) = audio_info.caps() {
                    if let Some(s) = caps.structure(0) {
                        info.codec = s.name().to_string().into();
                    }
                }
            }
        }
        
        // Extract metadata tags
        for tag_list in discover_info.tags() {
            for tag in tag_list.iter() {
                if let Some(value) = tag_list.get::<gst::tags::TagValue>(tag) {
                    info.metadata.insert(tag.to_string(), value.get().to_string());
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract image information
    fn extract_image_info(&self, path: &Path, info: &mut MediaInfo) -> Result<()> {
        // Create GStreamer pipeline to get image dimensions
        let pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! imagefreeze ! fakesink",
            path.to_str().unwrap()
        );
        
        let pipeline = gst::parse_launch(&pipeline_str)?;
        let bus = pipeline.bus().unwrap();
        
        // Start the pipeline
        pipeline.set_state(gst::State::Paused)?;
        
        // Wait for pipeline to be ready
        let mut width = None;
        let mut height = None;
        
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            match msg.view() {
                gst::MessageView::StreamsSelected(streams) => {
                    let stream_info = streams.stream_collection().get(0).unwrap();
                    if let Some(caps) = stream_info.caps() {
                        if let Some(s) = caps.structure(0) {
                            width = s.get::<i32>("width").ok();
                            height = s.get::<i32>("height").ok();
                        }
                    }
                    break;
                },
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!("Error extracting image info: {}", err.error()));
                },
                gst::MessageView::StateChanged(state_changed) => {
                    if state_changed.src().map(|s| s == pipeline.upcast_ref::<gst::Object>()).unwrap_or(false) 
                        && state_changed.current() == gst::State::Paused 
                        && state_changed.pending() == gst::State::VoidPending {
                        break;
                    }
                },
                _ => (),
            }
        }
        
        // Clean up
        pipeline.set_state(gst::State::Null)?;
        
        // Update info
        if let Some(w) = width {
            info.width = Some(w as u32);
        }
        if let Some(h) = height {
            info.height = Some(h as u32);
        }
        
        Ok(())
    }
    
    /// Generate video thumbnail
    fn generate_video_thumbnail(&self, path: &Path, options: &ThumbnailOptions) -> Result<PathBuf> {
        // Create output path
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let thumbnail_path = self.temp_dir.join(format!(
            "{}-thumb-{}x{}-{}.jpg",
            file_stem,
            options.width,
            options.height,
            options.position.unwrap_or(0.0)
        ));
        
        // Create GStreamer pipeline for thumbnail extraction
        let position_ns = (options.position.unwrap_or(0.0) * 1_000_000_000.0) as i64;
        let pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! videoconvert ! videoscale ! \
             video/x-raw,width={},height={} ! jpegenc quality={} ! filesink location=\"{}\"",
            path.to_str().unwrap(),
            options.width,
            options.height,
            options.quality,
            thumbnail_path.to_str().unwrap()
        );
        
        let pipeline = gst::parse_launch(&pipeline_str)?;
        
        // Set position for seeking
        pipeline.set_state(gst::State::Paused)?;
        
        // Wait for pipeline to be ready
        let bus = pipeline.bus().unwrap();
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            match msg.view() {
                gst::MessageView::StateChanged(state_changed) => {
                    if state_changed.src().map(|s| s == pipeline.upcast_ref::<gst::Object>()).unwrap_or(false) 
                        && state_changed.current() == gst::State::Paused 
                        && state_changed.pending() == gst::State::VoidPending {
                        break;
                    }
                },
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!("Error generating thumbnail: {}", err.error()));
                },
                _ => (),
            }
        }
        
        // Seek to position
        if position_ns > 0 {
            pipeline.seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                gst::ClockTime::from_nseconds(position_ns as u64),
            )?;
        }
        
        // Wait for seek to complete
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            match msg.view() {
                gst::MessageView::Eos(..) => break,
                gst::MessageView::SeekDone(..) => break,
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!("Error seeking: {}", err.error()));
                },
                _ => (),
            }
        }
        
        // Play for a short time to capture frame
        pipeline.set_state(gst::State::Playing)?;
        std::thread::sleep(Duration::from_millis(100));
        pipeline.set_state(gst::State::Paused)?;
        
        // Send EOS to flush buffers
        pipeline.send_event(gst::event::Eos::new());
        
        // Wait for EOS
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            match msg.view() {
                gst::MessageView::Eos(..) => break,
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!("Error generating thumbnail: {}", err.error()));
                },
                _ => (),
            }
        }
        
        // Clean up
        pipeline.set_state(gst::State::Null)?;
        
        // Check if thumbnail was created
        if !thumbnail_path.exists() {
            return Err(anyhow!("Failed to generate thumbnail"));
        }
        
        Ok(thumbnail_path)
    }
    
    /// Generate image thumbnail
    fn generate_image_thumbnail(&self, path: &Path, options: &ThumbnailOptions) -> Result<PathBuf> {
        // Create output path
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let thumbnail_path = self.temp_dir.join(format!(
            "{}-thumb-{}x{}.jpg",
            file_stem,
            options.width,
            options.height
        ));
        
        // Create GStreamer pipeline for image scaling
        let pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! videoconvert ! videoscale ! \
             video/x-raw,width={},height={} ! jpegenc quality={} ! filesink location=\"{}\"",
            path.to_str().unwrap(),
            options.width,
            options.height,
            options.quality,
            thumbnail_path.to_str().unwrap()
        );
        
        let pipeline = gst::parse_launch(&pipeline_str)?;
        let bus = pipeline.bus().unwrap();
        
        // Start the pipeline
        pipeline.set_state(gst::State::Playing)?;
        
        // Wait for EOS or error
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            match msg.view() {
                gst::MessageView::Eos(..) => break,
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!("Error generating thumbnail: {}", err.error()));
                },
                _ => (),
            }
        }
        
        // Clean up
        pipeline.set_state(gst::State::Null)?;
        
        // Check if thumbnail was created
        if !thumbnail_path.exists() {
            return Err(anyhow!("Failed to generate thumbnail"));
        }
        
        Ok(thumbnail_path)
    }
    
    /// Generate audio thumbnail (waveform image)
    fn generate_audio_thumbnail(&self, path: &Path, options: &ThumbnailOptions) -> Result<PathBuf> {
        // Create output path
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let thumbnail_path = self.temp_dir.join(format!(
            "{}-waveform-{}x{}.png",
            file_stem,
            options.width,
            options.height
        ));
        
        // Create GStreamer pipeline for waveform generation
        let pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! audioconvert ! \
             audiowaveform wave-mode=lines style=lines fill=true background-color=0x000000ff \
             foreground-color=0x00FF00FF scale-digitized=true ! \
             pngenc compression-level=6 ! filesink location=\"{}\"",
            path.to_str().unwrap(),
            thumbnail_path.to_str().unwrap()
        );
        
        let pipeline = gst::parse_launch(&pipeline_str)?;
        let bus = pipeline.bus().unwrap();
        
        // Start the pipeline
        pipeline.set_state(gst::State::Playing)?;
        
        // Wait for EOS or error
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(10)) {
            match msg.view() {
                gst::MessageView::Eos(..) => break,
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    
                    // Fall back to generic audio icon if waveform generation fails
                    return self.generate_generic_audio_thumbnail(options);
                },
                _ => (),
            }
        }
        
        // Clean up
        pipeline.set_state(gst::State::Null)?;
        
        // Check if thumbnail was created
        if !thumbnail_path.exists() {
            // Fall back to generic audio icon
            return self.generate_generic_audio_thumbnail(options);
        }
        
        Ok(thumbnail_path)
    }
    
    /// Generate a generic audio thumbnail (icon)
    fn generate_generic_audio_thumbnail(&self, options: &ThumbnailOptions) -> Result<PathBuf> {
        // Create output path
        let thumbnail_path = self.temp_dir.join(format!(
            "audio-icon-{}x{}.png",
            options.width,
            options.height
        ));
        
        // Create a simple audio icon (blue waveform on black background)
        let pipeline_str = format!(
            "videotestsrc pattern=black ! video/x-raw,width={},height={} ! \
             videooverlay text=\"Audio File\" font-desc=\"Sans 24\" ! \
             pngenc compression-level=6 ! filesink location=\"{}\"",
            options.width,
            options.height,
            thumbnail_path.to_str().unwrap()
        );
        
        let pipeline = gst::parse_launch(&pipeline_str)?;
        let bus = pipeline.bus().unwrap();
        
        // Start the pipeline
        pipeline.set_state(gst::State::Playing)?;
        
        // Wait for EOS or error
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            match msg.view() {
                gst::MessageView::Eos(..) => break,
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!("Error generating audio icon: {}", err.error()));
                },
                _ => (),
            }
        }
        
        // Send EOS to flush buffers
        pipeline.send_event(gst::event::Eos::new());
        
        // Wait for EOS
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            match msg.view() {
                gst::MessageView::Eos(..) => break,
                _ => (),
            }
        }
        
        // Clean up
        pipeline.set_state(gst::State::Null)?;
        
        // Check if thumbnail was created
        if !thumbnail_path.exists() {
            return Err(anyhow!("Failed to generate audio icon"));
        }
        
        Ok(thumbnail_path)
    }
}
