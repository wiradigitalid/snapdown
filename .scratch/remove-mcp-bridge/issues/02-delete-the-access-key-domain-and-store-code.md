# 02: Delete the Access Key domain, port, and SQLite store code

**What to build:** the Access Key ceremony's data layer — the `AccessKey` domain type, the
`AccessKeyStore` port, and `SqliteAccessKeyStore` — no longer exists in `snapdown-core` or
`snapdown-store`. Nothing in either crate's public surface (`lib.rs` re-exports, `domain/mod.rs`,
`ports/mod.rs`, `sqlite/mod.rs`) still names it. This scaffolding was never wired to a live server
(`snapdown-bridge` never depended on `snapdown-core`/`snapdown-store`, and `apps/desktop` never called
it either), so nothing outside its own tests consumes it — this ticket has no runtime behaviour to
preserve, only dead code to remove.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `crates/snapdown-core/src/domain/access_key.rs` is deleted; `crates/snapdown-core/src/domain/mod.rs`
      drops its `pub mod access_key;` line; `crates/snapdown-core/src/lib.rs` drops its
      `pub use domain::access_key::{AccessKey, AuthResult};` line.
- [ ] `crates/snapdown-core/src/ports/access_key_store.rs` is deleted; `crates/snapdown-core/src/ports/mod.rs`
      drops its `pub mod access_key_store;` and `pub use access_key_store::AccessKeyStore;` lines, and
      the `AccessKeyStore` name is dropped from `crates/snapdown-core/src/lib.rs`'s `pub use ports::{...}`
      list.
- [ ] `crates/snapdown-store/src/sqlite/access_key_store.rs` is deleted; `crates/snapdown-store/src/sqlite/mod.rs`
      drops its `pub mod access_key_store;` and `pub use access_key_store::SqliteAccessKeyStore;` lines,
      and `SqliteAccessKeyStore` is dropped from `crates/snapdown-store/src/lib.rs`'s `pub use sqlite::{...}`
      list.
- [ ] `crates/snapdown-store/tests/test_sqlite_access_keys.rs` is deleted.
- [ ] `crates/snapdown-store/src/sqlite/migrations.rs`: the `version: 4` ("create access_key table")
      entry is removed from the `MIGRATIONS` array. This is safe because `run_migrations` applies
      strictly by `version > current_version` with no requirement that version numbers be contiguous:
      a fresh install never creates the table; an already-migrated library (already past version 4)
      is untouched and simply keeps an inert, unqueried table. Confirm every existing
      `assert_eq!(..., get_schema_version()..., 9)` (or equivalent) test still holds — 9 remains the
      highest version in the array — rather than renumbering the remaining migrations.
- [ ] `cargo build --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
      -- -D warnings`, and `cargo test --workspace --no-fail-fast` all succeed from the repo root with
      nothing left referencing `AccessKey`, `AuthResult`, `AccessKeyStore`, or `SqliteAccessKeyStore`.
