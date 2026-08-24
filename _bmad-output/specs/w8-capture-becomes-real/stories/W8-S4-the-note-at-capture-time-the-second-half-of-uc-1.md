---
id: W8-S4
title: "W8-S4: The Note at capture time, the second half of UC-1"
type: 'feature'
wave: W8
status: done
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies:
  - W8-S3
files:
  - apps/desktop/src/components/CaptureNoteField.tsx
  - apps/desktop/src/components/CaptureOverlay.tsx
  - apps/desktop/src/services/capture.ts
  - apps/desktop/src/test/capture_overlay.test.tsx
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/commands/capture.rs
context:
  - _bmad-output/specs/w8-capture-becomes-real/SPEC.md
  - _bmad-output/specs/w8-capture-becomes-real/stories.yaml
  - _bmad-output/specs/w8-capture-becomes-real/dispatch-briefs/W8-S4-step1-plan.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .control/registry/components.yaml
  - .what/business-rules.md
  - .what/finding/02-rules/rules-finding.md
  - .what/finding/04-usecases/UC-1-capture-a-region-and-say-what-is-wrong.md
  - .what/finding/SRS-finding.md
  - .how/finding/01-ux/DESIGN.md
  - .how/_platform/inventory-screen.md
  - .constitution/project/codebase-stack-guide.md
  - web/ui/src/styles/tokens.css
warnings:
  - >-
    UC-1 step 5 reads "Snapdown captures exactly that region, and shows a focused note field
    anchored to it" - the pixel grab before the narration. BUG-18's `fix:` and the dispatch brief
    both prescribe the simpler seam instead: add `note` to CaptureRegionInput and invoke the single
    existing `capture_screen_region` command once, on save. This story follows BUG-18. The ordering
    difference is REPORTED, not resolved here, and it is a candidate for `.control/questions/`
    (assumptions class) rather than a corpus edit.
  - >-
    DESIGN.md's hint line says "Enter to save" while BR-34 requires multi-line Note text preserved
    verbatim, blank lines included. Resolved in this story as Enter saves and Shift+Enter inserts a
    newline, on the reading that BR-34 binds preservation and not the input affordance. Flagged so a
    reviewer can disagree cheaply.
  - >-
    `apps/desktop/src/test/capture_overlay.test.tsx` currently asserts that mouse-up calls
    `captureScreenRegion`. That assertion becomes WRONG under this story and must be rewritten, not
    deleted - it is the test that becomes
    `the_overlay_asks_for_a_note_before_it_writes_a_finding`.
  - >-
    `CaptureOverlay.tsx` currently begins with a UTF-8 BOM. Do not add one to the new files, and do
    not reformat the existing BOM away as part of this story unless a lint rule demands it.
deferred:
  - >-
    Whether the screen grab captures the overlay's own scrim over the selected region is a
    pre-existing question about W8-S1's grab path, not something this story introduces. Out of scope.
  - >-
    BR-35 (focus returns to the window that held it) and BR-36 (the transient toast with the running
    count) are not in this story's test list and are not in scope.
---

<intent-contract>

## Intent

**Context and defect rationale (`BUG-18`, `CAP-1`, `LC-029`, `FR-2`, `UC-1`):**

`UC-1` reads *"I press a key, box the thing that is wrong, **and say what is wrong with it**."*
The last clause has never existed.

`apps/desktop/src/components/CaptureOverlay.tsx:76-100` measures the region on mouse-up and calls
`captureScreenRegion` **immediately**:

```tsx
const handleMouseUp = async () => {
  ...
  const res = await captureScreenRegion({ x, y, width, height });
  if (onCaptureComplete) onCaptureComplete(res);
};
```

A grep for `note` across that file returns zero hits. It could not pass one if it wanted to:
`CaptureRegionInput` at `apps/desktop/src-tauri/src/commands/capture.rs:15-22` carries `x`, `y`,
`width`, `height`, `source_monitor` and nothing else. The Rust side then hardcodes the absence at
`capture.rs:130-135`:

```rust
let note = Note {
    id: format!("note-{finding_id}"),
    finding_id: finding_id.clone(),
    body: String::new(),      // always empty, by construction
    updated_at: captured_at,
};
```

**Every Finding Snapdown has ever stored has an empty Note.** `FR-2` — *"Write the Note at capture
time"* — is entirely unmet, and `BG-1`, the goal the product is built on, is that a note is
unambiguously attached to the image it describes.

`LC-029` `capture-note-field` is registered in `components.yaml` as `ui-composite`, `container:
desktop-app`, `component: finding`, `area: capture-pipeline`, `depends_on: [LC-001]`, with no
implementation. `inventory-readers.py:233` expects it at
`apps/desktop/src/components/CaptureNoteField.tsx` and has been reporting it UNREAD. `W6` saw this
and deferred it correctly as a `known_gap` — *"the capture path is not in this wave's scope"*. It is
this wave's scope.

This story builds `LC-029` exactly as `.how/finding/01-ux/DESIGN.md` already specifies, and carries
the typed Note through `CaptureRegionInput` into the stored `Note.body`.

---

### Governing constraints

1. **`BR-32` — Escape at any point before the save discards the Capture.** Nothing reaches the
   Vault and no row is written. Esc from the Narrating state MUST NOT save an empty Finding: a
   Finding with an empty Note reintroduced through the cancel path is the same defect wearing a
   different hat. `captureScreenRegion` MUST NOT be invoked on the Esc path.
2. **`BR-4` — a Note may be empty. A Finding with no words is still a Finding.** This is NOT in
   tension with the point above. Enter with an empty field MUST still save, because `UC-1`'s
   alternate flow says so in words: *"The Reviewer types nothing → the Finding saves with an empty
   Note."* What this story eliminates is the **absence of any way to write one**, not the Reviewer's
   right to write nothing. The note field MUST NOT validate emptiness or refuse a blank save.
3. **`BR-33` — the note field is focused the moment it appears**, and the Capture can be saved
   without touching the mouse.
4. **`BR-34` — multi-line Note text is preserved verbatim, blank lines included.** The field is a
   `textarea`, and the string sent to the command is the raw value with no trimming, no
   normalisation of newlines, and no collapsing of blank lines.
5. **`BR-31` — a region smaller than 8 x 8 pixels is refused; nothing is stored and the overlay
   stays open.** Enforced today at `CaptureOverlay.tsx:85-89` and at `capture.rs:43-45`. The new
   Narrating state MUST NOT become a second route to the command that skips either guard: the
   overlay refuses the drag before it ever enters Narrating, and the Rust guard stays.
6. **`.how/finding/01-ux/DESIGN.md` § Capture Overlay (`LC-001`) and note field (`LC-029`)** is the
   layout, and it is being executed, not invented:
   - The note field **anchors beneath the region**, and **flips above it when the region is near the
     screen foot**. It is never fixed to a screen corner and never covers the thing being described.
   - The **readout sits outside the region**, for the same reason. It already does.
   - The hint line is `--text-xs`, `--color-text-muted`, and reads `Enter to save · Esc to cancel`.
     It is the only instruction anywhere in the capture path.
   - The state table gains **Narrating** — *"Region stays lit, note field focused"* — between
     Dragging and Saving.
7. **`AD-10` — colour lives in exactly one file**, `web/ui/src/styles/tokens.css`. A lint rule
   refuses a colour literal anywhere else and it fails the build rather than warning. No new token
   may be introduced by this story, because that would require editing
   `.how/_platform/design-system.md`, which is corpus. Compose the field from tokens that already
   exist.
8. **`snapdown-core` stays free of IO.** There is a test, `snapdown_core_has_no_io_dependency`.
   `Note` already carries `body: String` at `crates/snapdown-core/src/domain/finding.rs:44-50` —
   this story **fills** that field and MUST NOT change the domain type.
9. **The corpus is not this story's to change.** No edit to `.what/`, `.how/`, or an `applied`
   `DEC-`. The two ordering/affordance tensions above are recorded in `warnings:` and reported, not
   resolved by editing a document.
10. **Never commit a captured screenshot or a fixture derived from one.** The repository is public.
    The cargo test's input PNG is synthesised programmatically in the test.

---

### The state machine

Five states, from DESIGN.md's table. Armed to Dragging to Narrating to Saving, with Error off to the
side.

| State | Trigger in | Rendering | Trigger out |
|---|---|---|---|
| Armed | mount, or a refused drag | full scrim, crosshair | mouse-down (button 0) to Dragging |
| Dragging | mouse-down | region lit with `--color-overlay-ring`; readout tracks the region | mouse-up to Narrating if at least 8x8, else back to Armed with the BR-31 message |
| Narrating | mouse-up on a valid region | region **stays lit**; `CaptureNoteField` mounted, anchored, focused; hint line visible | Enter to Saving; Esc to cancelled, no Finding |
| Saving | Enter | field disabled while the command is in flight | resolve to `onCaptureComplete`; reject to Error |
| Error | command rejected | existing `capture-error-toast` with `--color-danger` | — |

**Narrating is the state that did not exist.** The region must stay lit in it, which means the
selection box render condition can no longer be `drag.isDragging` alone.

---

## Approach

### 1. `apps/desktop/src/components/CaptureNoteField.tsx` — new, `LC-029`

The path is fixed by `inventory-readers.py:233` and `.how/_platform/inventory-screen.md:102`. Do not
put it anywhere else, or `inventory.py` keeps reporting `LC-029` UNREAD.

- Props: `region: { x: number; y: number; width: number; height: number }`, `value: string`,
  `onChange(value: string)`, `onSave()`, `onCancel()`.
- Renders, inside a positioned wrapper anchored to the region:
  - a `textarea` (compose `TextArea` from `@snapdown/ui`, or a plain `textarea` if `autoGrow`'s
    imperative height writes fight the anchoring — either is acceptable, the primitive is
    preferred), `data-testid="capture-note-field"`, `placeholder="What is wrong here?"`,
  - the hint line, `data-testid="capture-note-hint"`, text `Enter to save · Esc to cancel`,
    `fontSize: var(--text-xs)`, `color: var(--color-text-muted)`, `fontFamily: var(--font-ui)`.
- **Anchoring.** `position: absolute`, `left: region.x`, `top: region.y + region.height + GAP`.
  Flip when the field would not fit: if
  `region.y + region.height + FIELD_BLOCK_H > window.innerHeight`, render at
  `top: region.y - FIELD_BLOCK_H - GAP` instead. Expose the chosen side as
  `data-anchor="below" | "above"` so a test can assert the flip without reading computed pixels.
  `GAP` accounts for the existing readout, which sits at `bottom: -24px` of the selection box.
- **Focus (`BR-33`).** `autoFocus` on the textarea plus a `useEffect` that calls `.focus()` on
  mount. `autoFocus` alone is unreliable inside a freshly created Tauri webview.
- **Keys.** `onKeyDown` on the textarea:
  - `Enter` without `shiftKey`: `preventDefault()`, `stopPropagation()`, `onSave()`.
  - `Enter` with `shiftKey`: let it through, so a newline is inserted (`BR-34`).
  - `Escape`: `preventDefault()`, `stopPropagation()`, `onCancel()`.

  `stopPropagation` matters: `CaptureOverlay` also listens for Escape on `window`, and both firing
  would run the cancel path twice.
- **Colour.** Existing tokens only — `--color-surface-raised`, `--color-text`, `--color-border`,
  `--color-text-muted`, `--radius-md`, `--space-*`, `--font-ui`, `--text-xs`. No literals.
- The wrapper stops mouse events reaching the overlay (`onMouseDown` / `onMouseUp`
  `stopPropagation`), or a click into the field restarts a drag.

### 2. `apps/desktop/src/components/CaptureOverlay.tsx` — the Narrating state

- Add `phase: 'armed' | 'dragging' | 'narrating' | 'saving'` and `note: string` to component state,
  plus `pendingRegion: { x, y, width, height } | null` holding the rounded region measured at
  mouse-up.
- `handleMouseUp`: measure, apply the **existing** `BR-31` guard unchanged (message and all), and on
  success set `pendingRegion` and move to `narrating`. **It no longer calls
  `captureScreenRegion`.** That call moves to `handleSave`.
- `handleSave`: guard on `phase === 'narrating' && pendingRegion`, move to `saving`, call
  `captureScreenRegion({ ...pendingRegion, note })`, then `onCaptureComplete(res)`. On rejection set
  `errorMsg` and return to `narrating` so the typed text is not thrown away.
- `handleCancel`: `await dismissOverlay()` inside the existing try/catch, then `onDismiss?.()`. It
  MUST NOT call `captureScreenRegion`. This is the same body the window-level Escape handler already
  has; extract it and call it from both.
- The window-level `keydown` handler keeps handling Escape for the Armed and Dragging states.
- Selection box: render while `phase === 'dragging' || phase === 'narrating' || phase === 'saving'`
  and the box has area, so the region **stays lit** during Narrating. Keep the readout as it is.
- Mount `<CaptureNoteField ... />` when `phase === 'narrating' || phase === 'saving'`, passing
  `pendingRegion`. **This is the mount that makes `LC-029` reachable.**

### 3. `apps/desktop/src/services/capture.ts`

Add `note: string` to `CaptureRegionPayload`. Required, not optional: an optional field is how the
empty Note got here.

### 4. `apps/desktop/src-tauri/src/commands/capture.rs`

- Add `pub note: String` to `CaptureRegionInput`.
- Write it: `body: region.note.clone()` in place of `String::new()` at `capture.rs:133`. No
  trimming, no normalisation (`BR-34`).
- **Split the command for testability.** `tauri::test::mock_app` yields
  `STATUS_ENTRYPOINT_NOT_FOUND` on this platform, and the existing pattern for that is an
  `_impl(&AppState)` inner function a test calls directly — `bundle.rs:18`, `finding.rs:38`,
  `settings.rs:63`, `sharing.rs:172`, `startup.rs:23`. Follow it; do not invent a new seam.

  The one wrinkle is that `RegionCapturer::capture_region` needs a display, so the grab cannot sit
  inside the tested function. Pass the bytes in:

  ```rust
  pub fn capture_screen_region_impl(
      region: &CaptureRegionInput,
      captured_png_bytes: &[u8],
      state: &AppState,
  ) -> Result<CaptureResultDto, String>
  ```

  It holds everything from the `BR-31` guard through `create_finding` and returns the DTO. The
  `#[tauri::command]` wrapper keeps: the `BR-31` guard **before** the grab (so no display work is
  done for a refused region), the `RegionCapturer` call, the `_impl` call, the overlay-window close,
  and the `capture-completed` emit. The `BR-31` guard is present in **both** — the command's copy
  saves the grab, `_impl`'s copy is the one that actually guards the store, and neither entry point
  can reach `create_finding` without passing it.

- `apps/desktop/src-tauri/Cargo.toml`: add `image = { workspace = true }` to `[dev-dependencies]`,
  so the test can synthesise a PNG. It is already a workspace dependency with the `png` feature.

### 5. Tests

Four, named verbatim from `waves.yaml`.

**`apps/desktop/src/test/capture_overlay.test.tsx`** (vitest) — the existing
`capture_overlay_draws_selection_and_dimensions` case asserts mouse-up calls the command. Rewrite it
into the first test below; the drag/readout assertions in it are still correct and should be kept.

1. `the_overlay_asks_for_a_note_before_it_writes_a_finding`
   drag 50,50 to 250,200; assert the selection box and `200 × 150 px` readout as today; on mouse-up
   assert `captureScreenRegion` **has not been called**, and that `capture-note-field` is in the
   document, is the focused element, and that `capture-note-hint` reads
   `Enter to save · Esc to cancel`.
2. `enter_saves_the_note_with_the_finding`
   same drag; type `the CTA is unreadable` into the field; `keyDown` `Enter`; assert
   `captureScreenRegion` called **exactly once** with
   `{ x: 50, y: 50, width: 200, height: 150, note: 'the CTA is unreadable' }` and that
   `onCaptureComplete` fired with the mocked DTO. Add a `Shift+Enter` assertion in the same case:
   the value keeps its newline and the command is still not called (`BR-34`).
3. `esc_cancels_the_capture_and_writes_no_finding`
   same drag into Narrating; type some text; `keyDown` `Escape` on the field; assert
   `dismissOverlay` called once, `onDismiss` called once, and `captureScreenRegion` **never
   called** (`BR-32`). This is the test to write first.

**`apps/desktop/src-tauri/src/commands/capture.rs`** `#[cfg(test)] mod tests` (cargo) —

4. `a_capture_carries_its_note_through_to_the_stored_finding`
   Build an `AppState` from in-memory stores, following `settings.rs:437-464` verbatim. Write a
   `SettingKey::VaultPath` setting pointing at a `tempfile::TempDir` so nothing touches the
   operator's home directory. Synthesise a small valid PNG in memory with `image` (e.g. a 64x48
   `RgbaImage` with a two-colour pattern, encoded to PNG). Call `capture_screen_region_impl` with
   `note: "the CTA is unreadable\n\nsecond line"`. Read back through
   `state.finding_store.list_findings()` and assert the single `FindingDetail`'s `note.body` is that
   exact string, blank line included, and that `note.finding_id == finding.id`.

**Mutation is the acceptance criterion.** For each of the four, break the behaviour, watch it go
red, restore it. Specifically:

- restore `body: String::new()` — test 4 red;
- call `captureScreenRegion` from `handleMouseUp` again — tests 1 and 3 red;
- drop `note` from the payload — test 2 red;
- trim the note body in Rust — test 4 red on the trailing blank line.

Run cargo mutations with `--no-fail-fast`. cargo stops at the first failing binary otherwise, later
tests never run, and a live test reads as dead. That produced a false result in `W8-S2`.

### 6. Before closing — the reachability grep

```
grep -rn "<CaptureNoteField" apps/desktop/src web/ui/src
```

excluding `CaptureNoteField.tsx` itself and its tests. There must be **at least one hit, in
`CaptureOverlay.tsx`**. This repository has shipped four components built, unit-tested and mounted
nowhere — `CaptureOverlay`, `MarkerLayer`, `OrphanReportView`, `EmptyState` — leaving three
requirements unmet across four waves while every test passed. There is still no composition test
class (`OQ-23`), and `V12` checks that an `LC` is *registered*, not that it is *reached*.

---

## Boundaries and Constraints

**Always:**

- Enter a Narrating state between the drag and the command, with the region still lit and the field
  focused (`BR-33`, DESIGN.md state table).
- Anchor the field to the region and flip it above when the region is near the screen foot.
- Send the textarea's raw value through as `note`, untrimmed and unnormalised (`BR-34`).
- Save on Enter even when the field is empty (`BR-4`, `UC-1` alternate flow).
- Keep the `BR-31` guard on both entry paths, with its existing message.
- Use only tokens that already exist in `web/ui/src/styles/tokens.css` (`AD-10`).
- Mount `CaptureNoteField` inside `CaptureOverlay` and prove it with the grep.
- Write UTF-8 without a BOM, and watch for a stray cp1252 byte.

**Block if:**

- A change would require a new colour token, because that means editing
  `.how/_platform/design-system.md`, which is corpus. Report it instead.
- The `Enter` / `Shift+Enter` reading of `BR-34` is rejected in review — that is a `DEC-`, not a
  quiet edit.

**Never:**

- Never call `captureScreenRegion` on the Escape path, and never write a Finding on it (`BR-32`).
- Never refuse or validate an empty Note (`BR-4`).
- Never trim, collapse, or re-encode newlines in the Note body (`BR-34`).
- Never change `Note` in `snapdown-core`, and never pull IO into that crate.
- Never edit `.what/`, `.how/`, or an `applied` `DEC-`.
- Never write a colour literal outside `tokens.css`.
- Never commit a captured screenshot or a fixture derived from one.
- Never make `note` optional in `CaptureRegionPayload` or `CaptureRegionInput`.
- Never leave the sub-8x8 guard reachable-around via the Narrating state.
- Never push, and never leave a scratch file in the commit.

---

## I/O and Edge-Case Matrix

| Scenario | Input / state | Expected behaviour | Invariant / test |
|---|---|---|---|
| Valid drag released | 200x150 region, mouse-up | Narrating: region stays lit, readout stays, field mounted and focused, hint shown. **No command call** | `BR-33`, DESIGN.md state table, `the_overlay_asks_for_a_note_before_it_writes_a_finding` |
| Sub-8x8 drag released | 4x4 region, mouse-up | Refused with `Region must be at least 8x8 pixels`, overlay returns to Armed, no Narrating, no command call | `BR-31` |
| Enter with text | `the CTA is unreadable` | One `captureScreenRegion` call carrying the note; stored `Note.body` equals it exactly | `FR-2`, `enter_saves_the_note_with_the_finding`, `a_capture_carries_its_note_through_to_the_stored_finding` |
| Enter with nothing typed | empty field | Finding saves with an empty Note. No refusal, no validation message | `BR-4`, `UC-1` alternate flow from step 6 |
| Enter with multi-line text | `line one`, blank line, `line three` | Stored verbatim, blank line included, no trimming | `BR-34` |
| Shift+Enter | field focused | Newline inserted; command NOT called | `BR-34` |
| Esc in Narrating | text typed or not | Overlay dismissed, **no** `captureScreenRegion` call, no blob, no row | `BR-32`, `esc_cancels_the_capture_and_writes_no_finding` |
| Esc in Armed or Dragging | no region committed | Same dismissal as today, unchanged | `BR-32` |
| Region near the screen foot | `region.y + height + block > innerHeight` | Field renders **above** the region, `data-anchor="above"`; it never covers the region | DESIGN.md anchoring rule |
| Command rejects | Vault unreachable, grab fails | `capture-error-toast` with `--color-danger`; phase returns to Narrating with the typed text intact | `UC-1` failure flows from steps 4 and 7 |
| `_impl` called with a sub-8x8 region | direct call, bypassing the UI | `Err("Region must be at least 8x8 pixels")`, nothing stored | `BR-31`, no alternate route to the store |

</intent-contract>

## Code Map

- `apps/desktop/src/components/CaptureNoteField.tsx` — **new**, `LC-029`. The anchored, focused
  note field plus the hint line. Path is fixed by `inventory-readers.py:233`.
- `apps/desktop/src/components/CaptureOverlay.tsx` — `LC-001`. Gains the `narrating` phase, holds
  `pendingRegion` and `note`, keeps the region lit through Narrating, mounts `CaptureNoteField`,
  and moves the `captureScreenRegion` call from `handleMouseUp` to `handleSave`.
- `apps/desktop/src/services/capture.ts` — `note: string` added to `CaptureRegionPayload`.
- `apps/desktop/src/test/capture_overlay.test.tsx` — the three vitest cases; the existing
  mouse-up-captures assertion is rewritten, not deleted.
- `apps/desktop/src-tauri/src/commands/capture.rs` — `note` on `CaptureRegionInput`,
  `body: region.note.clone()`, the `capture_screen_region_impl` split, and the cargo test.
- `apps/desktop/src-tauri/Cargo.toml` — `image = { workspace = true }` under `[dev-dependencies]`.

## Tasks and Acceptance

**Execution, in this order:**

1. Write `esc_cancels_the_capture_and_writes_no_finding` first and watch it fail against today's
   code. It is the test the whole story exists to make true.
2. `apps/desktop/src/components/CaptureNoteField.tsx` — build `LC-029` per DESIGN.md: anchored,
   flipping, focused, Enter / Shift+Enter / Esc, hint line, tokens only.
3. `apps/desktop/src/components/CaptureOverlay.tsx` — add the phase machine, keep the region lit in
   Narrating, mount `CaptureNoteField`, move the command call to `handleSave`, extract the shared
   cancel path.
4. `apps/desktop/src/services/capture.ts` — add required `note: string`.
5. `apps/desktop/src-tauri/src/commands/capture.rs` — add `note` to `CaptureRegionInput`, write it
   into `Note.body`, split out `capture_screen_region_impl`, keep `BR-31` on both paths.
6. `apps/desktop/src-tauri/Cargo.toml` — `image` dev-dependency.
7. Write the remaining three tests; run each one red, then green.
8. Mutation-verify all four, `--no-fail-fast` on the cargo side.
9. Run the reachability grep for `<CaptureNoteField`.
10. Run every command in `AGENTS.md` § Code and read the four ways a verification run lies before
    trusting any of them.

**Acceptance criteria:**

- `handleMouseUp` no longer calls `captureScreenRegion`; a grep for `captureScreenRegion` in
  `CaptureOverlay.tsx` finds it only on the save path.
- `grep -rn "note" apps/desktop/src/components/CaptureOverlay.tsx` returns hits. It returns zero
  today.
- `CaptureRegionInput` carries `note: String`, and `Note.body` is written from it with no
  transformation.
- All four `waves.yaml` tests exist under their exact names and pass:
  `vitest::the_overlay_asks_for_a_note_before_it_writes_a_finding`,
  `vitest::enter_saves_the_note_with_the_finding`,
  `vitest::esc_cancels_the_capture_and_writes_no_finding`,
  `cargo::a_capture_carries_its_note_through_to_the_stored_finding`.
- Each of the four has been seen red under a mutation and green with it restored.
- `grep -rn "<CaptureNoteField" apps/desktop/src web/ui/src` hits `CaptureOverlay.tsx`.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace` are clean.
- `npm --prefix apps/desktop run typecheck`, `lint`, `test` and `build` are clean, as are the three
  `web/ui` scripts.
- No colour literal added outside `tokens.css`; no new token; no corpus file touched.
- No new file carries a BOM.

## Spec Change Log

### 2026-08-24 — Initial story specification (Step 1: plan only)

- Created the `W8-S4` specification against `BUG-18`, `CAP-1`, `FR-2`, `UC-1` and `LC-029`.
- Read the design rather than inventing one: DESIGN.md § *Capture Overlay (`LC-001`) and note field
  (`LC-029`)* supplies the anchoring, the flip, the readout placement, the hint line, and the
  five-row state table whose **Narrating** row is the state this story adds.
- Bound the plan to `BR-31`, `BR-32`, `BR-33`, `BR-34` and — the one most likely to be got
  backwards — `BR-4`, *a Note may be empty*. Enter on an empty field still saves; only Esc leaves no
  Finding.
- Fixed `LC-029`'s path at `apps/desktop/src/components/CaptureNoteField.tsx`, because
  `inventory-readers.py:233` and `inventory-screen.md:102` both name it and `inventory.py` reports
  UNREAD against that path.
- Chose the `capture_screen_region_impl(&CaptureRegionInput, &[u8], &AppState)` seam, with the grab
  left outside it, over `tauri::test::mock_app`, which yields `STATUS_ENTRYPOINT_NOT_FOUND` here.
- Recorded three tensions in `warnings:` rather than resolving them by editing the corpus: `UC-1`
  step 5's grab-before-narration ordering, the `Enter to save` / `BR-34` multi-line affordance, and
  the existing vitest case whose mouse-up assertion this story makes wrong.

## Design Notes

**Why the field is a textarea and Enter still saves.** DESIGN.md's hint line says `Enter to save`
and `BR-34` requires multi-line text preserved verbatim, blank lines included. Those only conflict
if `BR-34` is read as binding the input affordance; read as binding **preservation** — which is what
its wording and its `FR-7` link say — Enter-saves with Shift+Enter-newlines satisfies both, and it
is the affordance every chat and commit-message field in the world has trained the Reviewer on. It
is called out in `warnings:` so a reviewer can reject it cheaply; rejecting it is a `DEC-`, not an
edit.

**Why `note` is required rather than optional.** An optional field is exactly how the empty Note
survived five waves: every layer had a plausible default. Making it required in TypeScript and
`String` in Rust means a caller that forgets it does not compile.

**Why the `BR-31` guard is duplicated.** The command's copy exists so a refused region never pays
for a screen grab. `_impl`'s copy exists because `_impl` is now a second entry point to the store,
and the brief is explicit that the new state must not let a sub-8x8 region through by taking a
different path to the command. Two cheap guards on two entry points, not one guard trusted twice.

**Why the region must stay lit in Narrating.** DESIGN.md says so, and it is not decoration: the
Reviewer is typing about the thing in the box, and losing the box while describing it is the
usability failure the anchoring rule exists to prevent. It means the selection-box render condition
can no longer be `drag.isDragging`.

**What is not being fixed here.** Whether the grab picks up the overlay's own scrim over the
selected region belongs to `W8-S1`'s grab path, not to this story. `BR-35` (focus returns to the
prior window) and `BR-36` (the toast with the running count) are absent from this story's test list
and stay out.

## Verification

**Commands — all of them, from the repo root, per `AGENTS.md` § Code:**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix web/ui run typecheck && npm --prefix web/ui run lint && npm --prefix web/ui run test
npm --prefix apps/desktop run typecheck && npm --prefix apps/desktop run lint
npm --prefix apps/desktop run test && npm --prefix apps/desktop run build
```

**Targeted:**

```bash
cargo test -p snapdown --lib commands::capture --no-fail-fast
npm --prefix apps/desktop run test -- capture_overlay
grep -rn "<CaptureNoteField" apps/desktop/src web/ui/src
grep -rn "note" apps/desktop/src/components/CaptureOverlay.tsx
```

**Read before trusting any result:** the four ways a verification run lies, in `AGENTS.md` § Code.
`cmd | tail` reports `tail`'s exit code. `cmd; echo "EXIT=$?"` makes the harness report 0 whatever
`cmd` did — read the echoed value, never the notification's code. A long-lived worktree goes stale
the moment a story adds a dependency, so `npm --prefix <pkg> ci` before believing a local red. And
`Get-Process -Name Snapdown` before any `tauri build`: a leftover `Snapdown.exe` locks its own file
and fails the build with *Access is denied*, which reads like a permissions problem and is not.
