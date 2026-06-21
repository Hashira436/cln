mod commands;
mod error;
mod models;
mod modules;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state, managed by Tauri and injected into commands.
pub struct AppState {
    /// Reusable System instance for process/memory queries.
    /// Wrapped in RwLock for concurrent access from async commands.
    pub sys: Arc<RwLock<sysinfo::System>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sys: Arc::new(RwLock::new(sysinfo::System::new())),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Process management
            commands::processes::get_top_processes,
            commands::processes::kill_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
