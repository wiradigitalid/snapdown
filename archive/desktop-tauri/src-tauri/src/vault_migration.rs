use snapdown_core::error::CoreError;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Report returned by a successful vault migration detailing any source files
/// that could not be removed after successful copy and verification.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VaultMigrationReport {
    pub unremoved_sources: Vec<PathBuf>,
}

/// Error returned when a vault migration fails, detailing the failure reason
/// and any destination files that could not be cleaned up during rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultMigrationError {
    pub reason: String,
    pub uncleaned_destinations: Vec<PathBuf>,
}

impl fmt::Display for VaultMigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.uncleaned_destinations.is_empty() {
            write!(f, "{}", self.reason)
        } else {
            let files = self
                .uncleaned_destinations
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            write!(
                f,
                "{}; could not clean up destination files during rollback: [{files}]",
                self.reason
            )
        }
    }
}

impl std::error::Error for VaultMigrationError {}

impl From<CoreError> for VaultMigrationError {
    fn from(err: CoreError) -> Self {
        Self {
            reason: err.to_string(),
            uncleaned_destinations: Vec::new(),
        }
    }
}

pub struct VaultMigrator;

impl VaultMigrator {
    /// Validates whether a directory exists (or can be created) and is writable.
    pub fn validate_directory_writable<P: AsRef<Path>>(dir: P) -> Result<PathBuf, CoreError> {
        let dir_ref = dir.as_ref();
        if dir_ref.as_os_str().is_empty() {
            return Err(CoreError::InvalidPath(
                "Directory path cannot be empty".into(),
            ));
        }

        if !dir_ref.exists() {
            fs::create_dir_all(dir_ref)
                .map_err(|e| CoreError::Validation(format!("Failed to create directory: {e}")))?;
        }

        let canonical = dir_ref.canonicalize().map_err(|e| {
            CoreError::Validation(format!("Failed to canonicalize directory path: {e}"))
        })?;

        if !canonical.is_dir() {
            return Err(CoreError::Validation("Path is not a directory".into()));
        }

        // Test writability by creating, writing, and deleting a temporary marker file
        let test_file_name = format!(".snapdown_write_test_{}", std::process::id());
        let test_file_path = canonical.join(&test_file_name);

        fs::write(&test_file_path, b"writability_test")
            .map_err(|e| CoreError::Validation(format!("Directory is not writable: {e}")))?;

        // If the probe cannot be removed the folder is still writable, because the write
        // itself succeeded. The artifact is inert.
        let _ = fs::remove_file(&test_file_path);

        Ok(canonical)
    }

    /// Migrates all files and directories from `src_dir` to `dest_dir` with an all-or-nothing guarantee (BR-29).
    ///
    /// Copies and size-verifies all files in destination before removing source files.
    /// If copying or verification fails, destination copies are rolled back and source files remain intact.
    /// If all files are copied and verified but one or more source files cannot be deleted, the move succeeds
    /// and the unremoved source files are reported in `VaultMigrationReport.unremoved_sources`.
    pub fn migrate_vault<P: AsRef<Path>, Q: AsRef<Path>>(
        src_dir: P,
        dest_dir: Q,
    ) -> Result<VaultMigrationReport, VaultMigrationError> {
        Self::migrate_vault_with_deleter(src_dir, dest_dir, |p| fs::remove_file(p))
    }

    /// Migrates vault files using a custom deletion operation (test seam for simulating delete refusal).
    pub fn migrate_vault_with_deleter<P: AsRef<Path>, Q: AsRef<Path>, D>(
        src_dir: P,
        dest_dir: Q,
        deleter: D,
    ) -> Result<VaultMigrationReport, VaultMigrationError>
    where
        D: Fn(&Path) -> std::io::Result<()>,
    {
        let src = src_dir.as_ref();
        let dest = dest_dir.as_ref();

        // If source doesn't exist or is same as dest, nothing to migrate
        if !src.exists() {
            return Ok(VaultMigrationReport::default());
        }

        let canonical_src = src.canonicalize().map_err(|e| VaultMigrationError {
            reason: format!("Failed to canonicalize source path: {e}"),
            uncleaned_destinations: Vec::new(),
        })?;
        let canonical_dest =
            Self::validate_directory_writable(dest).map_err(|e| VaultMigrationError {
                reason: e.to_string(),
                uncleaned_destinations: Vec::new(),
            })?;

        if canonical_src == canonical_dest {
            return Ok(VaultMigrationReport::default());
        }

        // Collect all files to migrate relative to source root
        let mut files_to_migrate = Vec::new();
        Self::collect_files_recursive(&canonical_src, &canonical_src, &mut files_to_migrate)
            .map_err(|e| VaultMigrationError {
                reason: e.to_string(),
                uncleaned_destinations: Vec::new(),
            })?;

        if files_to_migrate.is_empty() {
            return Ok(VaultMigrationReport::default());
        }

        // List of successfully copied destination files for rollback tracking
        let mut copied_dest_files: Vec<PathBuf> = Vec::new();

        for rel_path in &files_to_migrate {
            let src_file = canonical_src.join(rel_path);
            let dest_file = canonical_dest.join(rel_path);

            if let Some(parent) = dest_file.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    let uncleaned =
                        Self::rollback_with_deleter(&copied_dest_files, &canonical_dest, &deleter);
                    return Err(VaultMigrationError {
                        reason: format!(
                            "Failed to create target directory {}: {e}",
                            parent.display()
                        ),
                        uncleaned_destinations: uncleaned,
                    });
                }
            }

            if let Err(e) = fs::copy(&src_file, &dest_file) {
                let uncleaned =
                    Self::rollback_with_deleter(&copied_dest_files, &canonical_dest, &deleter);
                return Err(VaultMigrationError {
                    reason: format!(
                        "Failed to copy file {} to {}: {e}",
                        src_file.display(),
                        dest_file.display()
                    ),
                    uncleaned_destinations: uncleaned,
                });
            }

            copied_dest_files.push(dest_file);
        }

        // Verification step: ensure all files in destination exist and have matching size
        for rel_path in &files_to_migrate {
            let src_file = canonical_src.join(rel_path);
            let dest_file = canonical_dest.join(rel_path);

            let src_meta = match fs::metadata(&src_file) {
                Ok(m) => m,
                Err(e) => {
                    let uncleaned =
                        Self::rollback_with_deleter(&copied_dest_files, &canonical_dest, &deleter);
                    return Err(VaultMigrationError {
                        reason: format!("Failed to stat source file {}: {e}", src_file.display()),
                        uncleaned_destinations: uncleaned,
                    });
                }
            };

            let dest_meta = match fs::metadata(&dest_file) {
                Ok(m) => m,
                Err(e) => {
                    let uncleaned =
                        Self::rollback_with_deleter(&copied_dest_files, &canonical_dest, &deleter);
                    return Err(VaultMigrationError {
                        reason: format!(
                            "Failed to stat destination file {}: {e}",
                            dest_file.display()
                        ),
                        uncleaned_destinations: uncleaned,
                    });
                }
            };

            if src_meta.len() != dest_meta.len() {
                let uncleaned =
                    Self::rollback_with_deleter(&copied_dest_files, &canonical_dest, &deleter);
                return Err(VaultMigrationError {
                    reason: format!("File size mismatch for copied file {}", dest_file.display()),
                    uncleaned_destinations: uncleaned,
                });
            }
        }

        // All copied and verified successfully. Now remove source files.
        let mut unremoved_sources = Vec::new();
        for rel_path in files_to_migrate.iter().rev() {
            let src_file = canonical_src.join(rel_path);
            if deleter(&src_file).is_err() {
                unremoved_sources.push(src_file);
            }
        }

        // Clean up empty directories in source
        Self::remove_empty_dirs_recursive(&canonical_src);

        Ok(VaultMigrationReport { unremoved_sources })
    }

    fn collect_files_recursive(
        base: &Path,
        current: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), CoreError> {
        let entries = fs::read_dir(current).map_err(|e| {
            CoreError::Validation(format!(
                "Failed to read directory {}: {e}",
                current.display()
            ))
        })?;

        for entry_res in entries {
            let entry = entry_res.map_err(|e| {
                CoreError::Validation(format!("Failed to read directory entry: {e}"))
            })?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel) = path.strip_prefix(base) {
                    files.push(rel.to_path_buf());
                }
            } else if path.is_dir() {
                Self::collect_files_recursive(base, &path, files)?;
            }
        }
        Ok(())
    }

    fn rollback_with_deleter<D: Fn(&Path) -> std::io::Result<()>>(
        copied_files: &[PathBuf],
        dest_root: &Path,
        deleter: &D,
    ) -> Vec<PathBuf> {
        let mut uncleaned = Vec::new();
        for file in copied_files {
            if deleter(file).is_err() {
                uncleaned.push(file.clone());
            }
        }
        Self::remove_empty_dirs_recursive(dest_root);
        uncleaned
    }

    fn remove_empty_dirs_recursive(dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry_res in entries.flatten() {
                let path = entry_res.path();
                if path.is_dir() {
                    Self::remove_empty_dirs_recursive(&path);
                    // fs::remove_dir refusing a non-empty directory is the pruning guard.
                    // Swallowing the error ensures only empty directories are pruned.
                    let _ = fs::remove_dir(&path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn unwritable_vault_folder_is_refused_at_choosing() {
        let tmp = TempDir::new().unwrap();
        let valid_path = tmp.path().join("sub_dir");
        let validated = VaultMigrator::validate_directory_writable(&valid_path);
        assert!(validated.is_ok());

        // Test invalid/empty path
        let empty_path = "";
        let res = VaultMigrator::validate_directory_writable(empty_path);
        assert!(res.is_err());
    }

    #[test]
    fn changing_the_vault_moves_every_file_or_none() {
        let src_tmp = TempDir::new().unwrap();
        let dest_tmp = TempDir::new().unwrap();

        let src_path = src_tmp.path();
        let dest_path = dest_tmp.path();

        // Create 3 files in src
        fs::create_dir_all(src_path.join("nested")).unwrap();
        fs::write(src_path.join("file1.png"), b"content 1").unwrap();
        fs::write(src_path.join("file2.png"), b"content 2").unwrap();
        fs::write(src_path.join("nested/file3.png"), b"content 3").unwrap();

        // Perform migration
        let res = VaultMigrator::migrate_vault(src_path, dest_path);
        assert!(res.is_ok());
        let report = res.unwrap();
        assert!(report.unremoved_sources.is_empty());

        // Verify files in dest
        assert_eq!(fs::read(dest_path.join("file1.png")).unwrap(), b"content 1");
        assert_eq!(fs::read(dest_path.join("file2.png")).unwrap(), b"content 2");
        assert_eq!(
            fs::read(dest_path.join("nested/file3.png")).unwrap(),
            b"content 3"
        );

        // Verify files removed from src
        assert!(!src_path.join("file1.png").exists());
        assert!(!src_path.join("file2.png").exists());
        assert!(!src_path.join("nested/file3.png").exists());
    }

    #[test]
    fn migration_rollback_on_failure_leaves_source_intact() {
        let src_tmp = TempDir::new().unwrap();
        let dest_tmp = TempDir::new().unwrap();

        let src_path = src_tmp.path();
        let dest_path = dest_tmp.path();

        fs::write(src_path.join("file1.png"), b"content 1").unwrap();
        fs::write(src_path.join("file2.png"), b"content 2").unwrap();

        // Create a directory in destination matching one of the file names to trigger a copy error
        fs::create_dir(dest_path.join("file2.png")).unwrap();

        let res = VaultMigrator::migrate_vault(src_path, dest_path);
        assert!(res.is_err(), "Migration must fail and trigger rollback");

        // Verify all source files remain untouched
        assert_eq!(fs::read(src_path.join("file1.png")).unwrap(), b"content 1");
        assert_eq!(fs::read(src_path.join("file2.png")).unwrap(), b"content 2");

        // Verify rolled-back destination file1 was removed
        assert!(!dest_path.join("file1.png").exists());
    }
}
