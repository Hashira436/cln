use serde::{Deserialize, Serialize};

/// DTO streamed to the frontend during heuristic file scans (Phase 3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub path: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub associated_reason: String,
}
