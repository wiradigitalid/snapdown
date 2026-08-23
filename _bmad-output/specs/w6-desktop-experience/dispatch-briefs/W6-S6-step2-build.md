# W6-S6 · Step 2 — BUILD

The plan is done and approved. Implement it.

Read `AGENTS.md` first. Run `bmad-build-auto` with the spec path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S6-hotkey-rows-chip-states-honest-conflicts-and-readable-badges.md`

The spec is complete and its `<intent-contract>` is the owner's. **Do not edit anything inside it.**
The four chip states, the badge wording, the conflict cases and the five tests are already written
there. This step ends when its frontmatter reads `status: done`.

## What is already right — do not replace it

The keystroke recorder exists and says what to do (`Click to record`). The owner asked to set hotkeys
by pressing keys rather than typing a string, and that interaction works. **The default is what is
missing** — `Capture Region` is bound to nothing, so on a fresh profile the capture hotkey does
nothing at all and the capture path is unreachable by keyboard.

`W6-S1` landed `HotkeyChip` and `Badge` in `@snapdown/ui`. Use them; do not invent a second chip.

## Three of the five tests look cosmetic and are not

- **A chip left listening after focus moves away swallows the next keystroke** the Reviewer meant for
  something else. That is a leak, not a visual state.
- **An internal conflict and an OS conflict have different remedies.** *Another Snapdown action uses
  this* is fixable here, in this panel. *Windows already uses this* is not. Wording them identically
  sends the Reviewer to the wrong place.
- **A badge distinguished only by colour is unreadable** to anyone who cannot separate those two
  colours. `DESIGN.md` says the badge reads **Active** or **Disabled** in words, and `EXPERIENCE.md`
  sets the same floor for Markers.

`DESIGN.md`'s own rule for this surface: **put a failure under the control that failed.** A hotkey
that could not be registered at startup says so before the Reviewer clicks it and finds out.

## Boundaries

- The Settings **frame** landed in `W6-S3`, the Quality Budget in `W6-S4`, the startup toggle in
  `W6-S5`. This group sits in column B. **Do not re-lay-out the panel.**
- **Colour lives in exactly one file** — `web/ui/src/styles/tokens.css`, both themes — and a lint rule
  refuses a literal anywhere else.
- `BR-26` looks local and is not: `finding` owns the capture action the hotkey triggers, and
  re-registration crosses both components. A hotkey change takes effect **without a restart**.
- `AppState` is a plain struct and `W6-S9` extracted `_impl` functions taking `&AppState`. That is the
  pattern for anything needing a test; reaching for `tauri::test` produces a binary that cannot start.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **A green unit test does not mean the component is reachable.** Grep for `<ComponentName` before
  closing.
- **A test that cannot fail is a review finding.** Assert behaviour, not a copy of the input.
- **Verification is run, not assumed.** Both halves of `AGENTS.md` § Code.
- **Write UTF-8 with NO BOM.** Three story files this wave arrived BOM-prefixed, and a BOM makes the
  frontmatter parser report the story as having no status at all.
- No scratch files in the commit, never a captured screenshot, and **do not push.**
- **Set the frontmatter to `status: done` when you are finished.**

## Done means

`cargo test --workspace`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and the `web/ui` and `apps/desktop` scripts
all exit **0**, the five named tests execute, and the spec's frontmatter reads `status: done`.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
