# Retrospective — Wave W1 (Settings)

- **Wave:** W1
- **Component:** `settings` (mode: catalog, risk_accepted: medium)
- **Initiative / Release:** `capture-to-markdown` / `r1`
- **Closed Date:** 2026-08-23
- **Stories Delivered:**
  - `W1-S1`: Cargo workspace, Tauri v2 shell, React webview, tray, and CI (`58e46ab`, PR #1)
  - `W1-S2`: library.db with migrations, the setting table, and the Vault blob adapter (`44a4810`, PR #2)
  - `W1-S3`: Settings screen — the Vault folder and the Quality Budget (`1997509`, PR #3)
  - `W1-S4`: Hotkey binding, OS registration, and honest conflict reporting (`288cf98`, PR #4)
  - `W1-S5`: Run at Windows startup, reflecting the real registration (`3acdf09`, PR #5)

---

## 1. What was delivered

1. **Foundational Architecture Substrate:** Established zero-I/O `snapdown-core` with strict port abstractions (`SettingsStore`, `BlobStore`, `Clock`, `EntropySource`, `HotkeyRegistrar`, `StartupRegistrar`), enforced by dependency-graph test and scoped clippy disallowed-methods rules.
2. **SQLite Database & Forward-Only Migrations:** Implemented `SqliteSettingsStore` in `snapdown-store` with forward-only idempotent migrations for `schema_version` and `setting` tables, WAL mode, corruption detection via quick_check, and default fallback.
3. **Vault Blob Storage Adapter:** Built `VaultBlobStore` enforcing canonical path resolution and strict root confinement, refusing traversal escapes (`..`), root slashes, and empty paths.
4. **Desktop Settings UI (Screen 12):** Created React Settings screen (`/settings`) consuming shared design tokens and primitives (`TextField`, `Button`, `Toast`, `ConfirmDialog`, `Checkbox`), providing interactive forms for Vault folder relocation (with atomic file migration & rollback), Quality Budget configuration (1600px / 75 defaults, bounds validation), Hotkey management (conflict detection, dynamic rebinding, startup warning alerts), and Windows startup registration (non-elevated HKCU autostart with direct uncached OS readback).
5. **CI Workflows:** Established `korpus.yml` gating corpus validator baseline and `desktop-ci.yml` running Rust build/test/clippy and React typecheck/lint/test/build on Windows runners.

---

## 2. Key Decisions Recorded & Retained

- **DEC-001 (Stack):** Locked Tauri v2, Rust 1.96, React 19 + Vite 7, SQLite via rusqlite.
- **First-Run State (MF-8):** First run is determined by checking whether the `setting` table holds zero rows, avoiding flag files or extra registry keys.
- **Single-Instance Lifecycle:** Secondary launches focus existing instance and exit 0.

---

## 3. Inventory Plan vs Reality

- `library.db` tables active: `setting`, `schema_version` (rows 8 & 9 of `inventory-db.md`).
- Later wave tables (`finding`, `note`, `marker`, `bundle`, etc.) remain planned and uncreated as intended.
- Screens active: Screen 12 (`/settings`).

---

## 4. Verification & Gate Status

- Rust workspace: 100% passing tests (27 unit/integration tests).
- Frontend web/ui & desktop: 100% passing vitest suites (32 tests total) and clean TypeScript typechecks.
- Corpus validator: Baseline matched with 0 unexpected violations.
- Wave W1 status transitioned to `closed`.
