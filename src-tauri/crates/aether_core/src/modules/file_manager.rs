use anyhow::{anyhow, Result};
use gstreamer as gst;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_pbutils::prelude::DiscovererStreamInfoExt;
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
        gst::init()?;

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

            },
        }

        self.media_info_cache.lock().unwrap().insert(path.to_path_buf(), info.clone());

        Ok(info)
    }


    pub fn generate_thumbnail(&self, path: &Path, options: Option<ThumbnailOptions>) -> Result<PathBuf> {
        let options = options.unwrap_or_default();


        let cache_key = path.to_path_buf();
        if let Some(thumbnail_path) = self.thumbnail_cache.lock().unwrap().get(&cache_key) {
            if thumbnail_path.exists() {
                return Ok(thumbnail_path.clone());
            }
        }


        let media_type = self.determine_media_type(path);


        let thumbnail_path = match media_type {
            MediaType::Video => self.generate_video_thumbnail(path, &options)?,
            MediaType::Image => self.generate_image_thumbnail(path, &options)?,
            MediaType::Audio => self.generate_audio_thumbnail(path, &options)?,
            MediaType::Unknown => return Err(anyhow!("Cannot generate thumbnail for unknown media type")),
        };


        self.thumbnail_cache.lock().unwrap().insert(cache_key, thumbnail_path.clone());

        Ok(thumbnail_path)
    }


    pub fn copy_file<F>(&self, source: &Path, destination: &Path, progress_callback: F) -> Result<()>
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        // Check if source exists
        if !source.exists() {
            return Err(anyhow!("Source path does not exist: {}", source));
        }

        // Create destination directory if it doesn't exist
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }


        let file_size = fs::metadata(source)?.len();


        let mut source_file = File::open(source)?;


        let mut dest_file = File::create(destination)?;


        let mut buffer = [0; 65536];
        let mut bytes_copied = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            dest_file.write_all(&buffer[..bytes_read])?;
            bytes_copied += bytes_read as u64;


            progress_callback(bytes_copied, file_size);
        }

        Ok(())
    }


    pub fn extract_frames(&self, video_path: &Path, output_dir: &Path, fps: f64) -> Result<Vec<PathBuf>> {

        if !video_path.exists() {
            return Err(anyhow!("Video file does not exist: {:?}", video_path));
        }


        fs::create_dir_all(output_dir)?;


        let _pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! videorate ! video/x-raw,framerate={}/1 ! \
             videoconvert ! jpegenc quality=90 ! multifilesink location=\"{}/frame-%04d.jpg\"",
            video_path.to_str().unwrap(),
            fps,
            output_dir.to_str().unwrap()
        );

        // TODO: GStreamer parse_launch API has changed - need to update to use manual pipeline construction
        return Err(anyhow::anyhow!("parse_launch not available in current GStreamer version").into());
    }

    pub fn cleanup(&self) -> Result<()> {

        self.media_info_cache.lock().unwrap().clear();
        self.thumbnail_cache.lock().unwrap().clear();


        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)?;
        }

        Ok(())
    }

    fn determine_media_type(&self, path: &Path) -> MediaType {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();


            if ["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"].contains(&ext.as_str()) {
                return MediaType::Video;
            }

            if ["mp3", "wav", "ogg", "flac", "aac", "m4a"].contains(&ext.as_str()) {
                return MediaType::Audio;
            }

            if ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"].contains(&ext.as_str()) {
                return MediaType::Image;
            }
        }

        MediaType::Unknown
    }

    fn extract_media_info_gstreamer(&self, path: &Path, info: &mut MediaInfo) -> Result<()> {

        let timeout = 5 * gst::ClockTime::SECOND;
        let discoverer = gst_pbutils::Discoverer::new(timeout)
            .map_err(|_| anyhow!("Failed to create GStreamer discoverer"))?;

        let uri = format!("file://{}", path.to_str().unwrap());
        let discover_info = discoverer.discover_uri(&uri)
            .map_err(|err| anyhow!("Failed to discover media info: {}", err))?;

        if let Some(duration) = discover_info.duration() {
            info.duration = Some(duration.seconds() as f64 + (duration.nseconds() as f64 / 1_000_000_000.0));
        }


        if let Some(video_info) = discover_info.video_streams().get(0) {
            info.width = Some(video_info.width());
            info.height = Some(video_info.height());


            if let Some(framerate) = video_info.framerate() {
                info.frame_rate = Some(framerate.numer() as f64 / framerate.denom() as f64);
            }


            if let Some(caps) = video_info.caps() {
                if let Some(s) = caps.structure(0) {
                    info.codec = s.name().to_string().into();
                }
            }
        }


        if let Some(audio_info) = discover_info.audio_streams().get(0) {
            info.sample_rate = Some(audio_info.sample_rate());
            info.channels = Some(audio_info.channels());


            if info.codec.is_none() {
                if let Some(caps) = audio_info.caps() {
                    if let Some(s) = caps.structure(0) {
                        info.codec = s.name().to_string().into();
                    }
                }
            }
        }

        for tag_list in discover_info.tags() {
            for (tag, value) in tag_list.iter() {
                if let Ok(serialized) = value.serialize() {
                    info.metadata.insert(tag.to_string(), serialized.to_string());
                }
            }
        }

        Ok(())
    }

    fn extract_image_info(&self, path: &Path, _info: &mut MediaInfo) -> Result<()> {
        let _pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! imagefreeze ! fakesink",
            path.to_str().unwrap()
        );

        // TODO: GStreamer parse_launch API has changed - need to update to use manual pipeline construction
        return Err(anyhow::anyhow!("parse_launch not available in current GStreamer version").into());
    }

    fn generate_video_thumbnail(&self, path: &Path, options: &ThumbnailOptions) -> Result<PathBuf> {

        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let thumbnail_path = self.temp_dir.join(format!(
            "{}-thumb-{}x{}-{}.jpg",
            file_stem,
            options.width,
            options.height,
            options.position.unwrap_or(0.0)
        ));

        let _position_ns = (options.position.unwrap_or(0.0) * 1_000_000_000.0) as i64;
        let _pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! videoconvert ! videoscale ! \
             video/x-raw,width={},height={} ! jpegenc quality={} ! filesink location=\"{}\"",
            options.width,
            options.height,
            options.quality,
            thumbnail_path.to_str().unwrap()
        );

        // TODO: GStreamer parse_launch API has changed - need to update to use manual pipeline construction
        return Err(anyhow::anyhow!("parse_launch not available in current GStreamer version").into());
    }

    fn generate_audio_thumbnail(&self, path: &Path, options: &ThumbnailOptions) -> Result<PathBuf> {

        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let thumbnail_path = self.temp_dir.join(format!(
            "{}-waveform-{}x{}.png",
            file_stem,
            options.width,
            options.height
        ));

        let _pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin ! audioconvert ! \
             audiowaveform wave-mode=lines style=lines fill=true background-color=0x000000ff \
             foreground-color=0x00FF00FF scale-digitized=true ! \
             pngenc compression-level=6 ! filesink location=\"{}\"\",
            path.to_str().unwrap(),
            thumbnail_path.to_str().unwrap()
        );

        // TODO: GStreamer parse_launch API has changed - need to update to use manual pipeline construction
        return Err(anyhow::anyhow!("parse_launch not available in current GStreamer version").into());
    }


    fn generate_generic_audio_thumbnail(&self, options: &ThumbnailOptions) -> Result<PathBuf> {

        let thumbnail_path = self.temp_dir.join(format!(
            "audio-icon-{}x{}.png",
            options.width,
            options.height
        ));


        let _pipeline_str = format!(
            "videotestsrc pattern=black ! video/x-raw,width={},height={} ! \
             videooverlay text=\"Audio File\" font-desc=\"Sans 24\" ! \
             pngenc compression-level=6 ! filesink location=\"{}\"",
            options.width,
            options.height,
            thumbnail_path.to_str().unwrap()
        );

        // TODO: GStreamer parse_launch API has changed - need to update to use manual pipeline construction
        return Err(anyhow::anyhow!("parse_launch not available in current GStreamer version").into());
    }
}
