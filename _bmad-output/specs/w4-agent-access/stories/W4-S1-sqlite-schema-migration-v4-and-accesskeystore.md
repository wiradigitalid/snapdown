---
id: W4-S1
title: SQLite schema migration v4 (access_key) and AccessKeyStore
wave: W4
status: in_progress
created: 2026-08-23
dependencies: []
files:
  - crates/snapdown-core/src/domain/access_key.rs
  - crates/snapdown-core/src/ports/access_key_store.rs
  - crates/snapdown-core/src/domain/mod.rs
  - crates/snapdown-core/src/ports/mod.rs
  - crates/snapdown-store/src/sqlite/access_key_store.rs
  - crates/snapdown-store/src/sqlite/migrations.rs
  - crates/snapdown-store/src/sqlite/mod.rs
  - crates/snapdown-store/tests/test_sqlite_access_keys.rs
---

# W4-S1: SQLite schema migration v4 (access_key) and AccessKeyStore

## User Story
As a reviewer granting temporary reading access to local coding agents, I want an `access_key` database table, `AccessKey` domain entity, and `AccessKeyStore` repository implementation with constant-time verification and credential store integration, so that an access key can be securely issued, queried, validated, and revoked.

## Acceptance Criteria
- [ ] Implement `AccessKey` domain entity and `AccessKeyStatus` in `crates/snapdown-core/src/domain/access_key.rs` with secure hashing (e.g. SHA-256 / argon2 / constant-time comparison).
- [ ] Declare `AccessKeyStore` port trait in `crates/snapdown-core/src/ports/access_key_store.rs`.
- [ ] Add migration v4 creating `access_key` table (`id TEXT PRIMARY KEY, key_hash TEXT NOT NULL, issued_at TEXT NOT NULL, revoked_at TEXT`) in `crates/snapdown-store/src/sqlite/migrations.rs`.
- [ ] Implement `SqliteAccessKeyStore` in `crates/snapdown-store/src/sqlite/access_key_store.rs` ensuring only one active key at a time (issuing a new key automatically revokes previous ones).
- [ ] Implement constant-time comparison logic (`subtle` or constant-time byte comparison) for token authentication.
- [ ] Full unit and integration test coverage (`cargo test`).
