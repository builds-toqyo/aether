use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use gstreamer as gst;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_editing_services as ges;
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
            gst::filename_to_uri(path)
                .with_context(|| format!("Failed to create URI for path {}", path.display()))
                .map_err(|e| EditingError::ImportError(e.to_string()))?
        } else {
            let abs_path = std::env::current_dir()
                .with_context(|| "Failed to get current directory")
                .map_err(|e| EditingError::ImportError(e.to_string()))?
                .join(path);
            gst::filename_to_uri(&abs_path)
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
                if let Err(e) = self.create_proxy_media(&uri, format, &path_canon) {
                    warn!("Failed to create proxy: {}", e);

                }
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


            match ges::UriClipAsset::request_async(&uri, Some(&structure)) {
                Ok(()) => debug!("Successfully requested GES asset for {}", uri),
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

        let duration = info.get_duration().unwrap_or(0);
        debug!("Media duration: {} ns ({:.2} seconds)", duration, duration as f64 / 1_000_000_000.0);


        let tags = info.get_tags();
        debug!("Extracted {} tag sets", if tags.is_some() { "some" } else { "no" });


        let title = tags.as_ref().and_then(|t| t.get::<gst::tags::Title>().ok().map(|t| t.get().to_string()));
        if let Some(ref t) = title {
            debug!("Found title: {}", t);
        }


        let artist = tags.as_ref().and_then(|t| t.get::<gst::tags::Artist>().ok().map(|t| t.get().to_string()));
        let album = tags.as_ref().and_then(|t| t.get::<gst::tags::Album>().ok().map(|t| t.get().to_string()));
        let genre = tags.as_ref().and_then(|t| t.get::<gst::tags::Genre>().ok().map(|t| t.get().to_string()));
        let comment = tags.as_ref().and_then(|t| t.get::<gst::tags::Comment>().ok().map(|t| t.get().to_string()));
        let copyright = tags.as_ref().and_then(|t| t.get::<gst::tags::Copyright>().ok().map(|t| t.get().to_string()));
        let creation_date = tags.as_ref().and_then(|t| t.get::<gst::tags::DateTime>().ok().map(|t| t.get().to_string()));


        let container_format = info.get_container_mime_type().map(|s| s.to_string());
        if let Some(ref fmt) = container_format {
            debug!("Container format: {}", fmt);
        }


        let has_video = !info.get_video_streams().is_empty();
        let has_audio = !info.get_audio_streams().is_empty();
        debug!("Media contains video: {}, audio: {}", has_video, has_audio);

        let media_type = if has_video {
            MediaType::Video
        } else if has_audio {
            MediaType::Audio
        } else {
            MediaType::Unknown
        };


        debug!("Processing {} video streams", info.get_video_streams().len());
        let video_streams = info.get_video_streams().iter().enumerate().map(|(i, stream)| {
            debug!("Analyzing video stream {}", i);
            let caps = stream.get_caps().unwrap_or_else(|| gst::Caps::new_empty());


            let structure = if caps.size() > 0 { caps.structure(0) } else { None };
            if let Some(s) = structure {
                debug!("Video stream {} caps structure: {}", i, s.to_string());
            } else {
                warn!("Video stream {} has no caps structure", i);
            }

            let width = structure.and_then(|s| s.get::<i32>("width").ok()).unwrap_or(0);
            let height = structure.and_then(|s| s.get::<i32>("height").ok()).unwrap_or(0);
            debug!("Video dimensions: {}x{}", width, height);


            let frame_rate = if stream.get_framerate_denom() != 0 {
                let fr = stream.get_framerate_num() as f64 / stream.get_framerate_denom() as f64;
                debug!("Frame rate: {:.2} fps ({}/{}))", fr, stream.get_framerate_num(), stream.get_framerate_denom());
                fr
            } else {
                warn!("Stream {} has zero denominator for framerate, defaulting to 0.0", i);
                0.0
            };


            let aspect_ratio = if width > 0 && height > 0 {
                let ar = width as f64 / height as f64;
                debug!("Aspect ratio: {:.3}", ar);
                Some(ar)
            } else {
                None
            };


            let bitrate = stream.get_bitrate().filter(|&b| b > 0);
            if let Some(br) = bitrate {
                debug!("Bitrate: {} bps ({:.2} Mbps)", br, br as f64 / 1_000_000.0);
            }

            let codec = stream.get_codec().unwrap_or_else(|| "unknown".to_string());
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


        debug!("Processing {} audio streams", info.get_audio_streams().len());
        let audio_streams = info.get_audio_streams().iter().enumerate().map(|(i, stream)| {
            debug!("Analyzing audio stream {}", i);
            let sample_rate = stream.get_sample_rate();
            let channels = stream.get_channels();
            let codec = stream.get_codec().unwrap_or_else(|| "unknown".to_string());

            debug!("Audio: {} channels, {} Hz, codec: {}", channels, sample_rate, codec);

            AudioStreamInfo {
                index: i as i32,
                sample_rate,
                channels,
                codec_name: codec,
                bit_depth: None,
            }
        }).collect();


        let path = gst::filename_from_uri(uri)
            .with_context(|| format!("Failed to convert URI back to path: {}", uri))
            .map_err(|e| EditingError::ImportError(e.to_string()))?;

        let path_buf = PathBuf::from(&path);
        let file_size = std::fs::metadata(&path_buf).ok().map(|m| m.len());
        if let Some(size) = file_size {
            debug!("File size: {} bytes ({:.2} MB)", size, size as f64 / (1024.0 * 1024.0));
        }

        info!("Media analysis complete for {}", path);

        Ok(MediaInfo {
            path: path_buf,
            duration,
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
        
        // Create thumbnail directory if it doesn't exist
        let thumb_dir = path.parent()
            .ok_or_else(|| EditingError::ImportError("Invalid path".to_string()))?
            .join(".thumbnails");
        
        std::fs::create_dir_all(&thumb_dir)
            .with_context(|| format!("Failed to create thumbnail directory: {}", thumb_dir.display()))
            .map_err(|e| EditingError::ImportError(e.to_string()))?;
        
        // Generate 3 thumbnails at different positions (25%, 50%, 75%)
        let positions = [0.25, 0.5, 0.75];
        
        for (i, position) in positions.iter().enumerate() {
            let thumb_path = thumb_dir.join(format!("thumb_{}.jpg", i));
            
            // Check if thumbnail already exists
            if thumb_path.exists() {
                debug!("Thumbnail already exists: {}", thumb_path.display());
                continue;
            }
            
            debug!("Generating thumbnail at position {} for {}", position, path.display());
            
            // Create pipeline for thumbnail extraction
            let pipeline = gst::parse_launch(&format!(
                "uridecodebin uri={} ! videoconvert ! videoscale ! video/x-raw,width=320,height=180 ! jpegenc ! filesink location={}",
                uri,
                thumb_path.display()
            ))
                .with_context(|| "Failed to create thumbnail pipeline")
                .map_err(|e| EditingError::ImportError(e.to_string()))?;
            
            // Set state to playing
            pipeline.set_state(gst::State::Playing)
                .with_context(|| "Failed to set pipeline to playing")
                .map_err(|e| EditingError::ImportError(e.to_string()))?;
            
            // Wait for EOS or error
            let bus = pipeline.bus().expect("Pipeline has no bus");
            let timeout = 10 * gst::ClockTime::SECOND;
            
            match bus.timed_pop_filtered(timeout, &[gst::MessageType::Eos, gst::MessageType::Error]) {
                Some(msg) => {
                    match msg.view() {
                        gst::MessageView::Eos(_) => {
                            debug!("Thumbnail generation complete for position {}", position);
                        }
                        gst::MessageView::Error(err) => {
                            error!("Error during thumbnail generation: {}", err.error());
                            return Err(EditingError::ImportError(format!("Thumbnail generation failed: {}", err.error())));
                        }
                        _ => {}
                    }
                }
                None => {
                    warn!("Thumbnail generation timeout for position {}", position);
                }
            }
            
            // Clean up
            pipeline.set_state(gst::State::Null)
                .with_context(|| "Failed to set pipeline to null")
                .map_err(|e| EditingError::ImportError(e.to_string()))?;
        }
        
        info!("Thumbnail generation complete for {}", path.display());
        Ok(())
    }

    fn create_proxy_media(&self, uri: &str, format: &str, path: &Path) -> Result<(), EditingError> {
        use gst::prelude::*;
        
        debug!("Creating proxy media for {} with format {}", path.display(), format);
        
        // Determine proxy settings based on format
        let (resolution, bitrate) = match format {
            "720p" => ("1280x720", 2000000),
            "1080p" => ("1920x1080", 5000000),
            "4K" => ("3840x2160", 15000000),
            _ => ("1280x720", 2000000),
        };
        
        // Create proxy directory
        let proxy_dir = path.parent()
            .ok_or_else(|| EditingError::ImportError("Invalid path".to_string()))?
            .join(".proxies");
        
        std::fs::create_dir_all(&proxy_dir)
            .with_context(|| format!("Failed to create proxy directory: {}", proxy_dir.display()))
            .map_err(|e| EditingError::ImportError(e.to_string()))?;
        
        let proxy_path = proxy_dir.join(format!("proxy_{}.mp4", path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("proxy")));
        
        // Check if proxy already exists
        if proxy_path.exists() {
            debug!("Proxy already exists: {}", proxy_path.display());
            return Ok(());
        }
        
        debug!("Creating proxy at: {}", proxy_path.display());
        
        // Create FFmpeg-style pipeline for proxy generation
        let pipeline_str = format!(
            "uridecodebin uri={} ! videoconvert ! videoscale ! video/x-raw,width={},height={} ! x264enc bitrate={} ! mp4mux ! filesink location={}",
            uri,
            resolution.split('x').next().unwrap_or("1280"),
            resolution.split('x').nth(1).unwrap_or("720"),
            bitrate,
            proxy_path.display()
        );
        
        let pipeline = gst::parse_launch(&pipeline_str)
            .with_context(|| "Failed to create proxy pipeline")
            .map_err(|e| EditingError::ImportError(e.to_string()))?;
        
        // Set state to playing
        pipeline.set_state(gst::State::Playing)
            .with_context(|| "Failed to set pipeline to playing")
            .map_err(|e| EditingError::ImportError(e.to_string()))?;
        
        // Wait for EOS or error with extended timeout for proxy generation
        let bus = pipeline.bus().expect("Pipeline has no bus");
        let timeout = 300 * gst::ClockTime::SECOND; // 5 minutes for proxy generation
        
        match bus.timed_pop_filtered(timeout, &[gst::MessageType::Eos, gst::MessageType::Error]) {
            Some(msg) => {
                match msg.view() {
                    gst::MessageView::Eos(_) => {
                        info!("Proxy creation complete: {}", proxy_path.display());
                    }
                    gst::MessageView::Error(err) => {
                        error!("Error during proxy creation: {}", err.error());
                        return Err(EditingError::ImportError(format!("Proxy creation failed: {}", err.error())));
                    }
                    _ => {}
                }
            }
            None => {
                warn!("Proxy creation timeout");
                return Err(EditingError::ImportError("Proxy creation timeout".to_string()));
            }
        }
        
        // Clean up
        pipeline.set_state(gst::State::Null)
            .with_context(|| "Failed to set pipeline to null")
            .map_err(|e| EditingError::ImportError(e.to_string()))?;
        
        info!("Proxy media created successfully: {}", proxy_path.display());
        Ok(())
    }


    pub fn get_proxy_path<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        let path = path.as_ref();
        let proxy_dir = path.parent()?.join(".proxies");
        let file_name = path.file_name()?.to_str()?;
        Some(proxy_dir.join(format!("proxy_{}.mp4", file_name)))
    }

    pub fn batch_import<P: AsRef<Path>>(&mut self, paths: Vec<P>, options: Option<ImportOptions>)
        -> Result<Vec<MediaInfo>, EditingError> {
        info!("Starting batch import of {} files", paths.len());
        
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        for (i, path) in paths.iter().enumerate() {
            info!("Importing file {}/{}: {}", i + 1, paths.len(), path.as_ref().display());
            
            match self.import_media(path, options.clone()) {
                Ok(media_info) => {
                    info!("Successfully imported: {}", path.as_ref().display());
                    results.push(media_info);
                }
                Err(e) => {
                    error!("Failed to import {}: {}", path.as_ref().display(), e);
                    errors.push((path.as_ref().to_path_buf(), e));
                }
            }
        }
        
        info!("Batch import complete: {} successful, {} failed", results.len(), errors.len());
        
        if !errors.is_empty() {
            warn!("Batch import had {} errors:", errors.len());
            for (path, error) in &errors {
                warn!("  {}: {}", path.display(), error);
            }
        }
        
        Ok(results)
    }

    pub fn validate_media<P: AsRef<Path>>(&self, path: P) -> Result<bool, EditingError> {
        let path = path.as_ref();
        
        // Check if file exists
        if !path.exists() {
            return Ok(false);
        }
        
        // Check if file is readable
        if !path.metadata().is_ok() {
            return Ok(false);
        }
        
        // Try to create URI to validate it's a valid media file
        let uri = match gst::filename_to_uri(path) {
            Ok(uri) => uri,
            Err(_) => return Ok(false),
        };
        
        // Quick validation by attempting to discover the media
        let timeout = 2 * gst::ClockTime::SECOND;
        let discoverer = gst_pbutils::Discoverer::new(timeout)
            .with_context(|| "Failed to create discoverer for validation")
            .map_err(|e| EditingError::ImportError(e.to_string()))?;
        
        match discoverer.discover_uri(&uri) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }


    pub fn get_ges_asset<P: AsRef<Path>>(&self, path: P) -> Option<ges::UriClipAsset> {

        let project = self.ges_project.as_ref()?;


        let path = path.as_ref();
        let uri = match if path.is_absolute() {
            gst::filename_to_uri(path)
        } else {
            let abs_path = match std::env::current_dir() {
                Ok(dir) => dir.join(path),
                Err(e) => {
                    error!("Failed to get current directory: {}", e);
                    return None;
                }
            };
            gst::filename_to_uri(&abs_path)
        } {
            Ok(uri) => uri,
            Err(e) => {
                error!("Failed to create URI for path {}: {}", path.display(), e);
                return None;
            }
        };


        if let Some(asset) = project.get_asset(&uri) {
            debug!("Found existing GES asset for {}", uri);
            return asset.downcast::<ges::UriClipAsset>().ok();
        }


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
        let asset = self.get_ges_asset(path)?;

        match asset.extract() {
            Ok(clip) => {
                debug!("Created GES clip from asset");
                Some(clip)
            },
            Err(e) => {
                error!("Failed to extract clip from asset: {}", e);
                None
            }
        }
    }
}
