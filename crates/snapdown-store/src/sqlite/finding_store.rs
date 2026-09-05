use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction};
use snapdown_core::domain::finding::{
    AnnotationShape, CropRect, CropRemap, Finding, FindingDetail, Marker, Note, VisualAnnotation,
};
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

        let visual_annotations = read_annotations(&conn, id)?;

        Ok(Some(FindingDetail {
            finding,
            note,
            markers,
            visual_annotations,
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

            let visual_annotations = read_annotations(&conn, &finding.id)?;

            details.push(FindingDetail {
                finding,
                note,
                markers,
                visual_annotations,
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

    fn update_finding_image(
        &self,
        finding_id: &str,
        image_path: &str,
        image_width: u32,
        image_height: u32,
    ) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE finding SET image_path = ?1, image_width = ?2, image_height = ?3 WHERE id = ?4;",
                rusqlite::params![image_path, image_width, image_height, finding_id],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        if rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "Finding not found: {finding_id}"
            )));
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

    fn add_annotation(
        &self,
        finding_id: &str,
        annotation_id: &str,
        data: &AnnotationShape,
        created_at: &str,
    ) -> Result<VisualAnnotation, CoreError> {
        validate_shape(data)?;

        let json = serde_json::to_string(data)
            .map_err(|e| CoreError::Validation(format!("Annotation could not be written: {e}")))?;

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

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

        // Read inside the transaction, so two annotations placed in the same millisecond cannot both
        // claim the same z-order.
        let next_position: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(position), 0) + 1 FROM visual_annotation WHERE finding_id = ?1;",
                rusqlite::params![finding_id],
                |row| row.get(0),
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        tx.execute(
            r#"
            INSERT INTO visual_annotation (id, finding_id, position, properties_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5);
            "#,
            rusqlite::params![annotation_id, finding_id, next_position, json, created_at],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(VisualAnnotation {
            id: annotation_id.to_string(),
            finding_id: finding_id.to_string(),
            data: data.clone(),
            created_at: created_at.to_string(),
        })
    }

    fn update_annotation(
        &self,
        finding_id: &str,
        annotation_id: &str,
        data: &AnnotationShape,
    ) -> Result<VisualAnnotation, CoreError> {
        validate_shape(data)?;

        let json = serde_json::to_string(data)
            .map_err(|e| CoreError::Validation(format!("Annotation could not be written: {e}")))?;

        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        // `created_at` is read back rather than invented: an update is not a creation, and the field
        // is the only record of when the Reviewer actually drew this.
        let created_at: Option<String> = conn
            .query_row(
                "SELECT created_at FROM visual_annotation WHERE id = ?1 AND finding_id = ?2;",
                rusqlite::params![annotation_id, finding_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let created_at = created_at.ok_or_else(|| {
            CoreError::NotFound(format!(
                "Annotation {annotation_id} not found on finding {finding_id}"
            ))
        })?;

        conn.execute(
            "UPDATE visual_annotation SET properties_json = ?1 WHERE id = ?2 AND finding_id = ?3;",
            rusqlite::params![json, annotation_id, finding_id],
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(VisualAnnotation {
            id: annotation_id.to_string(),
            finding_id: finding_id.to_string(),
            data: data.clone(),
            created_at,
        })
    }

    fn delete_annotation(&self, finding_id: &str, annotation_id: &str) -> Result<(), CoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "DELETE FROM visual_annotation WHERE id = ?1 AND finding_id = ?2;",
                rusqlite::params![annotation_id, finding_id],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        if rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "Annotation {annotation_id} not found on finding {finding_id}"
            )));
        }

        // The survivors are NOT renumbered, and that is the difference from `delete_marker`.
        //
        // A Marker's ordinal is what the Reviewer reads in the Markdown, so a gap in it is a defect.
        // An annotation's `position` is only z-order, and z-order is an ordering, not a sequence:
        // renumbering would rewrite every remaining row to change nothing anyone can see.
        Ok(())
    }

    fn reorder_annotations(
        &self,
        finding_id: &str,
        ordered_annotation_ids: &[&str],
    ) -> Result<(), CoreError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let existing: Vec<String> = {
            let mut stmt = tx
                .prepare(
                    "SELECT id FROM visual_annotation WHERE finding_id = ?1 ORDER BY position ASC;",
                )
                .map_err(|e| CoreError::Validation(e.to_string()))?;
            let rows = stmt
                .query_map(rusqlite::params![finding_id], |row| row.get(0))
                .map_err(|e| CoreError::Validation(e.to_string()))?;
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row.map_err(|e| CoreError::Validation(e.to_string()))?);
            }
            ids
        };

        // The whole set or nothing. A partial order would leave the annotations it omitted at
        // positions the caller never reasoned about, and the result would be an order nobody chose.
        if existing.len() != ordered_annotation_ids.len() {
            return Err(CoreError::Validation(format!(
                "Reorder count mismatch: finding has {} annotations, provided {}",
                existing.len(),
                ordered_annotation_ids.len()
            )));
        }
        for id in ordered_annotation_ids {
            if !existing.iter().any(|known| known == id) {
                return Err(CoreError::Validation(format!(
                    "Annotation {id} does not belong to finding {finding_id}"
                )));
            }
        }

        // `position` carries no UNIQUE constraint, so this needs no shift-out-of-the-way pass the
        // way `reorder_markers` does. It is still one transaction: a half-applied z-order is an
        // order nobody chose either.
        for (index, annotation_id) in ordered_annotation_ids.iter().enumerate() {
            tx.execute(
                "UPDATE visual_annotation SET position = ?1 WHERE id = ?2 AND finding_id = ?3;",
                rusqlite::params![(index + 1) as i64, annotation_id, finding_id],
            )
            .map_err(|e| CoreError::Validation(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }

    fn remap_markers_and_annotations_for_crop(
        &self,
        finding_id: &str,
        old_width: u32,
        old_height: u32,
        crop_rect_px: CropRect,
        new_width: u32,
        new_height: u32,
    ) -> Result<(), CoreError> {
        let remap = CropRemap::new(old_width, old_height, crop_rect_px, new_width, new_height);

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        let markers: Vec<(String, f64, f64)> = {
            let mut stmt = tx
                .prepare("SELECT id, x, y FROM marker WHERE finding_id = ?1 ORDER BY ordinal ASC;")
                .map_err(|e| CoreError::Validation(e.to_string()))?;
            let rows = stmt
                .query_map(rusqlite::params![finding_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                })
                .map_err(|e| CoreError::Validation(e.to_string()))?;
            let mut v = Vec::new();
            for r in rows {
                v.push(r.map_err(|e| CoreError::Validation(e.to_string()))?);
            }
            v
        };

        let mut any_marker_dropped = false;
        for (marker_id, x, y) in markers {
            match remap.remap_marker(x, y) {
                Some((nx, ny)) => {
                    tx.execute(
                        "UPDATE marker SET x = ?1, y = ?2 WHERE id = ?3 AND finding_id = ?4;",
                        rusqlite::params![nx, ny, marker_id, finding_id],
                    )
                    .map_err(|e| CoreError::Validation(e.to_string()))?;
                }
                None => {
                    tx.execute(
                        "DELETE FROM marker WHERE id = ?1 AND finding_id = ?2;",
                        rusqlite::params![marker_id, finding_id],
                    )
                    .map_err(|e| CoreError::Validation(e.to_string()))?;
                    any_marker_dropped = true;
                }
            }
        }

        // A drop must not leave a gap in the ordinal sequence the Reviewer reads as Markdown line
        // numbers - the same invariant `delete_marker` already enforces for a single deletion.
        if any_marker_dropped {
            renumber_markers_transaction(&tx, finding_id)?;
        }

        let annotations = read_annotations(&tx, finding_id)?;
        for annotation in annotations {
            match remap.remap_annotation(&annotation.data) {
                Some(new_shape) => {
                    let json = serde_json::to_string(&new_shape).map_err(|e| {
                        CoreError::Validation(format!("Annotation could not be written: {e}"))
                    })?;
                    tx.execute(
                        "UPDATE visual_annotation SET properties_json = ?1 WHERE id = ?2 AND finding_id = ?3;",
                        rusqlite::params![json, annotation.id, finding_id],
                    )
                    .map_err(|e| CoreError::Validation(e.to_string()))?;
                }
                None => {
                    tx.execute(
                        "DELETE FROM visual_annotation WHERE id = ?1 AND finding_id = ?2;",
                        rusqlite::params![annotation.id, finding_id],
                    )
                    .map_err(|e| CoreError::Validation(e.to_string()))?;
                }
            }
        }

        tx.commit()
            .map_err(|e| CoreError::Validation(e.to_string()))?;

        Ok(())
    }
}

/// Refuses a shape the burner would draw somewhere nobody can see.
///
/// Every coordinate in `AnnotationShape` is normalized to the image, the same convention
/// `Marker::validate_coordinates` enforces - and it is enforced here for the same reason: the burner
/// multiplies by the image's real dimensions, so an out-of-range value does not fail, it draws
/// off-canvas and is silently lost. A redaction box that lands off-canvas leaves the password on the
/// image.
///
/// Extent is checked as a fraction rather than as a second point: a zero-width box is a mis-click,
/// not a drawing, and a width that runs past the edge is clipped by the burner rather than refused -
/// dragging a box off the edge of the image is a normal thing to do.
fn validate_shape(shape: &AnnotationShape) -> Result<(), CoreError> {
    fn point(name: &str, x: f64, y: f64) -> Result<(), CoreError> {
        if !(0.0..=1.0).contains(&x) || x.is_nan() {
            return Err(CoreError::Validation(format!(
                "{name} x must be in [0.0, 1.0], got {x}"
            )));
        }
        if !(0.0..=1.0).contains(&y) || y.is_nan() {
            return Err(CoreError::Validation(format!(
                "{name} y must be in [0.0, 1.0], got {y}"
            )));
        }
        Ok(())
    }

    fn extent(name: &str, width: f64, height: f64) -> Result<(), CoreError> {
        // NaN first: `width <= 0.0` is FALSE for NaN, so the order here is what catches it.
        if width.is_nan() || height.is_nan() || width <= 0.0 || height <= 0.0 {
            return Err(CoreError::Validation(format!(
                "{name} must have a positive width and height, got {width} x {height}"
            )));
        }
        Ok(())
    }

    match shape {
        AnnotationShape::Rect {
            x,
            y,
            width,
            height,
            ..
        } => {
            point("Rect", *x, *y)?;
            extent("Rect", *width, *height)
        }
        AnnotationShape::Blur {
            x,
            y,
            width,
            height,
            ..
        } => {
            point("Blur", *x, *y)?;
            extent("Blur", *width, *height)
        }
        AnnotationShape::Text {
            x,
            y,
            width,
            height,
            ..
        } => {
            point("Text", *x, *y)?;
            extent("Text", *width, *height)
        }
        AnnotationShape::Arrow {
            start_x,
            start_y,
            end_x,
            end_y,
            ..
        } => {
            point("Arrow start", *start_x, *start_y)?;
            point("Arrow end", *end_x, *end_y)
        }
        AnnotationShape::Callout {
            x,
            y,
            width,
            height,
            tail_x,
            tail_y,
            ..
        } => {
            point("Callout", *x, *y)?;
            extent("Callout", *width, *height)?;
            point("Callout tail", *tail_x, *tail_y)
        }
    }
}

/// Every visual annotation on one Finding, in z-order.
///
/// Shared by both read paths on purpose. They already carried two copies of the note query and two
/// of the marker query, and the `visual_annotations: vec![]` this replaces was written twice too -
/// which is exactly how `get_finding` and `list_findings` came to disagree about what a Finding is.
fn read_annotations(
    conn: &Connection,
    finding_id: &str,
) -> Result<Vec<VisualAnnotation>, CoreError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, finding_id, properties_json, created_at
            FROM visual_annotation
            WHERE finding_id = ?1
            ORDER BY position ASC;
            "#,
        )
        .map_err(|e| CoreError::Validation(e.to_string()))?;

    let rows = stmt
        .query_map(rusqlite::params![finding_id], |row| {
            let id: String = row.get(0)?;
            let owner: String = row.get(1)?;
            let data: String = row.get(2)?;
            let created_at: String = row.get(3)?;
            Ok((id, owner, data, created_at))
        })
        .map_err(|e| CoreError::Validation(e.to_string()))?;

    let mut annotations = Vec::new();
    for row in rows {
        let (id, owner, data, created_at) =
            row.map_err(|e| CoreError::Validation(e.to_string()))?;
        // A row whose JSON will not parse is a corrupt row, and it is reported rather than skipped.
        // Silently dropping it would show the Reviewer an image missing the redaction box they drew
        // over a password - which is the one failure in `CAP-11` that cannot be taken back.
        let shape: AnnotationShape = serde_json::from_str(&data).map_err(|e| {
            CoreError::Validation(format!(
                "Annotation {id} on Finding {owner} could not be read: {e}"
            ))
        })?;
        annotations.push(VisualAnnotation {
            id,
            finding_id: owner,
            data: shape,
            created_at,
        });
    }

    Ok(annotations)
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
