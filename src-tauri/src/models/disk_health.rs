use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeHealth {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalDiskHealth {
    pub model: String,
    pub smart_status: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskHealthReport {
    pub volumes: Vec<VolumeHealth>,
    pub physical_disks: Vec<PhysicalDiskHealth>,
}
