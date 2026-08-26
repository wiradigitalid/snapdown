use serde::{Deserialize, Serialize};
use snapdown_core::domain::setting::HotkeyAction;
use snapdown_core::ports::HotkeyRegistrar;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyItemDto {
    pub action: String,
    pub shortcut: String,
    pub is_registered: bool,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettingsDto {
    pub hotkeys: Vec<HotkeyItemDto>,
    pub startup_warnings: Vec<String>,
}

#[tauri::command]
pub fn get_hotkeys(state: State<AppState>) -> Result<HotkeySettingsDto, String> {
    let registrar = state
        .hotkey_registrar
        .lock()
        .map_err(|e| format!("Failed to lock hotkey registrar: {e}"))?;

    let actions = [HotkeyAction::Capture, HotkeyAction::OpenEditor];
    let mut hotkeys = Vec::new();

    for action in actions {
        let action_str = action.as_str();
        let shortcut = registrar
            .get_shortcut(action_str)
            .unwrap_or_else(|| action.default_shortcut().to_string());
        let is_registered = registrar.is_registered(action_str);
        let is_active = !shortcut.trim().is_empty() && is_registered;
        let startup_error = registrar.get_startup_failures().get(&action).cloned();

        hotkeys.push(HotkeyItemDto {
            action: action_str.to_string(),
            shortcut,
            is_registered,
            is_active,
            startup_error,
        });
    }

    let startup_warnings = registrar
        .get_startup_failures()
        .iter()
        .map(|(action, err)| {
            format!(
                "Failed to register shortcut for action '{}' at startup: {}",
                action.as_str(),
                err
            )
        })
        .collect();

    Ok(HotkeySettingsDto {
        hotkeys,
        startup_warnings,
    })
}

#[tauri::command]
pub fn set_hotkey(action: String, shortcut: String, state: State<AppState>) -> Result<(), String> {
    let mut registrar = state
        .hotkey_registrar
        .lock()
        .map_err(|e| format!("Failed to lock hotkey registrar: {e}"))?;

    let action_enum = HotkeyAction::from_action_str(&action)
        .ok_or_else(|| format!("Unknown hotkey action: {action}"))?;

    registrar
        .validate_and_rebind(action_enum, &shortcut)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_hotkey(action: String, state: State<AppState>) -> Result<(), String> {
    let mut registrar = state
        .hotkey_registrar
        .lock()
        .map_err(|e| format!("Failed to lock hotkey registrar: {e}"))?;

    let action_enum = HotkeyAction::from_action_str(&action)
        .ok_or_else(|| format!("Unknown hotkey action: {action}"))?;

    registrar.clear(action_enum).map_err(|e| e.to_string())
}
