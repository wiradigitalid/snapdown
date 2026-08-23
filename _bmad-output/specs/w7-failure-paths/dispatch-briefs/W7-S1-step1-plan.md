# W7-S1 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W7**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first — its
`## Code` section carries the verification commands, and its second pitfall (*"A panic in the desktop
process takes the whole product with it"*) is precisely what this story exists to fix.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w7-failure-paths/`
- `story_id`: `W7-S1`

Resolve everything else from `{spec_folder}/stories.yaml` and `{spec_folder}/SPEC.md`.

## `BUG-12` — a corrupt `library.db` makes the application vanish on launch

`apps/desktop/src-tauri/src/lib.rs:109-119` opens five stores and `.expect()`s every one:

```rust
SqliteSettingsStore::open(&db_path).expect("Failed to initialize SqliteSettingsStore")
SqliteFindingStore::open(&db_path).expect("Failed to initialize SqliteFindingStore")
SqliteBundleStore::open(&db_path).expect("Failed to initialize SqliteBundleStore")
SqliteAccessKeyStore::open(&db_path).expect("Failed to initialize SqliteAccessKeyStore")
SqlitePublicationStore::open(&db_path).expect("Failed to initialize SqlitePublicationStore")
```

**Half the promise is kept and half is not.** Nothing IS created over the corrupt store — the panic
guarantees that much. But nothing is REPORTED either.

A Tauri release binary on Windows has **no console**. A panic inside the setup hook unwinds and the
process exits, so the Reviewer double-clicks `Snapdown.exe` and **nothing happens** — no window, no
tray icon, no message, no file named. The product does not appear to be broken; it appears not to be
there.

## The design is already written. Implement it; do not invent it

`.how/settings/SDD-settings.md` § Failure Behaviour, row **`LC-025` → `library.db`**, already
specifies the answer, and its middle column is the case here:

> Reported with the file's path, and **nothing is created over it** (`BR-118`). A store recreated
> beside a corrupt one is silent data loss.

Its third column is a second case worth planning for while you are here: *"A store that opens and
returns a schema version the code does not know is refused, named, and not migrated."*

`BR-118` states the same rule in `.what/settings/02-rules/rules-settings.md`:

> The settings store is opened, never created over. A store that cannot be read is reported with its
> path, and no fresh one is started beside it.

## `AD-11` is what makes this hard, and it must shape the plan

One process owns the tray, the hotkeys, the capture overlay and the Editor. At the moment a store
open fails there is **no surviving surface to report into** — and stderr reports to nobody, because
the release binary has no console.

`DEC-003` accepted this cost **in writing**, and that sentence is the reason this story exists:

> *"a panic in the editor's Tauri commands kills the tray, the hotkeys, and the overlay with it …
> this raises the bar on every unwrap in the command layer."*

The prediction was recorded and **never turned into a check**. Say in the plan what surface carries
the report — a native dialog before the main window exists is the obvious candidate, but make the
choice explicitly and say why, because it is the whole substance of the story.

## The trap, and it is the tempting fix

**Do not start a replacement store so the application keeps running.** That trades a visible failure
for **silent data loss**, and it is worse than the defect. `BR-118` is not in question and MUST NOT
be weakened to make the report easier. The unreadable file is left exactly as found.

## Scope — three groups were already swept and deliberately NOT registered

`BUG-12`'s register entry records them so the next sweep does not re-raise them. **Do not widen into
them:**

- `server/handlers.rs` — 26 unwraps, all `Header::from_bytes` over compile-time byte constants such
  as `b"Content-Type"`. These cannot fail.
- `lib.rs:226` — `.expect("error while running tauri application")` on the Tauri run call. If that
  fails there is nothing left to report with.
- bridge `mcp.rs:55,81` — `serde_json::to_string` on a well-formed response struct.

## The tests that matter

`waves.yaml` records four for this story and they MUST be carried through verbatim:

```
cargo::a_store_that_cannot_be_opened_yields_an_error_not_a_panic
cargo::the_startup_error_names_the_path_of_the_file_that_failed
cargo::a_corrupt_store_is_never_recreated_beside_itself
cargo::a_readable_store_still_starts_normally
```

**No test in this repository has ever opened a corrupt database.** Every store test uses
`open_in_memory()` — which is exactly why five waves passed over this. The corrupt-file fixture is
new work and it is the point of the story, not an afterthought.

Note the seam this repo already established for testing Tauri command logic: extract an
`_impl(&AppState)` function and have the `#[tauri::command]` wrapper delegate to it.
`commands/sharing.rs` is the worked example. `tauri::test::mock_app` produced
`STATUS_ENTRYPOINT_NOT_FOUND` twice on `W6-S9` and MUST NOT be reached for again.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal
  to escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code. Note the four ways a
  run lies, all recorded there — in particular `cmd; echo "EXIT=$?"` makes the harness report 0
  whatever `cmd` did.
- **Write UTF-8, and no BOM.** Three story files arrived with one during W6; it makes the
  frontmatter parser report the story as having no status.
- **Never commit a captured screenshot.** No scratch files in the commit. **Do not push.**

## Done means

`_bmad-output/specs/w7-failure-paths/stories/W7-S1-*.md` exists, carries an `<intent-contract>`, and
its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
