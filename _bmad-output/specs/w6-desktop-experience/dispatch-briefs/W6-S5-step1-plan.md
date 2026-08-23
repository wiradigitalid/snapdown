# W6-S5 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first — the
pitfall about a panic taking the whole product with it is half this story.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S5`

## Two things in one story, and they belong together

This story owns the **startup path** and the **honest-state question**. `SCN-02` is about a control
that must not claim a state it has not read; `BUG-12` is about a startup that must not fail silently.
Same surface, same principle.

## Part 1 — `SCN-02`, and the run that makes it a scenario

`.what/settings/05-scenarios/SCN-02-the-first-run-and-the-startup-default.md`. Read it in full; it has
already done the thinking and it must not be re-derived.

`AUDIT-4` photographed the shipped state on 2026-08-24: **`Run at Windows startup` is unchecked.** The
owner asked for it to default on. `BR-112` says the default applies to a first run where nothing was
configured — which is why the stored `startup.registered` key exists at all.

| Run | Stored | Registered | Must happen |
|---|---|---|---|
| 1 — fresh install | unset | none | Snapdown registers itself. Toggle **On** |
| 2 — Reviewer turns it off | `off` | removed | Toggle **Off** |
| **3 — next sign-in** | `off` | none | **Does NOT re-register.** Toggle **Off** |
| **4 — removed outside Snapdown** | `on` | none | Toggle **Off**, and Snapdown does not re-register |

**Run 3 is the whole scenario.** The naive implementation — *if not registered, register* — passes
runs 1 and 2 and fails run 3 **silently**, by doing exactly what the Reviewer asked it not to.
Nothing errors, nothing logs, and the Reviewer finds out by noticing Snapdown running again.

**Run 4 is the trap.** The stored value says `on`, the OS says otherwise. `FR-18` and `BR-114` settle
it: the control reflects the **actual** registration, so it shows Off. Snapdown does not re-register
to make the stored value true, because it cannot tell run 4 from run 3 — and being wrong in run 3's
direction overrides a decision, while being wrong in run 4's direction only costs a Reviewer one
click.

`SCN-02` also records a fifth run — the store lost while the registration survives — as **accepted,
not fixed**. Do not fix it. Do not treat it as a gap.

Two of the tests are about the control refusing to lie before it knows:

```
vitest::the_startup_toggle_renders_unknown_until_the_os_has_answered
vitest::the_startup_toggle_never_renders_a_definite_state_before_the_read_resolves
```

`W6-S1` landed a three-state `Toggle` in `@snapdown/ui` for exactly this. Use it; do not invent a
second one.

## Part 2 — `BUG-12`, severity high

`apps/desktop/src-tauri/src/lib.rs:109-119` opens all five stores with `.expect()`. An unreadable or
corrupt `library.db` panics **inside the Tauri setup hook**.

**A Tauri release binary on Windows has no console.** The Reviewer double-clicks `Snapdown.exe` and
**nothing happens at all** — no window, no tray, no message, no file named. The product does not look
broken; it looks absent. `AD-11` put the tray, the hotkeys, the overlay and the Editor in one process,
and `DEC-003` accepted that cost in writing.

`SDD-settings.md`'s Failure Behaviour already specifies the right answer — *reported with the file's
path, and nothing is created over it* — and **half of it is already kept**: the panic does guarantee
nothing is created over the corrupt store. The reporting half is simply not implemented.

```
cargo::an_unreadable_library_db_is_reported_with_its_path_and_not_recreated
cargo::a_corrupt_library_db_does_not_panic_the_setup_hook
```

Plan what the Reviewer actually sees. A process that exits silently is the defect; a process that
exits after saying which file it could not open, and where, is the fix. Decide the channel — a native
dialog, a log file the message names, or both — and say why.

`DEC-003`'s Cost section predicted this class in writing and nobody went and looked. That is the
useful part: **the prediction was recorded and never turned into a check.**

## Boundaries

- The Settings **frame** is `W6-S3`. The Quality Budget's contents are `W6-S4`. The hotkey rows are
  `W6-S6`. This story owns the Startup group's behaviour and the startup path beneath it.
- `NoopAutoStartBackend` already exists in `apps/desktop/src-tauri/src/startup/mod.rs` — `W6-S9` added
  it as test infrastructure. Use it rather than adding a second test double.
- `AppState` is a plain struct with public fields and `W6-S9` extracted `_impl` functions taking
  `&AppState` from the Tauri commands. **That is the pattern for anything here that needs testing**:
  a test that reaches for `tauri::test` produces a binary that cannot start at all, which cost that
  story two attempts.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Before writing `unwrap`/`expect` outside a test, ask what the Reviewer sees when it fires.**
  That question is this story's subject.
- **Verification is run, not assumed.** All of `AGENTS.md` § Code. Four traps are recorded there.
- **Write UTF-8, no BOM.** No scratch files in the commit, never a captured screenshot, and **do not
  push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S5-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
