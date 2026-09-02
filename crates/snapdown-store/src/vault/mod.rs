use std::fs;
use std::path::{Component, Path, PathBuf};

use snapdown_core::error::CoreError;
use snapdown_core::ports::BlobStore;

pub mod sweeper;
pub use sweeper::{OrphanScanReport, OrphanSweeper};

#[derive(Debug, Clone)]
pub struct VaultBlobStore {
    root: PathBuf,
}

impl VaultBlobStore {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, CoreError> {
        let root_path = root.as_ref();
        if !root_path.exists() {
            fs::create_dir_all(root_path).map_err(|e| CoreError::Validation(e.to_string()))?;
        }
        let canonical_root = root_path
            .canonicalize()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        if !canonical_root.is_dir() {
            return Err(CoreError::Validation(
                "Vault root must be a directory".into(),
            ));
        }

        Ok(Self {
            root: canonical_root,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Removes a whole folder and everything in it, reusing `resolve_path`'s traversal guard so a
    /// crafted relative path cannot escape the Vault root - the same protection every per-blob
    /// method already gets.
    ///
    /// This is the primitive Disassemble and Delete (ticket 16 of the Bundle Library spec) build on
    /// to remove a Bundle's own folder (`bundles/<id>/`, Markdown and image copies together) after
    /// its row is gone (`AD-2`: record first, then files). No folder-level delete existed anywhere
    /// in this store before it - every method on `BlobStore` is per-file.
    pub fn delete_folder(&self, relative_path: &str) -> Result<(), CoreError> {
        let path = self.resolve_path(relative_path)?;
        if !path.exists() {
            return Err(CoreError::NotFound(format!(
                "Folder not found: {relative_path}"
            )));
        }
        if !path.is_dir() {
            return Err(CoreError::Validation(format!(
                "Not a folder: {relative_path}"
            )));
        }
        fs::remove_dir_all(&path).map_err(|e| CoreError::Validation(e.to_string()))
    }

    /// Resolves and ensures that relative_path does not escape root.
    /// Rejects leading `/` or `\` that indicate absolute paths or root references,
    /// and ensures canonical target stays strictly inside `self.root`.
    fn resolve_path(&self, relative_path: &str) -> Result<PathBuf, CoreError> {
        let trimmed = relative_path.trim();
        if trimmed.is_empty() {
            return Err(CoreError::InvalidPath(
                "Empty blob path is forbidden".into(),
            ));
        }

        // Quick syntax reject on empty paths or invalid relative paths
        let rel = Path::new(relative_path);

        // Check components for ParentDir or Root/Prefix
        for comp in rel.components() {
            match comp {
                Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                    return Err(CoreError::InvalidPath(format!(
                        "Absolute, root, or parent traversal paths are forbidden: {relative_path}"
                    )));
                }
                _ => {}
            }
        }

        // Build target path
        let target = self.root.join(rel);

        // Canonicalize
        let canonical_target = if target.exists() {
            target
                .canonicalize()
                .map_err(|e| CoreError::Validation(e.to_string()))?
        } else {
            // Find existing ancestor
            let mut ancestor = target.as_path();
            let mut uncreated_parts = Vec::new();
            while !ancestor.exists() {
                if let Some(file_name) = ancestor.file_name() {
                    uncreated_parts.push(file_name);
                }
                if let Some(parent) = ancestor.parent() {
                    ancestor = parent;
                } else {
                    break;
                }
            }

            if !ancestor.exists() {
                return Err(CoreError::InvalidPath(
                    "Cannot resolve target path within vault root".into(),
                ));
            }

            let canonical_ancestor = ancestor
                .canonicalize()
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let mut final_path = canonical_ancestor;
            for part in uncreated_parts.into_iter().rev() {
                final_path.push(part);
            }
            final_path
        };

        // Strict root confinement check
        if !canonical_target.starts_with(&self.root) {
            return Err(CoreError::InvalidPath(format!(
                "Path escapes vault root: {relative_path}"
            )));
        }

        Ok(canonical_target)
    }
}

impl BlobStore for VaultBlobStore {
    fn read_blob(&self, relative_path: &str) -> Result<Vec<u8>, CoreError> {
        let path = self.resolve_path(relative_path)?;
        if !path.exists() {
            return Err(CoreError::NotFound(format!(
                "Blob not found: {relative_path}"
            )));
        }
        fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::NotFound(e.to_string())
            } else {
                CoreError::Validation(e.to_string())
            }
        })
    }

    fn write_blob(&self, relative_path: &str, bytes: &[u8]) -> Result<(), CoreError> {
        let path = self.resolve_path(relative_path)?;
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| CoreError::Validation(e.to_string()))?;
            }
        }
        fs::write(&path, bytes).map_err(|e| CoreError::Validation(e.to_string()))?;
        Ok(())
    }

    fn delete_blob(&self, relative_path: &str) -> Result<(), CoreError> {
        let path = self.resolve_path(relative_path)?;
        if !path.exists() {
            return Err(CoreError::NotFound(format!(
                "Blob not found: {relative_path}"
            )));
        }
        fs::remove_file(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::NotFound(e.to_string())
            } else {
                CoreError::Validation(e.to_string())
            }
        })?;
        Ok(())
    }

    fn blob_exists(&self, relative_path: &str) -> Result<bool, CoreError> {
        let path = self.resolve_path(relative_path)?;
        Ok(path.exists() && path.is_file())
    }
}
