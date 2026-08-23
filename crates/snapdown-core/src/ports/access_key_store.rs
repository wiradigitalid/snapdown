use crate::domain::access_key::{AccessKey, AuthResult};
use crate::error::CoreError;

pub trait AccessKeyStore: Send + Sync {
    fn get_active_key(&self) -> Result<Option<AccessKey>, CoreError>;
    fn save_key(&self, key: &AccessKey) -> Result<(), CoreError>;
    fn revoke_active_key(&self, revoked_at: &str) -> Result<(), CoreError>;
    fn verify_key(&self, secret: &str) -> Result<AuthResult, CoreError>;
}
