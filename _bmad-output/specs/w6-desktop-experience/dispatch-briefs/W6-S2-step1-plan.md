# W6-S2 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first — its
`## Code` section carries the verification commands and the pitfalls, and one of those pitfalls is
this story's whole reason for existing.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S2`

Resolve everything else from `{spec_folder}/stories.yaml` and `{spec_folder}/SPEC.md`.

## The story, and its priority order

**W6-S2 — the editor shell: persona naming, the navigation rail, one executable, and the overlay
mount.**

It carries **`BUG-4`, which is critical and outranks the rest of the story.** Plan it first.

### BUG-4 — the capture path does not work

`apps/desktop/src-tauri/src/commands/capture.rs:103-106` opens the overlay window at
`WebviewUrl::App("index.html?overlay=true")`. Nothing in `apps/desktop/src/main.tsx` or `App.tsx`
reads `window.location.search`. `apps/desktop` has exactly one html entry point and
`vite.config.ts` declares no `rollupOptions.input`.

So the overlay window mounts `<App />` — the full Editor shell, opening on Settings — instead of
`<CaptureOverlay />`. **There is no dim, no crosshair, no region drag, no note field, and no Finding
is ever created from the hotkey.** `FR-1`, `FR-2`, `UC-1` and `UC-2` are unmet in the shipped build.

It went unnoticed for four waves because `CaptureOverlay.tsx` has passing unit tests. The component
is correct; it is simply never mounted. **Plan a test that asserts the mount decision from the URL** —
not one that asserts `CaptureOverlay` renders, which is what already exists and already passes.

### Then the shell

`LC-028 editor-shell`, extracted out of `App.tsx` where it is inline JSX owned by no component.

- A left navigation rail, `200px`, replacing the top tab row. Not taste: `FR-29` needs Settings to fit
  1024×720 without scrolling, and a tab row costs ~64px of height on every surface.
- All four primary surfaces listed on every surface (`FR-28`). `sharing` and `agent-access` are frozen
  by `DEC-005` and their surfaces **MUST still be listed** (`BR-120`).
- The active item distinguished by **more than colour** — fill plus a left edge bar. `NFR-16`'s floor
  forbids state carried by colour alone.
- The Capture action pinned to the rail's foot.
- Window title `Snapdown Editor`, distinct from the tray's `Snapdown` (`DEC-003`, `FR-27`), and it
  does not change as the Reviewer moves between surfaces.
- `LC-028` depends on **nothing**. A frame that reads state is a frame that can fail to draw, and
  `FR-28` requires navigation to survive any surface's failure.
- A build assertion that exactly one desktop executable is produced (`BR-121`). This matters: a stale
  `desktop.exe` beside `Snapdown.exe` is what made the owner believe the product had no navigation.

## Documents that bind it

- `_bmad-output/specs/w6-desktop-experience/SPEC.md` and its `companions:` list
- `{spec_folder}/stories.yaml`, entry `W6-S2`
- `.how/settings/04-components/LC-028-editor-shell.md`
- `.how/settings/01-ux/DESIGN.md` § Editor shell
- `.what/settings/04-usecases/EXPERIENCE.md` § Information architecture
- `.control/registry/defects.yaml`, `BUG-4`
- `.how/_platform/ARCHITECTURE-SPINE.md` — `AD-10` and `AD-11`

W6-S1 has landed: every colour is a token, the lint rule refuses a literal, and `SegmentedControl`,
`HotkeyChip`, `EmptyState`, `ErrorState`, `Badge` and a three-state `Toggle` exist in `@snapdown/ui`.
Use them; do not invent new ones.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Commands are in `AGENTS.md` § Code.
- **Never commit a captured screenshot or an accessibility-tree dump.** This repository is public.
- **Do not push.** The coordinator pushes.

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S2-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
