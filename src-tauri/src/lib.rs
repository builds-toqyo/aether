pub mod commands;

use commands::editing::*;
use commands::rendering::*;
use std::sync::Mutex;

pub struct AppState {
    pub editing_state: Mutex<commands::editing::EditingState>,
    pub rendering_state: Mutex<commands::rendering::RenderingState>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            editing_state: Mutex::new(commands::editing::EditingState::new()),
            rendering_state: Mutex::new(commands::rendering::RenderingState::new()),
        }
    }
}

#[tauri::command]
pub fn init_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Editing Commands
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
            // Rendering Commands
            start_rendering,
            get_export_progress,
            cancel_export,
            get_active_exports,
            get_supported_formats,
            get_export_presets,
            get_video_codecs,
            get_audio_codecs,
            simulate_export_progress,
        ])
}
