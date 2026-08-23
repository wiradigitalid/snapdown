use crate::hotkey::DesktopHotkeyRegistrar;
use crate::startup::DesktopStartupRegistrar;
use snapdown_store::sqlite::{SqliteFindingStore, SqliteSettingsStore};
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub settings_store: Arc<SqliteSettingsStore>,
    pub finding_store: Arc<SqliteFindingStore>,
    pub hotkey_registrar: Arc<Mutex<DesktopHotkeyRegistrar>>,
    pub startup_registrar: Arc<Mutex<DesktopStartupRegistrar>>,
}
