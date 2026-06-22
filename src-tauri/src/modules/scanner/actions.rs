use std::env;
use std::fs;
use std::process::Command;
use uuid::Uuid;

use crate::error::AppError;

/// Evaluates Robocopy exit codes. Codes 0 through 3 are considered successful.
pub fn evaluate_robocopy_exit_code(code: i32) -> Result<(), AppError> {
    if code >= 0 && code <= 3 {
        Ok(())
    } else {
        Err(AppError::Internal(format!(
            "Robocopy failed with exit code: {}",
            code
        )))
    }
}

/// Nuke a directory bypassing Windows Explorer path limitations using robocopy mirroring.
pub fn nuke_directory_with_robocopy(target_dir: &str) -> Result<(), AppError> {
    // 1. Create an empty temp directory
    let temp_dir = env::temp_dir().join(format!("cln_empty_{}", Uuid::new_v4()));
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        return Err(AppError::Internal(format!(
            "Failed to create temp dir for nuke: {}",
            e
        )));
    }

    // 2. Run robocopy to mirror the empty dir into target_dir
    let status = Command::new("robocopy")
        .arg(&temp_dir)
        .arg(target_dir)
        .arg("/MIR")
        .arg("/MT:8")
        .arg("/NP")
        .arg("/NFL")
        .arg("/NDL")
        .arg("/R:1")
        .arg("/W:1")
        .status();

    // Always clean up the empty temp directory
    let _ = fs::remove_dir_all(&temp_dir);

    // 3. Evaluate robocopy result
    match status {
        Ok(exit_status) => {
            let code = exit_status.code().unwrap_or(16); // 16 is fatal error
            evaluate_robocopy_exit_code(code)?;

            // If successful, the target directory is now empty. We must remove it.
            if let Err(e) = fs::remove_dir_all(target_dir) {
                return Err(AppError::Internal(format!(
                    "Robocopy succeeded, but failed to remove empty target dir: {}",
                    e
                )));
            }
            Ok(())
        }
        Err(e) => Err(AppError::Internal(format!(
            "Failed to execute robocopy: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robocopy_exit_codes() {
        assert!(evaluate_robocopy_exit_code(0).is_ok());
        assert!(evaluate_robocopy_exit_code(1).is_ok());
        assert!(evaluate_robocopy_exit_code(2).is_ok());
        assert!(evaluate_robocopy_exit_code(3).is_ok());
        
        assert!(evaluate_robocopy_exit_code(4).is_err());
        assert!(evaluate_robocopy_exit_code(8).is_err());
        assert!(evaluate_robocopy_exit_code(16).is_err());
        assert!(evaluate_robocopy_exit_code(-1).is_err());
    }

    #[test]
    fn test_nuke_directory_with_robocopy() {
        let target_dir = env::temp_dir().join(format!("cln_nuke_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&target_dir).unwrap();

        let file_path = target_dir.join("test.txt");
        fs::write(&file_path, "dummy data").unwrap();

        let sub_dir = target_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("subtest.txt"), "more dummy data").unwrap();

        assert!(target_dir.exists());

        let result = nuke_directory_with_robocopy(&target_dir.to_string_lossy());
        
        assert!(result.is_ok(), "Nuke should succeed: {:?}", result.err());
        assert!(!target_dir.exists(), "Target directory should be removed");
    }
}
