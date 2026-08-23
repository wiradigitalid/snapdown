use std::io::Read;
use std::sync::Arc;

use snapdown_core::domain::access_key::AccessKey;
use snapdown_core::domain::bundle::{Bundle, BundleItem};
use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::ports::{AccessKeyStore, BlobStore, BundleStore, FindingStore};
use snapdown_store::sqlite::{SqliteAccessKeyStore, SqliteBundleStore, SqliteFindingStore};
use snapdown_store::vault::VaultBlobStore;
use tempfile::{NamedTempFile, TempDir};

use desktop_lib::server::LocalApiServer;

#[test]
fn local_api_server_routes_and_authentication_lifecycle() {
    let tmp_vault = TempDir::new().unwrap();
    let vault_store = Arc::new(VaultBlobStore::new(tmp_vault.path()).unwrap());

    let db_file = NamedTempFile::new().unwrap();
    let key_store = Arc::new(SqliteAccessKeyStore::open(db_file.path()).unwrap());
    let bundle_store = Arc::new(SqliteBundleStore::open(db_file.path()).unwrap());
    let finding_store = Arc::new(SqliteFindingStore::open(db_file.path()).unwrap());

    // 1. Issue an access key
    let secret = "sd_key_test_secret_123456789";
    let hash = AccessKey::sha256_hex(secret.as_bytes());
    let key = AccessKey::new("k-1".into(), hash, "2026-08-23T10:00:00Z".into(), None).unwrap();
    key_store.save_key(&key).unwrap();

    // 2. Populate finding & bundle in database and vault
    let fid = "018f2345-6789-7abc-8def-0123456789aa";
    let bid = "018f2345-6789-7abc-8def-0123456789bb";

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
        id: "n-1".into(),
        finding_id: fid.into(),
        body: "Sample note".into(),
        updated_at: "2026-08-23T10:00:00Z".into(),
    };
    finding_store.create_finding(&finding, &note, &[]).unwrap();

    let md_path = "bundles/test_bundle.md";
    let img_path = "bundles/018f2345-6789-7abc-8def-0123456789bb/burned_1.png";
    let sample_image_bytes = b"PNG IMAGE PAYLOAD TEST BYTES";

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

    // 3. Start local API server on dynamic loopback port
    let mut server = LocalApiServer::start(
        0,
        key_store.clone(),
        bundle_store.clone(),
        vault_store.clone(),
    )
    .expect("start local server");

    let base_url = format!("http://127.0.0.1:{}", server.port());

    // Test Route 1: GET /v1/health (Unauthenticated)
    {
        let resp = ureq::get(&format!("{base_url}/v1/health")).call().unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.header("X-Snapdown-Service"), Some("local-api"));
        let mut body = String::new();
        resp.into_reader().read_to_string(&mut body).unwrap();
        assert!(body.contains("\"status\":\"ok\""));
    }

    // Test Route 2: GET /v1/bundles without key -> 401 key_required
    {
        let resp = ureq::get(&format!("{base_url}/v1/bundles")).call();
        assert!(resp.is_err());
        let err = resp.unwrap_err();
        if let ureq::Error::Status(code, resp) = err {
            assert_eq!(code, 401);
            let mut body = String::new();
            resp.into_reader().read_to_string(&mut body).unwrap();
            assert!(body.contains("\"code\":\"key_required\""));
        } else {
            panic!("Expected status error");
        }
    }

    // Test Route 3: GET /v1/bundles with invalid key -> 401 key_invalid
    {
        let resp = ureq::get(&format!("{base_url}/v1/bundles"))
            .set("Authorization", "Bearer wrong_secret_token")
            .call();
        assert!(resp.is_err());
        let err = resp.unwrap_err();
        if let ureq::Error::Status(code, resp) = err {
            assert_eq!(code, 401);
            let mut body = String::new();
            resp.into_reader().read_to_string(&mut body).unwrap();
            assert!(body.contains("\"code\":\"key_invalid\""));
        } else {
            panic!("Expected status error");
        }
    }

    // Test Route 4: GET /v1/bundles with valid key -> 200 list
    {
        let resp = ureq::get(&format!("{base_url}/v1/bundles"))
            .set("Authorization", &format!("Bearer {secret}"))
            .call()
            .unwrap();
        assert_eq!(resp.status(), 200);
        let mut body = String::new();
        resp.into_reader().read_to_string(&mut body).unwrap();
        assert!(body.contains("Test Review Bundle"));
    }

    // Test Route 5: GET /v1/bundles/:id with valid key -> 200 markdown verbatim
    {
        let resp = ureq::get(&format!("{base_url}/v1/bundles/{bid}"))
            .set("X-Snapdown-Key", secret)
            .call()
            .unwrap();
        assert_eq!(resp.status(), 200);
        let mut body = String::new();
        resp.into_reader().read_to_string(&mut body).unwrap();
        assert!(body.contains("# Test Review Bundle\\n\\nFinding 1"));
        assert!(body.contains("burned_1.png"));
    }

    // Test Route 6: GET /v1/bundles/:id/images/:filename -> 200 image bytes
    {
        let resp = ureq::get(&format!("{base_url}/v1/bundles/{bid}/images/burned_1.png"))
            .set("Authorization", &format!("Bearer {secret}"))
            .call()
            .unwrap();
        assert_eq!(resp.status(), 200);
        let mut bytes = Vec::new();
        resp.into_reader().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, sample_image_bytes);
    }

    // Test Route 7: GET /v1/bundles/:id/images/traversal -> 400 bad_request
    {
        let resp = ureq::get(&format!(
            "{base_url}/v1/bundles/{bid}/images/..%2F..%2Fsecret.txt"
        ))
        .set("Authorization", &format!("Bearer {secret}"))
        .call();
        assert!(resp.is_err());
    }

    // Test Non-GET method -> 403 not_allowed (AD-5)
    {
        let resp = ureq::post(&format!("{base_url}/v1/bundles"))
            .set("Authorization", &format!("Bearer {secret}"))
            .call();
        assert!(resp.is_err());
        let err = resp.unwrap_err();
        if let ureq::Error::Status(code, resp) = err {
            assert_eq!(code, 403);
            let mut body = String::new();
            resp.into_reader().read_to_string(&mut body).unwrap();
            assert!(body.contains("\"code\":\"not_allowed\""));
        } else {
            panic!("Expected status error");
        }
    }

    server.stop();
}
