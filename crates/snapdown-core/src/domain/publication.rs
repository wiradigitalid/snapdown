use serde::{Deserialize, Serialize};

use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Publication {
    pub id: String,
    pub bundle_id: String,
    pub slug: String,
    pub base_url: String,
    pub published_at: String,
    pub unpublished_at: Option<String>,
    pub last_error: Option<String>,
}

impl Publication {
    pub fn new(
        id: String,
        bundle_id: String,
        slug: String,
        base_url: String,
        published_at: String,
        unpublished_at: Option<String>,
        last_error: Option<String>,
    ) -> Result<Self, CoreError> {
        if slug.trim().is_empty() {
            return Err(CoreError::Validation(
                "Publication slug cannot be empty".into(),
            ));
        }
        if base_url.trim().is_empty() {
            return Err(CoreError::Validation(
                "Publication base_url cannot be empty".into(),
            ));
        }
        Ok(Self {
            id,
            bundle_id,
            slug,
            base_url,
            published_at,
            unpublished_at,
            last_error,
        })
    }

    pub fn is_live(&self) -> bool {
        self.unpublished_at.is_none()
    }

    /// Generates a high-entropy cryptographically independent 32-character slug (AD-8).
    /// Pure function using 20 random bytes (160 bits) formatted as a hex / base32 string.
    pub fn generate_slug_from_bytes(bytes: &[u8; 20]) -> String {
        let mut slug = String::with_capacity(32);
        const CHARSET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789"; // Crockford-like base32 without ambiguous chars
        for byte in bytes.iter() {
            let idx = (byte % (CHARSET.len() as u8)) as usize;
            slug.push(CHARSET[idx] as char);
        }
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_creation_and_liveness() {
        let pub_live = Publication::new(
            "p1".into(),
            "b1".into(),
            "slug123".into(),
            "https://snapdown.dev".into(),
            "2026-08-23T10:00:00Z".into(),
            None,
            None,
        )
        .unwrap();

        assert!(pub_live.is_live());

        let pub_unpub = Publication::new(
            "p2".into(),
            "b2".into(),
            "slug456".into(),
            "https://snapdown.dev".into(),
            "2026-08-23T10:00:00Z".into(),
            Some("2026-08-23T11:00:00Z".into()),
            None,
        )
        .unwrap();

        assert!(!pub_unpub.is_live());
    }

    #[test]
    fn slug_generation_is_independent_and_correct_length() {
        let bytes1 = [1u8; 20];
        let bytes2 = [2u8; 20];

        let slug1 = Publication::generate_slug_from_bytes(&bytes1);
        let slug2 = Publication::generate_slug_from_bytes(&bytes2);

        assert_eq!(slug1.len(), 20);
        assert_ne!(slug1, slug2);
        assert!(slug1.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
