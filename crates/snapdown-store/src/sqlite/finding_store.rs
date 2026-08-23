use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction};
use snapdown_core::domain::finding::{Finding, FindingDetail, Marker, Note};
use snapdown_core::error::CoreError;
use snapdown_core::ports::FindingStore;

use crate::error::StoreError;
use crate::sqlite::migrations::run_migrations;

#[derive(Clone)]
pub struct SqliteFindingStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteFindingStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let path_ref = path.as_ref();

        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
        let mut conn = Connection::open_with_flags(path_ref, flags)?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

        {
            let mut integrity_stmt = conn.prepare("PRAGMA quick_check;")?;
            let integrity_res: String = integrity_stmt.query_row([], |row| row.get(0))?;
            if integrity_res != "ok" {
                return Err(StoreError::Corruption(integrity_res));
            }
        }

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

impl FindingStore for SqliteFindingStore {
    fn create_finding(
        &self,
        finding: &Finding,
        note: &Note,
        markers: &[Marker],
    ) -> Result<(), CoreError> {
        // Validate all marker coordinates and ordinals upfront
        for (idx, marker) in markers.iter().enumerate() {
            Marker::validate_coordinates(marker.x, marker.y)?;
            let expected_ordinal = (idx + 1) as u32;
            if marker.ordinal != expected_ordinal {
                return Err(CoreError::Validation(format!(
                    "Marker ordinal sequence must start at 1 and have no gaps. Expected {}, got {}",
                    expected_ordinal, marker.ordinal
                )));
            }
            if marker.finding_id != finding.id {
                return Err(CoreError::Validation(format!(
                    "Marker finding_id mismatch: expected {}, got {}",
                    finding.id, marker.finding_id
                )));
            }
        }

        if note.finding_id != finding.id {
            return Err(CoreError::Validation(format!(
                "Note finding_id mismatch: expected {}, got {}",
                finding.id, note.finding_id
            )));
        }

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Insert finding with derivation tracking columns (NFR-18, BR-105)
        tx.execute(
            r#"
            INSERT INTO finding (
                id, image_path, image_width, image_height, captured_at,
                source_monitor, region, resolved_long_edge, resolved_encoder_quality, budget_name
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);
            "#,
            rusqlite::params![
                finding.id,
                finding.image_path,
                finding.image_width,
                finding.image_height,
                finding.captured_at,
                finding.source_monitor,
                finding.region,
                finding.resolved_long_edge,
                finding.resolved_encoder_quality,
                finding.budget_name,
            ],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Insert note
        tx.execute(
            r#"
            INSERT INTO note (id, finding_id, body, updated_at)
            VALUES (?1, ?2, ?3, ?4);
            "#,
            rusqlite::params![note.id, note.finding_id, note.body, note.updated_at],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Insert markers
        for marker in markers {
            tx.execute(
                r#"
                INSERT INTO marker (id, finding_id, ordinal, x, y, comment)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6);
                "#,
                rusqlite::params![
                    marker.id,
                    marker.finding_id,
                    marker.ordinal,
                    marker.x,
                    marker.y,
                    marker.comment,
                ],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn get_finding(&self, id: &str) -> Result<Option<FindingDetail>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut finding_stmt = conn
            .prepare(
                r#"
                SELECT id, image_path, image_width, image_height, captured_at,
                       source_monitor, region, resolved_long_edge, resolved_encoder_quality, budget_name
                FROM finding WHERE id = ?1;
                "#,
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let finding_opt = finding_stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(Finding {
                    id: row.get(0)?,
                    image_path: row.get(1)?,
                    image_width: row.get(2)?,
                    image_height: row.get(3)?,
                    captured_at: row.get(4)?,
                    source_monitor: row.get(5)?,
                    region: row.get(6)?,
                    resolved_long_edge: row.get(7)?,
                    resolved_encoder_quality: row.get(8)?,
                    budget_name: row.get(9)?,
                })
            })
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let finding = match finding_opt {
            Some(f) => f,
            None => return Ok(None),
        };

        // Get note
        let mut note_stmt = conn
            .prepare("SELECT id, finding_id, body, updated_at FROM note WHERE finding_id = ?1;")
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let note_opt = note_stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    finding_id: row.get(1)?,
                    body: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let note = note_opt.unwrap_or_else(|| Note {
            id: String::new(),
            finding_id: id.to_string(),
            body: String::new(),
            updated_at: finding.captured_at.clone(),
        });

        // Get markers ordered by ordinal ASC
        let mut marker_stmt = conn
            .prepare(
                r#"
                SELECT id, finding_id, ordinal, x, y, comment
                FROM marker
                WHERE finding_id = ?1
                ORDER BY ordinal ASC;
                "#,
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let marker_rows = marker_stmt
            .query_map(rusqlite::params![id], |row| {
                Ok(Marker {
                    id: row.get(0)?,
                    finding_id: row.get(1)?,
                    ordinal: row.get(2)?,
                    x: row.get(3)?,
                    y: row.get(4)?,
                    comment: row.get(5)?,
                })
            })
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut markers = Vec::new();
        for m in marker_rows {
            markers.push(m.map_err(|e| CoreError::Validation(e.to_string()))?);
        }

        Ok(Some(FindingDetail {
            finding,
            note,
            markers,
        }))
    }

    fn list_findings(&self) -> Result<Vec<FindingDetail>, CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Order findings by captured_at DESC per BR-43
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, image_path, image_width, image_height, captured_at,
                       source_monitor, region, resolved_long_edge, resolved_encoder_quality, budget_name
                FROM finding
                ORDER BY captured_at DESC;
                "#,
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let finding_rows = stmt
            .query_map([], |row| {
                Ok(Finding {
                    id: row.get(0)?,
                    image_path: row.get(1)?,
                    image_width: row.get(2)?,
                    image_height: row.get(3)?,
                    captured_at: row.get(4)?,
                    source_monitor: row.get(5)?,
                    region: row.get(6)?,
                    resolved_long_edge: row.get(7)?,
                    resolved_encoder_quality: row.get(8)?,
                    budget_name: row.get(9)?,
                })
            })
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut findings = Vec::new();
        for f in finding_rows {
            findings.push(f.map_err(|e| CoreError::Validation(e.to_string()))?);
        }

        let mut details = Vec::with_capacity(findings.len());
        for finding in findings {
            let mut note_stmt = conn
                .prepare("SELECT id, finding_id, body, updated_at FROM note WHERE finding_id = ?1;")
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let note_opt = note_stmt
                .query_row(rusqlite::params![finding.id], |row| {
                    Ok(Note {
                        id: row.get(0)?,
                        finding_id: row.get(1)?,
                        body: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                })
                .optional()
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let note = note_opt.unwrap_or_else(|| Note {
                id: String::new(),
                finding_id: finding.id.clone(),
                body: String::new(),
                updated_at: finding.captured_at.clone(),
            });

            let mut marker_stmt = conn
                .prepare(
                    r#"
                    SELECT id, finding_id, ordinal, x, y, comment
                    FROM marker
                    WHERE finding_id = ?1
                    ORDER BY ordinal ASC;
                    "#,
                )
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let marker_rows = marker_stmt
                .query_map(rusqlite::params![finding.id], |row| {
                    Ok(Marker {
                        id: row.get(0)?,
                        finding_id: row.get(1)?,
                        ordinal: row.get(2)?,
                        x: row.get(3)?,
                        y: row.get(4)?,
                        comment: row.get(5)?,
                    })
                })
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let mut markers = Vec::new();
            for m in marker_rows {
                markers.push(m.map_err(|e| CoreError::Validation(e.to_string()))?);
            }

            details.push(FindingDetail {
                finding,
                note,
                markers,
            });
        }

        Ok(details)
    }

    fn delete_finding(&self, id: &str) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // With foreign_keys ON and CASCADE configured, deleting from finding cascades to note and marker
        conn.execute("DELETE FROM finding WHERE id = ?1;", rusqlite::params![id])
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn update_note(&self, finding_id: &str, body: &str, updated_at: &str) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Check if finding exists
        let finding_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM finding WHERE id = ?1);",
                rusqlite::params![finding_id],
                |row| row.get(0),
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        if !finding_exists {
            return Err(CoreError::NotFound(format!(
                "Finding not found: {finding_id}"
            )));
        }

        let rows_affected = conn
            .execute(
                "UPDATE note SET body = ?1, updated_at = ?2 WHERE finding_id = ?3;",
                rusqlite::params![body, updated_at, finding_id],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        if rows_affected == 0 {
            // Note row might need to be inserted if missing
            conn.execute(
                "INSERT INTO note (id, finding_id, body, updated_at) VALUES (?1, ?2, ?3, ?4);",
                rusqlite::params![format!("note-{finding_id}"), finding_id, body, updated_at],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;
        }

        Ok(())
    }

    fn add_marker(
        &self,
        finding_id: &str,
        marker_id: &str,
        x: f64,
        y: f64,
        comment: &str,
    ) -> Result<Marker, CoreError> {
        Marker::validate_coordinates(x, y)?;

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Check finding existence
        let finding_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM finding WHERE id = ?1);",
                rusqlite::params![finding_id],
                |row| row.get(0),
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        if !finding_exists {
            return Err(CoreError::NotFound(format!(
                "Finding not found: {finding_id}"
            )));
        }

        let max_ordinal: u32 = tx
            .query_row(
                "SELECT COALESCE(MAX(ordinal), 0) FROM marker WHERE finding_id = ?1;",
                rusqlite::params![finding_id],
                |row| row.get(0),
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let next_ordinal = max_ordinal + 1;

        tx.execute(
            r#"
            INSERT INTO marker (id, finding_id, ordinal, x, y, comment)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6);
            "#,
            rusqlite::params![marker_id, finding_id, next_ordinal, x, y, comment],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(Marker {
            id: marker_id.to_string(),
            finding_id: finding_id.to_string(),
            ordinal: next_ordinal,
            x,
            y,
            comment: comment.to_string(),
        })
    }

    fn update_marker(
        &self,
        finding_id: &str,
        marker_id: &str,
        x: f64,
        y: f64,
        comment: &str,
    ) -> Result<Marker, CoreError> {
        Marker::validate_coordinates(x, y)?;

        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT ordinal FROM marker WHERE id = ?1 AND finding_id = ?2;")
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let ordinal: u32 = stmt
            .query_row(rusqlite::params![marker_id, finding_id], |row| row.get(0))
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?
            .ok_or_else(|| {
                CoreError::NotFound(format!(
                    "Marker {marker_id} not found on finding {finding_id}"
                ))
            })?;

        conn.execute(
            "UPDATE marker SET x = ?1, y = ?2, comment = ?3 WHERE id = ?4 AND finding_id = ?5;",
            rusqlite::params![x, y, comment, marker_id, finding_id],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(Marker {
            id: marker_id.to_string(),
            finding_id: finding_id.to_string(),
            ordinal,
            x,
            y,
            comment: comment.to_string(),
        })
    }

    fn delete_marker(&self, finding_id: &str, marker_id: &str) -> Result<(), CoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let deleted_ordinal: Option<u32> = tx
            .query_row(
                "SELECT ordinal FROM marker WHERE id = ?1 AND finding_id = ?2;",
                rusqlite::params![marker_id, finding_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let ordinal = match deleted_ordinal {
            Some(ord) => ord,
            None => {
                return Err(CoreError::NotFound(format!(
                    "Marker {marker_id} not found on finding {finding_id}"
                )));
            }
        };

        // 1. Delete marker
        tx.execute(
            "DELETE FROM marker WHERE id = ?1 AND finding_id = ?2;",
            rusqlite::params![marker_id, finding_id],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        // 2. Renumber remaining markers with ordinal > deleted_ordinal in ascending order
        renumber_markers_transaction(&tx, finding_id)?;

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let _ = ordinal;
        Ok(())
    }

    fn reorder_markers(
        &self,
        finding_id: &str,
        ordered_marker_ids: &[&str],
    ) -> Result<(), CoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Check that ordered_marker_ids exactly matches existing markers for finding_id
        let existing_ids: Vec<String> = {
            let mut stmt = tx
                .prepare("SELECT id FROM marker WHERE finding_id = ?1 ORDER BY ordinal ASC;")
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let rows = stmt
                .query_map(rusqlite::params![finding_id], |row| row.get(0))
                .map_err(|e| CoreError::Validation(e.to_string()))?;

            let mut ids = Vec::new();
            for r in rows {
                ids.push(r.map_err(|e| CoreError::Validation(e.to_string()))?);
            }
            ids
        };

        if existing_ids.len() != ordered_marker_ids.len() {
            return Err(CoreError::Validation(format!(
                "Reorder count mismatch: finding has {} markers, provided {}",
                existing_ids.len(),
                ordered_marker_ids.len()
            )));
        }

        for id in ordered_marker_ids {
            if !existing_ids.contains(&id.to_string()) {
                return Err(CoreError::Validation(format!(
                    "Marker {id} does not belong to finding {finding_id}"
                )));
            }
        }

        // To avoid UNIQUE(finding_id, ordinal) collision during reordering,
        // first shift all ordinals to temporary negative or high range (e.g. + 10000)
        tx.execute(
            "UPDATE marker SET ordinal = ordinal + 10000 WHERE finding_id = ?1;",
            rusqlite::params![finding_id],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        // Now assign new ordinals 1..=N in order
        for (idx, marker_id) in ordered_marker_ids.iter().enumerate() {
            let new_ordinal = (idx + 1) as u32;
            tx.execute(
                "UPDATE marker SET ordinal = ?1 WHERE id = ?2 AND finding_id = ?3;",
                rusqlite::params![new_ordinal, marker_id, finding_id],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }
}

fn renumber_markers_transaction(tx: &Transaction, finding_id: &str) -> Result<(), CoreError> {
    let marker_ids: Vec<String> = {
        let mut stmt = tx
            .prepare("SELECT id FROM marker WHERE finding_id = ?1 ORDER BY ordinal ASC;")
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![finding_id], |row| row.get(0))
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let mut ids = Vec::new();
        for r in rows {
            ids.push(r.map_err(|e| CoreError::Validation(e.to_string()))?);
        }
        ids
    };

    // Shift to avoid unique constraint clash
    tx.execute(
        "UPDATE marker SET ordinal = ordinal + 10000 WHERE finding_id = ?1;",
        rusqlite::params![finding_id],
    )
    .map_err(|e| CoreError::Validation(e.to_string()))?;

    for (idx, id) in marker_ids.iter().enumerate() {
        let new_ord = (idx + 1) as u32;
        tx.execute(
            "UPDATE marker SET ordinal = ?1 WHERE id = ?2 AND finding_id = ?3;",
            rusqlite::params![new_ord, id, finding_id],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;
    }

    Ok(())
}
