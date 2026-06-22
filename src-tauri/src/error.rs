use serde::Serialize;

/// Unified error type for all CLN commands.
/// Every Tauri command returns `Result<T, AppError>`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Process not found: PID {0}")]
    ProcessNotFound(u32),

    #[error("Service error: {0}")]
    Service(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Remote execution error: {0}")]
    Remote(String),

    #[error("WMI query error: {0}")]
    Wmi(String),

    #[error("Event log error: {0}")]
    EventLog(String),

    #[error("SSH error: {0}")]
    Ssh(String),

    #[error("File operation error: {0}")]
    FileOp(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
