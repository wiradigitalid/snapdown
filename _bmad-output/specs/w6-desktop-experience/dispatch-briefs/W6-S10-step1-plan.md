# W6-S10 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first — the
third pitfall in its `## Code` section is precisely this story.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S10`

## The corpus has already done most of this story's thinking

`.what/settings/05-scenarios/SCN-01-the-vault-move-that-fails.md` is an **as-built record** written
from this exact file, and it names both defects and explains why one is worse than the other. Read it
before reading the code. Do not re-derive what it already says, and do not contradict it.

Its central finding, quoted:

> **On the success path (line 141).** After every copy is verified, the sources are removed with the
> same swallowed result. A source file that cannot be deleted leaves a **duplicate**: the image exists
> in both folders, the Setting now names the new one, and the old copy is unreferenced and unreported.
> On a Vault holding personal data, an unreported leftover copy is the failure mode that matters, and
> the product currently reports the move as fully successful.

## What the code does today, and what is right about it

`apps/desktop/src-tauri/src/vault_migration.rs`, `migrate_vault`:

The move is **copy every file → verify every copy → only then remove the sources**. The source is
untouched until the whole copy has succeeded. **That ordering is correct and is not what this story
changes.** `AD-2` needs a file never to exist in neither place, and copy-then-delete guarantees it.

Two `let _ =` are the defect:

| Line | Swallows | Consequence |
|---|---|---|
| **`:141`** | `fs::remove_file` on each **source** after verification | **The serious one.** A duplicate of an image survives in the old folder, unreferenced and unreported, while the move reports full success. `FR-15`'s orphan report scans the *current* Vault, so it can never see it |
| `:180` | `fs::remove_file` inside `rollback` | A destination copy that will not delete stays behind. The Reviewer is correctly told nothing moved, so the harm is a stray file, not a lie |

## Two more `let _ =` in the same file that the wave did NOT schedule

Judge them; do not silently fix them and do not silently ignore them. Say in the plan what you
concluded and why.

- **`:37`** — `fs::remove_file` on the probe file inside `validate_directory_writable`. It is a
  write-test artifact. Ask what is left behind when it fails, and whether the Reviewer ever sees it.
- **`:191`** — `fs::remove_dir` in `remove_empty_dirs_recursive`. This one may well be deliberate:
  `remove_dir` refusing a non-empty directory *is* the guard. If you conclude that, say so — a
  deliberate ignore with a stated reason is the right outcome, and `AGENTS.md` asks for exactly that
  reasoning before any `let _ =` is written or kept.

## What "reports" must mean here, and the trap inside it

`migrate_vault` returns `Result<(), CoreError>`. The tempting fix is to turn a failed source removal
into an `Err`. **That would be wrong**, and it is the trap this story has to avoid:

The files are already copied and verified. The move **succeeded**. Returning `Err` would tell the
Reviewer nothing moved when everything did, and `SCN-01`'s whole point is that the Reviewer is never
told something is in a state it is not. Compare `BR-20`'s shape in `W6-S9`: never claim a state you
have not achieved — in **either** direction.

So the move succeeds **and** reports what it could not clean up. Decide the shape in the plan: a
success value carrying the un-removed paths, a warning surfaced to the Settings surface, or both. Say
which, and say what the Reviewer sees. `.what/settings/04-usecases/EXPERIENCE.md` and `UC-14` bind
what they are told.

## The three tests, named in `waves.yaml` and to be carried verbatim

```
cargo::vault_move_reports_a_source_file_it_could_not_remove
cargo::vault_move_reports_a_destination_copy_it_could_not_clean_up
cargo::vault_move_failing_at_file_n_leaves_every_source_file_in_place
```

The third already has a sibling in the file — `migration_rollback_on_failure_leaves_source_intact`.
Read it before writing the new one, and do not duplicate it.

**These need a filesystem that refuses a delete.** Every test in this repository uses a writable temp
directory and a live path, which is exactly why five waves passed over this defect. `W6-S9` hit the
same wall and its Tauri-based attempt produced a test binary that could not start at all. Plan for a
test that actually runs: a read-only parent directory, a permissions change, or an injected
filesystem seam. If you choose a seam, keep it small and say why the alternative was rejected.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed**, and four traps are recorded in `AGENTS.md`. The newest:
  `cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did — read the echoed value.
- **A test that cannot fail is a review finding**, and so is a test file that cannot start.
- **Write UTF-8, no BOM.** No scratch files in the commit, never a captured screenshot, and **do not
  push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S10-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
