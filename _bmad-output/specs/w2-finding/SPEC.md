---
id: SPEC-w2-finding
companions:
  - .control/registry/index.yaml
  - .control/registry/components.yaml
  - .control/product-glossary.md
  - .what/finding/SRS-finding.md
  - .what/finding/03-domain/domain-model.md
  - .what/finding/02-rules/rules-finding.md
  - .what/business-rules.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l2-containers.md
  - .how/_platform/c4-l3-desktop-app.md
  - .how/_platform/inventory-db.md
  - .how/_platform/inventory-screen.md
  - .how/_platform/cross-cutting.md
  - .how/_platform/design-system.md
  - .how/finding/SDD-finding.md
  - .constitution/project/codebase-stack-guide.md
sources:
  - .what/_prd/capture-to-markdown/prd.md
  - .control/registry/requirements.yaml
  - .control/registry/usecases.yaml
  - .control/registry/waves.yaml
---

# W2 — Finding complete: the capture loop, the Editor, Markers, deletion

## Why

**The core interaction loop of Snapdown.** Having built the workspace, settings, and storage substrate in W1, Wave W2 delivers the primary value proposition of the desktop app: capturing a screen region, adding a note and spatial markers, managing findings in the editor, and maintaining vault hygiene through clean deletion and orphan reporting.

Who is affected: The Reviewer, during active software review. Everything in this wave runs locally inside `desktop-app`.

## Capabilities

- **CAP-1** — Capture a region with its note, fast and distraction-free (FR-1, FR-2, FR-3, FR-4, UC-1, UC-2, NFR-1, NFR-2, NFR-3)
  - **intent:** The Reviewer presses the capture hotkey, selects any rectangular region on any connected monitor, enters an optional short note, and returns to their workflow within 500 ms while image reduction and storage occur asynchronously.
  - **success:** On multi-monitor setups with mixed DPI scaling, pressing the capture hotkey dims all screens, draws a region with live dimension readouts, captures pixel-accurate unscaled bitmaps, saves a zero-byte placeholder immediately, applies the Quality Budget reduction asynchronously, and stores the `finding` and `note` in `library.db`.
- **CAP-2** — Mark exact spots on a screenshot and annotate them (FR-8, UC-5, AD-1, AD-3)
  - **intent:** The Reviewer clicks on a screenshot in the Editor to place numbered badges (1, 2, 3...) that correspond directly to lines in the structured note.
  - **success:** Badges appear in normalized coordinates `[0..1]`, draggable and removable, synchronized with single-table `marker` rows in `library.db`. Deleting or reordering markers automatically renumbers badges and note lines without gaps.
- **CAP-3** — Review, edit, and organize findings in the Editor (FR-6, FR-7, FR-9, UC-3, UC-4, UC-6)
  - **intent:** The Reviewer views the chronological list of captured findings, selects items, edits notes in Markdown, inspects dimensions and timestamps, and views real-time reduction status.
  - **success:** The Findings list (Screen 3) and Detail view (Screen 4) render responsive previews, support inline note editing with keyboard shortcuts, display placeholders while images are encoding, and update instantaneously.
- **CAP-5** — Clean deletion and orphan reporting (FR-13, FR-15, UC-7, UC-8, AD-2, NFR-5)
  - **intent:** The Reviewer deletes unwanted findings individually or in bulk, with guaranteed file and database synchronization, and inspects orphan report scans.
  - **success:** Deleting a finding removes image files from disk before deleting database records (or rolls back on error). The Orphan Sweeper (Screen 7) detects disk/db discrepancies without deleting unconfirmed files.

## Constraints

- **AD-1: Markers and Note lines are one sequence.** `marker` table rows hold both badge positions and line annotations; `ordinal` is both badge number and line number.
- **AD-2: Record and file lifetimes are coupled.** Hard deletion removes the vault image file before deleting the database row.
- **AD-3: Marker coordinates are normalised.** Positions are stored as `0.0..=1.0` relative to image dimensions.
- **AD-4: Quality Budget reduction happens once at capture.** Stored images are final; no on-the-fly recompression in the editor.
- **AD-6: No network calls.** Entire capture, edit, and deletion workflow is strictly local.
- **DPI Accuracy (RISK-1):** Multi-monitor captures must query native physical pixels and monitor scale factors correctly.
- **Zero-byte reservation (NFR-2):** Database record and vault file handle reservation occur synchronously; image compression and final write complete asynchronously.
- **Stack compliance:** Rust 1.96, Tauri v2, React 19, Vite 7, TypeScript 5, SQLite (`rusqlite`).

## Non-goals

- **Bundle composition & export.** Handled in Wave W3.
- **External agent handoff & MCP server.** Handled in Wave W4.
- **Web publishing & cloud sharing.** Handled in Wave W5.
- **Image editing beyond spatial markers.** No drawing pens, blur brushes, or shapes.

## Verification Suite

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix web/ui run typecheck
npm --prefix web/ui run lint
npm --prefix web/ui run test
npm --prefix apps/desktop run typecheck
npm --prefix apps/desktop run lint
npm --prefix apps/desktop run test
npm --prefix apps/desktop run build
uv run .constitution/method/scripts/validate.py --check
```
