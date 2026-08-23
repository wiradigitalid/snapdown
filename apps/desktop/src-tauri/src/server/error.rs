use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub detail: Option<serde_json::Value>,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub error: ErrorDetail,
}

impl ErrorEnvelope {
    pub fn new(code: &str, message: &str, detail: Option<serde_json::Value>) -> Self {
        let request_id = snapdown_core::id_from_parts(
            chrono::Utc::now().timestamp_millis() as u64,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        );
        Self {
            error: ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                detail,
                request_id,
            },
        }
    }

    pub fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_else(|_| b"{\"error\":{\"code\":\"unavailable\",\"message\":\"Serialization failed\",\"detail\":null,\"request_id\":\"\"}}".to_vec())
    }
}
