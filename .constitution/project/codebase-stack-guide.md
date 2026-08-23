---
status: Accepted
ratified_by: 90d55a77c0d90a586bea960e88167052cb9a3e47
---

# Stack — Codebase Guide

**Loaded when:** writing or reviewing code across any crate or container in Snapdown.

Distilled and ratified at the close of Wave W1.

## 1. Toolchains & Runtimes

- **Rust:** Rust 1.96 / 2021 edition. Managed via Cargo workspace at repository root.
- **Node.js:** Node 22+ / 24 LTS, npm with committed package-lock.json files (installs enforced with `npm ci`).
- **Desktop Runtime:** Tauri v2 (`tauri`, `tauri-build`, `tauri-plugin-single-instance`, `tauri-plugin-global-shortcut`, `tauri-plugin-autostart`).
- **Frontend:** React 19, Vite 7, TypeScript 5.

## 2. Workspace Layout & Crates

| Path | Purpose | Key Invariants |
| --- | --- | --- |
| `crates/snapdown-core` | Pure domain entities, value objects, ports | Zero I/O, no OS/network/clock calls. Enforced via `cargo test --test test_no_io` and `clippy.toml` `disallowed-methods`. |
| `crates/snapdown-store` | Database & filesystem adapters | SQLite `library.db` with forward-only idempotent migrations in transaction; WAL mode; integrity checks. `VaultBlobStore` with canonical path resolution and strict root confinement. |
| `apps/desktop` | Tauri v2 application host + React frontend | Single instance mutex, system tray, IPC command bridge, settings view. |
| `web/ui` | Shared React design system & tokens | Pure UI primitives (`Button`, `TextField`, `Modal`, `Toast`, `ConfirmDialog`, `Checkbox`). Strict token consumption from `tokens.css` with zero literal hex colors. |

## 3. Verification Commands

Run from repository root:

```bash
# Rust Workspace verification
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Shared UI verification
npm --prefix web/ui run typecheck
npm --prefix web/ui run lint
npm --prefix web/ui run test

# Desktop Frontend verification
npm --prefix apps/desktop run typecheck
npm --prefix apps/desktop run lint
npm --prefix apps/desktop run test
npm --prefix apps/desktop run build

# Desktop Application Build & Dev
npm --prefix apps/desktop run tauri build
npm --prefix apps/desktop run tauri dev

# Method Corpus Validation
uv run .constitution/method/scripts/validate.py --check
```
