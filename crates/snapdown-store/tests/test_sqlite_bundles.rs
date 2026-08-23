use snapdown_core::domain::bundle::{Bundle, BundleItem};
use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::ports::{BundleStore, FindingStore};
use snapdown_store::sqlite::{SqliteBundleStore, SqliteFindingStore};
use tempfile::NamedTempFile;

#[test]
fn migrations_v3_apply_cleanly_and_create_bundle_tables() {
    let temp = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open bundle store");

    assert_eq!(bundle_store.get_schema_version().unwrap(), 3);
}

#[test]
fn bundle_store_crud_and_cascade_operations() {
    let temp = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open bundle store");
    let finding_store = SqliteFindingStore::open(temp.path()).expect("open finding store");

    // 1. Create a Finding first to satisfy foreign key in bundle_item
    let fid = "018f2345-6789-7abc-8def-0123456789aa";
    let finding = Finding {
        id: fid.to_string(),
        image_path: "findings/finding-1.webp".to_string(),
        image_width: 1920,
        image_height: 1080,
        captured_at: "2026-08-23T10:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,1920,1080".to_string(),
    };
    let note = Note {
        id: "note-1".to_string(),
        finding_id: fid.to_string(),
        body: "Finding note".to_string(),
        updated_at: "2026-08-23T10:00:00Z".to_string(),
    };
    finding_store
        .create_finding(&finding, &note, &[])
        .expect("create finding");

    // 2. Create Bundle with BundleItem
    let bundle_id = "018f2345-6789-7abc-8def-0123456789bb";
    let bundle = Bundle::new(
        bundle_id.to_string(),
        "Wave 3 Test Bundle".to_string(),
        "# Summary Review\n\nFinding 1 description".to_string(),
        "bundles/w3_test.md".to_string(),
        "2026-08-23T12:00:00Z".to_string(),
    )
    .unwrap();

    let item = BundleItem::new(
        "bi-1".to_string(),
        bundle_id.to_string(),
        fid.to_string(),
        1,
        "bundles/w3_test/finding_1_burned.webp".to_string(),
    )
    .unwrap();

    bundle_store
        .create_bundle(&bundle, std::slice::from_ref(&item))
        .expect("create bundle");

    // 3. Read Bundle detail
    let detail = bundle_store
        .get_bundle(bundle_id)
        .expect("get bundle")
        .expect("bundle must exist");

    assert_eq!(detail.bundle.name, "Wave 3 Test Bundle");
    assert_eq!(detail.items.len(), 1);
    assert_eq!(detail.items[0].finding_id, fid);
    assert_eq!(detail.items[0].position, 1);

    // 4. Update Markdown
    bundle_store
        .update_bundle_markdown(bundle_id, "# Updated Markdown content")
        .expect("update markdown");
    let detail2 = bundle_store.get_bundle(bundle_id).unwrap().unwrap();
    assert_eq!(detail2.bundle.markdown, "# Updated Markdown content");

    // 5. List Bundles
    let list = bundle_store.list_bundles().expect("list bundles");
    assert_eq!(list.len(), 1);

    // 6. Delete Bundle
    bundle_store
        .delete_bundle(bundle_id)
        .expect("delete bundle");
    assert!(bundle_store.get_bundle(bundle_id).unwrap().is_none());
}
