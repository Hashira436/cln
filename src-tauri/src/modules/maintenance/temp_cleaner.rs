use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::temp_clean::{TempCleanResult, TempScanResult};

const WINDOWS_TEMP: &str = r"C:\Windows\Temp";
const SOFTWARE_DISTRIBUTION: &str = r"C:\Windows\SoftwareDistribution\Download";

pub fn temp_target_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(user_temp) = env::var("TEMP") {
        if !user_temp.is_empty() {
            paths.push(PathBuf::from(user_temp));
        }
    }

    if let Ok(user_temp) = env::var("TMP") {
        let tmp = PathBuf::from(&user_temp);
        if !user_temp.is_empty() && !paths.iter().any(|p| p == &tmp) {
            paths.push(tmp);
        }
    }

    paths.push(PathBuf::from(WINDOWS_TEMP));
    paths.push(PathBuf::from(SOFTWARE_DISTRIBUTION));

    paths
        .into_iter()
        .filter(|path| path.exists())
        .collect()
}

pub fn calculate_temp_size() -> TempScanResult {
    let paths = temp_target_paths();
    let mut total_bytes = 0u64;
    let mut file_count = 0u64;

    for path in &paths {
        scan_directory(path, &mut |file_path| {
            if let Ok(meta) = fs::metadata(file_path) {
                if meta.is_file() {
                    total_bytes = total_bytes.saturating_add(meta.len());
                    file_count += 1;
                }
            }
        });
    }

    TempScanResult {
        total_bytes,
        file_count,
        paths_scanned: paths
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
    }
}

pub fn clean_temp_files() -> TempCleanResult {
    let paths = temp_target_paths();
    let mut bytes_freed = 0u64;
    let mut files_deleted = 0u64;
    let mut files_skipped = 0u64;

    for path in &paths {
        clean_directory(path, &mut |file_path| match fs::metadata(file_path) {
            Ok(meta) if meta.is_file() => {
                let size = meta.len();
                match fs::remove_file(file_path) {
                    Ok(()) => {
                        bytes_freed = bytes_freed.saturating_add(size);
                        files_deleted += 1;
                    }
                    Err(err) if is_permission_denied(&err) => {
                        files_skipped += 1;
                    }
                    Err(_) => {
                        files_skipped += 1;
                    }
                }
            }
            Ok(_) => {}
            Err(err) if is_permission_denied(&err) => {
                files_skipped += 1;
            }
            Err(_) => {
                files_skipped += 1;
            }
        });
    }

    TempCleanResult {
        bytes_freed,
        files_deleted,
        files_skipped,
        paths_cleaned: paths
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
    }
}

fn scan_directory(path: &Path, on_file: &mut dyn FnMut(&Path)) {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            scan_directory(&entry_path, on_file);
        } else if entry_path.is_file() {
            on_file(&entry_path);
        }
    }
}

fn clean_directory(path: &Path, on_file: &mut dyn FnMut(&Path)) {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            clean_directory(&entry_path, on_file);
        } else if entry_path.is_file() {
            on_file(&entry_path);
        }
    }
}

fn is_permission_denied(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::PermissionDenied || err.raw_os_error() == Some(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn calculate_temp_size_counts_files_in_user_temp() {
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join(format!("cln-test-{}.tmp", std::process::id()));

        let mut file = File::create(&file_path).expect("create temp file");
        writeln!(file, "cln temp cleaner test").expect("write temp file");

        let result = calculate_temp_size();
        assert!(result.file_count >= 1);
        assert!(result.total_bytes >= 1);

        let _ = fs::remove_file(file_path);
    }
}
