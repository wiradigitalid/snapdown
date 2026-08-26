use serde::{Deserialize, Serialize};
use snapdown_core::domain::setting::{Setting, SettingKey, SettingValue};
use snapdown_core::ports::{Clock, SettingsStore, StartupRegistrar};
use snapdown_store::system::SystemClock;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupState {
    On,
    Off,
    Unreadable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupStatusDto {
    pub enabled: bool,
    pub state: StartupState,
}

pub fn get_startup_status_impl(state: &AppState) -> Result<StartupStatusDto, String> {
    let registrar = state
        .startup_registrar
        .lock()
        .map_err(|e| format!("Failed to lock startup registrar: {e}"))?;

    match registrar.is_enabled() {
        Ok(true) => Ok(StartupStatusDto {
            enabled: true,
            state: StartupState::On,
        }),
        Ok(false) => Ok(StartupStatusDto {
            enabled: false,
            state: StartupState::Off,
        }),
        Err(_) => Ok(StartupStatusDto {
            enabled: false,
            state: StartupState::Unreadable,
        }),
    }
}

pub fn set_startup_status_impl(
    enabled: bool,
    state: &AppState,
) -> Result<StartupStatusDto, String> {
    let registrar = state
        .startup_registrar
        .lock()
        .map_err(|e| format!("Failed to lock startup registrar: {e}"))?;

    if enabled {
        registrar.enable().map_err(|e| e.to_string())?;
    } else {
        registrar.disable().map_err(|e| e.to_string())?;
    }

    let is_now_enabled = registrar.is_enabled().map_err(|e| e.to_string())?;

    // Record preference expressed in store (SCN-02, BR-112)
    let clock = SystemClock::new();
    let record = Setting::new(
        SettingKey::StartupRegistered,
        SettingValue::String("expressed".to_string()),
        clock.now_rfc3339(),
    );
    let _ = state.settings_store.set(&record);

    let state_enum = if is_now_enabled {
        StartupState::On
    } else {
        StartupState::Off
    };

    Ok(StartupStatusDto {
        enabled: is_now_enabled,
        state: state_enum,
    })
}

#[tauri::command]
pub fn get_startup_status(state: State<AppState>) -> Result<StartupStatusDto, String> {
    get_startup_status_impl(&state)
}

#[tauri::command]
pub fn set_startup_status(
    enabled: bool,
    state: State<AppState>,
) -> Result<StartupStatusDto, String> {
    set_startup_status_impl(enabled, &state)
}
