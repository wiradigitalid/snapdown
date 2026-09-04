# 01: Delete the `mcp-bridge` executable and its workspace membership

**What to build:** `crates/snapdown-bridge` — the stdio MCP server, its `LocalApiClient`, and its
`main.rs` entry point — no longer exists anywhere in the tree, is no longer a workspace member, and is
no longer built or tested by CI. The one-executable-per-app CI guard in `desktop-ci.yml` no longer
names `snapdown-bridge.exe` as a second legitimate binary, since none is produced anymore.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `crates/snapdown-bridge/` (the whole directory: `src/lib.rs`, `src/main.rs`, `src/client.rs`,
      `src/mcp.rs`, `tests/test_bridge_mcp.rs`, `Cargo.toml`) is deleted.
- [ ] The root `Cargo.toml`'s `[workspace] members` list no longer has a `"crates/snapdown-bridge"`
      line.
- [ ] `Cargo.lock` is regenerated (via `cargo build`/`cargo test`) and committed with no
      `snapdown-bridge` entry left in it.
- [ ] `.github/workflows/desktop-ci.yml`'s `desktop-build` job: the `$known` array drops
      `"snapdown-bridge.exe"`, and the comment above it (currently explaining that
      `snapdown-bridge` is a legitimate second workspace binary) is rewritten so it no longer
      describes a binary that no longer exists — while keeping the part of the comment that still
      holds: the guard exists to catch a stray *second desktop* executable (`FR-27`, `BR-121`), not
      to require exactly one `.exe` in the directory.
- [ ] `cargo build --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
      -- -D warnings`, and `cargo test --workspace --no-fail-fast` all succeed from the repo root with
      nothing left referencing `snapdown_bridge`/`snapdown-bridge`.
