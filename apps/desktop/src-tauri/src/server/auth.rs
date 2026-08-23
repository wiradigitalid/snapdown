use snapdown_core::domain::access_key::AuthResult;
use snapdown_core::ports::AccessKeyStore;
use tiny_http::Request;

pub enum ExtractedAuth {
    Valid,
    Missing,
    Invalid,
}

pub fn authenticate_request<K: AccessKeyStore>(req: &Request, key_store: &K) -> ExtractedAuth {
    let mut token: Option<String> = None;

    for header in req.headers() {
        let name = header.field.as_str().as_str();
        let value = header.value.as_str();

        if name.eq_ignore_ascii_case("authorization") {
            if let Some(bearer) = value.strip_prefix("Bearer ") {
                token = Some(bearer.trim().to_string());
                break;
            } else if let Some(bearer) = value.strip_prefix("bearer ") {
                token = Some(bearer.trim().to_string());
                break;
            }
        } else if name.eq_ignore_ascii_case("x-snapdown-key") {
            token = Some(value.trim().to_string());
            break;
        }
    }

    let secret = match token {
        Some(s) if !s.is_empty() => s,
        _ => return ExtractedAuth::Missing,
    };

    match key_store.verify_key(&secret) {
        Ok(AuthResult::Valid) => ExtractedAuth::Valid,
        _ => ExtractedAuth::Invalid,
    }
}
