use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags};
use snapdown_core::domain::setting::{QualityBudget, Setting, SettingKey, SettingValue};
use snapdown_core::error::CoreError;
use snapdown_core::ports::SettingsStore;

use crate::error::StoreError;
use crate::sqlite::migrations::run_migrations;

#[derive(Clone)]
pub struct SqliteSettingsStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteSettingsStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let path_ref = path.as_ref();

        // Check if database file exists. If it exists, make sure it is not corrupt before proceeding.
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
        let mut conn = Connection::open_with_flags(path_ref, flags)?;

        // Apply pragmas
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

        // Quick integrity check to detect corruption
        {
            let mut integrity_stmt = conn.prepare("PRAGMA quick_check;")?;
            let integrity_res: String = integrity_stmt.query_row([], |row| row.get(0))?;
            if integrity_res != "ok" {
                return Err(StoreError::Corruption(integrity_res));
            }
        }

        // Run migrations
        run_migrations(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        run_migrations(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn is_empty(&self) -> Result<bool, StoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StoreError::Corruption(e.to_string()))?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM setting;", [], |row| row.get(0))?;
        Ok(count == 0)
    }

    pub fn get_schema_version(&self) -> Result<i64, StoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StoreError::Corruption(e.to_string()))?;
        crate::sqlite::migrations::get_schema_version(&conn)
    }
}

impl SettingsStore for SqliteSettingsStore {
    fn get(&self, key: &SettingKey) -> Result<Option<Setting>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT key, value, updated_at FROM setting WHERE key = ?1;")
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let key_str = key.as_str();
        let mut rows = stmt
            .query(rusqlite::params![key_str])
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| CoreError::Validation(e.to_string()))?
        {
            let raw_key: String = row
                .get(0)
                .map_err(|e| CoreError::Validation(e.to_string()))?;
            let raw_val: String = row
                .get(1)
                .map_err(|e| CoreError::Validation(e.to_string()))?;
            let updated_at: String = row
                .get(2)
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let parsed_key = SettingKey::from_key_str(&raw_key);
            let val = parse_setting_value(&parsed_key, &raw_val)?;

            Ok(Some(Setting {
                key: parsed_key,
                value: val,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    fn set(&self, setting: &Setting) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let key_str = setting.key.as_str();
        let value_str = serialize_setting_value(&setting.value)?;

        conn.execute(
            r#"
            INSERT INTO setting (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at;
            "#,
            rusqlite::params![key_str, value_str, setting.updated_at],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, key: &SettingKey) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        conn.execute(
            "DELETE FROM setting WHERE key = ?1;",
            rusqlite::params![key.as_str()],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn list_all(&self) -> Result<Vec<Setting>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT key, value, updated_at FROM setting ORDER BY key ASC;")
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let key: String = row.get(0)?;
                let val: String = row.get(1)?;
                let updated_at: String = row.get(2)?;
                Ok((key, val, updated_at))
            })
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut settings = Vec::new();
        for item in rows {
            let (raw_key, raw_val, updated_at) =
                item.map_err(|e| CoreError::Validation(e.to_string()))?;
            let parsed_key = SettingKey::from_key_str(&raw_key);
            if let Ok(val) = parse_setting_value(&parsed_key, &raw_val) {
                settings.push(Setting {
                    key: parsed_key,
                    value: val,
                    updated_at,
                });
            }
        }

        Ok(settings)
    }
}

fn serialize_setting_value(value: &SettingValue) -> Result<String, CoreError> {
    match value {
        SettingValue::String(s) => Ok(s.clone()),
        SettingValue::Boolean(b) => Ok(b.to_string()),
        SettingValue::Integer(i) => Ok(i.to_string()),
        SettingValue::QualityBudget(qb) => {
            serde_json::to_string(qb).map_err(|e| CoreError::Validation(e.to_string()))
        }
        SettingValue::Json(v) => {
            serde_json::to_string(v).map_err(|e| CoreError::Validation(e.to_string()))
        }
    }
}

fn parse_setting_value(key: &SettingKey, raw: &str) -> Result<SettingValue, CoreError> {
    match key {
        SettingKey::QualityBudget => {
            let qb: QualityBudget = serde_json::from_str(raw)
                .map_err(|e| CoreError::Validation(format!("Invalid QualityBudget JSON: {e}")))?;
            // Validate ranges
            QualityBudget::new(qb.max_long_edge, qb.encoder_quality)?;
            Ok(SettingValue::QualityBudget(qb))
        }
        SettingKey::RunAtStartup | SettingKey::OpenEditorAfterCapture => {
            if let Ok(b) = raw.parse::<bool>() {
                Ok(SettingValue::Boolean(b))
            } else {
                Err(CoreError::Validation(format!(
                    "Invalid boolean for key: {}",
                    key.as_str()
                )))
            }
        }
        SettingKey::VaultPath
        | SettingKey::HotkeyCapture
        | SettingKey::HotkeyOpenEditor
        | SettingKey::WebServiceAddress => Ok(SettingValue::String(raw.to_string())),
        SettingKey::Custom(_) => {
            if let Ok(b) = raw.parse::<bool>() {
                Ok(SettingValue::Boolean(b))
            } else if let Ok(i) = raw.parse::<i64>() {
                Ok(SettingValue::Integer(i))
            } else if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(raw) {
                Ok(SettingValue::Json(json_val))
            } else {
                Ok(SettingValue::String(raw.to_string()))
            }
        }
    }
}
