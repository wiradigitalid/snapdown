use crate::domain::setting::{Setting, SettingKey};
use crate::error::CoreError;

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

pub trait HotkeyRegistrar {
    fn register(&mut self, action: &str, shortcut: &str) -> Result<(), CoreError>;
    fn unregister(&mut self, action: &str) -> Result<(), CoreError>;
    fn is_registered(&self, action: &str) -> bool;
}

pub trait StartupRegistrar {
    fn is_enabled(&self) -> Result<bool, CoreError>;
    fn enable(&self) -> Result<(), CoreError>;
    fn disable(&self) -> Result<(), CoreError>;
}
