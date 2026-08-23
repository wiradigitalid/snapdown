use std::io::Cursor;
use std::thread;

use snapdown_bridge::client::LocalApiClient;
use snapdown_bridge::mcp::McpHandler;
use tiny_http::{Header, Method, Response, Server, StatusCode};

#[test]
fn test_mcp_handshake_and_tool_dispatch() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();

    // Spawn mock local api
    thread::spawn(move || {
        while let Ok(req) = server.recv() {
            let url = req.url().to_string();
            let method = req.method().clone();

            if method == Method::Get && url == "/v1/health" {
                let resp = Response::new(
                    StatusCode(200),
                    vec![
                        Header::from_bytes(&b"X-Snapdown-Service"[..], &b"local-api"[..]).unwrap(),
                    ],
                    Cursor::new(b"{\"status\":\"ok\"}"),
                    Some(15),
                    None,
                );
                let _ = req.respond(resp);
            } else if method == Method::Get && url == "/v1/bundles" {
                // Check auth
                let mut auth_valid = false;
                for h in req.headers() {
                    if h.field.as_str().as_str().eq_ignore_ascii_case("authorization")
                        && h.value == "Bearer valid_key"
                    {
                        auth_valid = true;
                        break;
                    }
                }

                if auth_valid {
                    let resp_bytes = b"[{\"id\":\"b-1\",\"name\":\"Test Bundle\",\"finding_count\":1,\"composed_at\":\"2026-08-23T10:00:00Z\"}]";
                    let resp = Response::new(
                        StatusCode(200),
                        vec![],
                        Cursor::new(resp_bytes),
                        Some(resp_bytes.len()),
                        None,
                    );
                    let _ = req.respond(resp);
                } else {
                    let err_bytes = b"{\"error\":{\"code\":\"key_required\",\"message\":\"Key required\",\"detail\":null,\"request_id\":\"\"}}";
                    let resp = Response::new(
                        StatusCode(401),
                        vec![],
                        Cursor::new(err_bytes),
                        Some(err_bytes.len()),
                        None,
                    );
                    let _ = req.respond(resp);
                }
            } else {
                let _ = req.respond(Response::empty(404));
            }
        }
    });

    let client = LocalApiClient::new(port);
    let mut handler = McpHandler::new(client);

    // 1. Initialize
    let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let init_resp = handler.handle_message(init_req).unwrap();
    assert!(init_resp.contains("snapdown-bridge"));
    assert!(init_resp.contains("2024-11-05"));

    // 2. Tools list
    let list_tools_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    let list_resp = handler.handle_message(list_tools_req).unwrap();
    assert!(list_resp.contains("mcp:set_access_key"));
    assert!(list_resp.contains("mcp:list_bundles"));
    assert!(list_resp.contains("mcp:read_bundle"));
    assert!(list_resp.contains("mcp:read_bundle_image"));

    // 3. Call list_bundles without key -> returns error in MCP content
    let list_bundles_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"mcp:list_bundles","arguments":{}}}"#;
    let err_bundle_resp = handler.handle_message(list_bundles_req).unwrap();
    assert!(err_bundle_resp.contains("key_required"));
    assert!(err_bundle_resp.contains("isError"));

    // 4. Set access key
    let set_key_req = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"mcp:set_access_key","arguments":{"key":"valid_key"}}}"#;
    let set_resp = handler.handle_message(set_key_req).unwrap();
    assert!(set_resp.contains("Access key configured successfully"));

    // 5. Call list_bundles with key -> returns list of bundles
    let list_bundles_req2 = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"mcp:list_bundles","arguments":{}}}"#;
    let ok_bundle_resp = handler.handle_message(list_bundles_req2).unwrap();
    assert!(ok_bundle_resp.contains("Test Bundle"));
    assert!(ok_bundle_resp.contains("b-1"));
}

#[test]
fn test_unreachable_server_fails_fast() {
    let client = LocalApiClient::new(59999);
    let mut handler = McpHandler::new(client);

    let list_bundles_req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"mcp:list_bundles","arguments":{}}}"#;
    let resp = handler.handle_message(list_bundles_req).unwrap();
    assert!(resp.contains("Snapdown is not running"));
    assert!(resp.contains("isError"));
}
