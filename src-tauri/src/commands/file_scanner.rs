use tauri::{AppHandle, Emitter};

use crate::error::AppError;
use crate::models::scan_result::ScanResult;

/// Phase 3 stub: demonstrates streaming scan hits to the UI via Tauri events.
#[tauri::command]
pub async fn search_files(app: AppHandle, query: String) -> Result<(), AppError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(AppError::Service("Search query cannot be empty.".into()));
    }

    let preview_results = vec![
        ScanResult {
            path: format!(
                "C:\\Users\\Public\\AppData\\Local\\{}",
                trimmed.to_lowercase()
            ),
            file_type: "Directory".into(),
            size_bytes: 0,
            associated_reason: "Scaffold hit — heuristic AppData association".into(),
        },
        ScanResult {
            path: format!(
                "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
                trimmed
            ),
            file_type: "Registry".into(),
            size_bytes: 0,
            associated_reason: "Scaffold hit — registry uninstall key".into(),
        },
        ScanResult {
            path: format!("C:\\ProgramData\\{}\\cache", trimmed),
            file_type: "File".into(),
            size_bytes: 4096,
            associated_reason: "Scaffold hit — ProgramData cache heuristic".into(),
        },
    ];

    for result in preview_results {
        app.emit("scan-result", &result)
            .map_err(|err| AppError::Internal(format!("Failed to emit scan-result: {err}")))?;
    }

    Ok(())
}
