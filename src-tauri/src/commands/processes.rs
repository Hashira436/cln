use tauri::State;

use crate::error::AppError;
use crate::models::process_info::ProcessInfo;
use crate::modules::processes::monitor;
use crate::AppState;

/// Tauri command: returns the top N processes sorted by RAM usage.
///
/// # Arguments
/// * `count` — number of processes to return (default: 5)
/// * `sort_by` — "memory" (default) or "cpu"
#[tauri::command]
pub async fn get_top_processes(
    state: State<'_, AppState>,
    count: Option<usize>,
    sort_by: Option<String>,
) -> Result<Vec<ProcessInfo>, AppError> {
    let count = count.unwrap_or(5);
    let sort_by = sort_by.unwrap_or_else(|| "memory".to_string());

    let mut sys = state.sys.write().await;

    let result = match sort_by.as_str() {
        "cpu" => monitor::get_top_by_cpu(&mut sys, count),
        _ => monitor::get_top_by_memory(&mut sys, count),
    };

    Ok(result)
}

/// Tauri command: kills a process by PID.
#[tauri::command]
pub async fn kill_process(
    state: State<'_, AppState>,
    pid: u32,
) -> Result<String, AppError> {
    let sys = state.sys.read().await;
    monitor::kill_process_by_pid(&sys, pid)
        .map(|_| format!("Process {} terminated successfully.", pid))
        .map_err(|e| map_kill_error(pid, e))
}

fn map_kill_error(pid: u32, message: String) -> AppError {
    let lower = message.to_lowercase();
    if lower.contains("not found") {
        AppError::ProcessNotFound(pid)
    } else if lower.contains("privilege") || lower.contains("permission") || lower.contains("access") {
        AppError::PermissionDenied(message)
    } else {
        AppError::Service(message)
    }
}
