use snapdown_core::error::CoreError;
use std::fs;
use std::path::{Path, PathBuf};

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

        let _ = fs::remove_file(&test_file_path);

        Ok(canonical)
    }

    /// Migrates all files and directories from `src_dir` to `dest_dir` with an all-or-nothing guarantee (BR-29).
    /// If moving any file fails, all copied files in `dest_dir` are deleted and `src_dir` remains intact.
    pub fn migrate_vault<P: AsRef<Path>, Q: AsRef<Path>>(
        src_dir: P,
        dest_dir: Q,
    ) -> Result<(), CoreError> {
        let src = src_dir.as_ref();
        let dest = dest_dir.as_ref();

        // If source doesn't exist or is same as dest, nothing to migrate
        if !src.exists() {
            return Ok(());
        }

        let canonical_src = src.canonicalize().map_err(|e| {
            CoreError::Validation(format!("Failed to canonicalize source path: {e}"))
        })?;
        let canonical_dest = Self::validate_directory_writable(dest)?;

        if canonical_src == canonical_dest {
            return Ok(());
        }

        // Collect all files to migrate relative to source root
        let mut files_to_migrate = Vec::new();
        Self::collect_files_recursive(&canonical_src, &canonical_src, &mut files_to_migrate)?;

        if files_to_migrate.is_empty() {
            return Ok(());
        }

        // List of successfully copied destination files for rollback tracking
        let mut copied_dest_files: Vec<PathBuf> = Vec::new();

        for rel_path in &files_to_migrate {
            let src_file = canonical_src.join(rel_path);
            let dest_file = canonical_dest.join(rel_path);

            if let Some(parent) = dest_file.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    Self::rollback(&copied_dest_files, &canonical_dest);
                    return Err(CoreError::Validation(format!(
                        "Failed to create target directory {}: {e}",
                        parent.display()
                    )));
                }
            }

            if let Err(e) = fs::copy(&src_file, &dest_file) {
                Self::rollback(&copied_dest_files, &canonical_dest);
                return Err(CoreError::Validation(format!(
                    "Failed to copy file {} to {}: {e}",
                    src_file.display(),
                    dest_file.display()
                )));
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
                    Self::rollback(&copied_dest_files, &canonical_dest);
                    return Err(CoreError::Validation(format!(
                        "Failed to stat source file {}: {e}",
                        src_file.display()
                    )));
                }
            };

            let dest_meta = match fs::metadata(&dest_file) {
                Ok(m) => m,
                Err(e) => {
                    Self::rollback(&copied_dest_files, &canonical_dest);
                    return Err(CoreError::Validation(format!(
                        "Failed to stat destination file {}: {e}",
                        dest_file.display()
                    )));
                }
            };

            if src_meta.len() != dest_meta.len() {
                Self::rollback(&copied_dest_files, &canonical_dest);
                return Err(CoreError::Validation(format!(
                    "File size mismatch for copied file {}",
                    dest_file.display()
                )));
            }
        }

        // All copied and verified successfully. Now remove source files.
        for rel_path in files_to_migrate.iter().rev() {
            let src_file = canonical_src.join(rel_path);
            let _ = fs::remove_file(&src_file);
        }

        // Clean up empty directories in source
        Self::remove_empty_dirs_recursive(&canonical_src);

        Ok(())
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

    fn rollback(copied_files: &[PathBuf], dest_root: &Path) {
        for file in copied_files {
            let _ = fs::remove_file(file);
        }
        Self::remove_empty_dirs_recursive(dest_root);
    }

    fn remove_empty_dirs_recursive(dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry_res in entries.flatten() {
                let path = entry_res.path();
                if path.is_dir() {
                    Self::remove_empty_dirs_recursive(&path);
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
