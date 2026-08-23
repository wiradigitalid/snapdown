---
id: W5-S1
title: SQLite schema migration v5 (publication) and PublicationStore
wave: W5
status: done
created: 2026-08-23
dependencies: []
files:
  - crates/snapdown-core/src/domain/publication.rs
  - crates/snapdown-core/src/ports/publication_store.rs
  - crates/snapdown-core/src/domain/mod.rs
  - crates/snapdown-core/src/ports/mod.rs
  - crates/snapdown-store/src/sqlite/publication_store.rs
  - crates/snapdown-store/src/sqlite/migrations.rs
  - crates/snapdown-store/src/sqlite/mod.rs
  - crates/snapdown-store/tests/test_sqlite_publications.rs
---

# W5-S1: SQLite schema migration v5 (publication) and PublicationStore

## User Story
As a reviewer preparing to publish bundles, I want a `publication` database table, a `Publication` domain model with CSPRNG base32 160-bit slug generation (independent from library IDs), and a `PublicationStore` repository implementation in SQLite with sticky error tracking so publication lifecycle can be reliably recorded locally.

## Acceptance Criteria
- [ ] Implement `Publication` entity in `crates/snapdown-core/src/domain/publication.rs` with `id`, `bundle_id`, `slug`, `base_url`, `published_at`, `unpublished_at`, `last_error`.
- [ ] Implement slug generation algorithm (160 bits from CSPRNG, unguessable, lower-case base32/alphanumeric, no library ID leaks per AD-8).
- [ ] Declare `PublicationStore` port trait in `crates/snapdown-core/src/ports/publication_store.rs`.
- [ ] Add migration v5 creating `publication` table in `crates/snapdown-store/src/sqlite/migrations.rs`.
- [ ] Implement `SqlitePublicationStore` in `crates/snapdown-store/src/sqlite/publication_store.rs`.
- [ ] Add integration test suite in `crates/snapdown-store/tests/test_sqlite_publications.rs`.
