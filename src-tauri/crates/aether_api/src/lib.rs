pub mod commands;
pub mod state;

pub use commands::*;
pub use state::*;

use tauri::Manager;


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
        ])
}
