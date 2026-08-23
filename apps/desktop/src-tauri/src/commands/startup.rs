use serde::{Deserialize, Serialize};
use snapdown_core::ports::StartupRegistrar;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupStatusDto {
    pub enabled: bool,
}

#[tauri::command]
pub fn get_startup_status(state: State<AppState>) -> Result<StartupStatusDto, String> {
    let registrar = state
        .startup_registrar
        .lock()
        .map_err(|e| format!("Failed to lock startup registrar: {e}"))?;
    let enabled = registrar.is_enabled().map_err(|e| e.to_string())?;
    Ok(StartupStatusDto { enabled })
}

#[tauri::command]
pub fn set_startup_status(
    enabled: bool,
    state: State<AppState>,
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
    let current = registrar.is_enabled().map_err(|e| e.to_string())?;
    Ok(StartupStatusDto { enabled: current })
}
