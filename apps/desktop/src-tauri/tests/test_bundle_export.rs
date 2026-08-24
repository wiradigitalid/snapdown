use std::sync::{Arc, Mutex};

use desktop_lib::commands::bundle::{create_bundle_impl, CreateBundleInput};
use desktop_lib::hotkey::DesktopHotkeyRegistrar;
use desktop_lib::startup::{DesktopStartupRegistrar, NoopAutoStartBackend};
use desktop_lib::state::AppState;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_core::domain::finding::{Finding, Marker, Note};
use snapdown_core::domain::setting::{Setting, SettingKey, SettingValue};
use snapdown_core::ports::{BlobStore, BundleStore, FindingStore, SettingsStore};
use snapdown_store::sqlite::{
    SqliteAccessKeyStore, SqliteBundleStore, SqliteFindingStore, SqlitePublicationStore,
    SqliteSettingsStore,
};
use snapdown_store::vault::VaultBlobStore;
use tempfile::{NamedTempFile, TempDir};

const BADGE_OUTER_RADIUS: i32 = 16;

fn make_gradient_png(w: u32, h: u32) -> Vec<u8> {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 255) / w.max(1)) as u8;
            let g = ((y * 255) / h.max(1)) as u8;
            let b = (((x + y) * 128) / (w + h).max(1)) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    encoder
        .write_image(img.as_raw(), w, h, ExtendedColorType::Rgba8)
        .expect("Failed to encode test gradient PNG fixture");
    bytes
}

fn make_valid_png_header_with_corrupt_idat(w: u32, h: u32) -> Vec<u8> {
    let raw_png = make_gradient_png(w, h);
    let mut corrupt_png = raw_png.clone();
    let pos = corrupt_png
        .windows(4)
        .position(|w| w == b"IDAT")
        .expect("IDAT chunk must exist in valid PNG");
    let idat_payload_start = pos + 4;
    for i in 0..16.min(corrupt_png.len() - idat_payload_start) {
        corrupt_png[idat_payload_start + i] ^= 0xFF;
    }
    corrupt_png
}

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

/// Covers:
/// - BUG-19 / FR-8 / AD-2 / UC-9 alt-1: For every returned BundleItem, a real file exists at image_path
///   and decodes to the Finding's dimensions.
/// - Covers both a Finding with active markers and a Finding with zero markers in one Bundle.
/// - BR-13 / UC-9 failure flow 1: Missing source image file refuses composition by naming the Finding.
#[test]
fn a_bundle_item_image_path_holds_a_file_that_decodes() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();

    let state = build_test_app(temp_db.path(), temp_vault.path(), None);
    let vault_store = VaultBlobStore::new(temp_vault.path()).unwrap();

    // Finding 1: 800x600 with active marker
    let fid1 = "f-item-1";
    let img_path1 = "findings/f1_src.png";
    let src_bytes1 = make_gradient_png(800, 600);
    vault_store.write_blob(img_path1, &src_bytes1).unwrap();

    let f1 = Finding {
        id: fid1.into(),
        image_path: img_path1.into(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-24T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,800,600".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note1 = Note {
        id: "n1".into(),
        finding_id: fid1.into(),
        body: "Note with markers".into(),
        updated_at: "2026-08-24T10:00:00Z".into(),
    };
    let m1 = Marker::new("m1".into(), fid1.into(), 1, 0.5, 0.5, "Badge text".into()).unwrap();
    state
        .finding_store
        .create_finding(&f1, &note1, &[m1])
        .unwrap();

    // Finding 2: 400x300 with zero markers (UC-9 alt-1)
    let fid2 = "f-item-2";
    let img_path2 = "findings/f2_src.png";
    let src_bytes2 = make_gradient_png(400, 300);
    vault_store.write_blob(img_path2, &src_bytes2).unwrap();

    let f2 = Finding {
        id: fid2.into(),
        image_path: img_path2.into(),
        image_width: 400,
        image_height: 300,
        captured_at: "2026-08-24T10:05:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,400,300".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note2 = Note {
        id: "n2".into(),
        finding_id: fid2.into(),
        body: "Note without markers".into(),
        updated_at: "2026-08-24T10:05:00Z".into(),
    };
    state
        .finding_store
        .create_finding(&f2, &note2, &[])
        .unwrap();

    let input = CreateBundleInput {
        name: "Export Integrity Bundle".to_string(),
        finding_ids: vec![fid1.to_string(), fid2.to_string()],
    };

    let result = create_bundle_impl(input, &state).expect("create_bundle_impl should succeed");
    assert_eq!(result.items.len(), 2);

    for item in &result.items {
        // 1. blob_exists is true
        assert!(
            vault_store.blob_exists(&item.image_path).unwrap(),
            "Blob must exist in vault at {}",
            item.image_path
        );

        // 2. read_blob returns bytes
        let bytes = vault_store
            .read_blob(&item.image_path)
            .expect("read_blob must succeed for item image");
        assert!(!bytes.is_empty(), "Read image bytes must not be empty");

        // 3. image::load_from_memory returns Ok
        let decoded = image::load_from_memory(&bytes)
            .expect("Exported bundle image must decode cleanly as PNG");

        // 4. Decoded dimensions match finding's dimensions
        if item.finding_id == fid1 {
            assert_eq!(decoded.width(), 800);
            assert_eq!(decoded.height(), 600);
        } else if item.finding_id == fid2 {
            assert_eq!(decoded.width(), 400);
            assert_eq!(decoded.height(), 300);
        }
    }

    // BR-13: Refusal when a Finding's image is missing from vault
    let fid3 = "f-item-missing";
    let f3 = Finding {
        id: fid3.into(),
        image_path: "findings/missing_file.png".into(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-24T10:10:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,800,600".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note3 = Note {
        id: "n3".into(),
        finding_id: fid3.into(),
        body: "Missing image finding".into(),
        updated_at: "2026-08-24T10:10:00Z".into(),
    };
    state
        .finding_store
        .create_finding(&f3, &note3, &[])
        .unwrap();

    let failing_input = CreateBundleInput {
        name: "Missing Source Image Bundle".to_string(),
        finding_ids: vec![fid3.to_string()],
    };
    let fail_result = create_bundle_impl(failing_input, &state);
    assert!(
        fail_result.is_err(),
        "Composition must be refused when source image file is missing"
    );
    let err_msg = fail_result.unwrap_err();
    assert!(
        err_msg.contains(fid3) && err_msg.contains("image file is missing from vault"),
        "Error message must name the missing finding id and indicate missing file from vault (BR-13), got: {err_msg}"
    );
}

/// Covers:
/// - FR-8 / SCN-04: Burned copy carries markers with non-empty comments.
/// - Markers with whitespace comments are never drawn (SCN-04).
/// - Pixels outside badge footprint are identical to the source.
#[test]
fn an_exported_bundle_image_carries_the_markers_of_its_finding() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();

    let state = build_test_app(temp_db.path(), temp_vault.path(), None);
    let vault_store = VaultBlobStore::new(temp_vault.path()).unwrap();

    let width = 600;
    let height = 400;
    let src_bytes = make_gradient_png(width, height);
    let src_img = image::load_from_memory(&src_bytes).unwrap().to_rgba8();

    let fid = "fid-markers";
    let img_path = "findings/src_markers.png";
    vault_store.write_blob(img_path, &src_bytes).unwrap();

    let m_active = Marker::new(
        "m-active".into(),
        fid.into(),
        1,
        0.25,
        0.25,
        "Active defect note".into(),
    )
    .unwrap();
    let m_whitespace =
        Marker::new("m-ws".into(), fid.into(), 2, 0.75, 0.75, "   \t\n ".into()).unwrap();

    let f = Finding {
        id: fid.into(),
        image_path: img_path.into(),
        image_width: width,
        image_height: height,
        captured_at: "2026-08-24T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,600,400".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "n-m".into(),
        finding_id: fid.into(),
        body: "Finding with markers".into(),
        updated_at: "2026-08-24T10:00:00Z".into(),
    };
    state
        .finding_store
        .create_finding(&f, &note, &[m_active, m_whitespace])
        .unwrap();

    let input = CreateBundleInput {
        name: "Marker Annotation Bundle".to_string(),
        finding_ids: vec![fid.to_string()],
    };

    let detail = create_bundle_impl(input, &state).unwrap();
    let item = &detail.items[0];

    let exported_bytes = vault_store.read_blob(&item.image_path).unwrap();
    let exported_img = image::load_from_memory(&exported_bytes).unwrap().to_rgba8();

    let cx_active = (0.25 * width as f64).round() as i32; // 150
    let cy_active = (0.25 * height as f64).round() as i32; // 100
    let cx_ws = (0.75 * width as f64).round() as i32; // 450
    let cy_ws = (0.75 * height as f64).round() as i32; // 300

    // 1. Active marker badge center differs from source
    assert_ne!(
        src_img.get_pixel(cx_active as u32, cy_active as u32),
        exported_img.get_pixel(cx_active as u32, cy_active as u32),
        "Active marker center must differ from source pixel"
    );

    // 2. SCN-04: Whitespace-only comment marker is NEVER drawn -> center is identical to source
    assert_eq!(
        src_img.get_pixel(cx_ws as u32, cy_ws as u32),
        exported_img.get_pixel(cx_ws as u32, cy_ws as u32),
        "Whitespace-only marker center must remain identical to source (SCN-04)"
    );

    // 3. Pixel away from every badge (e.g. 10, 10) is byte-identical
    let px: u32 = 10;
    let py: u32 = 10;
    let r_sq = BADGE_OUTER_RADIUS * BADGE_OUTER_RADIUS;
    let dist1_sq = (px as i32 - cx_active) * (px as i32 - cx_active)
        + (py as i32 - cy_active) * (py as i32 - cy_active);
    assert!(dist1_sq > r_sq);

    assert_eq!(
        src_img.get_pixel(px, py),
        exported_img.get_pixel(px, py),
        "Pixel ({px}, {py}) outside badge footprint must be identical to source"
    );
}

/// Covers:
/// - AD-4 / BR-8: The export burn takes already-reduced stored bytes and MUST NOT re-reduce them.
/// - Uses a fixture whose long edge exceeds default budget's max_long_edge (2560 > 1920).
#[test]
fn a_bundle_export_does_not_re_reduce_the_stored_image() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();

    let state = build_test_app(temp_db.path(), temp_vault.path(), None);
    let vault_store = VaultBlobStore::new(temp_vault.path()).unwrap();

    let width = 2560;
    let height = 1440;
    let src_bytes = make_gradient_png(width, height);

    let fid = "fid-large-stored";
    let img_path = "findings/large_stored.png";
    vault_store.write_blob(img_path, &src_bytes).unwrap();

    let f = Finding {
        id: fid.into(),
        image_path: img_path.into(),
        image_width: width,
        image_height: height,
        captured_at: "2026-08-24T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,2560,1440".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "n-large".into(),
        finding_id: fid.into(),
        body: "Large stored image".into(),
        updated_at: "2026-08-24T10:00:00Z".into(),
    };
    state.finding_store.create_finding(&f, &note, &[]).unwrap();

    let input = CreateBundleInput {
        name: "No Re-Reduction Bundle".to_string(),
        finding_ids: vec![fid.to_string()],
    };

    let detail = create_bundle_impl(input, &state).unwrap();
    let item = &detail.items[0];

    let exported_bytes = vault_store.read_blob(&item.image_path).unwrap();
    let decoded = image::load_from_memory(&exported_bytes)
        .expect("Exported bundle image must decode cleanly");

    // Decoded dimensions must preserve original 2560x1440 without re-reduction
    assert_eq!(
        decoded.width(),
        2560,
        "Exported width must remain 2560 without downscaling"
    );
    assert_eq!(
        decoded.height(),
        1440,
        "Exported height must remain 1440 without downscaling"
    );
}

/// Covers:
/// - BUG-20: MarkerBurner & bundle composition refuse a corrupt source image even with zero markers.
/// - Valid PNG header with corrupt IDAT payload fixture proves reach into the decoder.
/// - Refusal writes no DB row (list_bundles() is empty) and leaves no files under bundles/{id}/.
#[test]
fn a_corrupt_source_is_refused_even_when_no_marker_is_drawn() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();

    let state = build_test_app(temp_db.path(), temp_vault.path(), None);
    let vault_store = VaultBlobStore::new(temp_vault.path()).unwrap();

    // 1. Fixture reach proof: Valid PNG header + corrupt IDAT reaches decoder and fails
    let corrupt_bytes = make_valid_png_header_with_corrupt_idat(400, 300);
    let decode_result = image::load_from_memory(&corrupt_bytes);
    assert!(
        decode_result.is_err(),
        "Corrupt IDAT fixture must fail image::load_from_memory"
    );

    let fid = "fid-corrupt";
    let img_path = "findings/corrupt_header_valid.png";
    vault_store.write_blob(img_path, &corrupt_bytes).unwrap();

    let f = Finding {
        id: fid.into(),
        image_path: img_path.into(),
        image_width: 400,
        image_height: 300,
        captured_at: "2026-08-24T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,400,300".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "n-corrupt".into(),
        finding_id: fid.into(),
        body: "Corrupt source finding".into(),
        updated_at: "2026-08-24T10:00:00Z".into(),
    };
    // Finding with ZERO markers
    state.finding_store.create_finding(&f, &note, &[]).unwrap();

    let input = CreateBundleInput {
        name: "Corrupt Source Bundle".to_string(),
        finding_ids: vec![fid.to_string()],
    };

    let result = create_bundle_impl(input, &state);
    assert!(
        result.is_err(),
        "create_bundle_impl must return Err for corrupt source image even with zero markers"
    );

    // 2. Zero rows in bundle store
    let bundles = state.bundle_store.list_bundles().unwrap();
    assert_eq!(
        bundles.len(),
        0,
        "No bundle record must exist in DB on failure"
    );

    // 3. No bundle files left behind in vault
    let bundles_dir = temp_vault.path().join("bundles");
    if bundles_dir.exists() {
        let entries: Vec<_> = std::fs::read_dir(&bundles_dir).unwrap().collect();
        assert_eq!(
            entries.len(),
            0,
            "No files or subdirectories should remain under bundles/ on failed composition"
        );
    }
}

/// Covers:
/// - BUG-21 / FR-8 / AD-4: The composed Markdown document references BundleItem.image_path
///   (the Bundle's burned copy) rather than Finding.image_path.
/// - Verified both on the in-memory return and on the file stored in the vault.
#[test]
fn the_composed_markdown_references_the_bundles_burned_copy() {
    let temp_db = NamedTempFile::new().unwrap();
    let temp_vault = TempDir::new().unwrap();

    let state = build_test_app(temp_db.path(), temp_vault.path(), None);
    let vault_store = VaultBlobStore::new(temp_vault.path()).unwrap();

    let fid1 = "fid-md-ref-1";
    let img_src1 = "findings/source_clean_1.png";
    let bytes1 = make_gradient_png(640, 480);
    vault_store.write_blob(img_src1, &bytes1).unwrap();

    let f1 = Finding {
        id: fid1.into(),
        image_path: img_src1.into(),
        image_width: 640,
        image_height: 480,
        captured_at: "2026-08-24T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,640,480".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note1 = Note {
        id: "n-md-1".into(),
        finding_id: fid1.into(),
        body: "First finding note".into(),
        updated_at: "2026-08-24T10:00:00Z".into(),
    };
    let m1 = Marker::new("m-md-1".into(), fid1.into(), 1, 0.3, 0.3, "Defect 1".into()).unwrap();
    state
        .finding_store
        .create_finding(&f1, &note1, &[m1])
        .unwrap();

    let input = CreateBundleInput {
        name: "Burned Image Reference Review".to_string(),
        finding_ids: vec![fid1.to_string()],
    };

    let detail = create_bundle_impl(input, &state).expect("Bundle composition should succeed");
    let bundle_id = &detail.bundle.id;
    let expected_item_path = format!("bundles/{bundle_id}/finding_1_burned.png");
    let expected_md_ref = format!("![Finding 1](./{expected_item_path})");

    // 1. Returned Bundle.markdown references the bundle's burned copy
    assert!(
        detail.bundle.markdown.contains(&expected_md_ref),
        "Bundle markdown must reference the burned copy path '{}', got:\n{}",
        expected_md_ref,
        detail.bundle.markdown
    );
    assert!(
        !detail.bundle.markdown.contains(img_src1),
        "Bundle markdown must NOT reference the source finding clean image path '{}'",
        img_src1
    );

    // 2. Vault markdown file on disk matches byte-identically and references the burned copy
    let disk_bytes = vault_store
        .read_blob(&detail.bundle.markdown_path)
        .expect("Markdown file must exist in vault");
    let disk_markdown = String::from_utf8(disk_bytes).unwrap();
    assert_eq!(disk_markdown, detail.bundle.markdown);
    assert!(disk_markdown.contains(&expected_md_ref));
    assert!(!disk_markdown.contains(img_src1));
}
