# Pruned on 2026-08-30

This was the full Tauri v2 + React + Vite desktop app `DEC-007` replaced with `apps/desktop`
(Slint). It has been trimmed from ~141 MB to the ~260 KB left here: `node_modules/`, `dist/`,
build config (`package.json`, `vite.config.ts`, `tauri.conf.json`, `Cargo.toml`, …), and every
source file for a surface already rebuilt in Slint (capture overlay, capture note field, editor
shell, Settings and its four sections, hotkey/startup commands, the old icon set) are gone.
`apps/desktop/assets/app-icon.{png,ico}` is the current icon; the generation script and
intermediates that produced it lived in `.work/` and are also gone.

What is left, and why it is still worth reading before building the Slint equivalent:

| Kept | Covers | Read before |
|---|---|---|
| `src-tauri/src/server/` (`mod.rs`, `auth.rs`, `error.rs`, `handlers.rs`), `src-tauri/tests/test_local_api_server.rs` | The Local API that actually worked — the four endpoints `crates/snapdown-bridge` still calls with nothing listening | Building the fix for `BUG-59` |
| `archive/desktop-tauri/src/components/AgentAccessView.tsx`, `archive/desktop-tauri/src/services/agent_access.ts`, `src-tauri/src/commands/agent_access.rs`, both matching test files | The Agent access surface (issue/revoke a key) | Building the Agent access screen (`UC-17`, `UC-19`, and the surface `BUG-59`'s fix needs to create a key from) |
| `archive/desktop-tauri/src/components/FindingsView.tsx`, `archive/desktop-tauri/src/test/findings_view.test.tsx` | Findings list/management and its delete confirmation | `BUG-61`'s Findings list, delete (`UC-7`) |
| `archive/desktop-tauri/src/components/OrphanReportView.tsx`, `archive/desktop-tauri/src/test/orphan_report.test.tsx` | The orphan report | `BUG-61` / `UC-8` |
| `archive/desktop-tauri/src/components/BundleView.tsx`, `archive/desktop-tauri/src/services/bundle.ts`, `src-tauri/src/commands/bundle.rs`, both matching test files | Bundles list and detail | `BUG-61`'s Bundles list/detail (`UC-10`, `UC-11`) |
| `src-tauri/src/commands/sharing.rs`, `src-tauri/src/publish/` (`mod.rs`, `client.rs`), `src-tauri/tests/test_publish_client.rs` | The publish/unpublish flow and its client | `BUG-61`'s publish surface (`FR-23`, `FR-25`) — no frontend for this one was ever built (`BUG-2`), so there is no `.tsx` to read |
| `src-tauri/src/lib.rs`, `main.rs`, `state.rs`, `commands/mod.rs` | How the above wired together in the old app | Context only |
| `archive/desktop-tauri/src/types/finding.ts`, `archive/desktop-tauri/src/types/settings.ts` | The shapes the kept components/services use | Context only |

None of this builds or runs — config and dependencies are gone with everything else. It is prose
and structure to read, not code to compile. Once every row above has a Slint/Rust equivalent this
whole folder can go.
