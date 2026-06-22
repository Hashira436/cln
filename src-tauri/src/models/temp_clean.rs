use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempScanResult {
    pub total_bytes: u64,
    pub file_count: u64,
    pub paths_scanned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempCleanResult {
    pub bytes_freed: u64,
    pub files_deleted: u64,
    pub files_skipped: u64,
    pub paths_cleaned: Vec<String>,
}
