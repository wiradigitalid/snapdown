use serde::{Deserialize, Serialize};

use crate::error::CoreError;

pub const DEFAULT_MAX_LONG_EDGE_PX: u32 = 1600;
pub const DEFAULT_ENCODER_QUALITY: u8 = 75;

pub const MIN_LONG_EDGE_PX: u32 = 320;
pub const MAX_LONG_EDGE_PX: u32 = 7680;
pub const MIN_ENCODER_QUALITY: u8 = 10;
pub const MAX_ENCODER_QUALITY: u8 = 100;

pub const PRESET_SHARP_LONG_EDGE: u32 = 2560;
pub const PRESET_SHARP_QUALITY: u8 = 90;
pub const PRESET_BALANCED_LONG_EDGE: u32 = 1600;
pub const PRESET_BALANCED_QUALITY: u8 = 75;
pub const PRESET_SMALL_LONG_EDGE: u32 = 1280;
pub const PRESET_SMALL_QUALITY: u8 = 50;

pub const DEFAULT_HOTKEY_CAPTURE: &str = "CommandOrControl+Shift+S";
pub const DEFAULT_HOTKEY_OPEN_EDITOR: &str = "CommandOrControl+Shift+E";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAction {
    Capture,
    OpenEditor,
}

impl HotkeyAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::OpenEditor => "open_editor",
        }
    }

    pub fn to_setting_key(&self) -> SettingKey {
        match self {
            Self::Capture => SettingKey::HotkeyCapture,
            Self::OpenEditor => SettingKey::HotkeyOpenEditor,
        }
    }

    pub fn from_setting_key(key: &SettingKey) -> Option<Self> {
        match key {
            SettingKey::HotkeyCapture => Some(Self::Capture),
            SettingKey::HotkeyOpenEditor => Some(Self::OpenEditor),
            _ => None,
        }
    }

    pub fn from_action_str(s: &str) -> Option<Self> {
        match s {
            "capture" | "Capture" => Some(Self::Capture),
            "open_editor" | "OpenEditor" | "open-editor" => Some(Self::OpenEditor),
            _ => None,
        }
    }

    pub fn default_shortcut(&self) -> &'static str {
        match self {
            Self::Capture => DEFAULT_HOTKEY_CAPTURE,
            Self::OpenEditor => DEFAULT_HOTKEY_OPEN_EDITOR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedBudget {
    Auto,
    Sharp,
    Balanced,
    Small,
    Custom,
}

impl NamedBudget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Sharp => "sharp",
            Self::Balanced => "balanced",
            Self::Small => "small",
            Self::Custom => "custom",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Sharp => "Sharp",
            Self::Balanced => "Balanced",
            Self::Small => "Small",
            Self::Custom => "Custom",
        }
    }

    pub fn prose(&self) -> &'static str {
        match self {
            Self::Auto => "Sizes each capture to what it is. Most captures land near 120 KB.",
            Self::Sharp => "Keeps small text crisp. Files are larger.",
            Self::Balanced => "A middle setting that does not change with the capture.",
            Self::Small => "The smallest file that is still readable.",
            Self::Custom => "Custom limits set in Advanced.",
        }
    }

    pub fn fixed_pair(&self) -> Option<ResolvedPair> {
        match self {
            Self::Sharp => Some(ResolvedPair {
                max_long_edge: PRESET_SHARP_LONG_EDGE,
                encoder_quality: PRESET_SHARP_QUALITY,
            }),
            Self::Balanced => Some(ResolvedPair {
                max_long_edge: PRESET_BALANCED_LONG_EDGE,
                encoder_quality: PRESET_BALANCED_QUALITY,
            }),
            Self::Small => Some(ResolvedPair {
                max_long_edge: PRESET_SMALL_LONG_EDGE,
                encoder_quality: PRESET_SMALL_QUALITY,
            }),
            Self::Auto | Self::Custom => None,
        }
    }

    pub fn from_name_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "sharp" => Some(Self::Sharp),
            "balanced" => Some(Self::Balanced),
            "small" => Some(Self::Small),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedPair {
    pub max_long_edge: u32,
    pub encoder_quality: u8,
}

impl ResolvedPair {
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

pub fn derive_auto_budget(region_long_edge: u32) -> ResolvedPair {
    if region_long_edge <= 800 {
        ResolvedPair {
            max_long_edge: 1280,
            encoder_quality: 92,
        }
    } else if region_long_edge <= 1920 {
        ResolvedPair {
            max_long_edge: 1600,
            encoder_quality: 82,
        }
    } else {
        ResolvedPair {
            max_long_edge: 1600,
            encoder_quality: 70,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "QualityBudgetRaw", into = "QualityBudgetRaw")]
pub struct QualityBudget {
    pub named: NamedBudget,
    pub custom_pair: Option<ResolvedPair>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct QualityBudgetRaw {
    #[serde(default = "default_named_budget")]
    pub named: NamedBudget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_pair: Option<ResolvedPair>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_long_edge: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoder_quality: Option<u8>,
}

fn default_named_budget() -> NamedBudget {
    NamedBudget::Auto
}

impl From<QualityBudget> for QualityBudgetRaw {
    fn from(qb: QualityBudget) -> Self {
        Self {
            named: qb.named,
            custom_pair: qb.custom_pair,
            max_long_edge: None,
            encoder_quality: None,
        }
    }
}

impl From<QualityBudgetRaw> for QualityBudget {
    fn from(raw: QualityBudgetRaw) -> Self {
        if let (Some(edge), Some(qual)) = (raw.max_long_edge, raw.encoder_quality) {
            if raw.named == NamedBudget::Auto && raw.custom_pair.is_none() {
                let pair = ResolvedPair {
                    max_long_edge: edge,
                    encoder_quality: qual,
                };
                return Self {
                    named: NamedBudget::Custom,
                    custom_pair: Some(pair),
                };
            }
        }
        Self {
            named: raw.named,
            custom_pair: raw.custom_pair,
        }
    }
}

impl Default for QualityBudget {
    fn default() -> Self {
        Self {
            named: NamedBudget::Auto,
            custom_pair: None,
        }
    }
}

impl QualityBudget {
    pub fn new(named: NamedBudget, custom_pair: Option<ResolvedPair>) -> Self {
        Self { named, custom_pair }
    }

    pub fn prose(&self) -> &'static str {
        self.named.prose()
    }

    /// Resolves the effective (max_long_edge, encoder_quality) pair for a given captured region.
    /// Under Auto, derivation varies dynamically based on region dimensions (SCN-03, LC-003, OQ-3):
    /// - Small region (long edge <= 800 px, e.g. 312x118 tooltip): no downscale cap needed (1280 px),
    ///   encoder quality is high (92), preserving sharp text without lossy compression artifacts.
    /// - Medium region (800 < long edge <= 1920 px): max long edge 1600 px, encoder quality 82.
    /// - Large / 4K region (long edge > 1920 px, e.g. 3840x2160 dashboard): max long edge 1600 px,
    ///   encoder quality 70 (downscaling already removed high-frequency detail).
    pub fn resolve(&self, region_long_edge: u32) -> ResolvedPair {
        match self.named {
            NamedBudget::Sharp => self.named.fixed_pair().unwrap(),
            NamedBudget::Balanced => self.named.fixed_pair().unwrap(),
            NamedBudget::Small => self.named.fixed_pair().unwrap(),
            NamedBudget::Custom => self.custom_pair.unwrap_or(ResolvedPair {
                max_long_edge: DEFAULT_MAX_LONG_EDGE_PX,
                encoder_quality: DEFAULT_ENCODER_QUALITY,
            }),
            NamedBudget::Auto => derive_auto_budget(region_long_edge),
        }
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
    fn quality_budget_defaults_and_named_states() {
        let default_qb = QualityBudget::default();
        assert_eq!(default_qb.named, NamedBudget::Auto);
        assert_eq!(default_qb.custom_pair, None);

        let valid = ResolvedPair::new(1920, 80);
        assert!(valid.is_ok());

        let invalid_edge = ResolvedPair::new(100, 80);
        assert!(invalid_edge.is_err());

        let invalid_quality = ResolvedPair::new(1920, 5);
        assert!(invalid_quality.is_err());
    }

    #[test]
    fn fixed_presets_resolve_pinned_constants() {
        let sharp = QualityBudget::new(NamedBudget::Sharp, None);
        assert_eq!(
            sharp.resolve(3840),
            ResolvedPair {
                max_long_edge: 2560,
                encoder_quality: 90,
            }
        );

        let balanced = QualityBudget::new(NamedBudget::Balanced, None);
        assert_eq!(
            balanced.resolve(3840),
            ResolvedPair {
                max_long_edge: 1600,
                encoder_quality: 75,
            }
        );

        let small = QualityBudget::new(NamedBudget::Small, None);
        assert_eq!(
            small.resolve(3840),
            ResolvedPair {
                max_long_edge: 1280,
                encoder_quality: 50,
            }
        );
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

    #[test]
    fn hotkey_action_mapping_and_defaults() {
        assert_eq!(HotkeyAction::Capture.as_str(), "capture");
        assert_eq!(HotkeyAction::OpenEditor.as_str(), "open_editor");
        assert_eq!(
            HotkeyAction::Capture.to_setting_key(),
            SettingKey::HotkeyCapture
        );
        assert_eq!(
            HotkeyAction::OpenEditor.to_setting_key(),
            SettingKey::HotkeyOpenEditor
        );
        assert_eq!(
            HotkeyAction::from_setting_key(&SettingKey::HotkeyCapture),
            Some(HotkeyAction::Capture)
        );
        assert_eq!(
            HotkeyAction::from_setting_key(&SettingKey::HotkeyOpenEditor),
            Some(HotkeyAction::OpenEditor)
        );
        assert_eq!(HotkeyAction::from_setting_key(&SettingKey::VaultPath), None);
        assert_eq!(
            HotkeyAction::from_action_str("capture"),
            Some(HotkeyAction::Capture)
        );
        assert_eq!(
            HotkeyAction::from_action_str("open_editor"),
            Some(HotkeyAction::OpenEditor)
        );
        assert_eq!(
            HotkeyAction::Capture.default_shortcut(),
            DEFAULT_HOTKEY_CAPTURE
        );
        assert_eq!(
            HotkeyAction::OpenEditor.default_shortcut(),
            DEFAULT_HOTKEY_OPEN_EDITOR
        );
    }
}