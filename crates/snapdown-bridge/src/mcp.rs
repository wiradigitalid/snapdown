use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::client::LocalApiClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

pub struct McpHandler {
    client: LocalApiClient,
}

impl McpHandler {
    pub fn new(client: LocalApiClient) -> Self {
        Self { client }
    }

    pub fn handle_message(&mut self, line: &str) -> Option<String> {
        let req: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_) => {
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".into(),
                        data: None,
                    }),
                };
                return Some(serde_json::to_string(&err_resp).unwrap());
            }
        };

        let id = req.id.clone();
        let res = self.dispatch_method(&req.method, req.params);

        let response = match res {
            Ok(val) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(val),
                error: None,
            },
            Err((code, msg)) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code,
                    message: msg,
                    data: None,
                }),
            },
        };

        Some(serde_json::to_string(&response).unwrap())
    }

    fn dispatch_method(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, (i32, String)> {
        match method {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "snapdown-bridge",
                    "version": "0.1.0"
                }
            })),
            "notifications/initialized" => Ok(json!({})),
            "tools/list" => Ok(json!({
                "tools": [
                    {
                        "name": "mcp:set_access_key",
                        "description": "Configure the Snapdown Access Key in-memory for this bridge session",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "key": {
                                    "type": "string",
                                    "description": "The Access Key string generated from Snapdown"
                                }
                            },
                            "required": ["key"]
                        }
                    },
                    {
                        "name": "mcp:list_bundles",
                        "description": "List all composed review bundles in the Snapdown library",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "mcp:read_bundle",
                        "description": "Read exact CommonMark markdown content and image attachments for a given bundle",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "bundle_id": {
                                    "type": "string",
                                    "description": "Unique ID of the bundle"
                                }
                            },
                            "required": ["bundle_id"]
                        }
                    },
                    {
                        "name": "mcp:read_bundle_image",
                        "description": "Read raw image bytes for a specific bundle screenshot attachment",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "bundle_id": {
                                    "type": "string",
                                    "description": "Unique ID of the bundle"
                                },
                                "filename": {
                                    "type": "string",
                                    "description": "Filename of the image"
                                }
                            },
                            "required": ["bundle_id", "filename"]
                        }
                    }
                ]
            })),
            "tools/call" => {
                let params = params.ok_or((-32602, "Invalid params".into()))?;
                let tool_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing tool name".into()))?;
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));

                self.call_tool(tool_name, arguments)
            }
            _ => Err((-32601, format!("Method not found: {method}"))),
        }
    }

    fn call_tool(
        &mut self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, (i32, String)> {
        match name {
            "mcp:set_access_key" => {
                let key = args
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'key' argument".into()))?;

                self.client.set_access_key(key.to_string());
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Access key configured successfully."
                        }
                    ]
                }))
            }
            "mcp:list_bundles" => match self.client.list_bundles() {
                Ok(bundles) => Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&bundles).unwrap_or_default()
                        }
                    ]
                })),
                Err(err) => Ok(json!({
                    "isError": true,
                    "content": [
                        {
                            "type": "text",
                            "text": err
                        }
                    ]
                })),
            },
            "mcp:read_bundle" => {
                let bundle_id = args
                    .get("bundle_id")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'bundle_id' argument".into()))?;

                match self.client.read_bundle(bundle_id) {
                    Ok(bundle_detail) => Ok(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&bundle_detail).unwrap_or_default()
                            }
                        ]
                    })),
                    Err(err) => Ok(json!({
                        "isError": true,
                        "content": [
                            {
                                "type": "text",
                                "text": err
                            }
                        ]
                    })),
                }
            }
            "mcp:read_bundle_image" => {
                let bundle_id = args
                    .get("bundle_id")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'bundle_id' argument".into()))?;
                let filename = args
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'filename' argument".into()))?;

                match self.client.read_bundle_image(bundle_id, filename) {
                    Ok((b64_data, mime_type)) => Ok(json!({
                        "content": [
                            {
                                "type": "image",
                                "data": b64_data,
                                "mimeType": mime_type
                            }
                        ]
                    })),
                    Err(err) => Ok(json!({
                        "isError": true,
                        "content": [
                            {
                                "type": "text",
                                "text": err
                            }
                        ]
                    })),
                }
            }
            _ => Err((-32601, format!("Tool not found: {name}"))),
        }
    }
}
