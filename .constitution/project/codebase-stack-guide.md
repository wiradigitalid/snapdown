---
status: Accepted
ratified_by: 90d55a77c0d90a586bea960e88167052cb9a3e47
---

# Stack — Codebase Guide

**Loaded when:** writing or reviewing code across any crate or container in Snapdown.

Distilled and ratified at the close of Wave W1.

> **Corrected 2026-09-01, and this file BINDS — read the correction before the sections below.** It was
> distilled at W1 and describes the stack as it was then. `DEC-007` moved the desktop app from a Tauri
> host with a React webview onto **Slint**, and `OQ-27` deleted `web/ui` on 2026-09-01. Sections 1 and
> 2 are corrected here; anything further down this file that names Tauri, React, Vite, or an `npm`
> command is a record of the W1 repo, not an instruction, and MUST be checked against
> `.control/structure-codebase.md` before it is followed. `AGENTS.md` § Verification is the current
> authority on how to verify a change.

## 1. Toolchains & Runtimes

- **Rust:** Rust 1.96 / 2021 edition. Managed via Cargo workspace at repository root.
- **Desktop UI:** Slint, declared in `apps/desktop/ui/*.slint` and driven from Rust in
  `apps/desktop/src/`. `DEC-007`.
- **Web service:** Go, `apps/web-service`, with its own SQLite store.
- **Node.js:** **none.** There is no `package.json` anywhere in the active workspace as of 2026-09-01.
  `apps/desktop` lost its at `DEC-007`; `web/ui` was deleted at `OQ-27`.
- **Retired, listed so an old reference resolves:** Tauri v2 and its plugins, React 19, Vite 7,
  TypeScript 5. The Tauri implementation is kept at `archive/desktop-tauri/` as history and MUST NOT
  be imported or built.

## 2. Workspace Layout & Crates

| Path | Purpose | Key Invariants |
| --- | --- | --- |
| `crates/snapdown-core` | Pure domain entities, value objects, ports | Zero I/O, no OS/network/clock calls. Enforced via `cargo test --test test_no_io` and `clippy.toml` `disallowed-methods`. |
| `crates/snapdown-store` | Database & filesystem adapters | SQLite `library.db` with forward-only idempotent migrations in transaction; WAL mode; integrity checks. `VaultBlobStore` with canonical path resolution and strict root confinement. |
| `apps/desktop` | Slint desktop application - tray, hotkeys, capture overlay and Editor in one process (`AD-11`) | Colour only in `ui/theme.slint`, both themes. A UI component ships with a test proving something mounts it - see `codebase-conventions-guide.md`. |
| `apps/web-service` | Go service serving published Bundles | Its own SQLite store, separate from `library.db`. A reader that looks only at the Rust store will report its tables as missing. |
| `crates/snapdown-bridge` | The MCP executable | - |

The `web/ui` row stood here until 2026-09-01 and named `tokens.css` as the token source. That package
was deleted under `OQ-27`; the palette that ships is `apps/desktop/ui/theme.slint`, and the `apps/desktop`
row above was a Tauri/React description that `DEC-007` retired.

## 3. Verification Commands

Run from repository root:

```bash
# Rust workspace - this is the whole of the code verification now
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# The Go service
cd apps/web-service && go test ./...

# Method corpus validation - NOT proof the code compiles; a different question
uv run .constitution/method/scripts/validate.py --check
```

**Every `npm` line that stood in this block until 2026-09-01 now fails with `enoent`**, and they had
been failing for weeks before anyone deleted them: `apps/desktop` lost its `package.json` at `DEC-007`
and `web/ui` was removed at `OQ-27`. The two `npm run tauri` lines are retired with the Tauri host.
`AGENTS.md` § Verification carries the same list and is the one to trust if these ever disagree again.
