mod commands;
mod error;
mod models;
mod modules;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

/// Placeholder for Phase 5 SSH session metadata.
#[derive(Debug, Clone, Default)]
pub struct SshSessionSlot {
    pub host: String,
    pub username: String,
    pub connected_at: Option<String>,
}

/// Shared application state, managed by Tauri and injected into commands.
pub struct AppState {
    /// Reusable System instance for process/memory/disk queries.
    pub sys: Arc<RwLock<sysinfo::System>>,
    /// Reserved cache for upcoming remote-admin SSH sessions (Phase 5).
    pub ssh_sessions: Arc<RwLock<HashMap<String, SshSessionSlot>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sys: Arc::new(RwLock::new(sysinfo::System::new())),
            ssh_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::processes::get_top_processes,
            commands::processes::kill_process,
            commands::quick_fixes::get_temp_folder_stats,
            commands::quick_fixes::clean_temp_files,
            commands::quick_fixes::get_disk_health,
            commands::quick_fixes::get_critical_events,
            commands::file_scanner::search_files,
            commands::file_scanner::nuke_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
