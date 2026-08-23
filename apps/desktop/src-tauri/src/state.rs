use crate::hotkey::DesktopHotkeyRegistrar;
use snapdown_store::sqlite::SqliteSettingsStore;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub settings_store: Arc<SqliteSettingsStore>,
    pub hotkey_registrar: Arc<Mutex<DesktopHotkeyRegistrar>>,
}
