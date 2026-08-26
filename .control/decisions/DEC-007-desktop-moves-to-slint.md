---
type: decision
id: DEC-007
status: draft
touches: []
supersedes: DEC-001
superseded_by: null
created: "2026-08-26"
---

# DEC-007 — The desktop application's UI moves from a Tauri v2 + React webview to native Slint

## Decision

`apps/desktop` is a Slint UI (with the `i-slint-backend-winit` backend on Windows) rendered
in-process by the same Rust binary that owns the Library, replacing the Tauri v2 + React + Vite +
TypeScript webview DEC-001 named. The binary is `Snapdown.exe`, built from crate
`snapdown-desktop`. The previous implementation is archived at `archive/desktop-tauri` for
reference and is deliberately removed from the Cargo workspace, so it is not built or tested by
CI. Everything else DEC-001 settled — the Go web service, the MCP bridge, embedded SQLite, and
AD-11's one-process rule (tray, hotkeys, capture overlay, and the Library-owning window all in one
executable) — is unchanged; this decision narrows DEC-001, it does not replace it.

## Why

The repo owner instructed this migration directly, mid-session, as a decision already made
("kita sudah migrasi total ke slint" — "we've fully migrated to Slint") rather than one to weigh
here. What is recorded is the state that made the instruction the obviously correct thing to
execute, so the choice reads as more than an unexplained order to the next person who opens this
file:

- Every commit in the repository's recent history touching the desktop app — from the Graphite
  design system pass through window-dragging fixes, theme tokens, and the capture-canvas
  1:1-scale fix — landed on `apps/desktop-slint`. `apps/desktop` (Tauri) has no commit activity
  in the same window; it was the frozen half of the codebase, not the active one.
- The Slint app was already the more complete, more verified implementation at the point of this
  decision: it launches, renders, and its capture → annotate → marker → note flow is exercised
  live (screenshots in this session's own record). The Tauri app's own defect register
  (`BUG-11`) recorded that, for most of this project's life, no bundled `Snapdown.exe` had ever
  been verified to run — every UI claim against it was made against a Vite dev server, never a
  real build. `BUG-11` was later resolved (`W6-S11`), but the pattern it exposed — verification
  against something other than the shipped binary — never held for the Slint app, which this
  session ran and drove as the actual executable throughout.
- Slint gives per-monitor capture, global hotkeys, a system tray, and Windows startup
  registration from the same native Win32 surface DEC-001 already committed to for Tauri
  (`windows`, `winreg`), without a bundled webview or a Node/npm build step in the loop at all.
  DEC-001's own listed cost — "Tauri's webview is the platform's, not bundled... depends on the
  WebView2 runtime installed on the Reviewer's machine" — is fully retired by this move, not
  merely reduced.

## Cost

- **The desktop UI surface reset to a fraction of what DEC-005 had allowed to be built.**
  `apps/desktop-slint` was written from scratch and, at the point of this decision, implements
  only the Editor shell and the Capture Overlay (with its note field) — confirmed directly by
  reading `apps/desktop/ui/appwindow.slint`'s two exported components and by running this
  product's own `inventory.py` after repointing its readers at the new source tree. Every other
  screen the corpus promises — Findings management beyond the filmstrip, Bundle composition and
  detail, Publish/unpublish, Settings, Agent access, the Orphan report — has no Slint
  implementation yet. Some of this gap is DEC-005's own deliberate ordering (`sharing` and
  `agent-access` were already parked); the rest — `bundle`, and most of `finding` and
  `settings` beyond capture — had working React screens under the archived Tauri app and now do
  not exist in the active product at all. This is a real, not merely bookkeeping, regression in
  built surface, and it is the direct, accepted cost of this decision.
- **The Local API (AD-5) has no server in the new app.** The Tauri implementation served
  `/v1/health`, `/v1/bundles`, `/v1/bundles/{id}`, and `/v1/bundles/{id}/images/{filename}` over
  loopback HTTP from `apps/desktop/src-tauri/src/server/`. Nothing in `apps/desktop` (Slint)
  currently opens that port, so `agent-access`'s MCP path is unreachable at runtime even though
  `snapdown-bridge` and its MCP tool declarations are untouched.
- **`web/ui` (`@snapdown/ui`) is now unconsumed by anything in the active workspace.** It was
  built to be shared between the Tauri desktop webview and the published-Bundle reader (DEC-001's
  own stated reason for choosing React on both sides). The desktop side is gone; `apps/web-service`
  (Go) never consumed it. Whether `web/ui` still has a future — as the published-Bundle reader's
  frontend once `sharing` resumes, or not at all — is not decided here and is left open below.
  The React component inventory it still holds (`BundleComposer.tsx`, `MarkerLayer.tsx`,
  `ConfirmDialog.tsx`, `AgentAccessView.tsx`, and others) is not evidence any desktop screen is
  built; a future reconciliation pass MUST NOT credit a Slint screen as done because a same-named
  React file survives in an orphaned package.
- **The corpus now describes a stack the code does not run.** `.how/_platform/ARCHITECTURE-SPINE.md`'s
  Stack table (Tauri 2.x, React 19.x, Vite 7.x, TypeScript 5.x, Node 24.x build-time) and its C4
  containers describe the desktop app in terms this decision retires. Naming conventions written
  for TypeScript/React no longer apply to the desktop half of the product. This is the corpus debt
  this decision incurs; closing it is `wdi-blueprint intent platform`'s work at `apply`, not
  something recorded here.
- **Two crates of native Windows integration were re-hand-written rather than reused.** Tauri's
  plugin ecosystem (`tauri-plugin-global-shortcut`, `tauri-plugin-autostart`,
  `tauri-plugin-single-instance`, `tauri-plugin-window-state`) is gone; the Slint app now owns
  its own single-instance mutex, and ports the pre-existing `GlobalShortcutBackend` /
  `AutoStartBackend` trait abstractions onto the `global-hotkey` and `tray-icon` crates (the same
  two crates Tauri's plugins wrap internally) plus the `WindowsRegistryAutoStartBackend` that was
  already framework-agnostic. `tauri-plugin-window-state` (remembering window position/size
  across launches) has no replacement yet.

## Alternatives

Required here: `finding`, `bundle`, and `agent-access` all sit at `risk_accepted: low`.

| Option | Why not |
| --- | --- |
| Finish the Tauri app instead | Rejected by the owner's direct instruction, not weighed as a live option in this session; the Tauri app was also the stale half of the codebase by every commit-history signal available |
| Keep both and pick per-build | Reintroduces exactly the "which `Snapdown.exe` is real" ambiguity `BUG-11` and the stale-`desktop.exe` incident already cost this project; AD-11 already forbids a second desktop executable from one build, and maintaining two live UI stacks for one desktop app is the same failure at the source level instead of the binary level |
| Rebuild the missing screens in Slint before recording this decision | Would make the record true later instead of true now, and the corpus would describe an aspirational state indistinguishable from a shipped one — the exact failure this method's `mode`/depth rules exist to prevent |
| Keep `web/ui`'s React components as the desktop UI's design reference and hand-port them screen by screen | Not excluded — this is the likely path for rebuilding `bundle`/`settings`/`agent-access` in Slint — but it is future coding work, not part of this record |

## Reversal trigger

Any of these makes revisiting correct:

- Slint's Windows backend (`i-slint-backend-winit`) proves unable to match a capability the
  Tauri app already shipped — most concretely, per-monitor DPI-correct overlay capture — at the
  fidelity `NFR-1` requires. That reopens the desktop framework, not the process model.
- The `tray-icon` / `global-hotkey` crates prove unstable enough on Windows (a hung tray icon
  surviving process exit, a hotkey that silently stops firing) that Tauri's more heavily-used
  plugin equivalents would materially reduce defect load.
- Rebuilding `bundle`, `settings`, and `agent-access` in Slint turns out to cost materially more
  than resuming the archived React implementation would have, once actually attempted.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | Whether `web/ui` is kept (as a future published-Bundle reader or a Slint UI reference), archived alongside `archive/desktop-tauri`, or removed, is not decided here and should be filed to `wdi-question` before it goes stale by neglect |
| Note | This decision's `touches` is intentionally empty at `draft`. It reaches `.how/_platform/ARCHITECTURE-SPINE.md` (Stack table, C4 containers, Consistency Conventions), `.how/_platform/c4-l3-desktop-app.md`, and `.constitution/project/codebase-stack-guide.md` at `apply`, each through its own owning skill — none of that was hand-edited to reach this record |
| Source material | This session's own conversation and work: the icon fix, the tray/hotkey/autostart port from `archive/desktop-tauri` onto `tray-icon`/`global-hotkey`/the existing `WindowsRegistryAutoStartBackend`, the `apps/desktop-slint` → `apps/desktop` rename, the workspace/CI updates, and the `inventory-readers.py` fix that produced the screen/API gap this Cost section cites |
