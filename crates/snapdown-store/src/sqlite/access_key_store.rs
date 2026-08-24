use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use snapdown_core::domain::access_key::{AccessKey, AuthResult};
use snapdown_core::error::CoreError;
use snapdown_core::ports::AccessKeyStore;

use crate::error::StoreError;
use crate::sqlite::migrations::run_migrations;

#[derive(Clone)]
pub struct SqliteAccessKeyStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteAccessKeyStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let path_ref = path.as_ref();

        // Check if database file exists. If it exists, verify integrity on a read-only connection
        // before opening read-write or applying any modifying pragmas (BUG-15, BR-118).
        if path_ref.exists() {
            let ro_flags = OpenFlags::SQLITE_OPEN_READ_ONLY;
            let ro_conn = Connection::open_with_flags(path_ref, ro_flags)?;
            let mut integrity_stmt = ro_conn.prepare("PRAGMA quick_check;")?;
            let integrity_res: String = integrity_stmt.query_row([], |row| row.get(0))?;
            if integrity_res != "ok" {
                return Err(StoreError::Corruption(integrity_res));
            }
            drop(integrity_stmt);
            drop(ro_conn);
        }

        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
        let mut conn = Connection::open_with_flags(path_ref, flags)?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

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

    pub fn get_schema_version(&self) -> Result<i64, StoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StoreError::Corruption(e.to_string()))?;
        crate::sqlite::migrations::get_schema_version(&conn)
    }
}

impl AccessKeyStore for SqliteAccessKeyStore {
    fn get_active_key(&self) -> Result<Option<AccessKey>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, key_hash, issued_at, revoked_at FROM access_key WHERE revoked_at IS NULL ORDER BY issued_at DESC LIMIT 1;",
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let key_opt = stmt
            .query_row([], |row| {
                Ok(AccessKey {
                    id: row.get(0)?,
                    key_hash: row.get(1)?,
                    issued_at: row.get(2)?,
                    revoked_at: row.get(3)?,
                })
            })
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(key_opt)
    }

    fn save_key(&self, key: &AccessKey) -> Result<(), CoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // 1. Revoke any currently active keys to ensure exactly one active key
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        tx.execute(
            "UPDATE access_key SET revoked_at = ?1 WHERE revoked_at IS NULL;",
            rusqlite::params![now],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        // 2. Insert new key
        tx.execute(
            "INSERT INTO access_key (id, key_hash, issued_at, revoked_at) VALUES (?1, ?2, ?3, ?4);",
            rusqlite::params![key.id, key.key_hash, key.issued_at, key.revoked_at],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn revoke_active_key(&self, revoked_at: &str) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        conn.execute(
            "UPDATE access_key SET revoked_at = ?1 WHERE revoked_at IS NULL;",
            rusqlite::params![revoked_at],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn verify_key(&self, secret: &str) -> Result<AuthResult, CoreError> {
        let active_key = self.get_active_key()?;

        match active_key {
            Some(key) => {
                if key.verify_secret(secret) {
                    Ok(AuthResult::Valid)
                } else {
                    Ok(AuthResult::Invalid)
                }
            }
            None => Ok(AuthResult::NoKeyConfigured),
        }
    }
}
