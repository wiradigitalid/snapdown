use base64::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PublishClient {
    base_url: String,
    publish_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishFilePayload {
    pub filename: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequestPayload {
    pub markdown: String,
    pub files: Vec<PublishFilePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub status: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileResponse {
    pub slug: String,
    pub status: String,
}

impl PublishClient {
    pub fn new(base_url: String, publish_key: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            publish_key,
        }
    }

    pub fn publish(
        &self,
        slug: &str,
        markdown: &str,
        files: &[(&str, &[u8])],
    ) -> Result<PublishResponse, String> {
        let file_payloads: Vec<PublishFilePayload> = files
            .iter()
            .map(|(fname, bytes)| PublishFilePayload {
                filename: fname.to_string(),
                data_base64: BASE64_STANDARD.encode(bytes),
            })
            .collect();

        let payload = PublishRequestPayload {
            markdown: markdown.to_string(),
            files: file_payloads,
        };

        let mut req = ureq::put(&format!("{}/publish/{slug}", self.base_url))
            .timeout(std::time::Duration::from_secs(10));

        if let Some(ref key) = self.publish_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }

        let json_bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;

        match req.send_bytes(&json_bytes) {
            Ok(resp) => {
                let mut body = String::new();
                resp.into_reader()
                    .read_to_string(&mut body)
                    .map_err(|e| e.to_string())?;
                let pub_resp: PublishResponse = serde_json::from_str(&body)
                    .map_err(|e| format!("Invalid JSON response: {e}"))?;
                Ok(pub_resp)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let mut body = String::new();
                let _ = resp.into_reader().read_to_string(&mut body);
                Err(format!("Publish failed with status {code}: {body}"))
            }
            Err(ureq::Error::Transport(e)) => Err(format!("Publish connection error: {e}")),
        }
    }

    pub fn unpublish(&self, slug: &str) -> Result<(), String> {
        let mut req = ureq::delete(&format!("{}/publish/{slug}", self.base_url))
            .timeout(std::time::Duration::from_secs(10));

        if let Some(ref key) = self.publish_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }

        match req.call() {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(code, resp)) => {
                let mut body = String::new();
                let _ = resp.into_reader().read_to_string(&mut body);
                Err(format!("Unpublish failed with status {code}: {body}"))
            }
            Err(ureq::Error::Transport(e)) => Err(format!("Unpublish connection error: {e}")),
        }
    }

    pub fn reconcile(&self, slug: &str) -> Result<bool, String> {
        let mut req = ureq::get(&format!("{}/publish/{slug}", self.base_url))
            .timeout(std::time::Duration::from_secs(5));

        if let Some(ref key) = self.publish_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }

        match req.call() {
            Ok(resp) => {
                if resp.status() == 200 {
                    let mut body = String::new();
                    resp.into_reader()
                        .read_to_string(&mut body)
                        .map_err(|e| e.to_string())?;
                    let rec: ReconcileResponse = serde_json::from_str(&body)
                        .map_err(|e| format!("Invalid JSON response: {e}"))?;
                    Ok(rec.status == "served")
                } else {
                    Ok(false)
                }
            }
            Err(ureq::Error::Status(404, _)) => Ok(false),
            Err(ureq::Error::Status(code, resp)) => {
                let mut body = String::new();
                let _ = resp.into_reader().read_to_string(&mut body);
                Err(format!("Reconcile error {code}: {body}"))
            }
            Err(ureq::Error::Transport(e)) => Err(format!("Reconcile connection error: {e}")),
        }
    }
}
