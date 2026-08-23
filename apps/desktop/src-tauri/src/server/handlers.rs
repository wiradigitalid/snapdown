use std::io::Cursor;
use std::sync::Arc;

use serde::Serialize;
use snapdown_core::ports::{BlobStore, BundleStore};
use snapdown_store::sqlite::{SqliteAccessKeyStore, SqliteBundleStore};
use snapdown_store::vault::VaultBlobStore;
use tiny_http::{Header, Method, Request, Response, StatusCode};

use crate::server::auth::{authenticate_request, ExtractedAuth};
use crate::server::error::ErrorEnvelope;

#[derive(Serialize)]
struct BundleListItem {
    id: String,
    name: String,
    finding_count: usize,
    composed_at: String,
}

#[derive(Serialize)]
struct BundleDetailResponse {
    id: String,
    name: String,
    markdown: String,
    images: Vec<String>,
    composed_at: String,
}

pub fn handle_http_request(
    req: Request,
    key_store: Arc<SqliteAccessKeyStore>,
    bundle_store: Arc<SqliteBundleStore>,
    vault_store: Arc<VaultBlobStore>,
) {
    let method = req.method().clone();
    let url = req.url().to_string();

    // AD-5: Enforce read-only authority: any non-GET request is refused with 403 not_allowed
    if method != Method::Get {
        let envelope = ErrorEnvelope::new(
            "not_allowed",
            "Write operations are forbidden on the local agent API",
            None,
        );
        let resp = Response::new(
            StatusCode(403),
            vec![
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
            ],
            Cursor::new(envelope.to_json_bytes()),
            Some(envelope.to_json_bytes().len()),
            None,
        );
        let _ = req.respond(resp);
        return;
    }

    let clean_path = url.split('?').next().unwrap_or(&url);

    // Route 1: GET /v1/health (Unauthenticated)
    if clean_path == "/v1/health" {
        let body = b"{\"status\":\"ok\"}".to_vec();
        let resp = Response::new(
            StatusCode(200),
            vec![
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
            ],
            Cursor::new(body.clone()),
            Some(body.len()),
            None,
        );
        let _ = req.respond(resp);
        return;
    }

    // All subsequent routes require authentication
    match authenticate_request(&req, key_store.as_ref()) {
        ExtractedAuth::Missing => {
            let envelope = ErrorEnvelope::new(
                "key_required",
                "Access key required via Bearer authorization or X-Snapdown-Key header",
                None,
            );
            let resp = Response::new(
                StatusCode(401),
                vec![
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
                ],
                Cursor::new(envelope.to_json_bytes()),
                Some(envelope.to_json_bytes().len()),
                None,
            );
            let _ = req.respond(resp);
            return;
        }
        ExtractedAuth::Invalid => {
            let envelope = ErrorEnvelope::new(
                "key_invalid",
                "Provided access key is invalid or has been revoked",
                None,
            );
            let resp = Response::new(
                StatusCode(401),
                vec![
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
                ],
                Cursor::new(envelope.to_json_bytes()),
                Some(envelope.to_json_bytes().len()),
                None,
            );
            let _ = req.respond(resp);
            return;
        }
        ExtractedAuth::Valid => {}
    }

    // Route 2: GET /v1/bundles
    if clean_path == "/v1/bundles" {
        match bundle_store.list_bundles() {
            Ok(bundles) => {
                let items: Vec<BundleListItem> = bundles
                    .into_iter()
                    .map(|b| BundleListItem {
                        id: b.bundle.id,
                        name: b.bundle.name,
                        finding_count: b.items.len(),
                        composed_at: b.bundle.composed_at,
                    })
                    .collect();

                let json_bytes = serde_json::to_vec(&items).unwrap_or_default();
                let resp = Response::new(
                    StatusCode(200),
                    vec![
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                        Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
                    ],
                    Cursor::new(json_bytes.clone()),
                    Some(json_bytes.len()),
                    None,
                );
                let _ = req.respond(resp);
            }
            Err(err) => {
                let envelope = ErrorEnvelope::new("unavailable", &err.to_string(), None);
                let resp = Response::new(
                    StatusCode(503),
                    vec![
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                        Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
                    ],
                    Cursor::new(envelope.to_json_bytes()),
                    Some(envelope.to_json_bytes().len()),
                    None,
                );
                let _ = req.respond(resp);
            }
        }
        return;
    }

    // Route 3: GET /v1/bundles/:id/images/:filename
    if clean_path.starts_with("/v1/bundles/") && clean_path.contains("/images/") {
        let segments: Vec<&str> = clean_path.trim_start_matches('/').split('/').collect();
        if segments.len() == 5
            && segments[0] == "v1"
            && segments[1] == "bundles"
            && segments[3] == "images"
        {
            let bundle_id = segments[2];
            let filename = segments[4];

            // BR-84: Reject traversal attacks
            if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
                let envelope =
                    ErrorEnvelope::new("bad_request", "Invalid image filename path", None);
                let resp = Response::new(
                    StatusCode(400),
                    vec![
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                        Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
                    ],
                    Cursor::new(envelope.to_json_bytes()),
                    Some(envelope.to_json_bytes().len()),
                    None,
                );
                let _ = req.respond(resp);
                return;
            }

            let rel_image_path = format!("bundles/{bundle_id}/{filename}");
            match vault_store.read_blob(&rel_image_path) {
                Ok(bytes) => {
                    let content_type = if filename.ends_with(".png") {
                        "image/png"
                    } else if filename.ends_with(".webp") {
                        "image/webp"
                    } else {
                        "application/octet-stream"
                    };

                    let resp = Response::new(
                        StatusCode(200),
                        vec![
                            Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                                .unwrap(),
                            Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..])
                                .unwrap(),
                        ],
                        Cursor::new(bytes.clone()),
                        Some(bytes.len()),
                        None,
                    );
                    let _ = req.respond(resp);
                }
                Err(_) => {
                    let envelope = ErrorEnvelope::new(
                        "not_found",
                        &format!("Image not found: {filename}"),
                        None,
                    );
                    let resp = Response::new(
                        StatusCode(404),
                        vec![
                            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                .unwrap(),
                            Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..])
                                .unwrap(),
                        ],
                        Cursor::new(envelope.to_json_bytes()),
                        Some(envelope.to_json_bytes().len()),
                        None,
                    );
                    let _ = req.respond(resp);
                }
            }
            return;
        }
    }

    // Route 4: GET /v1/bundles/:id
    if let Some(bundle_id) = clean_path.strip_prefix("/v1/bundles/") {
        if !bundle_id.contains('/') {
            match bundle_store.get_bundle(bundle_id) {
                Ok(Some(detail)) => {
                    let images = detail
                        .items
                        .into_iter()
                        .map(|i| {
                            i.image_path
                                .split('/')
                                .next_back()
                                .unwrap_or(&i.image_path)
                                .to_string()
                        })
                        .collect();

                    let resp_obj = BundleDetailResponse {
                        id: detail.bundle.id,
                        name: detail.bundle.name,
                        markdown: detail.bundle.markdown, // AD-9 verbatim bytes
                        images,
                        composed_at: detail.bundle.composed_at,
                    };

                    let json_bytes = serde_json::to_vec(&resp_obj).unwrap_or_default();
                    let resp = Response::new(
                        StatusCode(200),
                        vec![
                            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                .unwrap(),
                            Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..])
                                .unwrap(),
                        ],
                        Cursor::new(json_bytes.clone()),
                        Some(json_bytes.len()),
                        None,
                    );
                    let _ = req.respond(resp);
                }
                Ok(None) => {
                    let envelope = ErrorEnvelope::new(
                        "not_found",
                        &format!("Bundle not found: {bundle_id}"),
                        None,
                    );
                    let resp = Response::new(
                        StatusCode(404),
                        vec![
                            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                .unwrap(),
                            Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..])
                                .unwrap(),
                        ],
                        Cursor::new(envelope.to_json_bytes()),
                        Some(envelope.to_json_bytes().len()),
                        None,
                    );
                    let _ = req.respond(resp);
                }
                Err(err) => {
                    let envelope = ErrorEnvelope::new("unavailable", &err.to_string(), None);
                    let resp = Response::new(
                        StatusCode(503),
                        vec![
                            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                .unwrap(),
                            Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..])
                                .unwrap(),
                        ],
                        Cursor::new(envelope.to_json_bytes()),
                        Some(envelope.to_json_bytes().len()),
                        None,
                    );
                    let _ = req.respond(resp);
                }
            }
            return;
        }
    }

    // Default route 404
    let envelope = ErrorEnvelope::new("not_found", "Route not found", None);
    let resp = Response::new(
        StatusCode(404),
        vec![
            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
        ],
        Cursor::new(envelope.to_json_bytes()),
        Some(envelope.to_json_bytes().len()),
        None,
    );
    let _ = req.respond(resp);
}
