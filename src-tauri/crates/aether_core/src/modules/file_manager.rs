use anyhow::{anyhow, Result};
use gstreamer as gst;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_pbutils::prelude::DiscovererStreamInfoExt;
use gst::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
            return Err(anyhow!("Source path does not exist: {}", source.display()));
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

        let pattern = format!("{}/frame-%04d.jpg", output_dir.to_str().unwrap());
        let pipeline = gst::Pipeline::new();

        let filesrc = gst::ElementFactory::make("filesrc")
            .property("location", video_path.to_str().unwrap())
            .build()
            .map_err(|e| anyhow!("Failed to create filesrc: {}", e))?;

        let decodebin = gst::ElementFactory::make("decodebin")
            .build()
            .map_err(|e| anyhow!("Failed to create decodebin: {}", e))?;

        let videorate = gst::ElementFactory::make("videorate")
            .build()
            .map_err(|e| anyhow!("Failed to create videorate: {}", e))?;

        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", gst::Caps::builder("video/x-raw")
                .field("framerate", gst::Fraction::new(fps as i32, 1))
                .build())
            .build()
            .map_err(|e| anyhow!("Failed to create capsfilter: {}", e))?;

        let videoconvert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|e| anyhow!("Failed to create videoconvert: {}", e))?;

        let jpegenc = gst::ElementFactory::make("jpegenc")
            .property("quality", 90i32)
            .build()
            .map_err(|e| anyhow!("Failed to create jpegenc: {}", e))?;

        let multifilesink = gst::ElementFactory::make("multifilesink")
            .property("location", pattern.as_str())
            .build()
            .map_err(|e| anyhow!("Failed to create multifilesink: {}", e))?;

        pipeline.add_many([&filesrc, &decodebin, &videorate, &capsfilter, &videoconvert, &jpegenc, &multifilesink])
            .map_err(|e| anyhow!("Failed to add elements: {}", e))?;

        filesrc.link(&decodebin)
            .map_err(|e| anyhow!("Failed to link filesrc to decodebin: {}", e))?;

        // Link decodebin to downstream chain via pad-added signal
        let videorate_ref = videorate.clone();
        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = videorate_ref.static_pad("sink");
            if let Some(sink_pad) = sink_pad {
                if sink_pad.is_linked() {
                    return;
                }
                let caps = src_pad.current_caps();
                if let Some(caps) = caps {
                    let s = caps.structure(0);
                    if let Some(s) = s {
                        if s.name().starts_with("video/") {
                            let _ = src_pad.link(&sink_pad);
                        }
                    }
                }
            }
        });

        gst::Element::link_many([&videorate, &capsfilter, &videoconvert, &jpegenc, &multifilesink])
            .map_err(|e| anyhow!("Failed to link downstream chain: {}", e))?;

        let bus = pipeline.bus().ok_or_else(|| anyhow!("Pipeline has no bus"))?;
        pipeline.set_state(gst::State::Playing)
            .map_err(|e| anyhow!("Failed to start pipeline: {}", e))?;

        let msg = bus.timed_pop_filtered(
            gst::ClockTime::from_seconds(60),
            &[gst::MessageType::Error, gst::MessageType::Eos],
        );

        pipeline.set_state(gst::State::Null)
            .map_err(|e| anyhow!("Failed to stop pipeline: {}", e))?;

        match msg {
            Some(msg) => match msg.view() {
                gst::MessageView::Error(err) => Err(anyhow!("Pipeline error: {}", err.error())),
                gst::MessageView::Eos(_) => {
                    // Collect generated frame paths
                    let mut frames = Vec::new();
                    for entry in fs::read_dir(output_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("jpg") {
                            frames.push(path);
                        }
                    }
                    frames.sort();
                    Ok(frames)
                }
                _ => Err(anyhow!("Unexpected pipeline message")),
            },
            None => Err(anyhow!("Pipeline timed out")),
        }
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


            let framerate = video_info.framerate();
            info.frame_rate = Some(framerate.numer() as f64 / framerate.denom() as f64);


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
        let pipeline = gst::Pipeline::new();
        let filesrc = gst::ElementFactory::make("filesrc")
            .property("location", path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesrc: {}", e))?;
        let decodebin = gst::ElementFactory::make("decodebin")
            .build().map_err(|e| anyhow!("decodebin: {}", e))?;
        let imagefreeze = gst::ElementFactory::make("imagefreeze")
            .build().map_err(|e| anyhow!("imagefreeze: {}", e))?;
        let fakesink = gst::ElementFactory::make("fakesink")
            .build().map_err(|e| anyhow!("fakesink: {}", e))?;

        pipeline.add_many([&filesrc, &decodebin, &imagefreeze, &fakesink])
            .map_err(|e| anyhow!("add: {}", e))?;
        filesrc.link(&decodebin).map_err(|e| anyhow!("link: {}", e))?;

        decodebin.connect_pad_added(move |_, src_pad| {
            if let Some(sink) = imagefreeze.static_pad("sink") {
                if !sink.is_linked() {
                    if let Some(caps) = src_pad.current_caps() {
                        if let Some(s) = caps.structure(0) {
                            if s.name().starts_with("image/") || s.name().starts_with("video/") {
                                let _ = src_pad.link(&sink);
                            }
                        }
                    }
                }
            }
        });
        imagefreeze.link(&fakesink).map_err(|e| anyhow!("link: {}", e))?;

        let bus = pipeline.bus().ok_or_else(|| anyhow!("no bus"))?;
        pipeline.set_state(gst::State::Playing).map_err(|e| anyhow!("play: {}", e))?;
        let msg = bus.timed_pop_filtered(gst::ClockTime::from_seconds(10), &[gst::MessageType::Error, gst::MessageType::Eos]);
        pipeline.set_state(gst::State::Null).map_err(|e| anyhow!("stop: {}", e))?;

        match msg {
            Some(msg) => match msg.view() {
                gst::MessageView::Error(err) => Err(anyhow!("err: {}", err.error())),
                gst::MessageView::Eos(_) => Ok(()),
                _ => Err(anyhow!("unexpected")),
            },
            None => Err(anyhow!("timeout")),
        }
    }

    fn generate_video_thumbnail(&self, path: &Path, options: &ThumbnailOptions) -> Result<PathBuf> {
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let thumbnail_path = self.temp_dir.join(format!(
            "{}-thumb-{}x{}-{}.jpg", file_stem, options.width, options.height,
            options.position.unwrap_or(0.0)
        ));
        let position_ns = (options.position.unwrap_or(0.0) * 1_000_000_000.0) as i64;

        let pipeline = gst::Pipeline::new();
        let filesrc = gst::ElementFactory::make("filesrc")
            .property("location", path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesrc: {}", e))?;
        let decodebin = gst::ElementFactory::make("decodebin")
            .build().map_err(|e| anyhow!("decodebin: {}", e))?;
        let videoconvert = gst::ElementFactory::make("videoconvert")
            .build().map_err(|e| anyhow!("videoconvert: {}", e))?;
        let videoscale = gst::ElementFactory::make("videoscale")
            .build().map_err(|e| anyhow!("videoscale: {}", e))?;
        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", gst::Caps::builder("video/x-raw")
                .field("width", options.width as i32).field("height", options.height as i32).build())
            .build().map_err(|e| anyhow!("capsfilter: {}", e))?;
        let jpegenc = gst::ElementFactory::make("jpegenc")
            .property("quality", options.quality as i32)
            .build().map_err(|e| anyhow!("jpegenc: {}", e))?;
        let filesink = gst::ElementFactory::make("filesink")
            .property("location", thumbnail_path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesink: {}", e))?;

        pipeline.add_many([&filesrc, &decodebin, &videoconvert, &videoscale, &capsfilter, &jpegenc, &filesink])
            .map_err(|e| anyhow!("add: {}", e))?;
        filesrc.link(&decodebin).map_err(|e| anyhow!("link: {}", e))?;

        decodebin.connect_pad_added(move |_, src_pad| {
            if let Some(sink) = videoconvert.static_pad("sink") {
                if !sink.is_linked() {
                    if let Some(caps) = src_pad.current_caps() {
                        if let Some(s) = caps.structure(0) {
                            if s.name().starts_with("video/") {
                                let _ = src_pad.link(&sink);
                            }
                        }
                    }
                }
            }
        });
        gst::Element::link_many([&videoconvert, &videoscale, &capsfilter, &jpegenc, &filesink])
            .map_err(|e| anyhow!("downstream: {}", e))?;

        let bus = pipeline.bus().ok_or_else(|| anyhow!("no bus"))?;
        pipeline.set_state(gst::State::Playing).map_err(|e| anyhow!("play: {}", e))?;

        if let Some(clock) = pipeline.clock() {
            let _ = pipeline.seek(1.0, gst::SeekFlags::FLUSH, gst::SeekType::Set,
                gst::ClockTime::from_nseconds(position_ns as u64), gst::SeekType::None, gst::ClockTime::NONE);
        }

        let msg = bus.timed_pop_filtered(gst::ClockTime::from_seconds(30),
            &[gst::MessageType::Error, gst::MessageType::Eos]);
        pipeline.set_state(gst::State::Null).map_err(|e| anyhow!("stop: {}", e))?;

        match msg {
            Some(msg) => match msg.view() {
                gst::MessageView::Error(err) => Err(anyhow!("err: {}", err.error())),
                gst::MessageView::Eos(_) => Ok(thumbnail_path),
                _ => Err(anyhow!("unexpected")),
            },
            None => Err(anyhow!("timeout")),
        }
    }

    fn generate_image_thumbnail(&self, path: &Path, options: &ThumbnailOptions) -> Result<PathBuf> {
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let _thumbnail_path = self.temp_dir.join(format!(
            "{}-thumbnail-{}x{}.png",
            file_stem,
            options.width,
            options.height
        ));
        // TODO: implement image thumbnail generation
        return Err(anyhow!("Image thumbnail generation not yet implemented"));
    }

    fn generate_audio_thumbnail(&self, path: &Path, options: &ThumbnailOptions) -> Result<PathBuf> {
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let thumbnail_path = self.temp_dir.join(format!(
            "{}-waveform-{}x{}.png", file_stem, options.width, options.height
        ));

        let pipeline = gst::Pipeline::new();
        let filesrc = gst::ElementFactory::make("filesrc")
            .property("location", path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesrc: {}", e))?;
        let decodebin = gst::ElementFactory::make("decodebin")
            .build().map_err(|e| anyhow!("decodebin: {}", e))?;
        let audioconvert = gst::ElementFactory::make("audioconvert")
            .build().map_err(|e| anyhow!("audioconvert: {}", e))?;
        let audiowaveform = gst::ElementFactory::make("audiowaveform")
            .property("wave-mode", "lines")
            .property("style", "lines")
            .property("fill", true)
            .property("background-color", 0x000000ff_u32)
            .property("foreground-color", 0x00FF00FF_u32)
            .property("scale-digitized", true)
            .build().map_err(|e| anyhow!("audiowaveform: {}", e))?;
        let pngenc = gst::ElementFactory::make("pngenc")
            .property("compression-level", 6i32)
            .build().map_err(|e| anyhow!("pngenc: {}", e))?;
        let filesink = gst::ElementFactory::make("filesink")
            .property("location", thumbnail_path.to_str().unwrap())
            .build().map_err(|e| anyhow!("filesink: {}", e))?;

        pipeline.add_many([&filesrc, &decodebin, &audioconvert, &audiowaveform, &pngenc, &filesink])
            .map_err(|e| anyhow!("add: {}", e))?;
        filesrc.link(&decodebin).map_err(|e| anyhow!("link: {}", e))?;

        decodebin.connect_pad_added(move |_, src_pad| {
            if let Some(sink) = audioconvert.static_pad("sink") {
                if !sink.is_linked() {
                    if let Some(caps) = src_pad.current_caps() {
                        if let Some(s) = caps.structure(0) {
                            if s.name().starts_with("audio/") {
                                let _ = src_pad.link(&sink);
                            }
                        }
                    }
                }
            }
        });
        gst::Element::link_many([&audioconvert, &audiowaveform, &pngenc, &filesink])
            .map_err(|e| anyhow!("downstream: {}", e))?;

        let bus = pipeline.bus().ok_or_else(|| anyhow!("no bus"))?;
        pipeline.set_state(gst::State::Playing).map_err(|e| anyhow!("play: {}", e))?;

        let msg = bus.timed_pop_filtered(gst::ClockTime::from_seconds(60),
            &[gst::MessageType::Error, gst::MessageType::Eos]);
        pipeline.set_state(gst::State::Null).map_err(|e| anyhow!("stop: {}", e))?;

        match msg {
            Some(msg) => match msg.view() {
                gst::MessageView::Error(err) => Err(anyhow!("err: {}", err.error())),
                gst::MessageView::Eos(_) => Ok(thumbnail_path),
                _ => Err(anyhow!("unexpected")),
            },
            None => Err(anyhow!("timeout")),
        }
    }
        
    fn generate_generic_audio_thumbnail(&self, options: &ThumbnailOptions) -> Result<PathBuf> {

        let thumbnail_path = self.temp_dir.join(format!(
            "audio-icon-{}x{}.png",
            options.width,
            options.height
        ));

        let pipeline = gst::Pipeline::new();

        let videotestsrc = gst::ElementFactory::make("videotestsrc")
            .property("pattern", "black")
            .build()
            .map_err(|e| anyhow!("Failed to create videotestsrc: {}", e))?;

        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", gst::Caps::builder("video/x-raw")
                .field("width", options.width as i32)
                .field("height", options.height as i32)
                .build())
            .build()
            .map_err(|e| anyhow!("Failed to create capsfilter: {}", e))?;

        let textoverlay = gst::ElementFactory::make("textoverlay")
            .property("text", "Audio File")
            .property("font-desc", "Sans 24")
            .build()
            .map_err(|e| anyhow!("Failed to create textoverlay: {}", e))?;

        let pngenc = gst::ElementFactory::make("pngenc")
            .property("compression-level", 6i32)
            .build()
            .map_err(|e| anyhow!("Failed to create pngenc: {}", e))?;

        let filesink = gst::ElementFactory::make("filesink")
            .property("location", thumbnail_path.to_str().unwrap())
            .build()
            .map_err(|e| anyhow!("Failed to create filesink: {}", e))?;

        pipeline.add_many([&videotestsrc, &capsfilter, &textoverlay, &pngenc, &filesink])
            .map_err(|e| anyhow!("Failed to add elements to pipeline: {}", e))?;

        gst::Element::link_many([&videotestsrc, &capsfilter, &textoverlay, &pngenc, &filesink])
            .map_err(|e| anyhow!("Failed to link elements: {}", e))?;

        let bus = pipeline.bus().ok_or_else(|| anyhow!("Pipeline has no bus"))?;
        pipeline.set_state(gst::State::Playing)
            .map_err(|e| anyhow!("Failed to start pipeline: {}", e))?;

        let msg = bus.timed_pop_filtered(
            gst::ClockTime::from_seconds(30),
            &[gst::MessageType::Error, gst::MessageType::Eos],
        );

        pipeline.set_state(gst::State::Null)
            .map_err(|e| anyhow!("Failed to stop pipeline: {}", e))?;

        match msg {
            Some(msg) => match msg.view() {
                gst::MessageView::Error(err) => {
                    Err(anyhow!("Pipeline error: {}", err.error()))
                }
                gst::MessageView::Eos(_) => Ok(thumbnail_path),
                _ => Err(anyhow!("Unexpected pipeline message")),
            },
            None => Err(anyhow!("Pipeline timed out")),
        }
    }
}
