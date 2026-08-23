pub mod access_key_store;
pub mod bundle_store;
pub mod publication_store;

pub use access_key_store::AccessKeyStore;
pub use bundle_store::BundleStore;
pub use publication_store::PublicationStore;

use crate::domain::finding::{Finding, FindingDetail, Marker, Note};
use crate::domain::setting::{Setting, SettingKey};
use crate::error::CoreError;

pub trait Clock {
    fn now_rfc3339(&self) -> String;
    fn now_unix_millis(&self) -> u64;
}

pub trait EntropySource {
    fn random_bytes_10(&self) -> [u8; 10];
}

pub trait SettingsStore {
    fn get(&self, key: &SettingKey) -> Result<Option<Setting>, CoreError>;
    fn set(&self, setting: &Setting) -> Result<(), CoreError>;
    fn delete(&self, key: &SettingKey) -> Result<(), CoreError>;
    fn list_all(&self) -> Result<Vec<Setting>, CoreError>;
}

pub trait BlobStore {
    fn read_blob(&self, relative_path: &str) -> Result<Vec<u8>, CoreError>;
    fn write_blob(&self, relative_path: &str, bytes: &[u8]) -> Result<(), CoreError>;
    fn delete_blob(&self, relative_path: &str) -> Result<(), CoreError>;
    fn blob_exists(&self, relative_path: &str) -> Result<bool, CoreError>;
}

pub trait FindingStore {
    fn create_finding(
        &self,
        finding: &Finding,
        note: &Note,
        markers: &[Marker],
    ) -> Result<(), CoreError>;

    fn get_finding(&self, id: &str) -> Result<Option<FindingDetail>, CoreError>;

    fn list_findings(&self) -> Result<Vec<FindingDetail>, CoreError>;

    fn delete_finding(&self, id: &str) -> Result<(), CoreError>;

    fn update_note(&self, finding_id: &str, body: &str, updated_at: &str) -> Result<(), CoreError>;

    fn add_marker(
        &self,
        finding_id: &str,
        marker_id: &str,
        x: f64,
        y: f64,
        comment: &str,
    ) -> Result<Marker, CoreError>;

    fn update_marker(
        &self,
        finding_id: &str,
        marker_id: &str,
        x: f64,
        y: f64,
        comment: &str,
    ) -> Result<Marker, CoreError>;

    fn delete_marker(&self, finding_id: &str, marker_id: &str) -> Result<(), CoreError>;

    fn reorder_markers(
        &self,
        finding_id: &str,
        ordered_marker_ids: &[&str],
    ) -> Result<(), CoreError>;
}

pub trait HotkeyRegistrar {
    fn register(&mut self, action: &str, shortcut: &str) -> Result<(), CoreError>;
    fn unregister(&mut self, action: &str) -> Result<(), CoreError>;
    fn is_registered(&self, action: &str) -> bool;
    fn get_shortcut(&self, action: &str) -> Option<String>;
}

pub trait StartupRegistrar {
    fn is_enabled(&self) -> Result<bool, CoreError>;
    fn enable(&self) -> Result<(), CoreError>;
    fn disable(&self) -> Result<(), CoreError>;
}
