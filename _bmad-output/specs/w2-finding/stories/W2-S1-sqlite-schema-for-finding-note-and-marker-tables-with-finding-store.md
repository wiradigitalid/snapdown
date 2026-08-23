---
title: 'W2-S1: SQLite schema for finding, note, and marker tables with FindingStore'
type: 'feature'
created: '2026-08-23'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - _bmad-output/specs/w2-finding/SPEC.md
  - .how/_platform/inventory-db.md
  - .how/_platform/cross-cutting.md
  - .how/finding/SDD-finding.md
  - .what/finding/SRS-finding.md
  - .what/finding/03-domain/domain-model.md
  - .what/finding/02-rules/rules-finding.md
  - .what/business-rules.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** `library.db` currently only supports schema version 1 with `schema_version` and `setting` tables. Snapdown needs to persist Findings, Notes, and Markers (`LC-004 finding-store`) with strict invariant guarantees (AD-1: single sequence for markers and note lines, AD-2: atomic creation/deletion coupling with vault storage, AD-3: normalized marker coordinates `[0.0..=1.0]`).

**Approach:**
1. Define pure domain entities in `crates/snapdown-core`: `Finding`, `Note`, `Marker`, `FindingWithDetails` (or `FindingDetail`), `Region` struct/tuple, coordinate validation for `[0.0..=1.0]`, and entity id creation using `id_from_parts` (UUIDv7).
2. Define `FindingStore` port trait in `crates/snapdown-core/src/ports/mod.rs`.
3. Add migration version 2 to `crates/snapdown-store/src/sqlite/migrations.rs` creating `finding`, `note`, and `marker` tables matching `.how/_platform/inventory-db.md` (rows 1–3).
4. Implement `SqliteFindingStore` in `crates/snapdown-store/src/sqlite/finding_store.rs` implementing `FindingStore`, handling transactional insert, update, retrieval, deletion, and atomic gap-free marker renumbering.
5. Provide comprehensive unit and integration tests verifying forward migration from v1 to v2, idempotent migration, CRUD operations, transaction rollbacks, coordinate range validation, and single-sequence marker renumbering.

## Boundaries & Constraints

**Always:**
- Migration v2 MUST create exactly three tables matching `inventory-db.md` (rows 1–3):
  - `finding` (`id TEXT PRIMARY KEY`, `image_path TEXT NOT NULL`, `image_width INTEGER NOT NULL`, `image_height INTEGER NOT NULL`, `captured_at TEXT NOT NULL`, `source_monitor TEXT NOT NULL`, `region TEXT NOT NULL`)
  - `note` (`id TEXT PRIMARY KEY`, `finding_id TEXT NOT NULL UNIQUE REFERENCES finding(id) ON DELETE CASCADE`, `body TEXT NOT NULL`, `updated_at TEXT NOT NULL`)
  - `marker` (`id TEXT PRIMARY KEY`, `finding_id TEXT NOT NULL REFERENCES finding(id) ON DELETE CASCADE`, `ordinal INTEGER NOT NULL`, `x REAL NOT NULL`, `y REAL NOT NULL`, `comment TEXT NOT NULL`, `UNIQUE(finding_id, ordinal)`)
- Foreign key constraints MUST be enabled (`PRAGMA foreign_keys = ON;`).
- Marker coordinates (`x`, `y`) MUST be validated in closed range `[0.0, 1.0]` (AD-3, BR-2). Out-of-bounds coordinates must be rejected with `CoreError::Validation` rather than clamped.
- Marker `ordinal` runs from 1 upward with no gaps (AD-1, BR-2). Marker deletion, addition, or reordering must maintain gap-free sequence starting at 1.
- Timestamps must be RFC 3339 UTC strings with explicit `Z` suffix (`cross-cutting.md`).
- Entity IDs must be lowercase UUIDv7 strings (`id_from_parts`).
- Maintain zero I/O and zero OS calls in `snapdown-core`.

**Block If:**
- Upstream requirements in `.what/`, `.how/`, `.control/`, or `.constitution/` demand tables outside `finding`, `note`, `marker`, `setting`, `schema_version` in this story.
- Invariants AD-1, AD-2, or AD-3 are violated.

**Never:**
- Do not create tables for `bundle`, `bundle_item`, `publication`, `access_key` (these belong to future waves W3/W4/W5).
- Do not implement Capture overlay UI, screen grabbing, image reduction pipeline, or React editor UI in this story (these belong to W2-S2 through W2-S6).
- Do not use autoincrement sequence for `marker.ordinal`.
- Do not allow empty gap in `marker.ordinal` for a finding.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Migrate v1 to v2 database | Existing DB at schema v1 | Applies migration 2; creates `finding`, `note`, `marker` tables; updates `schema_version` to 2 | Transaction rollback on failure; returns `StoreError` |
| Create Finding with Note and Markers | `Finding`, `Note`, `Vec<Marker>` | Transactionally inserts rows into `finding`, `note`, and `marker` tables | Rolls back entire insert if any table fails; returns `StoreError` |
| Get Finding with Note and Markers | Finding ID | Returns `Option<FindingDetail>` with finding metadata, note body, and markers ordered by `ordinal ASC` | Returns `Ok(None)` if finding does not exist |
| List Findings | None | Returns `Vec<FindingDetail>` ordered by `captured_at DESC` (BR-43) | Returns `StoreError` on DB failure |
| Update Note body | Finding ID and new body text | Updates `note.body` and `note.updated_at` | Returns `CoreError::NotFound` if finding doesn't exist |
| Add Marker to Finding | Finding ID, `x`, `y`, `comment` | Validates `x, y in [0.0, 1.0]`, assigns `ordinal = max_ordinal + 1`, inserts marker | Returns `CoreError::Validation` if coordinates invalid; `CoreError::NotFound` if finding missing |
| Update Marker position / comment | Finding ID, Marker ID, new `x, y`, new `comment` | Validates `x, y in [0.0, 1.0]`, updates marker record preserving `ordinal` (BR-48) | Returns `CoreError::Validation` if coords invalid; `CoreError::NotFound` if marker missing |
| Delete Marker and renumber | Finding ID, Marker ID | Deletes marker, decrements `ordinal` for all markers with `ordinal > deleted_ordinal` in single transaction (AD-1, BR-2) | Rolls back if renumbering fails; preserves sequence without gaps |
| Reorder Markers | Finding ID, new ordered Marker IDs | Updates `ordinal` for all markers of the finding in 1-based order (1..=N) in single transaction | Validates all marker IDs belong to finding; rolls back on mismatch |
| Delete Finding (DB row) | Finding ID | Deletes finding row; cascades deletion of note and markers | Returns `StoreError` on failure |
| Marker coordinates out of range | `x = 1.05` or `y = -0.01` | Rejected before DB write (AD-3) | Returns `CoreError::Validation` |
| Duplicate finding ID insert | Finding with already existing ID | Insert rejected | Returns `StoreError` |

</intent-contract>

## Code Map

- `crates/snapdown-core/src/domain/finding.rs` -- Domain entities: `Finding`, `Note`, `Marker`, `FindingDetail`, `Region`
- `crates/snapdown-core/src/domain/mod.rs` -- Export finding module
- `crates/snapdown-core/src/ports/mod.rs` -- `FindingStore` port trait definition
- `crates/snapdown-core/src/lib.rs` -- Export finding models and `FindingStore`
- `crates/snapdown-store/src/sqlite/migrations.rs` -- Migration 2 definition (`finding`, `note`, `marker` tables) and runner
- `crates/snapdown-store/src/sqlite/finding_store.rs` -- `SqliteFindingStore` implementation of `FindingStore` trait
- `crates/snapdown-store/src/sqlite/mod.rs` -- Export `finding_store` module
- `crates/snapdown-store/src/lib.rs` -- Export `SqliteFindingStore`
- `crates/snapdown-store/tests/test_sqlite_findings.rs` -- Integration tests for migration v2, Finding CRUD, Marker single-sequence renumbering, coordinate validation, and transaction guarantees
- `.constitution/project/inventory-readers.py` -- Update `derive_db` reader to recognize `finding`, `note`, and `marker` tables

## Tasks & Acceptance

**Execution:**
- `crates/snapdown-core/src/domain/finding.rs` -- Implement `Finding`, `Note`, `Marker`, `FindingDetail`, and `Region` domain types with normalized coordinate validation `[0.0, 1.0]` -- Provide pure domain entities adhering to AD-1, AD-3, BR-1, BR-2.
- `crates/snapdown-core/src/ports/mod.rs` -- Define `FindingStore` trait with transactional operations for finding, note, and marker manipulation -- Provide storage interface port.
- `crates/snapdown-store/src/sqlite/migrations.rs` -- Add migration 2 with DDL for `finding`, `note`, `marker` tables -- Match `inventory-db.md` schema rows 1–3.
- `crates/snapdown-store/src/sqlite/finding_store.rs` -- Implement `SqliteFindingStore` with atomic CRUD and gap-free marker renumbering -- Deliver `LC-004 finding-store`.
- `crates/snapdown-store/tests/test_sqlite_findings.rs` -- Add comprehensive integration test suite -- Cover named tests: `migrations_v2_apply_cleanly_and_idempotently`, `finding_store_crud_and_transaction_guarantees`, `marker_renumber_preserves_single_sequence_invariant`.
- `.constitution/project/inventory-readers.py` -- Update `derive_db` reader for finding/note/marker tables -- Ensure inventory generator stays accurate.

**Acceptance Criteria:**
- Given a v1 database, when migrations run, then schema version advances to 2 and `finding`, `note`, and `marker` tables exist with foreign keys and unique constraints.
- Given a new `Finding` with an associated `Note` and list of `Marker`s, when inserted via `FindingStore`, all records are persisted transactionally; if any part fails, no partial records remain.
- Given a `Finding` with 3 markers (ordinals 1, 2, 3), when marker 2 is deleted, marker 3 is renumbered to ordinal 2, leaving ordinals 1 and 2 with no gaps (AD-1, BR-2).
- Given an attempt to create or update a `Marker` with `x < 0.0`, `x > 1.0`, `y < 0.0`, or `y > 1.0`, then the operation is refused with `CoreError::Validation` (AD-3).
- Given multiple findings in the database, when `list_findings` is called, results are returned sorted by `captured_at DESC` (BR-43).
- Given `cargo clippy`, `cargo test`, `npm run lint`, and `npm run test` across workspace, all checks pass with zero warnings or errors.

## Spec Change Log

_None._

## Review Triage Log

_None._

## Auto Run Result

_None._

## Design Notes

### Schema Definition (Migration 2)
```sql
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
```

### Marker Renumbering Invariant (AD-1, BR-2)
When deleting a marker at ordinal $K$:
1. `DELETE FROM marker WHERE id = ?1 AND finding_id = ?2;`
2. `UPDATE marker SET ordinal = ordinal - 1 WHERE finding_id = ?2 AND ordinal > ?3;` (Executed in order or with temporary offset if necessary to satisfy unique constraint during transaction).

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: All Rust files formatted cleanly
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Clippy clean with no warnings across workspace
- `cargo test --workspace` -- expected: All unit and integration tests pass, including:
  - `cargo::migrations_v2_apply_cleanly_and_idempotently`
  - `cargo::finding_store_crud_and_transaction_guarantees`
  - `cargo::marker_renumber_preserves_single_sequence_invariant`
- `npm --prefix web/ui run typecheck` -- expected: Shared UI clean
- `npm --prefix web/ui run lint` -- expected: Shared UI linter clean
- `npm --prefix web/ui run test` -- expected: Shared UI tests pass
- `npm --prefix apps/desktop run typecheck` -- expected: Desktop frontend clean
- `npm --prefix apps/desktop run lint` -- expected: Desktop frontend linter clean
- `npm --prefix apps/desktop run test` -- expected: Desktop frontend tests pass
- `uv run .constitution/method/scripts/validate.py --check` -- expected: Validator passes baseline comparison
