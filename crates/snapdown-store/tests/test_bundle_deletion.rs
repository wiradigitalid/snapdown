use snapdown_core::domain::bundle::{Bundle, BundleItem};
use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::ports::{BlobStore, BundleStore, FindingStore};
use snapdown_store::sqlite::{SqliteBundleStore, SqliteFindingStore};
use snapdown_store::vault::VaultBlobStore;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn bundle_deletion_with_file_synchronization_and_cascade() {
    let tmp_vault = TempDir::new().unwrap();
    let vault_store = VaultBlobStore::new(tmp_vault.path()).unwrap();

    let db_file = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(db_file.path()).unwrap();
    let finding_store = SqliteFindingStore::open(db_file.path()).unwrap();

    let fid = "018f2345-6789-7abc-8def-0123456789aa";
    let bid = "018f2345-6789-7abc-8def-0123456789bb";

    // 1. Create finding in shared database
    let finding = Finding {
        id: fid.into(),
        image_path: "findings/f1.png".into(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-23T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,800,600".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "n1".into(),
        finding_id: fid.into(),
        body: "Note".into(),
        updated_at: "2026-08-23T10:00:00Z".into(),
    };
    finding_store.create_finding(&finding, &note, &[]).unwrap();

    // 2. Write bundle files on disk
    let md_path = "bundles/test_bundle.md";
    let img_path = "bundles/test_bundle/burned_1.png";
    vault_store.write_blob(md_path, b"# Test Bundle").unwrap();
    vault_store
        .write_blob(img_path, b"burned image bytes")
        .unwrap();

    assert!(vault_store.blob_exists(md_path).unwrap());
    assert!(vault_store.blob_exists(img_path).unwrap());

    // 3. Create bundle in store
    let bundle = Bundle::new(
        bid.into(),
        "Test Bundle".into(),
        "# Test Bundle".into(),
        md_path.into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();

    let item = BundleItem::new("bi-1".into(), bid.into(), fid.into(), 1, img_path.into()).unwrap();
    bundle_store.create_bundle(&bundle, &[item]).unwrap();

    // Verify bundle created
    assert!(bundle_store.get_bundle(bid).unwrap().is_some());

    // 4. Perform synchronized deletion: remove files and delete DB record
    vault_store.delete_blob(md_path).unwrap();
    vault_store.delete_blob(img_path).unwrap();
    bundle_store.delete_bundle(bid).unwrap();

    // 5. Verify neither DB record nor files exist
    assert!(bundle_store.get_bundle(bid).unwrap().is_none());
    assert!(!vault_store.blob_exists(md_path).unwrap());
    assert!(!vault_store.blob_exists(img_path).unwrap());
}
