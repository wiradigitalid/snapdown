use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use snapdown_core::domain::bundle::{Bundle, BundleDetail, BundleItem};
use snapdown_core::error::CoreError;
use snapdown_core::ports::BundleStore;

use crate::error::StoreError;
use crate::sqlite::migrations::run_migrations;

#[derive(Clone)]
pub struct SqliteBundleStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteBundleStore {
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

impl BundleStore for SqliteBundleStore {
    fn create_bundle(&self, bundle: &Bundle, items: &[BundleItem]) -> Result<(), CoreError> {
        for item in items {
            if item.bundle_id != bundle.id {
                return Err(CoreError::Validation(format!(
                    "BundleItem bundle_id mismatch: expected {}, got {}",
                    bundle.id, item.bundle_id
                )));
            }
        }

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Insert bundle
        tx.execute(
            r#"
            INSERT INTO bundle (id, name, markdown, markdown_path, composed_at)
            VALUES (?1, ?2, ?3, ?4, ?5);
            "#,
            rusqlite::params![
                bundle.id,
                bundle.name,
                bundle.markdown,
                bundle.markdown_path,
                bundle.composed_at,
            ],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Insert bundle items
        for item in items {
            tx.execute(
                r#"
                INSERT INTO bundle_item (id, bundle_id, finding_id, position, image_path)
                VALUES (?1, ?2, ?3, ?4, ?5);
                "#,
                rusqlite::params![
                    item.id,
                    item.bundle_id,
                    item.finding_id,
                    item.position,
                    item.image_path,
                ],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn get_bundle(&self, id: &str) -> Result<Option<BundleDetail>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, markdown, markdown_path, composed_at FROM bundle WHERE id = ?1;",
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let bundle_opt = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(Bundle {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    markdown: row.get(2)?,
                    markdown_path: row.get(3)?,
                    composed_at: row.get(4)?,
                })
            })
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let bundle = match bundle_opt {
            Some(b) => b,
            None => return Ok(None),
        };

        // Get bundle items ordered by position ASC
        let mut item_stmt = conn
            .prepare(
                r#"
                SELECT id, bundle_id, finding_id, position, image_path
                FROM bundle_item
                WHERE bundle_id = ?1
                ORDER BY position ASC;
                "#,
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let item_rows = item_stmt
            .query_map(rusqlite::params![id], |row| {
                Ok(BundleItem {
                    id: row.get(0)?,
                    bundle_id: row.get(1)?,
                    finding_id: row.get(2)?,
                    position: row.get(3)?,
                    image_path: row.get(4)?,
                })
            })
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut items = Vec::new();
        for item in item_rows {
            items.push(item.map_err(|e| CoreError::Validation(e.to_string()))?);
        }

        Ok(Some(BundleDetail { bundle, items }))
    }

    fn list_bundles(&self) -> Result<Vec<BundleDetail>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT id, name, markdown, markdown_path, composed_at FROM bundle ORDER BY composed_at DESC;")
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let bundle_rows = stmt
            .query_map([], |row| {
                Ok(Bundle {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    markdown: row.get(2)?,
                    markdown_path: row.get(3)?,
                    composed_at: row.get(4)?,
                })
            })
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut bundles = Vec::new();
        for b in bundle_rows {
            bundles.push(b.map_err(|e| CoreError::Validation(e.to_string()))?);
        }

        let mut details = Vec::with_capacity(bundles.len());
        for bundle in bundles {
            let mut item_stmt = conn
                .prepare(
                    r#"
                    SELECT id, bundle_id, finding_id, position, image_path
                    FROM bundle_item
                    WHERE bundle_id = ?1
                    ORDER BY position ASC;
                    "#,
                )
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let item_rows = item_stmt
                .query_map(rusqlite::params![bundle.id], |row| {
                    Ok(BundleItem {
                        id: row.get(0)?,
                        bundle_id: row.get(1)?,
                        finding_id: row.get(2)?,
                        position: row.get(3)?,
                        image_path: row.get(4)?,
                    })
                })
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let mut items = Vec::new();
            for item in item_rows {
                items.push(item.map_err(|e| CoreError::Validation(e.to_string()))?);
            }

            details.push(BundleDetail { bundle, items });
        }

        Ok(details)
    }

    fn update_bundle_name_and_markdown(
        &self,
        id: &str,
        name: &str,
        markdown: &str,
    ) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let rows = conn
            .execute(
                "UPDATE bundle SET name = ?1, markdown = ?2 WHERE id = ?3;",
                rusqlite::params![name, markdown, id],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        if rows == 0 {
            return Err(CoreError::NotFound(format!("Bundle not found: {id}")));
        }

        Ok(())
    }

    fn delete_bundle(&self, id: &str) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // With foreign_keys ON, deleting bundle cascades to bundle_item
        conn.execute("DELETE FROM bundle WHERE id = ?1;", rusqlite::params![id])
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }
}
