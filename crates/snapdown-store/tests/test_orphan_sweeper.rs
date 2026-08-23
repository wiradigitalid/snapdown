use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::ports::{BlobStore, FindingStore};
use snapdown_store::sqlite::SqliteFindingStore;
use snapdown_store::vault::{OrphanSweeper, VaultBlobStore};
use tempfile::TempDir;

#[test]
fn delete_finding_removes_vault_file_before_db_row() {
    let tmp = TempDir::new().unwrap();
    let vault_store = VaultBlobStore::new(tmp.path()).unwrap();
    let finding_store = SqliteFindingStore::open_in_memory().unwrap();

    let fid = "018f2345-6789-7abc-8def-0123456789aa";
    let img_path = "findings/test_delete.png";

    // Write file to vault
    vault_store
        .write_blob(img_path, b"test image bytes")
        .unwrap();
    assert!(vault_store.blob_exists(img_path).unwrap());

    // Register finding
    let f = Finding {
        id: fid.into(),
        image_path: img_path.into(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-23T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,800,600".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let n = Note {
        id: "n1".into(),
        finding_id: fid.into(),
        body: "Note".into(),
        updated_at: "2026-08-23T10:00:00Z".into(),
    };
    finding_store.create_finding(&f, &n, &[]).unwrap();

    // Synchronous deletion: remove file then remove DB record
    vault_store.delete_blob(img_path).unwrap();
    assert!(!vault_store.blob_exists(img_path).unwrap());

    finding_store.delete_finding(fid).unwrap();
    assert!(finding_store.get_finding(fid).unwrap().is_none());
}

#[test]
fn orphan_sweeper_detects_unreferenced_blobs() {
    let tmp = TempDir::new().unwrap();
    let vault_store = VaultBlobStore::new(tmp.path()).unwrap();
    let finding_store = SqliteFindingStore::open_in_memory().unwrap();

    vault_store
        .write_blob("findings/orphan_1.png", b"1")
        .unwrap();
    vault_store
        .write_blob("findings/orphan_2.png", b"2")
        .unwrap();

    let report = OrphanSweeper::scan_orphans(&finding_store, &vault_store).unwrap();
    assert_eq!(report.orphan_files.len(), 2);
    assert_eq!(report.missing_files.len(), 0);

    let cleaned = OrphanSweeper::clean_orphans(&vault_store, &report.orphan_files).unwrap();
    assert_eq!(cleaned, 2);

    let report2 = OrphanSweeper::scan_orphans(&finding_store, &vault_store).unwrap();
    assert_eq!(report2.orphan_files.len(), 0);
}
