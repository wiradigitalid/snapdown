# W8-S4 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W8**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w8-capture-becomes-real/`
- `story_id`: `W8-S4`

`W8-S1`, `W8-S2` and `W8-S3` have landed. Capture, reduction and the marker burn are all real now.
This story is the one that makes the captured image mean something.

## The defect — half of a use case that was never built

`apps/desktop/src/components/CaptureOverlay.tsx:76-100` measures the region on mouse-up and calls
`captureScreenRegion` **immediately**:

```tsx
const handleMouseUp = async () => {
  ...
  const res = await captureScreenRegion({ x, y, width, height });
  if (onCaptureComplete) onCaptureComplete(res);
};
```

A grep for `note` across that file returns **zero hits**. And it could not pass one if it wanted to —
`CaptureRegionInput` at `apps/desktop/src-tauri/src/commands/capture.rs:16-22` has five fields and
none of them is a note.

The Rust side then hardcodes the absence, at `capture.rs:130-135`:

```rust
let note = Note {
    id: format!("note-{finding_id}"),
    finding_id: finding_id.clone(),
    body: String::new(),      // <-- always empty, by construction
    updated_at: captured_at,
};
```

**Every Finding Snapdown has ever stored has an empty Note.**

## Why this one matters more than a missing input usually would

`UC-1` reads: *"I press a key, box the thing that is wrong, **and say what is wrong with it**."*
This story is that last clause. `FR-2` — *"Write the Note at capture time"* — is entirely unmet.

`BG-1`, the goal the whole product is built on, is that **a note is unambiguously attached to the
image it describes**. A Finding with no Note is an image with nothing said about it — the exact thing
Snapdown exists to prevent.

`LC-029` `capture-note-field` is registered in `components.yaml` as `ui-composite`, `container:
desktop-app`, `component: finding`, `area: capture-pipeline`, `depends_on: [LC-001]` — with no
implementation. `inventory.py` has been reporting it UNREAD.

`W6` saw this and deferred it correctly as a `known_gap` — *"the capture path is not in this wave's
scope"*. It is this wave's scope.

## The design is already written — you are executing it, not inventing it

`.how/finding/01-ux/DESIGN.md` § *Capture Overlay (`LC-001`) and note field (`LC-029`)* specifies it:

```
        ╔═════════════════════════════════╗
  dim   ║   the selected region, sharp    ║   dim
        ╚═════════════════════════════════╝
              1408 × 620          ← readout, --font-mono
        ┌─────────────────────────────────┐
        │ What is wrong here?             │  ← LC-029, anchored beneath
        └─────────────────────────────────┘
              Enter to save · Esc to cancel
```

Four things it binds, and the second is the one most likely to be got wrong:

1. **The note field anchors beneath the region**, and **flips above it when the region is near the
   screen foot**. It is not fixed to a screen corner, and it never covers the thing being described.
2. **The readout sits outside the region.** Same reason.
3. The hint line is `--text-xs`, `--color-text-muted`, and is **the only instruction anywhere in the
   capture path**.
4. The state table gains a **Narrating** state — *"Region stays lit, note field focused"* — between
   Dragging and Saving. Read that table; it is five rows and it is the state machine for this story.

**Colour lives in exactly one file** — `web/ui/src/styles/tokens.css`. A lint rule refuses a colour
literal anywhere else, and it will fail your build rather than warn.

## Esc, and what it must not do

**Esc cancels the WHOLE capture and leaves NO Finding.** It MUST NOT save an empty one. A Finding
with an empty Note is precisely the state this story exists to eliminate, and reintroducing it
through the cancel path would be the same defect wearing a different hat.

`waves.yaml` names a test for exactly this, and it is the one to write first.

## What must survive

- **`BR-31` — a region smaller than 8×8 is refused.** Already enforced at `CaptureOverlay.tsx:85-89`
  and in `capture.rs`. The new state between drag and capture MUST NOT let a sub-8×8 region through
  by taking a different path to the command.
- **`snapdown-core` stays free of IO.** There is a test, `snapdown_core_has_no_io_dependency`.
- **`Note` already has the field you need.** `crates/snapdown-core/src/domain/finding.rs:44-50`
  carries `body: String`. You are filling it, not changing the domain type.

## The tests

`waves.yaml` records four, carried through verbatim:

```
vitest::the_overlay_asks_for_a_note_before_it_writes_a_finding
vitest::enter_saves_the_note_with_the_finding
vitest::esc_cancels_the_capture_and_writes_no_finding
cargo::a_capture_carries_its_note_through_to_the_stored_finding
```

The last one is the seam: `tauri::test::mock_app` yields `STATUS_ENTRYPOINT_NOT_FOUND` on this
platform, so the Tauri commands here are split into an `_impl(&AppState)` inner function that a test
can call directly. Follow that existing pattern rather than inventing a new one.

**Mutation is the acceptance criterion.** Break each behaviour, watch the test go red, put it back.
Use `--no-fail-fast` when you do: cargo stops at the first failing binary otherwise, later tests
never run, and a live test reads as dead. That produced a false result in `W8-S2`.

## Before you close, grep

**`grep -rn "<CaptureNoteField" apps/desktop/src web/ui/src`**, excluding the component's own file
and its tests.

This repository has shipped **four** components that were built, unit-tested and mounted nowhere —
`CaptureOverlay`, `MarkerLayer`, `OrphanReportView`, `EmptyState` — leaving three requirements unmet
across four waves while every test passed. There is still no composition test class (`OQ-23`), and
`V12` will not catch it: it checks that an `LC` is *registered*, not that it is *reached*.

A green unit test does not mean the component is reachable.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** Unknown cause → `wdi-systematic-debugging` first; a
  third failed attempt is an escalation.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code, and read the four ways a
  verification run lies recorded there.
- **Write UTF-8, no BOM, and watch for a lone cp1252 byte.** Three story files in this wave have
  arrived with one or the other.
- **No scratch files in the commit. Do not push.**

## Done means

`_bmad-output/specs/w8-capture-becomes-real/stories/W8-S4-*.md` exists, carries an
`<intent-contract>`, and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
