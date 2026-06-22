use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::error::AppError;
use crate::models::scan_result::ScanResult;
use crate::modules::scanner::{actions, core};

/// Searches for files asynchronously using jwalk, streaming results back via Tauri events.
#[tauri::command]
pub async fn search_files(app: AppHandle, path: String, query: String) -> Result<(), AppError> {
    let trimmed_query = query.trim().to_string();
    if trimmed_query.is_empty() {
        return Err(AppError::Service("Search query cannot be empty.".into()));
    }

    let start_path = path.trim().to_string();
    if start_path.is_empty() {
        return Err(AppError::Service("Search path cannot be empty.".into()));
    }

    let (tx, mut rx) = mpsc::channel::<ScanResult>(500);

    // Spawn the scanner core logic
    tokio::spawn(async move {
        let _ = core::scan_directory(&start_path, &trimmed_query, tx).await;
    });

    // Spawn the event emitter loop
    tokio::spawn(async move {
        let mut batch = Vec::new();
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        let _ = app.emit("scan-result", &batch);
                        batch.clear();
                    }
                }
                msg = rx.recv() => {
                    match msg {
                        Some(result) => {
                            batch.push(result);
                            if batch.len() >= 50 {
                                let _ = app.emit("scan-result", &batch);
                                batch.clear();
                                interval.reset();
                            }
                        }
                        None => {
                            // Channel closed, flush remaining results
                            if !batch.is_empty() {
                                let _ = app.emit("scan-result", &batch);
                            }
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

/// Nukes a directory bypassing path limitations using Robocopy mirroring.
#[tauri::command]
pub async fn nuke_directory(path: String) -> Result<(), AppError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::Service("Path cannot be empty.".into()));
    }
    
    // Call the blocking nuke logic inside spawn_blocking
    let path_clone = trimmed.to_string();
    tokio::task::spawn_blocking(move || actions::nuke_directory_with_robocopy(&path_clone))
        .await
        .map_err(|e| AppError::Internal(format!("Task panicked: {}", e)))?
}
