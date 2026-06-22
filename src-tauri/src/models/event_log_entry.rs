use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub channel: String,
    pub level: String,
    pub provider: String,
    pub event_id: u32,
    pub message: String,
    pub timestamp: String,
}
