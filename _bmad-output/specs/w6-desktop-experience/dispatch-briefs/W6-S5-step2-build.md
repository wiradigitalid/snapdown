# W6-S5 · Step 2 — BUILD

The plan is done and approved. Implement it.

Read `AGENTS.md` first. Run `bmad-build-auto` with the spec path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S5-run-at-windows-startup-on-by-first-run-default-and-never-assumed.md`

The spec is complete and its `<intent-contract>` is the owner's. **Do not edit anything inside it.**
The `startup.registered` key, the four runs, the setup-hook failure path and the seven tests are all
already written there. This step ends when its frontmatter reads `status: done`.

## The two things that are easiest to get wrong

**Run 3.** The naive implementation — *if not registered, register* — passes a fresh install and passes
the moment the Reviewer unchecks the toggle, then **fails on the next sign-in silently**, by
re-registering against an explicit instruction. Nothing errors and nothing logs; the Reviewer finds out
by noticing Snapdown running again. `startup.registered` exists to tell *unset* from *off*, which is the
one question the OS cannot answer.

**Run 4.** Stored value says `on`, the OS says otherwise. The control shows **Off** and Snapdown does
**not** re-register — being wrong in run 3's direction overrides a decision, while being wrong in run
4's direction costs one click.

`SCN-02`'s **fifth** run is recorded as accepted, not fixed. Do not fix it.

## `BUG-12` — what the Reviewer sees is the whole subject

`lib.rs:109-119` opens five stores with `.expect()`. A Tauri release binary on Windows has no console,
so a corrupt `library.db` means double-clicking `Snapdown.exe` does **nothing at all** — no window, no
tray, no message, no file named. The product does not look broken; it looks absent.

The plan's answer: open the stores fallibly, and on failure show a native dialog naming the **exact
database path** and the reason, write it to a log, and exit cleanly rather than panicking. Half of the
right behaviour is already kept by accident — the panic does guarantee nothing is created over the
corrupt store — so **keep that**: report, and do not recreate.

## The startup toggle must not lie before it knows

`W6-S1` landed a three-state `Toggle` in `@snapdown/ui`. The indeterminate state is what shows before
Windows has answered, and it **MUST NOT** render on or off first. Use that component; do not invent a
second one.

## Boundaries

- The Settings **frame** landed in `W6-S3`, the Quality Budget in `W6-S4`. The hotkey rows are
  `W6-S6`. **Do not re-lay-out the panel.**
- `NoopAutoStartBackend` already exists in `startup/mod.rs` — `W6-S9` added it. Use it rather than
  adding a second test double.
- `AppState` is a plain struct and `W6-S9` extracted `_impl` functions taking `&AppState` from the
  Tauri commands. **That is the pattern for anything needing a test.** Reaching for `tauri::test`
  produces a binary that cannot start at all, which cost that story two attempts.
- Migration numbering: v7 landed with `W6-S4`. If this story needs one, it takes **v8**.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Before writing `unwrap`/`expect` outside a test, ask what the Reviewer sees when it fires.**
- **Verification is run, not assumed.** Both halves of `AGENTS.md` § Code. Four traps are recorded
  there; the newest is that `cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did.
- **A test that cannot fail is a review finding.**
- **Write UTF-8, no BOM, keep trailing newlines.** No scratch files in the commit. **Do not push.**
- **Set the frontmatter to `status: done` when you are finished.**

## Done means

`cargo test --workspace`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and the `web/ui` and `apps/desktop` scripts
all exit **0**, the seven named tests execute, and the spec's frontmatter reads `status: done`.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
