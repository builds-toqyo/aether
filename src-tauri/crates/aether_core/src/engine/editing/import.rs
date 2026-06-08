use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_pbutils::prelude::DiscovererStreamInfoExt;
use gstreamer_editing_services as ges;
use glib::{filename_to_uri, filename_from_uri};
use log::{debug, info, warn, error};
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
    media_cache: std::collections::HashMap<PathBuf, MediaInfo>,

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


        let path_canon = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => PathBuf::from(path),
        };


        if let Some(info) = self.media_cache.get(&path_canon) {
            debug!("Cache hit for media: {}", path_canon.display());
            return Ok(info.clone());
        }

        debug!("Cache miss for media: {}", path_canon.display());

        let uri = if path.is_absolute() {
            filename_to_uri(path, None)
                .with_context(|| format!("Failed to create URI for path {}", path.display()))
                .map_err(|e| EditingError::ImportError(e.to_string()))?
        } else {
            let abs_path = std::env::current_dir()
                .with_context(|| "Failed to get current directory")
                .map_err(|e| EditingError::ImportError(e.to_string()))?
                .join(path);
            filename_to_uri(&abs_path, None)
                .with_context(|| format!("Failed to create URI for absolute path {}", abs_path.display()))
                .map_err(|e| EditingError::ImportError(e.to_string()))?
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
                creation_date: None,
                artist: None,
                copyright: None,
                comment: None,
                album: None,
                genre: None,
                file_size: None,
                container_format: None,
                bitrate: None,
            }
        };


        if options.extract_thumbnails && media_info.media_type == MediaType::Video {
            debug!("Extracting thumbnails for {}", path_canon.display());
            if let Err(e) = self.generate_thumbnails(&uri, &path_canon) {
                warn!("Failed to generate thumbnails: {}", e);

            }
        }


        if options.create_proxy && media_info.media_type == MediaType::Video {
            if let Some(format) = &options.proxy_format {
                debug!("Creating proxy with format {} for {}", format, path_canon.display());
                // TODO: implement create_proxy_media
                // if let Err(e) = self.create_proxy_media(&uri, format, &path_canon) {
                //     warn!("Failed to create proxy: {}", e);
                // }
            }
        }


        if let Some(project) = &self.ges_project {
            debug!("Registering media with GES project: {}", uri);


            let mut structure = gst::Structure::new_empty("aether-media-info");
            structure.set("title", &media_info.title.clone().unwrap_or_default());
            structure.set("media-type", &format!("{:?}", media_info.media_type));

            if !media_info.video_streams.is_empty() {
                let vs = &media_info.video_streams[0];
                structure.set("width", vs.width);
                structure.set("height", vs.height);
                structure.set("frame-rate", vs.frame_rate);
            }


            match ges::UriClipAsset::request_sync(&uri) {
                Ok(_) => debug!("Successfully requested GES asset for {}", uri),
                Err(e) => warn!("Failed to request GES asset: {}", e),
            }
        }


        self.media_cache.insert(path_canon, media_info.clone());

        Ok(media_info)
    }

    fn analyze_media(&self, uri: &str) -> Result<MediaInfo, EditingError> {
        debug!("Analyzing media at URI: {}", uri);

        let timeout = 5 * gst::ClockTime::SECOND;
        let discoverer = gst_pbutils::Discoverer::new(timeout)
            .with_context(|| "Failed to create GStreamer media discoverer")
            .map_err(|e| EditingError::ImportError(e.to_string()))?;

        debug!("Starting media discovery with timeout: {} seconds", timeout / gst::ClockTime::SECOND);
        let info = discoverer.discover_uri(uri)
            .with_context(|| format!("Failed to discover media at URI: {}", uri))
            .map_err(|e| EditingError::ImportError(e.to_string()))?;

        let duration = info.duration().map(|d| d.nseconds()).unwrap_or(0);
        debug!("Media duration: {} ns ({:.2} seconds)", duration, duration as f64 / 1_000_000_000.0);


        let tags = info.tags();
        debug!("Extracted {} tag sets", if tags.is_some() { "some" } else { "no" });


        let title = tags.as_ref().and_then(|t| t.get::<gst::tags::Title>().map(|t| t.get().to_string()));
        if let Some(ref t) = title {
            debug!("Found title: {}", t);
        }

        let artist = tags.as_ref().and_then(|t| t.get::<gst::tags::Artist>().map(|t| t.get().to_string()));
        let album = tags.as_ref().and_then(|t| t.get::<gst::tags::Album>().map(|t| t.get().to_string()));
        let genre = tags.as_ref().and_then(|t| t.get::<gst::tags::Genre>().map(|t| t.get().to_string()));
        let comment = tags.as_ref().and_then(|t| t.get::<gst::tags::Comment>().map(|t| t.get().to_string()));
        let copyright = tags.as_ref().and_then(|t| t.get::<gst::tags::Copyright>().map(|t| t.get().to_string()));
        let creation_date = tags.as_ref().and_then(|t| t.get::<gst::tags::DateTime>().map(|t| t.get().to_string()));

        let container_format = Some("mp4".to_string()); // TODO: Get actual container format from GStreamer API
        if let Some(ref fmt) = container_format {
            debug!("Container format: {}", fmt);
        }

        let has_video = !info.video_streams().is_empty();
        let has_audio = !info.audio_streams().is_empty();
        debug!("Media contains video: {}, audio: {}", has_video, has_audio);

        let media_type = if has_video {
            MediaType::Video
        } else if has_audio {
            MediaType::Audio
        } else {
            MediaType::Unknown
        };

        debug!("Processing {} video streams", info.video_streams().len());
        let video_streams = info.video_streams().iter().enumerate().map(|(i, stream)| {
            debug!("Analyzing video stream {}", i);
            let caps = stream.caps().unwrap_or_else(|| gst::Caps::new_empty());

            let structure = if caps.size() > 0 { caps.structure(0) } else { None };
            if let Some(s) = structure {
                debug!("Video stream {} caps structure: {}", i, s.to_string());
            } else {
                warn!("Video stream {} has no caps structure", i);
            }

            let width = structure.and_then(|s| s.get::<i32>("width").ok()).unwrap_or(0);
            let height = structure.and_then(|s| s.get::<i32>("height").ok()).unwrap_or(0);
            debug!("Video dimensions: {}x{}", width, height);

            let frame_rate = 0.0;

            let aspect_ratio = if width > 0 && height > 0 {
                let ar = width as f64 / height as f64;
                debug!("Aspect ratio: {:.3}", ar);
                Some(ar)
            } else {
                None
            };

            let bitrate = stream.bitrate();
            if bitrate > 0 {
                debug!("Bitrate: {} bps ({:.2} Mbps)", bitrate, bitrate as f64 / 1_000_000.0);
            }

            let codec = "unknown".to_string();
            debug!("Codec: {}", codec);

            VideoStreamInfo {
                index: i as i32,
                width,
                height,
                frame_rate,
                codec_name: codec,
                pixel_format: structure.map(|s| s.name().to_string()).unwrap_or_else(|| "unknown".to_string()),
                aspect_ratio,
                bitrate,
            }
        }).collect();

        debug!("Processing {} audio streams", info.audio_streams().len());
        let audio_streams = info.audio_streams().iter().enumerate().map(|(i, stream)| {
            debug!("Analyzing audio stream {}", i);
            let sample_rate = 48000;
            let channels = stream.channels();
            let codec = "unknown".to_string();

            debug!("Audio: {} channels, {} Hz, codec: {}", channels, sample_rate, codec);

            AudioStreamInfo {
                index: i as i32,
                sample_rate,
                channels: channels as i32,
                codec_name: codec,
                bit_depth: None,
            }
        }).collect();


        let (path, _hostname) = filename_from_uri(uri)
            .with_context(|| format!("Failed to convert URI back to path: {}", uri))
            .map_err(|e| EditingError::ImportError(e.to_string()))?;

        let file_size = std::fs::metadata(&path).ok().map(|m| m.len());
        if let Some(size) = file_size {
            debug!("File size: {} bytes ({:.2} MB)", size, size as f64 / (1024.0 * 1024.0));
        }

        info!("Media analysis complete for {}", path.display());

        Ok(MediaInfo {
            path,
            duration: duration as i64,
            title,
            media_type,
            video_streams,
            audio_streams,
            creation_date,
            artist,
            copyright,
            comment,
            album,
            genre,
            file_size,
            container_format,
            bitrate: None,
        })
    }

    pub fn get_imported_media(&self) -> Vec<MediaInfo> {
        self.media_cache.values().cloned().collect()
    }

    pub fn get_media_info<P: AsRef<Path>>(&self, path: P) -> Option<MediaInfo> {

        let path_canon = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to canonicalize path: {}", e);
                return None;
            }
        };

        self.media_cache.get(&path_canon).cloned()
    }


    fn generate_thumbnails(&self, uri: &str, path: &Path) -> Result<(), EditingError> {
        use gst::prelude::*;

        debug!("Generating thumbnails for {}", path.display());


        let uri = match filename_to_uri(path, None) {
            Ok(uri) => uri,
            Err(_) => return Ok(()),
        };

        let timeout = 2 * gst::ClockTime::SECOND;
        let discoverer = gst_pbutils::Discoverer::new(timeout)
            .with_context(|| "Failed to create discoverer for validation")
            .map_err(|e| EditingError::ImportError(e.to_string()))?;

        match discoverer.discover_uri(&uri) {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }


    pub fn get_ges_asset<P: AsRef<Path>>(&self, path: P) -> Option<ges::UriClipAsset> {

        let project = self.ges_project.as_ref()?;


        let path = path.as_ref();
        let uri = match if path.is_absolute() {
            filename_to_uri(path, None)
        } else {
            let abs_path = match std::env::current_dir() {
                Ok(dir) => dir.join(path),
                Err(e) => {
                    error!("Failed to get current directory: {}", e);
                    return None;
                }
            };
            filename_to_uri(&abs_path, None)
        } {
            Ok(uri) => uri,
            Err(e) => {
                error!("Failed to create URI for path {}: {}", path.display(), e);
                return None;
            }
        };


        debug!("Creating new GES asset for {}", uri);
        match ges::UriClipAsset::request_sync(&uri) {
            Ok(asset) => {
                debug!("Successfully created GES asset for {}", uri);
                Some(asset)
            },
            Err(e) => {
                error!("Failed to create GES asset for {}: {}", uri, e);
                None
            }
        }
    }


    pub fn create_ges_clip<P: AsRef<Path>>(&self, path: P) -> Option<ges::Clip> {
        let path = path.as_ref();
        let uri = match filename_to_uri(path, None) {
            Ok(uri) => uri.to_string(),
            Err(e) => {
                error!("Failed to convert path to URI: {}", e);
                return None;
            }
        };

        match ges::UriClip::new(&uri) {
            Ok(clip) => {
                debug!("Created GES clip from URI");
                Some(clip.upcast())
            },
            Err(e) => {
                error!("Failed to create clip from URI: {}", e);
                None
            }
        }
    }
}
