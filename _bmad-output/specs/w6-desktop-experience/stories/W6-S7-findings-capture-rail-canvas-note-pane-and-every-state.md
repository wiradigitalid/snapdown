---
id: W6-S7
title: 'W6-S7: Findings — capture rail, canvas, note pane, and every state'
type: 'feature'
wave: W6
status: done
created: '2026-08-23'
dependencies:
  - W6-S1
  - W6-S2
files:
  - web/ui/src/components/FindingsEditor.tsx
  - web/ui/src/components/MarkerLayer.tsx
  - web/ui/src/components/MarkerBadge.tsx
  - web/ui/src/index.ts
  - apps/desktop/src/components/FindingsView.tsx
  - apps/desktop/src/components/OrphanReportView.tsx
  - apps/desktop/src/services/finding.ts
  - apps/desktop/src/App.tsx
  - web/ui/src/test/findings_editor.test.tsx
  - web/ui/src/test/marker_layer.test.tsx
  - apps/desktop/src/test/findings_view.test.tsx
  - apps/desktop/src/test/orphan_report.test.tsx
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S7-step1-plan.md
  - .how/finding/01-ux/DESIGN.md
  - .what/finding/04-usecases/EXPERIENCE.md
  - .what/finding/05-scenarios/SCN-04-the-note-line-deleted-without-its-marker.md
  - .what/finding/03-domain/state-machines.md
  - .control/registry/defects.yaml
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/design-system.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
The desktop application currently suffers from two severe defects and a broken findings surface layout:
1. **`BUG-5` (Critical): The Editor never renders a Finding's image.** `MarkerLayer` is exported from `web/ui/src/index.ts` but mounted nowhere in the application. `FindingsEditor.tsx` renders only metadata, a Note textarea, and `{f.markers.length} markers` as static text. `FindingsView.tsx` contains no `<img>`, no `convertFileSrc`, and no canvas. As a consequence, the screenshot the Note describes is not on screen, Markers cannot be placed, moved, or deleted (`FR-8`, `UC-5`), and `AD-1` (*Markers and Note lines are one sequence*) has no user interface. The core value promise `BG-1` (*a note is unambiguously attached to the image it describes*) is invisible to the Reviewer. The defect went undetected because `MarkerLayer` had passing isolated unit tests without any composition tests verifying mounting in the Findings view.
2. **`BUG-6` (High): The orphan report is unreachable.** `OrphanReportView.tsx` exists and has passing unit tests, but is mounted nowhere in the application and has no route, tab, or entry point. `FR-15` and `UC-8` are unreachable, and half of `NFR-5`'s enforcement mechanism cannot be opened.
3. **Findings Surface Layout & Experience Deficiencies:**
   - The surface does not implement the three-region layout specified in `.how/finding/01-ux/DESIGN.md` (200px capture rail, flexible canvas, 320px note pane with marker list beneath the note).
   - Panels use fixed heights leaving roughly a third of the window dark beneath them instead of taking 100% available viewport height.
   - Missing keyboard accessibility path for markers: without a marker list in the note pane, markers cannot be operated from a keyboard (`EXPERIENCE.md`).
   - Missing handling for `SCN-04` (*The note line deleted without its marker*): deleting a line from the free-text note must leave the marker in place, renumber nothing, report the unbound marker in the note pane, and NEVER annotate an app-only state onto the exported image.
   - Incomplete state implementations: `empty` (should collapse to centered `EmptyState` teaching the bound capture `HotkeyChip`), `loading` (skeleton rail thumbnails), `nothing-selected` (distinct prompt on canvas), `image-missing` (warning panel linking to orphan report `BUG-6`), and `error` (`ErrorState` with retry).

**Approach:**
1. **Fix BUG-5 (Critical - Image Rendering & Marker Canvas Mount):**
   - In `apps/desktop/src/components/FindingsView.tsx` and `web/ui/src/components/FindingsEditor.tsx`, render the Finding's screenshot using Tauri's `convertFileSrc` over the resolved Vault image path.
   - Mount `<MarkerLayer />` directly over the image in the canvas area, enabling click-to-place (normalized `[0.0, 1.0]` coordinates), drag-reposition, and marker selection.
   - Display image dimensions and stored file size beneath the canvas (e.g. `1408 × 620 px · 184 KB`).
   - Implement the repository's first composition tests in `apps/desktop/src/test/findings_view.test.tsx` and `web/ui/src/test/findings_editor.test.tsx` asserting the `<img>` element and `MarkerLayer` canvas are present in the mounted DOM, and clicking the canvas triggers marker placement and note synchronization.
2. **Fix BUG-6 (High - Reachable Orphan Report):**
   - Provide an entry point to `OrphanReportView` (`LC-030`) from the Findings surface's `image-missing` state as specified in `.how/finding/01-ux/DESIGN.md` (and optionally a header action).
   - Allow seamless navigation to and from the Orphan Report.
   - Add automated tests verifying that clicking the action in `image-missing` state mounts `OrphanReportView`.
3. **Rebuild Findings Surface (`LC-006` / `LC-007`):**
   - **Capture Rail (200px):** Vertical scroll of capture thumbnails (`--rail-thumb-width: 176px`), newest first, timestamp beneath. Checkboxes appear on hover or focus for multi-select (`FR-9`). Pinned rail footer displays selection count and `Compose ->` button bridging to Bundle creation (`FR-9`).
   - **Canvas (Flex Growing Center):** Centered image fitted to pane with transparent overlay `<MarkerLayer />`, dimension readout, and stored byte size beneath.
   - **Note Pane (320px):** Note `TextArea` with auto-save / debounced persistence (`FR-7`), plus a dedicated Marker list beneath the Note.
     - Marker list items are focusable, display badge numbers (`1`, `2`, ...), position/comment, and delete action. Provides the keyboard accessibility floor (`EXPERIENCE.md`).
     - **SCN-04 Asymmetry Implementation:**
       - Deleting a numbered line from Note free-text: Marker stays on canvas at its position and number; Note retains text; Note pane highlights marker as unbound ("Unbound / No note line"); Image canvas renders normal marker without app-only annotations; Bundle composition remains allowed.
       - Deleting a Marker from image/list: Marker is deleted, matching Note line is removed, remaining markers and note lines renumber contiguously (`1, 2, ...`).
     - Delete Finding button at bottom of note pane with single confirmation dialog (`BR-5`, `BR-6`, `FR-13`).
   - **State Matrix Implementation:**
     - `empty`: 3 columns collapse to one centered `EmptyState` ("No findings yet") displaying the bound capture combination rendered as a `HotkeyChip` (not a button).
     - `loading`: Rail displays skeleton thumbnail placeholders; canvas and note pane maintain frame structure without layout shift.
     - `nothing-selected`: Rail populated, canvas displays muted prompt ("Select a finding"), note pane empty and inert.
     - `populated`: Full 3-column layout active with loaded finding details.
     - `image-missing`: Canvas displays `--color-warning-bg` warning panel naming the missing file and providing "Open Orphan Report" action (`BUG-6`).
     - `error`: Centered `ErrorState` ("The Library could not be read") with Retry button.
   - **Styling & Accessibility:** Full height flex layout (100% height), strict token consumption (`tokens.css`, zero color literals, `AD-10`), WCAG 2.2 AA keyboard navigation, and theme-invariant marker badges (`--color-marker`, `--color-marker-text`, `--color-marker-ring`).

## Boundaries & Constraints

**Always:**
- The Finding's image MUST be rendered in the canvas using Tauri `convertFileSrc` over the resolved Vault path (`BUG-5`, `FR-6`).
- `<MarkerLayer />` MUST be mounted directly over the rendered image in the canvas (`BUG-5`, `FR-8`, `LC-007`).
- Automated composition tests MUST assert the presence of `<img>` and `[data-testid="marker-layer"]` in the mounted Findings view (`BUG-5`).
- The Orphan Report (`LC-030`) MUST be reachable from the Findings surface's `image-missing` state (`BUG-6`, `FR-15`, `UC-8`).
- Marker coordinate math MUST use normalized floats in `[0.0, 1.0]` relative to the image bounds (`AD-3`).
- Implement the `SCN-04` asymmetry strictly:
  - Deleting a Note line leaves its Marker in place and numbered; the note pane reports it as unbound.
  - Deleting a Marker removes its Note line and renumbers remaining markers contiguously.
- An unbound marker MUST NEVER be annotated or flagged on the image itself (`SCN-04`).
- The note pane Marker list MUST be fully keyboard-navigable (Tab / Arrow keys, Enter/Space/Delete actions) to fulfill the accessibility floor (`EXPERIENCE.md`).
- Panels MUST take 100% of available height (`height: 100%`) without fixed-height cutoffs.
- All styles MUST use design tokens from `tokens.css` (`AD-10`, `NFR-16`, `NFR-17`); zero hex/rgb color literals permitted outside `tokens.css`.
- Marker badge styling MUST remain theme-invariant (`--color-marker: #f59e0b`, `--color-marker-text: #000000`, `--color-marker-ring: #ffffff`).

**Block If:**
- Image loading or blob path resolution escapes the Vault root or violates security confinement (`AD-2`, `LC-005`).
- Any race condition between note text editing and marker sequence operations corrupts marker ordinals or note contents.

**Never:**
- Do not introduce inline color literals in TSX or CSS files (`AD-10`).
- Do not silently delete markers when a Reviewer deletes text lines in the Note (`SCN-04`).
- Do not burn app-only warning indicators or transient editing states into the rendered/exported image (`SCN-04`).
- Do not add graphic annotation tools (arrows, callouts, highlights, blur) outside numbered markers (Non-Goal, `SPEC.md`).
- Do not commit captured screenshots or test fixture binaries (`BUG-7`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Initial Load (Empty Library) | `findings.length === 0`, `isLoading === false` | 3 columns collapse to centered `EmptyState` ("No findings yet"), bound capture shortcut rendered via `HotkeyChip` | Recovery on new capture event or reload |
| Initial Load (Populated Library) | `findings.length > 0`, default selection | First finding selected; 200px rail displays thumbnails with timestamps; canvas displays image with `MarkerLayer`; 320px note pane displays Note and Marker list | Graceful fallback if selection invalid |
| Loading State | `isLoading === true` | Rail renders skeleton thumbnail placeholders; canvas and note pane retain structural frame with zero layout shift | Transition to populated/empty when resolved |
| No Finding Selected | Findings list populated, `selectedFindingId === null` | Rail active; canvas shows muted "Select a finding"; note pane inert and empty | Selecting any thumbnail restores detail |
| Image Render & Marker Overlay | Finding with valid image path selected | `<img>` loaded via `convertFileSrc`; `<MarkerLayer />` positioned over image; dimension readout (e.g. `1408 × 620 px · 184 KB`) displayed beneath | Image error triggers `image-missing` state |
| Marker Placement | Click on canvas at `(clientX, clientY)` | Computes normalized coordinates `(x, y)` in `[0, 1]`; adds Marker; appends numbered line `N. <comment>` to Note; updates Marker list in note pane (`AD-1`, `UC-5`) | Rejects clicks outside image boundary |
| Marker Drag Reposition | Pointer drag on existing marker badge | Marker moves with pointer (80% opacity); on release, updates `(x, y)` via IPC `update_marker`; Note line preserved | Keeps marker within `[0.0, 1.0]` bounds |
| Marker Deletion via List or Image | Click delete on Marker `K` in note pane or canvas | Marker `K` removed; line `K` removed from Note; remaining markers `> K` renumber contiguously to `K, K+1, ...` (`SCN-04` reverse) | IPC `delete_marker` updates store |
| Note Line Deleted by User | User deletes line `2` in free-text Note textarea | Marker 2 remains on canvas at position `(x, y)` with badge `2`; remaining note lines untouched; Note pane displays Marker 2 as "Unbound / No note line"; image canvas renders normal badge (`SCN-04`) | SCN-04 compliance; bundle composition still allowed |
| Image File Missing on Disk | Database record exists, image blob missing in Vault | Canvas replaces image with `--color-warning-bg` panel stating "Image file missing: <path>"; renders "Open Orphan Report" button (`BUG-6`) | Clicking button navigates to `OrphanReportView` |
| Orphan Report Navigation | User clicks "Open Orphan Report" | Surface switches to `OrphanReportView` (`LC-030`); displays missing/orphan file scan; provides back button to return to Findings | Scan/clean operations execute cleanly |
| Multi-Select Compose Bridge | User checks 2 findings in capture rail | Rail footer displays "2 selected" and active "Compose ->" button; clicking invokes compose workflow / switches to Bundle composer (`FR-9`) | Footer hidden when 0 items selected |
| Finding Deletion | User clicks "Delete Finding" in note pane | Displays single confirmation dialog stating finding ID and file removal; on confirm, deletes blob and row via `deleteFinding` (`BR-5`, `BR-6`, `FR-13`) | Error toast if file delete fails |
| Library Store Failure | IPC `list_findings` rejects with error | Surface renders centered `ErrorState` ("The Library could not be read") with "Retry" action button | Retries fetch on click |

</intent-contract>

## Code Map

- `web/ui/src/components/FindingsEditor.tsx` -- Composite Findings component (`LC-006`); renders 3-column layout (200px capture rail, flex canvas with `<MarkerLayer />`, 320px note pane with Note `TextArea` and keyboard-accessible Marker list), manages all 6 states (`empty`, `loading`, `nothing-selected`, `populated`, `image-missing`, `error`), enforces `SCN-04` asymmetry, and wires multi-select compose bridge (`FR-9`).
- `web/ui/src/components/MarkerLayer.tsx` -- Marker canvas overlay (`LC-007`); maps normalized coordinates `[0.0, 1.0]`, handles click placement, pointer drag repositioning, badge selection, and keyboard focus states.
- `web/ui/src/components/MarkerBadge.tsx` -- Theme-invariant numbered marker badge primitive (`--color-marker`, `--color-marker-text`, `--color-marker-ring`).
- `web/ui/src/index.ts` -- Barrel export for shared UI components, types, and props.
- `apps/desktop/src/components/FindingsView.tsx` -- Desktop container for Findings surface; connects to Tauri IPC (`listFindings`, `saveNote`, `deleteFinding`, `addMarker`, `updateMarker`, `deleteMarker`, `getSettings`), resolves Vault image paths with `convertFileSrc`, manages selection and view mode (Findings vs Orphan Report).
- `apps/desktop/src/components/OrphanReportView.tsx` -- Orphan files report (`LC-030`); scans and cleans unreferenced vault files or missing finding references (`BUG-6`, `FR-15`, `UC-8`), includes back navigation to Findings.
- `apps/desktop/src/services/finding.ts` -- Tauri IPC service wrapper for finding and orphan operations.
- `apps/desktop/src/App.tsx` -- Main application container integrating `FindingsView` and navigation shell.
- `web/ui/src/test/findings_editor.test.tsx` -- Unit and interaction tests for `FindingsEditor`: 3-column layout, state transitions, marker list keyboard navigation, SCN-04 asymmetry, and multi-select footer.
- `web/ui/src/test/marker_layer.test.tsx` -- Tests for coordinate normalization, marker placement, dragging, and keyboard interaction on `MarkerLayer`.
- `apps/desktop/src/test/findings_view.test.tsx` -- Composition tests for `FindingsView`: asserts `<img>` and `MarkerLayer` canvas presence in DOM (`BUG-5`), image path resolution with `convertFileSrc`, marker add/delete/drag IPC integration, and orphan report transition (`BUG-6`).
- `apps/desktop/src/test/orphan_report.test.tsx` -- Tests for `OrphanReportView` scanning, discrepancy display, and file cleanup.

## Tasks & Acceptance

**Execution:**
- [ ] `web/ui/src/components/MarkerLayer.tsx` -- Update `MarkerLayer` to support image overlay positioning, normalized `[0, 1]` coordinate mapping, marker selection, drag repositioning, and keyboard focus handling.
- [ ] `web/ui/src/components/FindingsEditor.tsx` -- Rebuild `FindingsEditor` as a 3-column responsive layout taking 100% viewport height:
  - **Capture Rail (200px):** Thumbnails (`--rail-thumb-width: 176px`), timestamps, hover/focus multi-select checkboxes, and pinned footer with selection count and "Compose ->" button (`FR-9`).
  - **Canvas (Flex):** Centered image container with `convertFileSrc` source, mounted `<MarkerLayer />` overlay, dimensions and stored size readout beneath (`BUG-5`).
  - **Note Pane (320px):** Note `TextArea` with auto-save / debounced persistence, focusable Marker list beneath Note with badge number, coordinates/comment, delete button, and unbound status reporting (`SCN-04`).
  - **SCN-04 Asymmetry:** Deleting a Note line leaves Marker intact and flags row as unbound in pane without annotating canvas image; deleting a Marker removes line and renumbers remaining contiguously.
  - **All 6 States:** `empty` (centered `EmptyState` with `HotkeyChip`), `loading` (skeleton placeholders), `nothing-selected` (canvas selection prompt), `populated`, `image-missing` (warning panel with "Open Orphan Report" action, `BUG-6`), `error` (centered `ErrorState` with retry).
- [ ] `apps/desktop/src/components/FindingsView.tsx` -- Refactor `FindingsView` container:
  - Fetch settings to resolve `vault_path`, convert image paths with Tauri `convertFileSrc`.
  - Connect marker CRUD handlers (`addMarker`, `updateMarker`, `deleteMarker`) and note auto-save.
  - Handle view switching between Findings and `OrphanReportView` when triggered from image-missing state or header action (`BUG-6`).
- [ ] `apps/desktop/src/components/OrphanReportView.tsx` -- Add back button / navigation header to return to Findings view when opened from Findings surface.
- [ ] `web/ui/src/test/findings_editor.test.tsx` -- Implement unit tests for `FindingsEditor` testing all 6 states, 3-column layout, marker list keyboard accessibility, and SCN-04 unbound marker display.
- [ ] `web/ui/src/test/marker_layer.test.tsx` -- Add tests for marker placement, drag repositioning, and coordinate bounds clamping.
- [ ] `apps/desktop/src/test/findings_view.test.tsx` -- Implement composition tests:
  - Assert `<img>` element is present and receives converted file source (`BUG-5`).
  - Assert `<MarkerLayer />` canvas is mounted over the image (`BUG-5`).
  - Assert clicking canvas invokes `addMarker` IPC and updates note lines.
  - Assert deleting marker invokes `deleteMarker` IPC and renumbers remaining markers.
  - Assert image-missing state renders warning banner and "Open Orphan Report" button.
  - Assert clicking orphan report button transitions view to `OrphanReportView` (`BUG-6`).
- [ ] `apps/desktop/src/test/orphan_report.test.tsx` -- Verify orphan scanning, cleanup, and return navigation.

**Acceptance Criteria:**
- Given a selected Finding with an image, when `FindingsView` is rendered, the `<img>` element is present in the DOM with a valid `convertFileSrc` source and `<MarkerLayer />` is mounted directly over it (`BUG-5`, `FR-6`, `FR-8`).
- Given the mounted Findings surface, clicking on the image canvas places a new numbered Marker in normalized coordinates `[0, 1]` and adds the corresponding numbered line to the Note (`AD-1`, `UC-5`).
- Given a Finding with 3 markers, when the user deletes line 2 in the Note textarea, Marker 2 remains on the image canvas at badge 2, the note pane reports Marker 2 as unbound, no error/warning graphic is rendered on the canvas image, and Bundle composition remains permitted (`SCN-04`).
- Given a Finding with 3 markers, when the user deletes Marker 2 from the image or note pane list, Marker 2 is removed, Note line 2 is removed, and Marker 3 renumbers to 2 with its note line renumbering contiguously (`SCN-04` reverse, `AD-1`).
- Given a Finding whose image file is missing on disk, the canvas renders a warning panel naming the missing file with an "Open Orphan Report" action, and clicking it mounts `OrphanReportView` (`BUG-6`, `FR-15`, `LC-030`).
- Given the Findings surface, the layout consists of 3 columns (200px capture rail, flexible canvas, 320px note pane) spanning 100% available viewport height without dark gaps.
- Given an empty library, the view renders a centered `EmptyState` displaying the bound capture combination as a `HotkeyChip` (not a button).
- Given the note pane, the Marker list beneath the Note is keyboard-navigable, with each marker row focusable and actionable via keyboard.
- Given zero findings selected, multi-select footer is hidden; when 1 or more findings are selected via rail checkboxes, footer displays count and active "Compose ->" button (`FR-9`).
- Given automated verification, all Vitest test suites (including new composition tests in `findings_view.test.tsx`), TypeScript typechecks, and ESLint checks pass with zero errors, and zero color literals exist outside `tokens.css`.

## Verification

**Commands:**
- `npm --prefix web/ui run typecheck` -- expected: TypeScript compiles cleanly with zero errors across all components
- `npm --prefix web/ui run lint` -- expected: ESLint passes with zero warnings/errors (no colour literals outside tokens.css)
- `npm --prefix web/ui run test` -- expected: All Vitest suites pass in `web/ui` (including findings_editor, marker_layer, components, tokens, contrast)
- `npm --prefix apps/desktop run typecheck` -- expected: Zero TypeScript errors in desktop app
- `npm --prefix apps/desktop run lint` -- expected: ESLint passes with zero warnings/errors in desktop app
- `npm --prefix apps/desktop run test` -- expected: All Vitest suites pass in `apps/desktop` (including findings_view composition tests, orphan_report, editor_shell, mount, shell)
- `npm --prefix apps/desktop run build` -- expected: Production Vite build succeeds
- `cargo test --workspace` -- expected: All Rust crate unit and integration tests pass

## Spec Change Log

- 2026-08-23: Completed W6-S7 Step 2 BUILD. Implemented 3-column FindingsEditor layout, MarkerLayer image canvas overlay (BUG-5), OrphanReportView navigation entry point (BUG-6), SCN-04 asymmetry handling, and composition test suite.