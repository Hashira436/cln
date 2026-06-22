use jwalk::WalkDir;
use tokio::sync::mpsc::Sender;

use crate::error::AppError;
use crate::models::scan_result::ScanResult;

/// Asynchronous directory traversal that sends results through a channel.
pub async fn scan_directory(
    start_path: &str,
    query: &str,
    tx: Sender<ScanResult>,
) -> Result<(), AppError> {
    let start_path_val = start_path.to_string();
    let query_val = query.to_lowercase();

    tokio::task::spawn_blocking(move || {
        for entry in WalkDir::new(&start_path_val)
            .skip_hidden(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path_str = entry.path().to_string_lossy().to_string();
            
            // Check if path contains the query (case-insensitive)
            if path_str.to_lowercase().contains(&query_val) {
                let metadata = entry.metadata().ok();
                let is_dir = entry.file_type().is_dir();
                let size_bytes = metadata.map(|m| m.len()).unwrap_or(0);
                
                let result = ScanResult {
                    path: path_str,
                    file_type: if is_dir { "Directory".into() } else { "File".into() },
                    size_bytes,
                    associated_reason: "Name Match".into(),
                };

                // Send to channel; stop if receiver is dropped
                if tx.blocking_send(result).is_err() {
                    break;
                }
            }
        }
    })
    .await
    .map_err(|e| AppError::Internal(format!("Scanner task panicked: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tokio::sync::mpsc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_scan_directory() {
        let temp_dir = env::temp_dir().join(format!("test_scan_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        
        let target_file = temp_dir.join("match_me.txt");
        fs::write(&target_file, "content").unwrap();
        
        let ignore_file = temp_dir.join("ignore.txt");
        fs::write(&ignore_file, "content").unwrap();

        let (tx, mut rx) = mpsc::channel(10);
        let temp_dir_str = temp_dir.to_string_lossy().to_string();
        
        scan_directory(&temp_dir_str, "match_me", tx).await.unwrap();

        let mut found_match = false;
        while let Some(res) = rx.recv().await {
            if res.path.contains("match_me.txt") {
                found_match = true;
            }
            if res.path.contains("ignore.txt") {
                panic!("Should not have found ignore.txt");
            }
        }

        assert!(found_match, "Should have found the matching file");
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
