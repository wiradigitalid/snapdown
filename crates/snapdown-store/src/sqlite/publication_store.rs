use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use snapdown_core::domain::publication::Publication;
use snapdown_core::error::CoreError;
use snapdown_core::ports::PublicationStore;

use crate::error::StoreError;
use crate::sqlite::migrations::run_migrations;

#[derive(Clone)]
pub struct SqlitePublicationStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqlitePublicationStore {
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

impl PublicationStore for SqlitePublicationStore {
    fn get_by_bundle_id(&self, bundle_id: &str) -> Result<Option<Publication>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, bundle_id, slug, base_url, published_at, unpublished_at, last_error FROM publication WHERE bundle_id = ?1;",
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let pub_opt = stmt
            .query_row(rusqlite::params![bundle_id], |row| {
                Ok(Publication {
                    id: row.get(0)?,
                    bundle_id: row.get(1)?,
                    slug: row.get(2)?,
                    base_url: row.get(3)?,
                    published_at: row.get(4)?,
                    unpublished_at: row.get(5)?,
                    last_error: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(pub_opt)
    }

    fn get_by_slug(&self, slug: &str) -> Result<Option<Publication>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, bundle_id, slug, base_url, published_at, unpublished_at, last_error FROM publication WHERE slug = ?1;",
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let pub_opt = stmt
            .query_row(rusqlite::params![slug], |row| {
                Ok(Publication {
                    id: row.get(0)?,
                    bundle_id: row.get(1)?,
                    slug: row.get(2)?,
                    base_url: row.get(3)?,
                    published_at: row.get(4)?,
                    unpublished_at: row.get(5)?,
                    last_error: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(pub_opt)
    }

    fn save(&self, publication: &Publication) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO publication (id, bundle_id, slug, base_url, published_at, unpublished_at, last_error)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(bundle_id) DO UPDATE SET
                slug = excluded.slug,
                base_url = excluded.base_url,
                published_at = excluded.published_at,
                unpublished_at = excluded.unpublished_at,
                last_error = excluded.last_error;
            "#,
            rusqlite::params![
                publication.id,
                publication.bundle_id,
                publication.slug,
                publication.base_url,
                publication.published_at,
                publication.unpublished_at,
                publication.last_error,
            ],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn mark_unpublished(&self, bundle_id: &str, unpublished_at: &str) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        conn.execute(
            "UPDATE publication SET unpublished_at = ?1 WHERE bundle_id = ?2;",
            rusqlite::params![unpublished_at, bundle_id],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn set_last_error(&self, bundle_id: &str, error: Option<&str>) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        conn.execute(
            "UPDATE publication SET last_error = ?1 WHERE bundle_id = ?2;",
            rusqlite::params![error, bundle_id],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn delete_by_bundle_id(&self, bundle_id: &str) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        conn.execute(
            "DELETE FROM publication WHERE bundle_id = ?1;",
            rusqlite::params![bundle_id],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn list_active(&self) -> Result<Vec<Publication>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, bundle_id, slug, base_url, published_at, unpublished_at, last_error FROM publication WHERE unpublished_at IS NULL ORDER BY published_at DESC;",
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Publication {
                    id: row.get(0)?,
                    bundle_id: row.get(1)?,
                    slug: row.get(2)?,
                    base_url: row.get(3)?,
                    published_at: row.get(4)?,
                    unpublished_at: row.get(5)?,
                    last_error: row.get(6)?,
                })
            })
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut pubs = Vec::new();
        for r in rows {
            pubs.push(r.map_err(|e| CoreError::Validation(e.to_string()))?);
        }

        Ok(pubs)
    }
}
