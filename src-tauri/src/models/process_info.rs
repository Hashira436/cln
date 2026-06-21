use serde::{Deserialize, Serialize};

/// Data transfer object for a single process, sent to the frontend via IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// OS process ID
    pub pid: u32,
    /// Process name (executable filename)
    pub name: String,
    /// Resident memory in megabytes
    pub memory_mb: f64,
    /// CPU usage as a percentage (0.0–100.0+)
    pub cpu_percent: f32,
}
