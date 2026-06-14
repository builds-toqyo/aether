pub mod commands;

use commands::editing::*;
use commands::rendering::*;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub editing_state: Arc<Mutex<commands::editing::EditingState>>,
    pub rendering_state: Arc<Mutex<commands::rendering::RenderingState>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            editing_state: Arc::new(Mutex::new(commands::editing::EditingState::new())),
            rendering_state: Arc::new(Mutex::new(commands::rendering::RenderingState::new())),
        }
    }
}

pub fn init_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![

            create_project,
            open_project,
            save_project,
            get_current_project,
            close_project,
            update_project_settings,
            create_track,
            delete_track,
            get_timeline_tracks,
            update_track,
            add_clip,
            remove_clip,
            get_timeline_clips,
            update_clip,
            move_clip,
            import_media,
            get_media_items,
            remove_media,
            analyze_media,

            start_rendering,
            get_export_progress,
            cancel_export,
            get_active_exports,
            get_supported_formats,
            get_export_presets,
            get_video_codecs,
            get_audio_codecs,
            simulate_export_progress,

            color_analyze_histogram,
            color_analyze_vectorscope,
            color_analyze_waveform,
        ])
}
