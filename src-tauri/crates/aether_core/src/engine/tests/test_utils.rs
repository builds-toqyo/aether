use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::env;
use std::fs;
use std::time::Duration;
use anyhow::{Result, Context};

use crate::engine::editing::EditingEngine;
use crate::engine::rendering::RenderingEngine;
use crate::engine::integration::IntegratedExporter;

pub const TEST_VIDEO_ENV: &str = "VIDEO_TEST_PATH";

pub const DEFAULT_TEST_VIDEO: &str = "tests/assets/test_video.mp4";

pub fn get_test_video_path() -> PathBuf {
    match env::var(TEST_VIDEO_ENV) {
        Ok(path) => PathBuf::from(path),
        Err(_) => PathBuf::from(DEFAULT_TEST_VIDEO),
    }
}

pub fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
    let temp_dir = env::temp_dir().join(format!("aether_test_{}", prefix));
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

pub fn create_test_output_path(prefix: &str, extension: &str) -> Result<PathBuf> {
    let temp_dir = create_temp_dir(prefix)?;
    let timestamp = chrono::Utc::now().timestamp();
    let path = temp_dir.join(format!("test_output_{}_{}.{}", prefix, timestamp, extension));
    Ok(path)
}

pub fn init_gstreamer() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        gstreamer::init().expect("Failed to initialize GStreamer");
    });
}

pub fn create_test_editing_engine() -> Result<Arc<Mutex<EditingEngine>>> {
    init_gstreamer();
    
    let engine = EditingEngine::new()?;
    engine.init_project("test_project")?;
    
    Ok(Arc::new(Mutex::new(engine)))
}

pub fn create_test_rendering_engine() -> Result<Arc<Mutex<RenderingEngine>>> {
    let engine = RenderingEngine::new()?;
    Ok(Arc::new(Mutex::new(engine)))
}

pub fn import_test_video(engine: &Arc<Mutex<EditingEngine>>) -> Result<String> {
    let test_video = get_test_video_path();
    
    if !test_video.exists() {
        anyhow::bail!(
            "Test video not found at {}. Set the {} environment variable to a valid video path.",
            test_video.display(),
            TEST_VIDEO_ENV
        );
    }
    
    let clip_id = engine.lock().unwrap().import_media(&test_video)?;
    Ok(clip_id)
}

pub fn wait_for_condition<F>(condition: F, timeout: Duration, poll_interval: Duration) -> Result<()>
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    
    while !condition() {
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for condition");
        }
        
        std::thread::sleep(poll_interval);
    }
    
    Ok(())
}

pub fn check_file_exists_with_content<P: AsRef<Path>>(path: P) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.len() > 0,
        Err(_) => false,
    }
}

pub fn create_simple_test_timeline(engine: &Arc<Mutex<EditingEngine>>) -> Result<String> {
    let clip_id = import_test_video(engine)?;
    
    let timeline = engine.lock().unwrap().timeline();
    let mut timeline = timeline.lock().unwrap();
    
    let video_track_id = timeline.add_track(crate::engine::editing::types::TrackType::Video)?;
    
    timeline.add_clip_to_track(&clip_id, &video_track_id, 0)?;
    
    Ok(clip_id)
}

pub fn ensure_test_assets_dir() -> Result<PathBuf> {
    let assets_dir = PathBuf::from("tests/assets");
    fs::create_dir_all(&assets_dir)?;
    Ok(assets_dir)
}

pub fn download_test_video_if_needed() -> Result<PathBuf> {
    let test_video = get_test_video_path();
    
    if test_video.exists() {
        return Ok(test_video);
    }
    
    let assets_dir = ensure_test_assets_dir()?;
    let target_path = assets_dir.join("test_video.mp4");
    
    let url = "https://sample-videos.com/video123/mp4/240/big_buck_bunny_240p_1mb.mp4";
    
    println!("Downloading test video from {}", url);
    
    let response = reqwest::blocking::get(url)
        .context("Failed to download test video")?;
    
    let content = response.bytes()
        .context("Failed to read test video content")?;
    
    fs::write(&target_path, content)
        .context("Failed to write test video to disk")?;
    
    println!("Downloaded test video to {}", target_path.display());
    
    Ok(target_path)
}

pub fn create_mock_progress_callback<T: Clone + Send + 'static>() -> (
    Arc<Mutex<Vec<T>>>,
    impl Fn(T) + Send + 'static,
) {
    let progress_history = Arc::new(Mutex::new(Vec::new()));
    let progress_history_clone = progress_history.clone();
    
    let callback = move |progress: T| {
        progress_history_clone.lock().unwrap().push(progress);
    };
    
    (progress_history, callback)
}
