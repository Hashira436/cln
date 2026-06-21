use crate::models::process_info::ProcessInfo;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Refreshes system data and returns the top `count` processes sorted by memory usage (descending).
///
/// This function is pure business logic — no Tauri dependency.
pub fn get_top_by_memory(sys: &mut System, count: usize) -> Vec<ProcessInfo> {
    // Refresh only process-level memory and CPU — skip disk, users, network.
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new()
            .with_memory()
            .with_cpu(),
    );

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .filter(|p| {
            // Exclude idle/system pseudo-processes with no name or no memory
            let name = p.name().to_string_lossy().to_string();
            !name.is_empty() && p.memory() > 0
        })
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().to_string(),
            memory_mb: p.memory() as f64 / 1_048_576.0, // bytes → MB
            cpu_percent: p.cpu_usage(),
        })
        .collect();

    // Sort by memory descending
    processes.sort_by(|a, b| {
        b.memory_mb
            .partial_cmp(&a.memory_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    processes.truncate(count);
    processes
}

/// Refreshes system data and returns the top `count` processes sorted by CPU usage (descending).
pub fn get_top_by_cpu(sys: &mut System, count: usize) -> Vec<ProcessInfo> {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new()
            .with_memory()
            .with_cpu(),
    );

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .filter(|p| {
            let name = p.name().to_string_lossy().to_string();
            !name.is_empty() && p.memory() > 0
        })
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().to_string(),
            memory_mb: p.memory() as f64 / 1_048_576.0,
            cpu_percent: p.cpu_usage(),
        })
        .collect();

    processes.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    processes.truncate(count);
    processes
}

/// Kills a process by PID. Returns `Ok(())` on success, or an error message.
pub fn kill_process_by_pid(sys: &System, pid: u32) -> Result<(), String> {
    let sysinfo_pid = sysinfo::Pid::from_u32(pid);
    match sys.process(sysinfo_pid) {
        Some(process) => {
            if process.kill() {
                Ok(())
            } else {
                Err(format!(
                    "Failed to kill process with PID {}. It may require elevated privileges.",
                    pid
                ))
            }
        }
        None => Err(format!("Process with PID {} not found.", pid)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_top_by_memory_returns_limited_results() {
        let mut sys = System::new();
        let result = get_top_by_memory(&mut sys, 5);
        assert!(result.len() <= 5);
    }

    #[test]
    fn test_get_top_by_memory_sorted_descending() {
        let mut sys = System::new();
        let result = get_top_by_memory(&mut sys, 10);
        for window in result.windows(2) {
            assert!(window[0].memory_mb >= window[1].memory_mb);
        }
    }

    #[test]
    fn test_kill_nonexistent_process() {
        let sys = System::new();
        let result = kill_process_by_pid(&sys, 999_999_999);
        assert!(result.is_err());
    }
}
