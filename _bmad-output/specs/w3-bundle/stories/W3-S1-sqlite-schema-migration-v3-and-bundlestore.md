---
id: W3-S1
title: SQLite Schema Migration v3 (bundle, bundle_item) and BundleStore
wave: W3
status: done
created: 2026-08-23
dependencies: []
files:
  - crates/snapdown-core/src/domain/bundle.rs
  - crates/snapdown-core/src/ports/bundle_store.rs
  - crates/snapdown-store/src/sqlite/bundle_store.rs
  - crates/snapdown-store/src/sqlite/migrations.rs
  - crates/snapdown-store/tests/test_sqlite_bundles.rs
---

# W3-S1: SQLite Schema Migration v3 (bundle, bundle_item) and BundleStore

## User Story
As a user creating multi-finding documentation bundles, I want `bundle` and `bundle_item` tables and a `BundleStore` repository implementation so that findings can be organized into structured, persistent bundles.

## Acceptance Criteria
- [ ] Implement `Bundle` and `BundleItem` domain value entities in `crates/snapdown-core/src/domain/bundle.rs`.
- [ ] Declare `BundleStore` port trait in `crates/snapdown-core/src/ports/bundle_store.rs`.
- [ ] Add migration v3 creating `bundle` and `bundle_item` tables with foreign keys and unique constraints in `crates/snapdown-store/src/sqlite/migrations.rs`.
- [ ] Implement `SqliteBundleStore` with transactional create, read, update, list, and delete methods.
- [ ] Full unit and integration test coverage (`cargo test`).
