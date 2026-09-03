use snapdown_core::domain::bundle::{Bundle, BundleItem};
use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::ports::{BlobStore, BundleStore, FindingStore};
use snapdown_store::sqlite::{SqliteBundleStore, SqliteFindingStore};
use snapdown_store::vault::VaultBlobStore;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn migrations_apply_cleanly_and_create_bundle_tables() {
    let temp = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open bundle store");

    assert_eq!(bundle_store.get_schema_version().unwrap(), 9);
}

/// Ticket 15 ("An edited Bundle says so", ticket 09's option B): a database written before
/// `updated_at` existed opens, migrates to v9, and every Bundle already in it reads `updated_at`
/// equal to its own `composed_at` - "never edited", which is true of a row nothing has touched since
/// migration v8 left it.
///
/// The backfill this proves was SEEN RED FIRST: with the version-9 `Migration` entry commented out
/// of `MIGRATIONS`, `run_migrations` had nothing whose `version > 8` to apply, so
/// `get_schema_version()` stayed at 8 and this test's own `assert_eq!(..., 9)` failed for real
/// (`left: 8, right: 9`, not a panic from the column read further down) - then the entry was
/// restored and this test went green. See this ticket's final report for the exact command run.
#[test]
fn opening_a_pre_migration_database_backfills_updated_at_from_composed_at() {
    let temp = NamedTempFile::new().unwrap();

    // The OLD schema, built by hand exactly as migration v3/v6/v8 left it - no `updated_at` column
    // at all - stamped at schema_version 8. `bundle_item` must exist too (migration v6's shape, no
    // `finding_id` foreign key): stamping `schema_version` straight to 8 skips migrations 1-8
    // entirely (`run_migrations` only applies a migration whose `version > current_version`), so
    // this fixture must build every table `get_bundle` reads, not just `bundle`, or the read below
    // fails on a table that was never created rather than on the thing this test means to prove.
    // Two Bundle rows, so the backfill is proven for more than one.
    {
        let conn = rusqlite::Connection::open(temp.path()).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
            CREATE TABLE bundle (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                markdown TEXT NOT NULL,
                markdown_path TEXT NOT NULL,
                composed_at TEXT NOT NULL
            );
            CREATE TABLE bundle_item (
                id TEXT PRIMARY KEY,
                bundle_id TEXT NOT NULL,
                finding_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                image_path TEXT NOT NULL,
                UNIQUE(bundle_id, finding_id)
            );
            INSERT INTO bundle (id, name, markdown, markdown_path, composed_at)
            VALUES ('b-old-1', 'Old Bundle One', '# Old One', 'bundles/old1.md', '2026-01-01T00:00:00Z');
            INSERT INTO bundle (id, name, markdown, markdown_path, composed_at)
            VALUES ('b-old-2', 'Old Bundle Two', '# Old Two', 'bundles/old2.md', '2026-02-02T00:00:00Z');
            INSERT INTO schema_version (version, applied_at) VALUES (8, '2026-01-01T00:00:00Z');
            "#,
        )
        .expect("build the pre-migration fixture");
    }

    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open must migrate cleanly");
    assert_eq!(bundle_store.get_schema_version().unwrap(), 9);

    let one = bundle_store
        .get_bundle("b-old-1")
        .unwrap()
        .expect("the first pre-migration Bundle must still be there");
    assert_eq!(one.bundle.updated_at, one.bundle.composed_at);
    assert_eq!(one.bundle.updated_at, "2026-01-01T00:00:00Z");

    let two = bundle_store
        .get_bundle("b-old-2")
        .unwrap()
        .expect("the second pre-migration Bundle must still be there");
    assert_eq!(two.bundle.updated_at, two.bundle.composed_at);
    assert_eq!(two.bundle.updated_at, "2026-02-02T00:00:00Z");
}

#[test]
fn bundle_store_crud_and_cascade_operations() {
    let temp = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open bundle store");
    let finding_store = SqliteFindingStore::open(temp.path()).expect("open finding store");

    // 1. Create a Finding first
    let fid = "018f2345-6789-7abc-8def-0123456789aa";
    let finding = Finding {
        id: fid.to_string(),
        image_path: "findings/finding-1.webp".to_string(),
        image_width: 1920,
        image_height: 1080,
        captured_at: "2026-08-23T10:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,1920,1080".to_string(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
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

    // 4. Update name and Markdown together (ticket 14, BR-5: the two never make sense apart), moving
    //    the last-edited time (ticket 15) to exactly the instant named.
    assert_eq!(
        detail.bundle.updated_at, detail.bundle.composed_at,
        "a freshly created Bundle must read as never edited"
    );
    bundle_store
        .update_bundle_name_and_markdown(
            bundle_id,
            "Renamed Bundle",
            "# Updated Markdown content",
            "2026-08-24T09:00:00Z",
        )
        .expect("update name and markdown");
    let detail2 = bundle_store.get_bundle(bundle_id).unwrap().unwrap();
    assert_eq!(detail2.bundle.name, "Renamed Bundle");
    assert_eq!(detail2.bundle.markdown, "# Updated Markdown content");
    assert_eq!(detail2.bundle.updated_at, "2026-08-24T09:00:00Z");
    assert_eq!(
        detail2.bundle.composed_at, detail.bundle.composed_at,
        "composed_at must never move"
    );

    // 5. List Bundles
    let list = bundle_store.list_bundles().expect("list bundles");
    assert_eq!(list.len(), 1);

    // 6. Delete Bundle
    bundle_store
        .delete_bundle(bundle_id)
        .expect("delete bundle");
    assert!(bundle_store.get_bundle(bundle_id).unwrap().is_none());
}

#[test]
fn deleting_a_finding_leaves_its_bundle_item_in_place() {
    // BUG-1 / FR-13: Deleting a Finding must NOT cascade-delete bundle_item rows
    let temp = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open bundle store");
    let finding_store = SqliteFindingStore::open(temp.path()).expect("open finding store");

    let fid1 = "018f2345-6789-7abc-8def-0123456789a1";
    let fid2 = "018f2345-6789-7abc-8def-0123456789a2";

    let finding1 = Finding {
        id: fid1.to_string(),
        image_path: "findings/finding-1.webp".to_string(),
        image_width: 1920,
        image_height: 1080,
        captured_at: "2026-08-23T10:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,1920,1080".to_string(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note1 = Note {
        id: "note-1".to_string(),
        finding_id: fid1.to_string(),
        body: "Finding 1 note".to_string(),
        updated_at: "2026-08-23T10:00:00Z".to_string(),
    };
    finding_store
        .create_finding(&finding1, &note1, &[])
        .expect("create finding 1");

    let finding2 = Finding {
        id: fid2.to_string(),
        image_path: "findings/finding-2.webp".to_string(),
        image_width: 1280,
        image_height: 720,
        captured_at: "2026-08-23T10:05:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,1280,720".to_string(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note2 = Note {
        id: "note-2".to_string(),
        finding_id: fid2.to_string(),
        body: "Finding 2 note".to_string(),
        updated_at: "2026-08-23T10:05:00Z".to_string(),
    };
    finding_store
        .create_finding(&finding2, &note2, &[])
        .expect("create finding 2");

    let bundle_id = "018f2345-6789-7abc-8def-0123456789b1";
    let bundle = Bundle::new(
        bundle_id.to_string(),
        "Multi-Item Review Bundle".to_string(),
        "# Summary\n\n- Finding 1\n- Finding 2".to_string(),
        "bundles/review_b1.md".to_string(),
        "2026-08-23T12:00:00Z".to_string(),
    )
    .unwrap();

    let item1 = BundleItem::new(
        "bi-1".to_string(),
        bundle_id.to_string(),
        fid1.to_string(),
        1,
        "bundles/review_b1/finding_1_burned.webp".to_string(),
    )
    .unwrap();

    let item2 = BundleItem::new(
        "bi-2".to_string(),
        bundle_id.to_string(),
        fid2.to_string(),
        2,
        "bundles/review_b1/finding_2_burned.webp".to_string(),
    )
    .unwrap();

    bundle_store
        .create_bundle(&bundle, &[item1, item2])
        .expect("create bundle with 2 items");

    // Verify bundle has 2 items initially
    let detail_before = bundle_store.get_bundle(bundle_id).unwrap().unwrap();
    assert_eq!(detail_before.items.len(), 2);

    // Delete finding 1 from finding_store
    finding_store
        .delete_finding(fid1)
        .expect("delete finding 1");

    // Finding 1 is gone from finding store
    assert!(finding_store.get_finding(fid1).unwrap().is_none());

    // Bundle MUST still retain BOTH bundle_item records (FR-13)
    let detail_after = bundle_store.get_bundle(bundle_id).unwrap().unwrap();
    assert_eq!(
        detail_after.items.len(),
        2,
        "Bundle items must not be deleted when a source finding is deleted"
    );
    assert_eq!(detail_after.items[0].finding_id, fid1);
    assert_eq!(detail_after.items[0].position, 1);
    assert_eq!(detail_after.items[1].finding_id, fid2);
    assert_eq!(detail_after.items[1].position, 2);
}

#[test]
fn deleting_a_finding_leaves_the_bundle_markdown_byte_identical() {
    // AD-9 & FR-13: Markdown content in bundle must remain byte-identical after finding deletion
    let temp = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open bundle store");
    let finding_store = SqliteFindingStore::open(temp.path()).expect("open finding store");

    let fid = "018f2345-6789-7abc-8def-0123456789ac";
    let finding = Finding {
        id: fid.to_string(),
        image_path: "findings/finding-golden.webp".to_string(),
        image_width: 1920,
        image_height: 1080,
        captured_at: "2026-08-23T10:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,1920,1080".to_string(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "note-golden".to_string(),
        finding_id: fid.to_string(),
        body: "Golden Note Body".to_string(),
        updated_at: "2026-08-23T10:00:00Z".to_string(),
    };
    finding_store
        .create_finding(&finding, &note, &[])
        .expect("create finding");

    let bundle_id = "018f2345-6789-7abc-8def-0123456789bc";
    let original_markdown = "# Executive Report\n\n## Finding 1\nGolden Note Body\n\n![Finding 1](bundles/018f2345-6789-7abc-8def-0123456789bc/finding_1_burned.webp)\n";

    let bundle = Bundle::new(
        bundle_id.to_string(),
        "Executive Report".to_string(),
        original_markdown.to_string(),
        "bundles/executive_report.md".to_string(),
        "2026-08-23T12:00:00Z".to_string(),
    )
    .unwrap();

    let item = BundleItem::new(
        "bi-golden".to_string(),
        bundle_id.to_string(),
        fid.to_string(),
        1,
        "bundles/018f2345-6789-7abc-8def-0123456789bc/finding_1_burned.webp".to_string(),
    )
    .unwrap();

    bundle_store
        .create_bundle(&bundle, &[item])
        .expect("create bundle");

    // Delete finding
    finding_store.delete_finding(fid).expect("delete finding");

    // Retrieve bundle
    let loaded = bundle_store.get_bundle(bundle_id).unwrap().unwrap();
    assert_eq!(
        loaded.bundle.markdown, original_markdown,
        "Bundle markdown column must remain byte-identical"
    );
    assert_eq!(
        loaded.bundle.markdown.as_bytes(),
        original_markdown.as_bytes()
    );
}

#[test]
fn deleting_a_finding_leaves_the_bundles_own_image_copy_in_the_vault() {
    // SCN-05 / FR-13: The Bundle keeps its own burned copy of the image in the Vault
    let tmp_vault = TempDir::new().unwrap();
    let vault_store = VaultBlobStore::new(tmp_vault.path()).unwrap();

    let db_file = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(db_file.path()).unwrap();
    let finding_store = SqliteFindingStore::open(db_file.path()).unwrap();

    let fid = "018f2345-6789-7abc-8def-0123456789ad";
    let bid = "018f2345-6789-7abc-8def-0123456789bd";

    let finding_img = "findings/f_original.webp";
    let bundle_burned_img = "bundles/018f2345-6789-7abc-8def-0123456789bd/finding_1_burned.webp";

    // Write original finding image and burned bundle copy
    vault_store
        .write_blob(finding_img, b"ORIGINAL_FINDING_PIXELS")
        .unwrap();
    vault_store
        .write_blob(bundle_burned_img, b"BURNED_BUNDLE_COPY_PIXELS")
        .unwrap();

    let finding = Finding {
        id: fid.into(),
        image_path: finding_img.into(),
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
        id: "n-ad".into(),
        finding_id: fid.into(),
        body: "Note AD".into(),
        updated_at: "2026-08-23T10:00:00Z".into(),
    };
    finding_store.create_finding(&finding, &note, &[]).unwrap();

    let bundle = Bundle::new(
        bid.into(),
        "Burned Copy Test".into(),
        "# Burned Copy".into(),
        "bundles/burned.md".into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();
    let item = BundleItem::new(
        "bi-ad".into(),
        bid.into(),
        fid.into(),
        1,
        bundle_burned_img.into(),
    )
    .unwrap();
    bundle_store.create_bundle(&bundle, &[item]).unwrap();

    // Finding deletion removes finding DB row and finding image blob
    vault_store.delete_blob(finding_img).unwrap();
    finding_store.delete_finding(fid).unwrap();

    // Original finding image is deleted
    assert!(!vault_store.blob_exists(finding_img).unwrap());

    // Bundle's burned copy in vault MUST remain present and intact (FR-13, SCN-05)
    assert!(
        vault_store.blob_exists(bundle_burned_img).unwrap(),
        "Bundle burned image copy must stay intact in the vault"
    );
    let burned_bytes = vault_store.read_blob(bundle_burned_img).unwrap();
    assert_eq!(burned_bytes, b"BURNED_BUNDLE_COPY_PIXELS");
}

#[test]
fn deleting_a_bundle_still_cascades_to_its_items() {
    // FR-14: Deleting a Bundle still cascade-deletes all of its bundle_item rows
    let temp = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open bundle store");

    let bundle_id = "018f2345-6789-7abc-8def-0123456789be";
    let bundle = Bundle::new(
        bundle_id.to_string(),
        "Cascade Test Bundle".to_string(),
        "# Cascade Test".to_string(),
        "bundles/cascade.md".to_string(),
        "2026-08-23T12:00:00Z".to_string(),
    )
    .unwrap();

    let item1 = BundleItem::new(
        "bi-c1".to_string(),
        bundle_id.to_string(),
        "fid-1".to_string(),
        1,
        "bundles/cascade/img1.webp".to_string(),
    )
    .unwrap();

    let item2 = BundleItem::new(
        "bi-c2".to_string(),
        bundle_id.to_string(),
        "fid-2".to_string(),
        2,
        "bundles/cascade/img2.webp".to_string(),
    )
    .unwrap();

    bundle_store
        .create_bundle(&bundle, &[item1, item2])
        .expect("create bundle");

    // Verify items exist
    let detail = bundle_store.get_bundle(bundle_id).unwrap().unwrap();
    assert_eq!(detail.items.len(), 2);

    // Delete bundle
    bundle_store
        .delete_bundle(bundle_id)
        .expect("delete bundle");

    // Verify bundle is None
    assert!(bundle_store.get_bundle(bundle_id).unwrap().is_none());

    // Direct SQLite check to verify bundle_item table has 0 rows for this bundle_id
    let conn = rusqlite::Connection::open(temp.path()).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM bundle_item WHERE bundle_id = ?1;",
            rusqlite::params![bundle_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "All bundle_item records must be cascade-deleted");
}
