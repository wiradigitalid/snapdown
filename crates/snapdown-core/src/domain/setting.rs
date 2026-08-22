use serde::{Deserialize, Serialize};

use crate::error::CoreError;

pub const DEFAULT_MAX_LONG_EDGE_PX: u32 = 1600;
pub const DEFAULT_ENCODER_QUALITY: u8 = 75;

pub const MIN_LONG_EDGE_PX: u32 = 320;
pub const MAX_LONG_EDGE_PX: u32 = 7680;
pub const MIN_ENCODER_QUALITY: u8 = 10;
pub const MAX_ENCODER_QUALITY: u8 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityBudget {
    pub max_long_edge: u32,
    pub encoder_quality: u8,
}

impl Default for QualityBudget {
    fn default() -> Self {
        Self {
            max_long_edge: DEFAULT_MAX_LONG_EDGE_PX,
            encoder_quality: DEFAULT_ENCODER_QUALITY,
        }
    }
}

impl QualityBudget {
    pub fn new(max_long_edge: u32, encoder_quality: u8) -> Result<Self, CoreError> {
        if !(MIN_LONG_EDGE_PX..=MAX_LONG_EDGE_PX).contains(&max_long_edge) {
            return Err(CoreError::Validation(format!(
                "max_long_edge must be between {MIN_LONG_EDGE_PX} and {MAX_LONG_EDGE_PX}, got {max_long_edge}"
            )));
        }
        if !(MIN_ENCODER_QUALITY..=MAX_ENCODER_QUALITY).contains(&encoder_quality) {
            return Err(CoreError::Validation(format!(
                "encoder_quality must be between {MIN_ENCODER_QUALITY} and {MAX_ENCODER_QUALITY}, got {encoder_quality}"
            )));
        }
        Ok(Self {
            max_long_edge,
            encoder_quality,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingKey {
    VaultPath,
    HotkeyCapture,
    HotkeyOpenEditor,
    QualityBudget,
    RunAtStartup,
    OpenEditorAfterCapture,
    WebServiceAddress,
    Custom(String),
}

impl SettingKey {
    pub fn as_str(&self) -> &str {
        match self {
            Self::VaultPath => "vault_path",
            Self::HotkeyCapture => "hotkey_capture",
            Self::HotkeyOpenEditor => "hotkey_open_editor",
            Self::QualityBudget => "quality_budget",
            Self::RunAtStartup => "run_at_startup",
            Self::OpenEditorAfterCapture => "open_editor_after_capture",
            Self::WebServiceAddress => "web_service_address",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn from_key_str(s: &str) -> Self {
        match s {
            "vault_path" => Self::VaultPath,
            "hotkey_capture" => Self::HotkeyCapture,
            "hotkey_open_editor" => Self::HotkeyOpenEditor,
            "quality_budget" => Self::QualityBudget,
            "run_at_startup" => Self::RunAtStartup,
            "open_editor_after_capture" => Self::OpenEditorAfterCapture,
            "web_service_address" => Self::WebServiceAddress,
            other => Self::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    String(String),
    Boolean(bool),
    Integer(i64),
    QualityBudget(QualityBudget),
    Json(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    pub key: SettingKey,
    pub value: SettingValue,
    pub updated_at: String,
}

impl Setting {
    pub fn new(key: SettingKey, value: SettingValue, updated_at: String) -> Self {
        Self {
            key,
            value,
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_budget_defaults_and_validation() {
        let default_qb = QualityBudget::default();
        assert_eq!(default_qb.max_long_edge, 1600);
        assert_eq!(default_qb.encoder_quality, 75);

        let valid = QualityBudget::new(1920, 80);
        assert!(valid.is_ok());

        let invalid_edge = QualityBudget::new(100, 80);
        assert!(invalid_edge.is_err());

        let invalid_quality = QualityBudget::new(1920, 5);
        assert!(invalid_quality.is_err());
    }

    #[test]
    fn setting_key_roundtrip() {
        let keys = vec![
            (SettingKey::VaultPath, "vault_path"),
            (SettingKey::HotkeyCapture, "hotkey_capture"),
            (SettingKey::HotkeyOpenEditor, "hotkey_open_editor"),
            (SettingKey::QualityBudget, "quality_budget"),
            (SettingKey::RunAtStartup, "run_at_startup"),
            (
                SettingKey::OpenEditorAfterCapture,
                "open_editor_after_capture",
            ),
            (SettingKey::WebServiceAddress, "web_service_address"),
            (SettingKey::Custom("custom_key".to_string()), "custom_key"),
        ];

        for (key, key_str) in keys {
            assert_eq!(key.as_str(), key_str);
            assert_eq!(SettingKey::from_key_str(key_str), key);
        }
    }
}
