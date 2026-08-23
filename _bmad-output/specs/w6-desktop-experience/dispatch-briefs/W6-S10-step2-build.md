# W6-S10 · Step 2 — BUILD

The plan is done and approved. Implement it.

Read `AGENTS.md` first. Run `bmad-build-auto` with the spec path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S10-the-vault-move-reports-a-file-it-could-not-remove.md`

The spec is complete and its `<intent-contract>` is the owner's. **Do not edit anything inside it.**
The migration report shape, the error shape, the edge-case matrix and the test list are all already
written there. This step ends when its frontmatter reads `status: done`.

## The one thing that is easiest to get wrong, restated because it is worth restating

**A source file that will not delete does NOT make the move fail.**

Every file has been copied and verified. The move **succeeded**. Returning `Err` would tell the
Reviewer nothing moved when everything did — and `SCN-01`'s whole point is that the Reviewer is never
told something is in a state it is not. `migrate_vault` returns `Ok(report)` with the un-removed paths
inside it.

Compare `BR-20`, which `W6-S9` just landed: never claim a state you have not achieved. This is the
same rule pointing the other way, and getting it backwards here would be a worse defect than the one
being fixed.

## The two `let _ =` that stay, and why they must carry a comment

The plan judged both and both keep their swallow — but `AGENTS.md` asks for the reasoning to be
written down before a `let _ =` is kept, so write it in the code:

- **`:37`** — the writability probe file. If the probe cannot be removed the folder is still writable,
  because the write itself succeeded. The artifact is inert.
- **`:191`** — `fs::remove_dir` refusing a non-empty directory **is** the pruning guard. Removing the
  error handling would mean removing the guard.

A deliberate ignore with a stated reason is the correct outcome. An unexplained one is a review
finding.

## The test seam

The plan chose an injected delete function rather than a read-only file, and that is the right call:
`set_readonly` does not reliably prevent deletion on Windows, and a test that passes for the wrong
reason is worse than no test. Keep the seam small — a delete operation the migrator calls, swapped in
tests. Do not build a filesystem abstraction layer.

**These tests must actually run.** `W6-S9` lost two attempts to a test binary that could not start at
all because it reached for `tauri::test`. `vault_migration.rs` is plain Rust with no Tauri types in
it; keep the tests that way and they will run.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** All of `AGENTS.md` § Code. Four traps are recorded there; the
  newest is that `cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did — read the
  echoed value.
- **A test that cannot fail is a review finding**, and so is a test file that cannot start.
- **Write UTF-8, no BOM, keep the trailing newline.** No scratch files in the commit — `W6-S9` left a
  `test_ro.rs` at the repo root. Never a captured screenshot. **Do not push.**
- **Set the frontmatter to `status: done` when you are finished.** Two of the last three stories were
  judged incomplete because the work was done and the status was not.

## Done means

`cargo test --workspace`, `cargo fmt --all -- --check`, and
`cargo clippy --workspace --all-targets -- -D warnings` all exit **0**, the three named tests execute,
and the spec's frontmatter reads `status: done`.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
