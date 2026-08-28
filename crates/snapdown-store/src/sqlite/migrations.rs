use rusqlite::{Connection, Transaction};

use crate::error::StoreError;

pub struct Migration {
    pub version: i64,
    pub description: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "create setting and schema_version tables",
        sql: r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS setting (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
    "#,
    },
    Migration {
        version: 2,
        description: "create finding, note, and marker tables",
        sql: r#"
        CREATE TABLE IF NOT EXISTS finding (
            id TEXT PRIMARY KEY,
            image_path TEXT NOT NULL,
            image_width INTEGER NOT NULL,
            image_height INTEGER NOT NULL,
            captured_at TEXT NOT NULL,
            source_monitor TEXT NOT NULL,
            region TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS note (
            id TEXT PRIMARY KEY,
            finding_id TEXT NOT NULL UNIQUE,
            body TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(finding_id) REFERENCES finding(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS marker (
            id TEXT PRIMARY KEY,
            finding_id TEXT NOT NULL,
            ordinal INTEGER NOT NULL,
            x REAL NOT NULL,
            y REAL NOT NULL,
            comment TEXT NOT NULL,
            FOREIGN KEY(finding_id) REFERENCES finding(id) ON DELETE CASCADE,
            UNIQUE(finding_id, ordinal)
        );
    "#,
    },
    Migration {
        version: 3,
        description: "create bundle and bundle_item tables",
        sql: r#"
        CREATE TABLE IF NOT EXISTS bundle (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            markdown TEXT NOT NULL,
            markdown_path TEXT NOT NULL,
            composed_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS bundle_item (
            id TEXT PRIMARY KEY,
            bundle_id TEXT NOT NULL,
            finding_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            image_path TEXT NOT NULL,
            FOREIGN KEY(bundle_id) REFERENCES bundle(id) ON DELETE CASCADE,
            FOREIGN KEY(finding_id) REFERENCES finding(id) ON DELETE CASCADE,
            UNIQUE(bundle_id, finding_id)
        );
    "#,
    },
    Migration {
        version: 4,
        description: "create access_key table",
        sql: r#"
        CREATE TABLE IF NOT EXISTS access_key (
            id TEXT PRIMARY KEY,
            key_hash TEXT NOT NULL,
            issued_at TEXT NOT NULL,
            revoked_at TEXT
        );
    "#,
    },
    Migration {
        version: 5,
        description: "create publication table",
        sql: r#"
        CREATE TABLE IF NOT EXISTS publication (
            id TEXT PRIMARY KEY,
            bundle_id TEXT NOT NULL UNIQUE,
            slug TEXT NOT NULL UNIQUE,
            base_url TEXT NOT NULL,
            published_at TEXT NOT NULL,
            unpublished_at TEXT,
            last_error TEXT,
            FOREIGN KEY(bundle_id) REFERENCES bundle(id) ON DELETE CASCADE
        );
    "#,
    },
    Migration {
        version: 6,
        description: "drop finding_id foreign key constraint from bundle_item table",
        sql: r#"
        CREATE TABLE IF NOT EXISTS bundle_item_v6 (
            id TEXT PRIMARY KEY,
            bundle_id TEXT NOT NULL,
            finding_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            image_path TEXT NOT NULL,
            FOREIGN KEY(bundle_id) REFERENCES bundle(id) ON DELETE CASCADE,
            UNIQUE(bundle_id, finding_id)
        );

        INSERT INTO bundle_item_v6 (id, bundle_id, finding_id, position, image_path)
        SELECT id, bundle_id, finding_id, position, image_path FROM bundle_item;

        DROP TABLE bundle_item;

        ALTER TABLE bundle_item_v6 RENAME TO bundle_item;
    "#,
    },
    Migration {
        version: 7,
        description:
            "add quality budget derivation and resolution tracking columns to finding table",
        sql: r#"
        ALTER TABLE finding ADD COLUMN resolved_long_edge INTEGER;
        ALTER TABLE finding ADD COLUMN resolved_encoder_quality INTEGER;
        ALTER TABLE finding ADD COLUMN budget_name TEXT;
    "#,
    },
    // THIS TABLE WAS DESIGNED BEFORE IT WAS BUILT, and the design is
    // `.how/finding/05-model/data-model.md`. `BUG-72` reported "NO `visual_annotation` TABLE", which
    // was true of the code and not of the corpus - the shape had been specified and never
    // implemented. The column names here are the ones that document chose.
    //
    // `properties_json` is one JSON column rather than the union of five shapes' fields.
    // `AnnotationShape` is an internally-tagged enum whose five variants share almost nothing: an
    // Arrow has two endpoints and no size, a Callout has a tail and a font, a Blur has a radius. A
    // column per field would be a table of mostly-NULLs whose validity would live in Rust anyway.
    //
    // TWO DELIBERATE DEPARTURES from that design, both recorded in the data model beside it:
    //
    //   no `kind` column.  The design has one. `AnnotationShape` is `#[serde(tag = "kind")]`, so the
    //     tag is ALREADY the first key inside `properties_json` - a column would be a second copy of
    //     one fact, and the copy is the one that goes stale. Nothing queries by kind; every read is
    //     "all of this Finding's, in order".
    //   `position` added. The design has no z-order at all, and the burn needs one: what was drawn
    //     later covers what was drawn earlier, and that is what the Reviewer saw on the canvas.
    Migration {
        version: 8,
        description: "create visual_annotation table",
        sql: r#"
        CREATE TABLE IF NOT EXISTS visual_annotation (
            id TEXT PRIMARY KEY,
            finding_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            properties_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(finding_id) REFERENCES finding(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_visual_annotation_finding
            ON visual_annotation(finding_id, position);
    "#,
    },
];

pub fn run_migrations(conn: &mut Connection) -> Result<(), StoreError> {
    // First, ensure schema_version table exists so we can inspect current version
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
    "#,
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version;",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for migration in MIGRATIONS {
        if migration.version > current_version {
            let tx = conn.transaction()?;
            apply_migration(&tx, migration)?;
            tx.commit()?;
        }
    }

    Ok(())
}

fn apply_migration(tx: &Transaction, migration: &Migration) -> Result<(), StoreError> {
    tx.execute_batch(migration.sql)?;
    let applied_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    tx.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2);",
        rusqlite::params![migration.version, applied_at],
    )?;
    Ok(())
}

pub fn get_schema_version(conn: &Connection) -> Result<i64, StoreError> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='schema_version';")?;
    let exists = stmt.exists([])?;
    if !exists {
        return Ok(0);
    }

    let version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version;",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(version)
}
