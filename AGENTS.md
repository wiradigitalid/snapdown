# Agent Rules — Snapdown (mattpocock/skills evaluation branch)

This branch (`eval/mattpocock-skills`) is a temporary evaluation of
https://github.com/mattpocock/skills instead of WDI Method. WDI Method's skills,
`_bmad/`, and the process rules that used to live in this file are intentionally
removed here — they are untouched on `main` and this branch will either merge in
as-is or have its process commit dropped via `git rebase --onto main`, replaying
just the real coding commits back onto `main` (which still has WDI Method).

The stack facts and pitfalls below are project truth, not method process — kept
as-is for this evaluation.

## Code

Tauri v2 desktop app. Rust workspace (`crates/snapdown-core` pure domain, `crates/snapdown-store`
adapters, `crates/snapdown-bridge` the MCP executable), a React + Vite webview in `apps/desktop`, a
shared UI package in `web/ui` consumed as `@snapdown/ui`, and a Go service in `apps/web-service`.
`.control/structure-codebase.md` is the map; this section does not duplicate it.

### Verification — run all of it, from the repo root

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`apps/desktop` has **no `package.json`** — `DEC-007` moved the desktop app off React onto Slint, so the
three `npm --prefix apps/desktop` lines that used to be here fail with `enoent` and always will. They
were left behind by that decision and cost a session's time before being noticed.

`web/ui` still has one, and nothing in the active workspace builds or tests it: the desktop app no
longer consumes `@snapdown/ui` and the Go service never did (`OQ-27`). Run it only when changing
`web/ui` itself:

```bash
npm --prefix web/ui ci
npm --prefix web/ui run typecheck && npm --prefix web/ui run lint && npm --prefix web/ui run test
```

Three CI jobs cover these: `rust-check`, `web-check`, and `web-service`. A green `korpus.yml` is
**not** proof the code compiles — it validates the corpus, and they answer different questions.

**`cargo build` does NOT build this application.** A Tauri app needs the Tauri CLI; without it the
release binary requests `devUrl` from `tauri.conf.json` and shows `ERR_CONNECTION_REFUSED` instead of
the frontend. The CLI is currently absent from this repository entirely — see `BUG-11`. Until that is
fixed, **a locally built `Snapdown.exe` is not the application**, and any UI finding taken from one is
a finding about the build.

**Four ways a verification run lies, all hit on 2026-08-23:**

- **`cmd | tail` reports the exit code of `tail`, not of `cmd`.** A `cargo build` that failed with
  *package ID specification did not match any packages* was reported as exit 0 because it was piped.
  Check `${PIPESTATUS[0]}`, or redirect to `/dev/null` and read `$?`.
- **The coordinator's own worktree goes stale the moment a story adds a dependency.** `web/ui`
  typecheck failed locally on missing `@types/node` while CI was green: CI runs `npm ci` from the
  lockfile, a long-lived worktree does not. Run `npm --prefix <pkg> ci` before believing a local red.
- **`cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did.** The script's exit code is
  `echo`'s, and `echo` always succeeds — so the background-task notification says *exit code 0* while
  the echoed line says `EXIT=1`. A `tauri build` that died on *Access is denied* was reported as a
  success this way. Read the echoed value, never the notification's code.

### Pitfalls

**A green unit test does not mean the component is reachable.** This is the most expensive mistake
this repository has made. On 2026-08-23 a sweep found **four** components built, unit-tested, and
mounted nowhere: `CaptureOverlay` (the capture path — `BUG-4`), `MarkerLayer` (marker annotation —
`BUG-5`), `OrphanReportView` (`BUG-6`), and `EmptyState`. Three requirements — `FR-1`/`FR-2`, `FR-8`,
`FR-15` — were unmet in a build whose tests all passed, for four waves.

There is no composition test class here yet (`OQ-23`). Until there is, **before closing any story
that adds a component, grep for `<ComponentName` across `apps/desktop/src` and `web/ui/src`,
excluding its own file and its tests.** No hit means nobody can reach it. `V12` will not catch this:
it checks that an `LC` is *registered*, not that it is *reached*.

**A panic in the desktop process takes the whole product with it.** `AD-11` puts the tray, the
hotkeys, the capture overlay and the Editor in one process, and `DEC-003` accepted that cost in
writing: *"a panic in the editor's Tauri commands kills the tray, the hotkeys, and the overlay with
it."* A Tauri release binary on Windows has no console, so a panic in the setup hook means the
Reviewer double-clicks the exe and **nothing happens at all** — see `BUG-12`, five `.expect()` calls
on store opens. Before writing `unwrap`/`expect` outside a test, ask what the Reviewer sees when it
fires. Genuinely infallible cases exist and are fine — `Header::from_bytes` over a compile-time byte
constant is one — but they are rarer than they look.

**`let _ =` on a Result an invariant depends on is a defect, not a style.** Five instances found on
2026-08-23 across two files: `vault_migration.rs` swallowed both `fs::remove_file` results, and
`bundle.rs` swallowed the Markdown write, the Vault open, the unpublish, and two blob deletes. The
worst of them leaves a published Bundle **live on the internet** after the Reviewer deletes it. It
reads as deliberate, clippy ignores it at default levels, and no test catches it because every store
test uses a writable temp directory. Before writing one, ask what invariant the call is holding up.

**Colour lives in exactly one file.** `web/ui/src/styles/tokens.css`, defined for both themes
(`AD-10`). A lint rule refuses a colour literal anywhere else. The four deliberately theme-invariant
groups — `--color-marker*`, `--color-overlay-scrim`, `--color-overlay-ring`, `--canvas-checker` —
are the one exception and they live in that file too, each with a comment saying why.

**An image test that asserts a signature and a dimension is a test that a fake header passes.** That
is not a hypothetical: a 17-byte `PNG` + width + height passed every image assertion in this
repository for five waves and three audits, and `FR-4` and `NFR-3` were reported met on the strength
of a correct number inside a fabrication. **Decode the output.** The defect such a test misses is
never the presence of a signature check — it is the absence of a decode. And the prohibition cannot
be a test: any mechanical form of it scans the test sources, which asserts a copy of its own input
and passes by construction the moment the offending line is deleted. It lives here instead, and its
positive half is `every_image_producing_path_decodes_its_own_output`, which fails when an image
producer is added without one.

**A test that asserts a literal is a test that cannot fail.** `contrast.test.ts` originally hardcoded
its own copy of the token values; changing a token to a 2:1 ratio left it green. It now parses
`tokens.css` and was verified by mutation. Assert the behaviour, not a copy of the input.

**Two SQLite stores, not one.** `library.db` (Rust, `crates/snapdown-store/src/sqlite/migrations.rs`)
and the web service's own (Go, `apps/web-service/internal/store/store.go`). A reader that looks at
only the first will report the second's tables as missing — that is what
`.constitution/project/inventory-readers.py` did for two waves.

**Never commit a captured screenshot.** This repository is public and the product brief forbids it.
`.gitignore` covers every image outside the app icon set, and `korpus.yml` now refuses tracked images,
raw accessibility-tree dumps, and operator home-directory paths.

**Scrubbing a leak from a branch is not finished when the branch is rewritten.** `git filter-branch`
leaves its own backup at `refs/original/refs/heads/<branch>`, and the old objects stay reachable
through it. Delete that ref, `git reflog expire --expire=now --all`, then `git gc --prune=now`. Verify
with `git log --all --oneline -- <path>` returning nothing. This was missed once, on 2026-08-23, and
found only because the tag cleanup prompted a second look.

**A defect register entry is a claim about code at a moment, and it goes stale silently.** `BUG-12`
read `status: open` for a day after `W6-S5` had already fixed it — that story was not scoped to the
defect and closed it as a side effect of needing a fallible startup path. Wave `W7` was opened
against it, a planner was dispatched, and it wrote a complete implementation plan for code that
already existed. Nothing caught it until a review read the code instead of the register. **Before
planning against a defect row, grep for the symbols its `fix:` describes.** The same read found the
opposite failure too: `BUG-3` and `BUG-10` carried `blocked_by: DEC-005` when that decision says in
its own words *"This decision does not forbid a fix. It forbids new work"* — so a public,
unauthenticated HTML-injection path sat unfixed because a register field misquoted a decision.

**A sweep's exclusions expire when the code they reasoned about changes.** `BUG-12` deliberately left
`lib.rs:347` unregistered because *"if that fails there is nothing left to report with"* — true while
every store open panicked before reaching it. `W6-S5` made it the **routine** exit path and the
premise quietly stopped holding, which is now `BUG-16`. Second time this has happened. When you
change a path, re-read what was excused on the strength of it.

**A writing pragma is a write.** All five SQLite stores ran `journal_mode = WAL` before
`PRAGMA quick_check`, which mutates page 1 and creates `-wal`/`-shm` — so a corrupt store *was*
written to while the Reviewer was shown a dialog promising it had not been (`BUG-15`). Check
integrity on a read-only connection first.

**Prove a corrupt-file fixture actually reaches the code you think it does.** Every corrupt-database
test here used garbage bytes, which SQLite rejects at `Connection::open` before a single pragma runs
— so the byte-identity assertion passed without ever executing the defect. A valid header with
corrupt pages is what reaches it. And note the subtler result from the same story:
`a_failed_open_leaves_no_wal_or_shm_file_beside_the_database` **still passes with the bug present**,
because SQLite removes those files on a clean close. It is accurate to its name and insensitive to
the defect — belt-and-braces, not a guard. **Mutation is the only way to tell those apart.**

**A test fixture must be legal on Windows, and CI will not tell you.** A `sharing` fixture used a
slug of `test<slug>&42`; `store.go` joins the slug into a filesystem path and Windows refuses `<`
and `>`. The Go job runs on `ubuntu-latest`, so it would have been green in CI and red on every
developer machine here.

**Stale binaries mislead.** Renaming the product left `desktop.exe` beside `Snapdown.exe` in
`target/release/`, the owner ran the old one, and reported four defects that did not exist. `FR-27`
now makes a second desktop executable a build failure.

**A dispatched worktree branches from `main`, not from the branch you are on.** `worker-start
--worktree new-child` without `--base-branch` gave W6-S9's planner a checkout at `main`'s tip, so the
whole wave — `SPEC.md`, `stories.yaml`, every dispatch brief — was absent, and the worker rebuilt them
from scratch as new files. Pass `--base-branch` explicitly for any wave work, and take only the files
the brief asked for out of a worktree that got this wrong: its reconstructed registries are guesses,
and overwriting the real ones with them loses everything the wave has written.

**A leftover `Snapdown.exe` process locks its own file and fails the next build.** A binary launched
by an earlier UI audit was still running hours later; `tauri build` died with *failed to remove file
`Snapdown.exe`: Access is denied (os error 5)*, which reads like a permissions problem and is not.
`Get-Process -Name Snapdown` before rebuilding, and treat a still-running instance as cleanup the
same way a stale worktree is.
