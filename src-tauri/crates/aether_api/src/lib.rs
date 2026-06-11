pub mod commands;
pub mod state;
pub mod engine_proxy;
pub mod render_proxy;

pub use commands::*;
pub use state::*;

pub fn init_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![

            create_node,
            delete_node,
            connect_nodes,
            disconnect_nodes,
            execute_graph,
            get_node_result,
            get_graph_info,

            get_timeline_info,
            timeline_playback_control,
            timeline_seek,
            timeline_move_clip,
            timeline_trim_clip,
            timeline_add_clip,
            timeline_remove_clip,
            timeline_create_track,
            timeline_delete_track,

            get_preview_info,
            preview_playback_control,
            preview_seek,
            preview_get_frame,
            preview_get_frame_range,
            preview_update_settings,
            preview_get_settings,
            preview_clear_cache,
            preview_get_performance_stats,
            preview_export_frame,

            project_init,
            project_save,
            project_load,
            project_get_recent,
            project_auto_save,
            media_import,
            media_get_info,
            media_get_all,
            media_remove,
            media_export,
            export_get_status,
            export_cancel,


            rendering_start_job,
            rendering_cancel_job,
            rendering_pause_job,
            rendering_resume_job,
            rendering_get_job_status,
            rendering_get_job_progress,
            rendering_get_all_jobs,
            rendering_get_queue,
            rendering_get_formats,
            rendering_get_presets,
            rendering_estimate_time,
            rendering_get_performance_stats,
            rendering_cleanup_completed,
        ])
}
