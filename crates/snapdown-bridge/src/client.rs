use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Clone)]
pub struct LocalApiClient {
    base_url: String,
    access_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
    pub detail: Option<serde_json::Value>,
    pub request_id: String,
}

impl LocalApiClient {
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            access_key: None,
        }
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            base_url,
            access_key: None,
        }
    }

    pub fn set_access_key(&mut self, key: String) {
        self.access_key = Some(key);
    }

    pub fn check_health(&self) -> Result<bool, String> {
        match ureq::get(&format!("{}/v1/health", self.base_url))
            .timeout(std::time::Duration::from_millis(500))
            .call()
        {
            Ok(resp) => Ok(resp.status() == 200),
            Err(ureq::Error::Transport(_)) => {
                Err("Snapdown is not running (connection refused)".into())
            }
            Err(ureq::Error::Status(code, _)) => {
                Err(format!("Health check failed with status {code}"))
            }
        }
    }

    pub fn list_bundles(&self) -> Result<serde_json::Value, String> {
        let mut req = ureq::get(&format!("{}/v1/bundles", self.base_url))
            .timeout(std::time::Duration::from_secs(3));

        if let Some(ref key) = self.access_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }

        match req.call() {
            Ok(resp) => {
                let mut body = String::new();
                resp.into_reader()
                    .read_to_string(&mut body)
                    .map_err(|e| e.to_string())?;
                let val: serde_json::Value =
                    serde_json::from_str(&body).map_err(|e| e.to_string())?;
                Ok(val)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let err_detail = parse_error_response(code, resp);
                Err(err_detail)
            }
            Err(ureq::Error::Transport(_)) => {
                Err("Snapdown is not running (connection refused)".into())
            }
        }
    }

    pub fn read_bundle(&self, bundle_id: &str) -> Result<serde_json::Value, String> {
        let mut req = ureq::get(&format!("{}/v1/bundles/{bundle_id}", self.base_url))
            .timeout(std::time::Duration::from_secs(3));

        if let Some(ref key) = self.access_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }

        match req.call() {
            Ok(resp) => {
                let mut body = String::new();
                resp.into_reader()
                    .read_to_string(&mut body)
                    .map_err(|e| e.to_string())?;
                let val: serde_json::Value =
                    serde_json::from_str(&body).map_err(|e| e.to_string())?;
                Ok(val)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let err_detail = parse_error_response(code, resp);
                Err(err_detail)
            }
            Err(ureq::Error::Transport(_)) => {
                Err("Snapdown is not running (connection refused)".into())
            }
        }
    }

    pub fn read_bundle_image(
        &self,
        bundle_id: &str,
        filename: &str,
    ) -> Result<(String, String), String> {
        let mut req = ureq::get(&format!(
            "{}/v1/bundles/{bundle_id}/images/{filename}",
            self.base_url
        ))
        .timeout(std::time::Duration::from_secs(5));

        if let Some(ref key) = self.access_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }

        match req.call() {
            Ok(resp) => {
                let mime = resp
                    .header("Content-Type")
                    .unwrap_or("image/png")
                    .to_string();

                let mut bytes = Vec::new();
                resp.into_reader()
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;

                let b64_data = BASE64_STANDARD.encode(&bytes);
                Ok((b64_data, mime))
            }
            Err(ureq::Error::Status(code, resp)) => {
                let err_detail = parse_error_response(code, resp);
                Err(err_detail)
            }
            Err(ureq::Error::Transport(_)) => {
                Err("Snapdown is not running (connection refused)".into())
            }
        }
    }
}

fn parse_error_response(_code: u16, resp: ureq::Response) -> String {
    parse_error_response_reader(_code, resp.into_reader())
}

pub fn parse_error_response_reader<R: Read>(code: u16, mut reader: R) -> String {
    let mut body = String::new();
    if let Err(e) = reader.read_to_string(&mut body) {
        return format!("internal: HTTP {code} (failed to read error response: {e})");
    }

    if let Ok(env) = serde_json::from_str::<ApiErrorEnvelope>(&body) {
        format!("{}: {}", env.error.code, env.error.message)
    } else if body.trim().is_empty() {
        format!("internal: HTTP {code} (empty error response)")
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    struct FailingReader;
    impl io::Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "connection reset",
            ))
        }
    }

    #[test]
    fn a_failed_error_body_read_never_yields_an_empty_message() {
        let err_msg = parse_error_response_reader(502, FailingReader);
        assert!(!err_msg.is_empty());
        assert!(!err_msg.trim().is_empty());

        let invalid_utf8_bytes: &[u8] = &[0xFF, 0xFE, 0xFD];
        let invalid_utf8_err = parse_error_response_reader(500, invalid_utf8_bytes);
        assert!(!invalid_utf8_err.is_empty());

        let empty_reader: &[u8] = b"";
        let empty_body_err = parse_error_response_reader(404, empty_reader);
        assert!(!empty_body_err.is_empty());
    }

    #[test]
    fn the_status_code_survives_a_failed_error_body_read() {
        let err_msg = parse_error_response_reader(502, FailingReader);
        assert!(err_msg.contains("502"));
        assert!(err_msg.contains("internal:"));

        let invalid_utf8_bytes: &[u8] = &[0xFF, 0xFE];
        let invalid_utf8_err = parse_error_response_reader(500, invalid_utf8_bytes);
        assert!(invalid_utf8_err.contains("500"));

        let empty_reader: &[u8] = b"";
        let empty_body_err = parse_error_response_reader(404, empty_reader);
        assert!(empty_body_err.contains("404"));
    }

    #[test]
    fn a_readable_error_envelope_is_still_parsed_as_before() {
        let json_body = br#"{"error":{"code":"key_required","message":"Key required","detail":null,"request_id":"req-1"}}"#;
        let err_msg = parse_error_response_reader(401, &json_body[..]);
        assert_eq!(err_msg, "key_required: Key required");

        let plain_body = b"Service Unavailable";
        let plain_err = parse_error_response_reader(503, &plain_body[..]);
        assert_eq!(plain_err, "Service Unavailable");
    }
}
