use std::path::{Path, PathBuf};
use anyhow::Result;
use gstreamer as gst;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_editing_services as ges;
use crate::engine::editing::types::{
    EditingError, MediaInfo, MediaType, VideoStreamInfo, AudioStreamInfo
};

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub analyze: bool,
    
    pub extract_thumbnails: bool,
    
    pub create_proxy: bool,
    
    pub proxy_format: Option<String>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            analyze: true,
            extract_thumbnails: true,
            create_proxy: false,
            proxy_format: None,
        }
    }
}

pub struct MediaImporter {
    media_cache: std::collections::HashMap<String, MediaInfo>,
    
    ges_project: Option<ges::Project>,
}

impl MediaImporter {
    pub fn new() -> Result<Self, EditingError> {
        Ok(Self {
            media_cache: std::collections::HashMap::new(),
            ges_project: None,
        })
    }
    
    pub fn set_ges_project(&mut self, project: ges::Project) {
        self.ges_project = Some(project);
    }
    
    pub fn import_media<P: AsRef<Path>>(&mut self, path: P, options: Option<ImportOptions>) 
        -> Result<MediaInfo, EditingError> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();
        
        if let Some(info) = self.media_cache.get(&path_str) {
            return Ok(info.clone());
        }
        
        let uri = if path.is_absolute() {
            gst::filename_to_uri(path)?
        } else {
            let abs_path = std::env::current_dir()?.join(path);
            gst::filename_to_uri(&abs_path)?
        };
        
        let options = options.unwrap_or_default();
        let media_info = if options.analyze {
            self.analyze_media(&uri)?
        } else {
            MediaInfo {
                path: PathBuf::from(path),
                duration: 0,
                title: None,
                media_type: MediaType::Unknown,
                video_streams: Vec::new(),
                audio_streams: Vec::new(),
            }
        };
        
        if let Some(project) = &self.ges_project {
            let _ = ges::UriClipAsset::request_async(&uri, None::<gst::StructureRef>);
        }
        
        self.media_cache.insert(path_str, media_info.clone());
        
        Ok(media_info)
    }
    
    fn analyze_media(&self, uri: &str) -> Result<MediaInfo, EditingError> {
        let timeout = 5 * gst::ClockTime::SECOND;
        let discoverer = gst_pbutils::Discoverer::new(timeout)
            .map_err(|e| EditingError::ImportError(format!("Failed to create discoverer: {}", e)))?;
        
        let info = discoverer.discover_uri(uri)
            .map_err(|e| EditingError::ImportError(format!("Failed to discover media: {}", e)))?;
        
        let duration = info.get_duration().unwrap_or(0);
        let tags = info.get_tags();
        
        let title = tags.and_then(|t| t.get::<gst::tags::Title>().ok().map(|t| t.get().to_string()));
        
        let has_video = !info.get_video_streams().is_empty();
        let has_audio = !info.get_audio_streams().is_empty();
        
        let media_type = if has_video {
            MediaType::Video
        } else if has_audio {
            MediaType::Audio
        } else {
            MediaType::Unknown
        };
        
        let video_streams = info.get_video_streams().iter().enumerate().map(|(i, stream)| {
            let caps = stream.get_caps().unwrap_or_else(|| gst::Caps::new_empty());
            let structure = caps.structure(0);
            
            let width = structure.and_then(|s| s.get::<i32>("width").ok()).unwrap_or(0);
            let height = structure.and_then(|s| s.get::<i32>("height").ok()).unwrap_or(0);
            
            let frame_rate = stream.get_framerate_num() as f64 / stream.get_framerate_denom() as f64;
            
            VideoStreamInfo {
                index: i as i32,
                width,
                height,
                frame_rate,
                codec_name: stream.get_codec().unwrap_or_else(|| "unknown".to_string()),
                pixel_format: structure.map(|s| s.name().to_string()).unwrap_or_else(|| "unknown".to_string()),
            }
        }).collect();
        
        let audio_streams = info.get_audio_streams().iter().enumerate().map(|(i, stream)| {
            AudioStreamInfo {
                index: i as i32,
                sample_rate: stream.get_sample_rate(),
                channels: stream.get_channels(),
                codec_name: stream.get_codec().unwrap_or_else(|| "unknown".to_string()),
                bit_depth: None, // Not directly available from discoverer
            }
        }).collect();
        
        let path = gst::filename_from_uri(uri)
            .map_err(|e| EditingError::ImportError(format!("Failed to convert URI to path: {}", e)))?;
        
        Ok(MediaInfo {
            path: PathBuf::from(path),
            duration,
            title,
            media_type,
            video_streams,
            audio_streams,
        })
    }
    
    pub fn get_imported_media(&self) -> Vec<MediaInfo> {
        self.media_cache.values().cloned().collect()
    }
    
    pub fn get_media_info<P: AsRef<Path>>(&self, path: P) -> Option<MediaInfo> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.media_cache.get(&path_str).cloned()
    }
}
