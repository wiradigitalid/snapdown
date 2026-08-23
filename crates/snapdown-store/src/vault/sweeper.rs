use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use snapdown_core::error::CoreError;
use snapdown_core::ports::{BlobStore, FindingStore};

use crate::vault::VaultBlobStore;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrphanScanReport {
    pub total_vault_files: usize,
    pub referenced_files: usize,
    pub orphan_files: Vec<String>,
    pub missing_files: Vec<String>,
}

pub struct OrphanSweeper;

impl OrphanSweeper {
    /// Compares SQLite finding records against files in the vault findings directory.
    pub fn scan_orphans<F: FindingStore>(
        finding_store: &F,
        vault_store: &VaultBlobStore,
    ) -> Result<OrphanScanReport, CoreError> {
        let findings = finding_store.list_findings()?;
        let mut referenced_set: HashSet<String> = HashSet::new();
        let mut missing_files: Vec<String> = Vec::new();

        for item in findings {
            let p = item.finding.image_path.clone();
            if !p.is_empty() {
                // Check if file exists on disk
                if !vault_store.blob_exists(&p).unwrap_or(false) {
                    missing_files.push(p.clone());
                }
                referenced_set.insert(p);
            }
        }

        // Scan all files under vault root/findings
        let findings_dir = vault_store.root().join("findings");
        let mut vault_files: Vec<String> = Vec::new();

        if findings_dir.exists() && findings_dir.is_dir() {
            Self::collect_files_recursive(&findings_dir, vault_store.root(), &mut vault_files)?;
        }

        let mut orphan_files: Vec<String> = Vec::new();
        for file in &vault_files {
            let normalized = file.replace('\\', "/");
            if !referenced_set.contains(&normalized) {
                orphan_files.push(normalized);
            }
        }

        Ok(OrphanScanReport {
            total_vault_files: vault_files.len(),
            referenced_files: referenced_set.len(),
            orphan_files,
            missing_files,
        })
    }

    /// Deletes unreferenced orphan files found on disk.
    pub fn clean_orphans(
        vault_store: &VaultBlobStore,
        orphan_files: &[String],
    ) -> Result<usize, CoreError> {
        let mut deleted_count = 0;
        for rel_path in orphan_files {
            if vault_store.delete_blob(rel_path).is_ok() {
                deleted_count += 1;
            }
        }
        Ok(deleted_count)
    }

    fn collect_files_recursive(
        dir: &Path,
        root: &Path,
        files: &mut Vec<String>,
    ) -> Result<(), CoreError> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    Self::collect_files_recursive(&p, root, files)?;
                } else if p.is_file() {
                    if let Ok(rel) = p.strip_prefix(root) {
                        files.push(rel.to_string_lossy().to_string().replace('\\', "/"));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::SqliteFindingStore;
    use snapdown_core::domain::finding::{Finding, Note};
    use tempfile::TempDir;

    #[test]
    fn orphan_sweeper_detects_unreferenced_and_missing_files() {
        let tmp_vault = TempDir::new().unwrap();
        let vault_store = VaultBlobStore::new(tmp_vault.path()).unwrap();
        let finding_store = SqliteFindingStore::open_in_memory().unwrap();

        // 1. Write referenced file on disk
        vault_store
            .write_blob("findings/finding1.png", b"image bytes")
            .unwrap();

        // 2. Write orphan file on disk (not in DB)
        vault_store
            .write_blob("findings/orphan.png", b"orphan bytes")
            .unwrap();

        // 3. Register finding in DB pointing to finding1.png and missing finding2.png
        let f1 = Finding {
            id: "fid-1".into(),
            image_path: "findings/finding1.png".into(),
            image_width: 800,
            image_height: 600,
            captured_at: "2026-08-23T10:00:00Z".into(),
            source_monitor: "DISPLAY1".into(),
            region: "0,0,800,600".into(),
        };
        let n1 = Note {
            id: "n1".into(),
            finding_id: "fid-1".into(),
            body: "Note".into(),
            updated_at: "2026-08-23T10:00:00Z".into(),
        };
        finding_store.create_finding(&f1, &n1, &[]).unwrap();

        let f2 = Finding {
            id: "fid-2".into(),
            image_path: "findings/finding2.png".into(),
            image_width: 800,
            image_height: 600,
            captured_at: "2026-08-23T11:00:00Z".into(),
            source_monitor: "DISPLAY1".into(),
            region: "0,0,800,600".into(),
        };
        let n2 = Note {
            id: "n2".into(),
            finding_id: "fid-2".into(),
            body: "Note 2".into(),
            updated_at: "2026-08-23T11:00:00Z".into(),
        };
        finding_store.create_finding(&f2, &n2, &[]).unwrap();

        let report = OrphanSweeper::scan_orphans(&finding_store, &vault_store).unwrap();

        assert_eq!(report.total_vault_files, 2);
        assert_eq!(report.referenced_files, 2);
        assert_eq!(report.orphan_files, vec!["findings/orphan.png".to_string()]);
        assert_eq!(
            report.missing_files,
            vec!["findings/finding2.png".to_string()]
        );

        // Clean orphans
        let cleaned = OrphanSweeper::clean_orphans(&vault_store, &report.orphan_files).unwrap();
        assert_eq!(cleaned, 1);
        assert!(!vault_store.blob_exists("findings/orphan.png").unwrap());
    }
}
