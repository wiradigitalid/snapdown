# W7-S1 · Step 1 — PLAN ONLY (re-scoped 2026-08-24)

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

> **This brief replaces an earlier one, and you should know why.** The first version of this story
> was written against `BUG-12` — five `.expect()` store opens that made a corrupt `library.db` kill
> the process silently. **`BUG-12` was already fixed** by `W6-S5` (commit `aa30434`), which was not
> scoped to it and closed it as a side effect. The register row still read `open`, a planner was
> dispatched, and it produced a complete implementation plan for code that already existed. **Read
> the code before you plan against a defect row.** That is the lesson and it applies to this brief
> too: verify each claim below against `HEAD` before building on it.

## Method position

WDI Method, **G5 Release**, wave **W7**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w7-failure-paths/`
- `story_id`: `W7-S1`

## What is already at `HEAD` — do not rebuild any of it

`apps/desktop/src-tauri/src/lib.rs`:

| Symbol | Lines |
|---|---|
| `StartupError::DatabaseOpen { path, source }` | `:51-59` |
| `StoresBundle` | `:61-67` |
| `init_app_stores(&Path) -> Result<StoresBundle, StartupError>` | `:69-102` |
| `format_startup_error_message` | `:105-120` |
| `show_native_message_dialog` via `MessageBoxW` | `:122-152` |
| `report_startup_error`, writes `startup-error.log` | `:154-163` |
| the fallible setup hook returning `Err` | `:226-233` |

Two of this story's four named tests already exist in
`apps/desktop/src-tauri/tests/test_startup.rs` under **different names**:
`an_unreadable_library_db_is_reported_with_its_path_and_not_recreated` and
`a_corrupt_library_db_does_not_panic_the_setup_hook`.

## The three defects this story actually carries

### `BUG-15` — HIGH, and it is the reason this story is still worth running

All five stores share one shape. `crates/snapdown-store/src/sqlite/settings_store.rs:22-38`, and
identically at `:28`/`:33` in `finding_store.rs`, `bundle_store.rs`, `access_key_store.rs`,
`publication_store.rs`:

```rust
let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
let mut conn = Connection::open_with_flags(path_ref, flags)?;
conn.pragma_update(None, "journal_mode", "WAL")?;      // <- WRITES to the file
conn.pragma_update(None, "foreign_keys", "ON")?;
conn.pragma_update(None, "busy_timeout", 5000)?;
// only now:
let mut integrity_stmt = conn.prepare("PRAGMA quick_check;")?;
```

Switching an existing database to WAL **mutates page 1** and creates `library.db-wal` and
`library.db-shm` beside it. Both happen **before** anything checks whether the file is intact. So for
a database with a **valid header and corrupt B-tree pages** — the case `quick_check` exists to catch
— the store *is* written to and files *are* created next to it, and the Reviewer is then shown a
message saying nothing was touched. That sentence is false, and it is the one thing they would rely
on before taking a backup.

`BR-118` — *"opened, never created over … no fresh one is started beside it"* — and the SDD's *"a
store recreated beside a corrupt one is silent data loss"* are both broken by the code path that
reports them.

**Why no test sees it, and this is the part to fix properly.** The only corrupt fixture anyone has
ever used is **garbage bytes**, which SQLite rejects at `Connection::open` before a single pragma
runs. The byte-identity assertion passes without ever reaching the code that breaks it. The
superseded version of this story named this exact case in its own edge-case matrix and then specified
a fixture that avoided it.

**The fixture is the story.** Build a database with a valid SQLite header and corrupt pages —
create a real database, then overwrite a page in the middle. Assert byte-identity **and** the absence
of `-wal` and `-shm`.

`SQLITE_OPEN_CREATE` is **not** the defect and MUST stay: creating a store that is absent is first
run, and `BR-118` forbids creating one *over* a corrupt file, which that flag does not do.

### `BUG-16` — medium

`lib.rs:346-347`:

```rust
.run(tauri::generate_context!())
.expect("error while running tauri application");
```

A setup hook returning `Err` makes `Builder::run` return `Err`, and that `.expect()` turns it into a
panic. The dialog is shown first, so the Reviewer is not left in the dark — what is wrong is that the
process exits by panic rather than by a matched exit.

`BUG-12` excluded this line from its sweep for a reason that was **true at the time**: *"If that
fails there is nothing left to report with."* That held while every store open panicked before
reaching it. `W6-S5` made it the **routine** exit path, and the premise quietly stopped being true.

### `BUG-17` — medium

`lib.rs:122-152` calls `MessageBoxW(null_mut(), …, MB_OK | MB_ICONERROR)` — no owner window, and
none of `MB_SETFOREGROUND`, `MB_TOPMOST`, `MB_SYSTEMMODAL`. Windows may not bring an unowned message
box to the foreground when the process has no foreground activation, which is exactly a
double-clicked exe failing at setup with Explorer still focused. The Reviewer then sees a taskbar
flash and nothing else — **the symptom `BUG-12` existed to end**.

Add `MB_SETFOREGROUND | MB_TOPMOST`. Two further properties nobody has written down, worth a line in
the plan: the call **blocks the setup hook** until someone clicks OK, and
`tauri-plugin-single-instance` is in the dependency list, so a second launch while the box is up
behaves in a way nothing specifies.

**Whether the dialog is actually visible is a MANUAL check.** Do not plan an automated test for it,
and do not assert that the flag constants were passed — that tests a copy of the input, which is the
literal-assert failure this repository has landed three times. `OQ-24` records that this project has
no working way to run a UI verification.

## The tests

`waves.yaml` records seven, and they MUST be carried through verbatim:

```
cargo::a_store_that_cannot_be_opened_yields_an_error_not_a_panic
cargo::the_startup_error_names_the_path_of_the_file_that_failed
cargo::a_corrupt_store_is_never_recreated_beside_itself
cargo::a_readable_store_still_starts_normally
cargo::a_valid_header_with_corrupt_pages_is_not_written_to_before_it_is_checked
cargo::a_failed_open_leaves_no_wal_or_shm_file_beside_the_database
cargo::a_store_failure_exits_without_panicking
```

The first four: two exist under other names and are **renamed, not rewritten**, so the registry and
the suite agree; `a_readable_store_still_starts_normally` has no coverage today.
`a_corrupt_store_is_never_recreated_beside_itself` must be **re-fixtured** against the valid-header
case, or it keeps passing for the wrong reason.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal
  to escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code, and read the four ways a
  run lies recorded there.
- **Write UTF-8, and no BOM.**
- **Never commit a captured screenshot.** No scratch files in the commit. **Do not push.**

## Done means

`_bmad-output/specs/w7-failure-paths/stories/W7-S1-*.md` exists, carries an `<intent-contract>`, and
its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
