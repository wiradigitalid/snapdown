use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;

use desktop_lib::commands::bundle::{
    copy_bundle_to_clipboard_impl, create_bundle_impl, delete_bundle_impl, CreateBundleInput,
};
use desktop_lib::commands::finding::delete_finding_impl;
use desktop_lib::hotkey::DesktopHotkeyRegistrar;
use desktop_lib::startup::{DesktopStartupRegistrar, NoopAutoStartBackend};
use desktop_lib::state::AppState;
use snapdown_core::domain::bundle::{Bundle, BundleItem};
use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::domain::publication::Publication;
use snapdown_core::domain::setting::{Setting, SettingKey, SettingValue};
use snapdown_core::ports::{BlobStore, BundleStore, FindingStore, PublicationStore, SettingsStore};
use snapdown_store::sqlite::{
    SqliteAccessKeyStore, SqliteBundleStore, SqliteFindingStore, SqlitePublicationStore,
    SqliteSettingsStore,
};
use snapdown_store::vault::VaultBlobStore;
use tempfile::{NamedTempFile, TempDir};
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn build_test_app(
    db_path: &std::path::Path,
    vault_path: &std::path::Path,
    web_service_url: Option<String>,
) -> AppState {
    let settings_store = Arc::new(SqliteSettingsStore::open(db_path).unwrap());
    let finding_store = Arc::new(SqliteFindingStore::open(db_path).unwrap());
    let bundle_store = Arc::new(SqliteBundleStore::open(db_path).unwrap());
    let access_key_store = Arc::new(SqliteAccessKeyStore::open(db_path).unwrap());
    let publication_store = Arc::new(SqlitePublicationStore::open(db_path).unwrap());

    // Configure vault path in settings
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    settings_store
        .set(&Setting::new(
            SettingKey::VaultPath,
            SettingValue::String(vault_path.to_string_lossy().to_string()),
            now.clone(),
        ))
        .unwrap();

    if let Some(url) = web_service_url {
        settings_store
            .set(&Setting::new(
                SettingKey::WebServiceAddress,
                SettingValue::String(url),
                now,
            ))
            .unwrap();
    }

    let hotkey_registrar = Arc::new(Mutex::new(DesktopHotkeyRegistrar::new(
        settings_store.clone(),
        None,
    )));
    let startup_registrar = Arc::new(Mutex::new(DesktopStartupRegistrar::new(Arc::new(
        NoopAutoStartBackend,
    ))));

    AppState {
        settings_store,
        finding_store,
        bundle_store,
        access_key_store,
        publication_store,
        hotkey_registrar,
        startup_registrar,
    }
}

#[test]
fn composition_that_cannot_open_the_vault_is_refused_not_silently_skipped() {
    // BUG-9 / FR-10 / AD-2: When vault path is invalid, create_bundle must return Err and write zero rows
    let temp_db = NamedTempFile::new().unwrap();
    let temp_file_as_vault = NamedTempFile::new().unwrap(); // A file, not a directory -> VaultBlobStore::new fails

    let state = build_test_app(temp_db.path(), temp_file_as_vault.path(), None);

    let input = CreateBundleInput {
        name: "Failing Vault Bundle".to_string(),
        finding_ids: vec![],
    };

    let result = create_bundle_impl(input, &state);
    assert!(
        result.is_err(),
        "create_bundle must return error when vault cannot open"
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("Failed to open vault"),
        "Error message should mention failed vault open, got: {err_msg}"
    );

    // Verify 0 rows in bundle store
    let bundles = state.bundle_store.list_bundles().unwrap();
    assert_eq!(bundles.len(), 0, "No bundle row should be inserted in DB");
}

#[test]
fn composition_that_cannot_write_its_markdown_writes_no_bundle_row() {
    // BUG-9 / FR-10 / AD-2: If writing markdown blob fails, create_bundle must abort and write no DB row
    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();

    // Create a regular file at temp_vault/bundles so fs::create_dir_all / fs::write fails
    let bundles_blocker = temp_vault.path().join("bundles");
    std::fs::write(
        &bundles_blocker,
        b"I am a file, blocking directory creation",
    )
    .unwrap();

    let state = build_test_app(temp_db.path(), temp_vault.path(), None);

    let input = CreateBundleInput {
        name: "Unwritable Markdown Bundle".to_string(),
        finding_ids: vec![],
    };

    let result = create_bundle_impl(input, &state);
    assert!(
        result.is_err(),
        "create_bundle must fail if writing markdown fails"
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("Failed to write bundle markdown file"),
        "Error message should mention markdown write failure, got: {err_msg}"
    );

    // Verify 0 bundle rows inserted into database
    let bundles = state.bundle_store.list_bundles().unwrap();
    assert_eq!(bundles.len(), 0, "No bundle row should be inserted in DB");
}

#[test]
fn deleting_a_published_bundle_whose_unpublish_fails_aborts_and_reports() {
    // BUG-9 / BR-20 / BR-23: When unpublish fails, delete_bundle must abort, return Err, and keep local DB & files
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();

    // Mock web service returning 500 on DELETE /publish/<slug>
    thread::spawn(move || {
        while let Ok(req) = server.recv() {
            if req.method() == &Method::Delete {
                let resp = Response::new(
                    StatusCode(500),
                    vec![
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    ],
                    Cursor::new(br#"{"error":{"code":"internal_error","message":"Server Error"}}"#),
                    None,
                    None,
                );
                let _ = req.respond(resp);
            } else {
                let _ = req.respond(Response::empty(404));
            }
        }
    });

    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();
    let web_service_url = format!("http://127.0.0.1:{port}");

    let state = build_test_app(
        temp_db.path(),
        temp_vault.path(),
        Some(web_service_url.clone()),
    );

    let bid = "b-published-fail-test";
    let md_path = "bundles/b-published-fail-test.md";
    let img_path = "bundles/b-published-fail-test/finding_1_burned.webp";

    // Write vault files
    let vault_store = VaultBlobStore::new(temp_vault.path()).unwrap();
    vault_store
        .write_blob(md_path, b"# Published Markdown")
        .unwrap();
    vault_store.write_blob(img_path, b"IMAGE_BYTES").unwrap();

    // Insert bundle and item in store
    let bundle = Bundle::new(
        bid.into(),
        "Published Bundle".into(),
        "# Published Markdown".into(),
        md_path.into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();
    let item = BundleItem::new(
        "bi-pub-1".into(),
        bid.into(),
        "fid-1".into(),
        1,
        img_path.into(),
    )
    .unwrap();
    state.bundle_store.create_bundle(&bundle, &[item]).unwrap();

    // Insert live publication record
    let pub_record = Publication::new(
        "pub-1".into(),
        bid.into(),
        "slug-unpub-fail".into(),
        web_service_url,
        "2026-08-23T10:00:00Z".into(),
        None,
        None,
    )
    .unwrap();
    state.publication_store.save(&pub_record).unwrap();

    // Attempt delete_bundle
    let result = delete_bundle_impl(bid, &state);
    assert!(
        result.is_err(),
        "delete_bundle must fail when unpublish fails"
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("Unpublish failed"),
        "Error message should mention unpublish failure, got: {err_msg}"
    );

    // Invariant checks (BR-20, BR-23):
    // 1. Bundle and items must still exist in SQLite
    let loaded_bundle = state.bundle_store.get_bundle(bid).unwrap();
    assert!(
        loaded_bundle.is_some(),
        "Bundle record must not be deleted from DB"
    );
    assert_eq!(
        loaded_bundle.unwrap().items.len(),
        1,
        "Bundle items must not be deleted"
    );

    // 2. Publication record must remain live and carry last_error
    let loaded_pub = state
        .publication_store
        .get_by_bundle_id(bid)
        .unwrap()
        .unwrap();
    assert!(
        loaded_pub.is_live(),
        "Publication must remain marked as live"
    );
    assert!(
        loaded_pub.last_error.is_some(),
        "Publication must record last_error"
    );

    // 3. Vault files must remain on disk
    assert!(
        vault_store.blob_exists(md_path).unwrap(),
        "Markdown file must stay on disk"
    );
    assert!(
        vault_store.blob_exists(img_path).unwrap(),
        "Image file must stay on disk"
    );
}

#[test]
fn deleting_a_bundle_reports_an_image_copy_it_could_not_remove() {
    // BUG-9 / FR-14 / NFR-5: When a file cannot be removed, delete_bundle reports error and does not silently succeed
    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();

    let state = build_test_app(temp_db.path(), temp_vault.path(), None);

    let bid = "b-unremovable-file-test";
    let md_path = "bundles/b-unremovable-file-test.md";
    let img_path = "bundles/b-unremovable-file-test/locked_burned.webp";

    let vault_store = VaultBlobStore::new(temp_vault.path()).unwrap();
    vault_store
        .write_blob(md_path, b"# Unremovable File Test")
        .unwrap();
    vault_store.write_blob(img_path, b"LOCKED_BYTES").unwrap();

    let bundle = Bundle::new(
        bid.into(),
        "Locked Bundle".into(),
        "# Unremovable File Test".into(),
        md_path.into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();
    let item = BundleItem::new(
        "bi-lock-1".into(),
        bid.into(),
        "fid-lock-1".into(),
        1,
        img_path.into(),
    )
    .unwrap();
    state.bundle_store.create_bundle(&bundle, &[item]).unwrap();

    // Make the image file unremovable so deletion fails
    let full_img_path = temp_vault
        .path()
        .join("bundles")
        .join("b-unremovable-file-test")
        .join("locked_burned.webp");

    #[cfg(windows)]
    let _lock = {
        use std::os::windows::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&full_img_path)
            .unwrap()
    };

    #[cfg(not(windows))]
    {
        let parent = full_img_path.parent().unwrap();
        let mut p_perms = std::fs::metadata(parent).unwrap().permissions();
        p_perms.set_readonly(true);
        std::fs::set_permissions(parent, p_perms).unwrap();
    }

    // Call delete_bundle
    let result = delete_bundle_impl(bid, &state);

    #[cfg(windows)]
    drop(_lock);

    #[cfg(not(windows))]
    {
        let parent = full_img_path.parent().unwrap();
        let mut cleanup_perms = std::fs::metadata(parent).unwrap().permissions();
        cleanup_perms.set_readonly(false);
        let _ = std::fs::set_permissions(parent, cleanup_perms);
    }

    assert!(
        result.is_err(),
        "delete_bundle must fail when image file cannot be deleted"
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("Failed to delete bundle image"),
        "Error message should mention failed image deletion, got: {err_msg}"
    );

    // Bundle record is not deleted from DB
    let loaded = state.bundle_store.get_bundle(bid).unwrap();
    assert!(
        loaded.is_some(),
        "Bundle record must remain in DB when file cleanup fails"
    );
}

#[test]
fn a_bundle_whose_source_finding_is_gone_still_copies_the_same_bytes() {
    // Golden markdown & clipboard guarantee: deleting a finding does not alter bundle clipboard output
    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();

    let state = build_test_app(temp_db.path(), temp_vault.path(), None);

    let fid = "fid-clipboard-source";
    let bid = "b-clipboard-golden";
    let md_path = "bundles/b-clipboard-golden.md";
    let img_path = "bundles/b-clipboard-golden/finding_1_burned.webp";

    let vault_store = VaultBlobStore::new(temp_vault.path()).unwrap();
    vault_store
        .write_blob(md_path, b"# Golden Copy Content")
        .unwrap();
    vault_store.write_blob(img_path, b"BURNED_PIXELS").unwrap();
    vault_store
        .write_blob("findings/fid-clipboard-source.webp", b"FINDING_PIXELS")
        .unwrap();

    let finding = Finding {
        id: fid.into(),
        image_path: "findings/fid-clipboard-source.webp".into(),
        image_width: 1920,
        image_height: 1080,
        captured_at: "2026-08-23T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,1920,1080".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "note-cb".into(),
        finding_id: fid.into(),
        body: "Clipboard note".into(),
        updated_at: "2026-08-23T10:00:00Z".into(),
    };
    state
        .finding_store
        .create_finding(&finding, &note, &[])
        .unwrap();

    let expected_markdown = "# Summary Report\n\n- Finding Note: Clipboard note\n- Burned Image: ![Image](bundles/b-clipboard-golden/finding_1_burned.webp)\n";
    let bundle = Bundle::new(
        bid.into(),
        "Summary Report".into(),
        expected_markdown.into(),
        md_path.into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();
    let item =
        BundleItem::new("bi-cb-1".into(), bid.into(), fid.into(), 1, img_path.into()).unwrap();
    state.bundle_store.create_bundle(&bundle, &[item]).unwrap();

    // 1. Copy bundle before finding deletion
    let copy_before = copy_bundle_to_clipboard_impl(bid, &state).expect("copy before deletion");
    assert_eq!(copy_before, expected_markdown);

    // 2. Delete source finding from library
    delete_finding_impl(fid, &state).expect("delete finding");
    assert!(state.finding_store.get_finding(fid).unwrap().is_none());

    // 3. Copy bundle after finding deletion
    let copy_after = copy_bundle_to_clipboard_impl(bid, &state).expect("copy after deletion");
    assert_eq!(
        copy_after, expected_markdown,
        "Clipboard copy must be byte-identical after finding deletion"
    );
    assert_eq!(copy_before.as_bytes(), copy_after.as_bytes());
}