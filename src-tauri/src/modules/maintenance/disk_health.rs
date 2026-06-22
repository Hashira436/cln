use sysinfo::{Disks, System};

use crate::models::disk_health::{DiskHealthReport, PhysicalDiskHealth, VolumeHealth};

pub fn get_volume_health(sys: &mut System) -> Vec<VolumeHealth> {
    sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let free = disk.available_space();
            let used_percent = if total == 0 {
                0.0
            } else {
                ((total - free) as f64 / total as f64) * 100.0
            };

            VolumeHealth {
                name: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().to_string_lossy().into_owned(),
                total_bytes: total,
                free_bytes: free,
                used_percent,
            }
        })
        .collect()
}

#[cfg(windows)]
pub fn get_physical_disk_health() -> Result<Vec<PhysicalDiskHealth>, String> {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    #[derive(Debug, Deserialize)]
    struct Win32DiskDrive {
        #[serde(rename = "Model")]
        model: Option<String>,
        #[serde(rename = "Status")]
        status: Option<String>,
        #[serde(rename = "Size")]
        size: Option<u64>,
    }

    let com = COMLibrary::new().map_err(|e| format!("COM init failed: {e}"))?;
    let wmi = WMIConnection::new(com).map_err(|e| format!("WMI connection failed: {e}"))?;

    let drives: Vec<Win32DiskDrive> = wmi
        .raw_query("SELECT Model, Status, Size FROM Win32_DiskDrive")
        .map_err(|e| format!("WMI query failed: {e}"))?;

    Ok(drives
        .into_iter()
        .map(|drive| {
            let status = drive
                .status
                .unwrap_or_else(|| "Unknown".to_string())
                .trim()
                .to_string();

            PhysicalDiskHealth {
                model: drive.model.unwrap_or_else(|| "Unknown disk".to_string()),
                smart_status: normalize_smart_status(&status),
                size_bytes: drive.size.unwrap_or(0),
            }
        })
        .collect())
}

#[cfg(not(windows))]
pub fn get_physical_disk_health() -> Result<Vec<PhysicalDiskHealth>, String> {
    Ok(Vec::new())
}

pub fn build_disk_health_report(sys: &mut System) -> Result<DiskHealthReport, String> {
    let volumes = get_volume_health(sys);
    let physical_disks = get_physical_disk_health()?;

    Ok(DiskHealthReport {
        volumes,
        physical_disks,
    })
}

fn normalize_smart_status(status: &str) -> String {
    let upper = status.to_uppercase();
    if upper.contains("PRED") || upper.contains("FAIL") {
        "PredFail".to_string()
    } else if upper == "OK" {
        "OK".to_string()
    } else if upper.is_empty() || upper == "UNKNOWN" {
        "Unknown".to_string()
    } else {
        status.to_string()
    }
}
