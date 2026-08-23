use snapdown_store::sqlite::SqliteSettingsStore;
use std::sync::Arc;

pub struct AppState {
    pub settings_store: Arc<SqliteSettingsStore>,
}
