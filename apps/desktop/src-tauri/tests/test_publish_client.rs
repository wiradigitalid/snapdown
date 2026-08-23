use std::io::Cursor;
use std::thread;

use desktop_lib::publish::PublishClient;
use snapdown_core::domain::bundle::Bundle;
use snapdown_core::domain::publication::Publication;
use snapdown_core::ports::{BundleStore, PublicationStore};
use snapdown_store::sqlite::{SqliteBundleStore, SqlitePublicationStore};
use tempfile::NamedTempFile;
use tiny_http::{Header, Method, Response, Server, StatusCode};

#[test]
fn test_publish_client_and_unpublish_lifecycle() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();

    // Mock web-api
    thread::spawn(move || {
        let mut published_slugs = std::collections::HashSet::new();

        while let Ok(req) = server.recv() {
            let url = req.url().to_string();
            let method = req.method().clone();

            if method == Method::Put && url.starts_with("/publish/") {
                let slug = url.trim_start_matches("/publish/").to_string();
                published_slugs.insert(slug.clone());
                let resp = Response::new(
                    StatusCode(200),
                    vec![
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    ],
                    Cursor::new(format!("{{\"status\":\"published\",\"slug\":\"{slug}\"}}")),
                    None,
                    None,
                );
                let _ = req.respond(resp);
            } else if method == Method::Delete && url.starts_with("/publish/") {
                let slug = url.trim_start_matches("/publish/").to_string();
                published_slugs.remove(&slug);
                let resp = Response::new(
                    StatusCode(200),
                    vec![
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    ],
                    Cursor::new(b"{\"status\":\"unpublished\"}"),
                    None,
                    None,
                );
                let _ = req.respond(resp);
            } else if method == Method::Get && url.starts_with("/publish/") {
                let slug = url.trim_start_matches("/publish/").to_string();
                if published_slugs.contains(&slug) {
                    let resp = Response::new(
                        StatusCode(200),
                        vec![
                            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                .unwrap(),
                        ],
                        Cursor::new(format!("{{\"status\":\"served\",\"slug\":\"{slug}\"}}")),
                        None,
                        None,
                    );
                    let _ = req.respond(resp);
                } else {
                    let resp = Response::new(
                        StatusCode(404),
                        vec![
                            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                .unwrap(),
                        ],
                        Cursor::new(
                            b"{\"error\":{\"code\":\"not_found\",\"message\":\"Not found\"}}",
                        ),
                        None,
                        None,
                    );
                    let _ = req.respond(resp);
                }
            } else {
                let _ = req.respond(Response::empty(404));
            }
        }
    });

    let client = PublishClient::new(format!("http://127.0.0.1:{port}"), None);

    // 1. Publish bundle
    let pub_res = client
        .publish("slug-test-1", "# Markdown", &[("img1.png", b"BYTES")])
        .expect("publish");
    assert_eq!(pub_res.status, "published");

    // 2. Reconcile -> true
    assert!(client.reconcile("slug-test-1").unwrap());

    // 3. Unpublish
    client.unpublish("slug-test-1").expect("unpublish");

    // 4. Reconcile -> false
    assert!(!client.reconcile("slug-test-1").unwrap());
}

#[test]
fn test_publish_store_sticky_error_behavior() {
    let temp = NamedTempFile::new().unwrap();
    let pub_store = SqlitePublicationStore::open(temp.path()).unwrap();
    let bundle_store = SqliteBundleStore::open(temp.path()).unwrap();

    let bid = "018f2345-6789-7abc-8def-0123456789aa";
    let bundle = Bundle::new(
        bid.into(),
        "Bundle".into(),
        "# Doc".into(),
        "b.md".into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();
    bundle_store.create_bundle(&bundle, &[]).unwrap();

    let pub_record = Publication::new(
        "p-1".into(),
        bid.into(),
        "slug-sticky".into(),
        "http://127.0.0.1:8080".into(),
        "2026-08-23T10:00:00Z".into(),
        None,
        None,
    )
    .unwrap();
    pub_store.save(&pub_record).unwrap();

    // Sticky error on unpublish failure
    pub_store
        .set_last_error(bid, Some("Connection refused by web-api"))
        .unwrap();

    let loaded = pub_store.get_by_bundle_id(bid).unwrap().unwrap();
    assert!(loaded.is_live()); // Remains live locally (BR-20, BR-96)
    assert_eq!(
        loaded.last_error,
        Some("Connection refused by web-api".into())
    );
}
