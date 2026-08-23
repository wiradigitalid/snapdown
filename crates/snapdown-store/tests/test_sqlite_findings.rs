use rusqlite::Connection;
use snapdown_core::domain::finding::{Finding, Marker, Note};
use snapdown_core::error::CoreError;
use snapdown_core::ports::FindingStore;
use snapdown_store::sqlite::migrations::{get_schema_version, run_migrations};
use snapdown_store::sqlite::SqliteFindingStore;
use tempfile::NamedTempFile;

#[test]
fn migrations_v2_apply_cleanly_and_idempotently() {
    let mut conn = Connection::open_in_memory().expect("open in memory db");

    // Apply v1 migration manually
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
        INSERT INTO schema_version (version, applied_at) VALUES (1, '2026-08-22T00:00:00Z');
        CREATE TABLE IF NOT EXISTS setting (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .expect("setup v1");

    assert_eq!(get_schema_version(&conn).unwrap(), 1);

    // Run migrations up to v2
    run_migrations(&mut conn).expect("run migrations to v2");
    assert_eq!(get_schema_version(&conn).unwrap(), 2);

    // Verify tables exist
    {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name ASC;")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"finding".to_string()));
        assert!(tables.contains(&"note".to_string()));
        assert!(tables.contains(&"marker".to_string()));
        assert!(tables.contains(&"setting".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    // Idempotency: Running migrations again should succeed without modifying version
    run_migrations(&mut conn).expect("run migrations idempotent");
    assert_eq!(get_schema_version(&conn).unwrap(), 2);
}

#[test]
fn finding_store_crud_and_transaction_guarantees() {
    let temp = NamedTempFile::new().unwrap();
    let store = SqliteFindingStore::open(temp.path()).expect("open finding store");

    assert_eq!(store.get_schema_version().unwrap(), 2);

    let finding_id = "018f2345-6789-7abc-8def-0123456789ab";
    let finding = Finding {
        id: finding_id.to_string(),
        image_path: "findings/finding-1.webp".to_string(),
        image_width: 1920,
        image_height: 1080,
        captured_at: "2026-08-23T10:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,1920,1080".to_string(),
    };

    let note = Note {
        id: "note-1".to_string(),
        finding_id: finding_id.to_string(),
        body: "Primary note body text".to_string(),
        updated_at: "2026-08-23T10:00:00Z".to_string(),
    };

    let marker1 = Marker::new(
        "marker-1".to_string(),
        finding_id.to_string(),
        1,
        0.25,
        0.5,
        "First observation point".to_string(),
    )
    .unwrap();

    let marker2 = Marker::new(
        "marker-2".to_string(),
        finding_id.to_string(),
        2,
        0.75,
        0.8,
        "Second observation point".to_string(),
    )
    .unwrap();

    // Create Finding with Note and Markers
    store
        .create_finding(&finding, &note, &[marker1.clone(), marker2.clone()])
        .expect("create finding");

    // Retrieve Finding
    let detail = store
        .get_finding(finding_id)
        .expect("get finding")
        .expect("finding must exist");
    assert_eq!(detail.finding.id, finding_id);
    assert_eq!(detail.finding.image_width, 1920);
    assert_eq!(detail.note.body, "Primary note body text");
    assert_eq!(detail.markers.len(), 2);
    assert_eq!(detail.markers[0].ordinal, 1);
    assert_eq!(detail.markers[1].ordinal, 2);

    // Update Note
    store
        .update_note(finding_id, "Updated note text", "2026-08-23T11:00:00Z")
        .expect("update note");
    let detail2 = store.get_finding(finding_id).unwrap().unwrap();
    assert_eq!(detail2.note.body, "Updated note text");
    assert_eq!(detail2.note.updated_at, "2026-08-23T11:00:00Z");

    // Add Marker
    let marker3 = store
        .add_marker(finding_id, "marker-3", 0.1, 0.2, "Third observation point")
        .expect("add marker");
    assert_eq!(marker3.ordinal, 3);

    let detail3 = store.get_finding(finding_id).unwrap().unwrap();
    assert_eq!(detail3.markers.len(), 3);

    // Update Marker position & comment
    let updated_m2 = store
        .update_marker(
            finding_id,
            "marker-2",
            0.8,
            0.9,
            "Updated second observation",
        )
        .expect("update marker");
    assert_eq!(updated_m2.ordinal, 2);
    assert_eq!(updated_m2.x, 0.8);
    assert_eq!(updated_m2.comment, "Updated second observation");

    // List findings (ordered by captured_at DESC)
    let finding2 = Finding {
        id: "018f2345-6789-7abc-8def-0123456789ac".to_string(),
        image_path: "findings/finding-2.webp".to_string(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-23T12:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "10,10,800,600".to_string(),
    };
    let note2 = Note {
        id: "note-2".to_string(),
        finding_id: "018f2345-6789-7abc-8def-0123456789ac".to_string(),
        body: "Second finding note".to_string(),
        updated_at: "2026-08-23T12:00:00Z".to_string(),
    };
    store
        .create_finding(&finding2, &note2, &[])
        .expect("create second finding");

    let list = store.list_findings().expect("list findings");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].finding.id, finding2.id); // Newer first (captured at 12:00:00Z)
    assert_eq!(list[1].finding.id, finding.id); // Older second (captured at 10:00:00Z)

    // Delete Finding with Cascade
    store.delete_finding(finding_id).expect("delete finding");
    assert!(store.get_finding(finding_id).unwrap().is_none());

    let list_after_delete = store.list_findings().unwrap();
    assert_eq!(list_after_delete.len(), 1);
    assert_eq!(list_after_delete[0].finding.id, finding2.id);
}

#[test]
fn marker_renumber_preserves_single_sequence_invariant() {
    let store = SqliteFindingStore::open_in_memory().expect("open memory store");

    let fid = "018f2345-6789-7abc-8def-0123456789aa";
    let finding = Finding {
        id: fid.to_string(),
        image_path: "findings/test.webp".to_string(),
        image_width: 1000,
        image_height: 1000,
        captured_at: "2026-08-23T00:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,1000,1000".to_string(),
    };
    let note = Note {
        id: "note-fid".to_string(),
        finding_id: fid.to_string(),
        body: "Note".to_string(),
        updated_at: "2026-08-23T00:00:00Z".to_string(),
    };
    let m1 = Marker::new("m1".into(), fid.into(), 1, 0.1, 0.1, "One".into()).unwrap();
    let m2 = Marker::new("m2".into(), fid.into(), 2, 0.2, 0.2, "Two".into()).unwrap();
    let m3 = Marker::new("m3".into(), fid.into(), 3, 0.3, 0.3, "Three".into()).unwrap();
    let m4 = Marker::new("m4".into(), fid.into(), 4, 0.4, 0.4, "Four".into()).unwrap();

    store
        .create_finding(&finding, &note, &[m1, m2, m3, m4])
        .expect("create with 4 markers");

    // Delete middle marker m2 (ordinal 2)
    store.delete_marker(fid, "m2").expect("delete marker m2");

    let detail = store.get_finding(fid).unwrap().unwrap();
    assert_eq!(detail.markers.len(), 3);
    assert_eq!(detail.markers[0].id, "m1");
    assert_eq!(detail.markers[0].ordinal, 1);
    assert_eq!(detail.markers[1].id, "m3");
    assert_eq!(detail.markers[1].ordinal, 2); // Renumbered from 3 to 2
    assert_eq!(detail.markers[2].id, "m4");
    assert_eq!(detail.markers[2].ordinal, 3); // Renumbered from 4 to 3

    // Delete first marker m1 (ordinal 1)
    store.delete_marker(fid, "m1").expect("delete marker m1");
    let detail2 = store.get_finding(fid).unwrap().unwrap();
    assert_eq!(detail2.markers.len(), 2);
    assert_eq!(detail2.markers[0].id, "m3");
    assert_eq!(detail2.markers[0].ordinal, 1); // Renumbered from 2 to 1
    assert_eq!(detail2.markers[1].id, "m4");
    assert_eq!(detail2.markers[1].ordinal, 2); // Renumbered from 3 to 2

    // Reorder markers
    store
        .reorder_markers(fid, &["m4", "m3"])
        .expect("reorder markers");
    let detail3 = store.get_finding(fid).unwrap().unwrap();
    assert_eq!(detail3.markers.len(), 2);
    assert_eq!(detail3.markers[0].id, "m4");
    assert_eq!(detail3.markers[0].ordinal, 1);
    assert_eq!(detail3.markers[1].id, "m3");
    assert_eq!(detail3.markers[1].ordinal, 2);
}

#[test]
fn coordinate_validation_and_transaction_rollback_guarantees() {
    let store = SqliteFindingStore::open_in_memory().expect("open memory store");

    let fid = "018f2345-6789-7abc-8def-0123456789ab";
    let finding = Finding {
        id: fid.to_string(),
        image_path: "findings/test.webp".to_string(),
        image_width: 1000,
        image_height: 1000,
        captured_at: "2026-08-23T00:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,1000,1000".to_string(),
    };
    let note = Note {
        id: "note-fid".to_string(),
        finding_id: fid.to_string(),
        body: "Note".to_string(),
        updated_at: "2026-08-23T00:00:00Z".to_string(),
    };

    // Out of bounds coordinates on add_marker
    let err_add = store.add_marker(fid, "m1", 1.05, 0.5, "Out of bounds");
    match err_add {
        Err(CoreError::Validation(_)) => {}
        other => panic!("Expected CoreError::Validation, got {:?}", other),
    }

    let err_add_neg = store.add_marker(fid, "m1", 0.5, -0.05, "Out of bounds negative");
    match err_add_neg {
        Err(CoreError::Validation(_)) => {}
        other => panic!("Expected CoreError::Validation, got {:?}", other),
    }

    // Out of bounds marker when creating finding
    let invalid_marker = Marker {
        id: "inv-1".to_string(),
        finding_id: fid.to_string(),
        ordinal: 1,
        x: 1.5,
        y: 0.5,
        comment: "Invalid".to_string(),
    };
    let err_create = store.create_finding(&finding, &note, &[invalid_marker]);
    match err_create {
        Err(CoreError::Validation(_)) => {}
        other => panic!("Expected CoreError::Validation, got {:?}", other),
    }

    // Ensure finding was not created (rollback)
    assert!(store.get_finding(fid).unwrap().is_none());
}
