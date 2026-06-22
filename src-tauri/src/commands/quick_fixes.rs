use tauri::State;

use crate::error::AppError;
use crate::models::disk_health::DiskHealthReport;
use crate::models::event_log_entry::EventLogEntry;
use crate::models::temp_clean::{TempCleanResult, TempScanResult};
use crate::modules::maintenance;
use crate::AppState;

#[tauri::command]
pub async fn get_temp_folder_stats() -> Result<TempScanResult, AppError> {
    tokio::task::spawn_blocking(maintenance::calculate_temp_size)
        .await
        .map_err(|err| AppError::Internal(format!("Temp scan task failed: {err}")))
}

#[tauri::command]
pub async fn clean_temp_files() -> Result<TempCleanResult, AppError> {
    tokio::task::spawn_blocking(maintenance::clean_temp_files)
        .await
        .map_err(|err| AppError::Internal(format!("Temp clean task failed: {err}")))
}

#[tauri::command]
pub async fn get_disk_health(state: State<'_, AppState>) -> Result<DiskHealthReport, AppError> {
    let mut sys = state.sys.write().await;
    maintenance::build_disk_health_report(&mut sys).map_err(map_maintenance_error)
}

#[tauri::command]
pub async fn get_critical_events() -> Result<Vec<EventLogEntry>, AppError> {
    tokio::task::spawn_blocking(maintenance::get_critical_events_last_24h)
        .await
        .map_err(|err| AppError::Internal(format!("Event log task failed: {err}")))?
        .map_err(map_maintenance_error)
}

fn map_maintenance_error(message: String) -> AppError {
    let lower = message.to_lowercase();

    if lower.contains("wmi") || lower.contains("com init") {
        AppError::Wmi(message)
    } else if lower.contains("event") || lower.contains("evt") {
        AppError::EventLog(message)
    } else if lower.contains("permission") || lower.contains("access denied") {
        AppError::PermissionDenied(message)
    } else {
        AppError::Service(message)
    }
}
