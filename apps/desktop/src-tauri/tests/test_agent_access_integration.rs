use std::sync::Arc;

use desktop_lib::server::LocalApiServer;
use snapdown_core::domain::access_key::AccessKey;
use snapdown_core::domain::bundle::{Bundle, BundleItem};
use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::ports::{AccessKeyStore, BlobStore, BundleStore, FindingStore};
use snapdown_store::sqlite::{SqliteAccessKeyStore, SqliteBundleStore, SqliteFindingStore};
use snapdown_store::vault::VaultBlobStore;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_agent_access_e2e_lifecycle_refusals_and_traversal_guards() {
    let tmp_vault = TempDir::new().unwrap();
    let vault_store = Arc::new(VaultBlobStore::new(tmp_vault.path()).unwrap());

    let db_file = NamedTempFile::new().unwrap();
    let key_store = Arc::new(SqliteAccessKeyStore::open(db_file.path()).unwrap());
    let bundle_store = Arc::new(SqliteBundleStore::open(db_file.path()).unwrap());
    let finding_store = Arc::new(SqliteFindingStore::open(db_file.path()).unwrap());

    // 1. Populate Finding and Bundle
    let fid = "018f2345-6789-7abc-8def-0123456789aa";
    let bid = "018f2345-6789-7abc-8def-0123456789bb";

    let finding = Finding {
        id: fid.into(),
        image_path: "findings/f1.png".into(),
        image_width: 1920,
        image_height: 1080,
        captured_at: "2026-08-23T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,1920,1080".into(),
    };
    let note = Note {
        id: "n-1".into(),
        finding_id: fid.into(),
        body: "Sample review note".into(),
        updated_at: "2026-08-23T10:00:00Z".into(),
    };
    finding_store.create_finding(&finding, &note, &[]).unwrap();

    let md_path = "bundles/018f2345-6789-7abc-8def-0123456789bb.md";
    let img_path = "bundles/018f2345-6789-7abc-8def-0123456789bb/burned_1.png";
    let sample_image_bytes = b"SAMPLE BURNED PNG BYTES";

    vault_store
        .write_blob(md_path, b"# Test Review Bundle\n\nFinding 1")
        .unwrap();
    vault_store
        .write_blob(img_path, sample_image_bytes)
        .unwrap();

    let bundle = Bundle::new(
        bid.into(),
        "Test Review Bundle".into(),
        "# Test Review Bundle\n\nFinding 1".into(),
        md_path.into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();
    let item = BundleItem::new("bi-1".into(), bid.into(), fid.into(), 1, img_path.into()).unwrap();
    bundle_store.create_bundle(&bundle, &[item]).unwrap();

    // 2. Start Local API Server
    let mut server = LocalApiServer::start(
        0,
        key_store.clone(),
        bundle_store.clone(),
        vault_store.clone(),
    )
    .expect("start server");

    let base_url = format!("http://127.0.0.1:{}", server.port());

    // 3. Verify GET /v1/health is unauthenticated
    let health_resp = ureq::get(&format!("{base_url}/v1/health")).call().unwrap();
    assert_eq!(health_resp.status(), 200);
    assert_eq!(health_resp.header("X-Snapdown-Service"), Some("local-api"));

    // 4. Query without key -> 401 key_required (distinct error envelope, never empty list)
    let no_key_err = ureq::get(&format!("{base_url}/v1/bundles"))
        .call()
        .unwrap_err();
    if let ureq::Error::Status(code, resp) = no_key_err {
        assert_eq!(code, 401);
        let body = resp.into_string().unwrap();
        assert!(body.contains("\"code\":\"key_required\""));
    } else {
        panic!("Expected status error 401");
    }

    // 5. Issue access key
    let secret = "sd_key_integration_test_secret_123";
    let hash = AccessKey::sha256_hex(secret.as_bytes());
    let key = AccessKey::new("k-1".into(), hash, "2026-08-23T10:00:00Z".into(), None).unwrap();
    key_store.save_key(&key).unwrap();

    // 6. Query with valid key -> 200 list with bundles
    let list_resp = ureq::get(&format!("{base_url}/v1/bundles"))
        .set("Authorization", &format!("Bearer {secret}"))
        .call()
        .unwrap();
    assert_eq!(list_resp.status(), 200);
    let list_body = list_resp.into_string().unwrap();
    assert!(list_body.contains("Test Review Bundle"));
    assert!(list_body.contains(bid));

    // 7. Query bundle detail -> 200 verbatim markdown and image list
    let bundle_resp = ureq::get(&format!("{base_url}/v1/bundles/{bid}"))
        .set("X-Snapdown-Key", secret)
        .call()
        .unwrap();
    assert_eq!(bundle_resp.status(), 200);
    let bundle_body = bundle_resp.into_string().unwrap();
    assert!(bundle_body.contains("# Test Review Bundle\\n\\nFinding 1"));
    assert!(bundle_body.contains("burned_1.png"));

    // 8. Query bundle image -> 200 raw image bytes
    let img_resp = ureq::get(&format!("{base_url}/v1/bundles/{bid}/images/burned_1.png"))
        .set("Authorization", &format!("Bearer {secret}"))
        .call()
        .unwrap();
    assert_eq!(img_resp.status(), 200);
    let mut fetched_bytes = Vec::new();
    img_resp
        .into_reader()
        .read_to_end(&mut fetched_bytes)
        .unwrap();
    assert_eq!(fetched_bytes, sample_image_bytes);

    // 9. Path traversal security attempt -> refused with 400 bad_request
    let traversal_err = ureq::get(&format!(
        "{base_url}/v1/bundles/{bid}/images/..%2Fsecret.txt"
    ))
    .set("Authorization", &format!("Bearer {secret}"))
    .call()
    .unwrap_err();
    if let ureq::Error::Status(code, resp) = traversal_err {
        assert_eq!(code, 400);
        let body = resp.into_string().unwrap();
        assert!(body.contains("\"code\":\"bad_request\""));
    } else {
        panic!("Expected 400 for path traversal");
    }

    // 10. Revoke key -> immediate refusal with 401 key_invalid (NFR-13)
    key_store.revoke_active_key("2026-08-23T12:00:00Z").unwrap();
    let revoked_err = ureq::get(&format!("{base_url}/v1/bundles"))
        .set("Authorization", &format!("Bearer {secret}"))
        .call()
        .unwrap_err();
    if let ureq::Error::Status(code, resp) = revoked_err {
        assert_eq!(code, 401);
        let body = resp.into_string().unwrap();
        assert!(body.contains("\"code\":\"key_invalid\""));
    } else {
        panic!("Expected 401 for revoked key");
    }

    server.stop();
}
