#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let app = src_tauri::init_app()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(debug_assertions)]
    let app = {
        use tauri::Manager;
        app.setup(|app| {
            app.get_webview_window("main").unwrap().open_devtools();
            Ok(())
        })
    };

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
