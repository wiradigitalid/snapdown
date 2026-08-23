---
type: scenario
id: SCN-01
component: settings
branches_from: UC-14
created: "2026-08-23"
---

# SCN-01 — The Vault move that fails

Branches from `UC-14` step 7. It is a scenario rather than an alternate flow because it has a shape of
its own — a partial state that must not survive — and folding it in would have pushed that flow past
eight steps and hidden the thing worth reading.

It was first written as *the Vault move that fails halfway*, and that title was wrong: the
implementation has no halfway state on the source side, and the title survived a rewrite that removed
the thing it named.

**This is an as-built record.** The behaviour below was read from
`apps/desktop/src-tauri/src/vault_migration.rs` and is what the code does, not what would be nice.

## Setup

The Reviewer has 31 Findings and 4 Bundles in `C:\Users\<user>\SnapdownVault`. They choose
`D:\projects\acme\vault` and confirm. Partway through, the 19th file fails: it is open in an image
viewer, `D:` runs out of space, or the network drive drops.

## What the code actually does, and why it is the right shape

The move is **copy every file, verify every copy, and only then remove the sources.** The source is
untouched until the whole copy has succeeded (`vault_migration.rs:138`).

That makes the failure case much smaller than it first appears:

1. The copy stops at the first failure — it does not skip the file and continue.
2. `rollback` deletes the destination copies written so far, then prunes the empty directories it
   made (`vault_migration.rs:178`).
3. **No source file was ever touched.** All 35 remain where they were.
4. The location Setting is unchanged — it is written after the move returns, never before.
5. The Reviewer is told, in one message, that nothing moved and which file stopped it.

The failure this scenario was written to guard against — a half-state nothing on disk explains — is
prevented by ordering rather than by compensation. Copy-then-delete never has a moment where a file
exists in neither place, which is what `AD-2` actually needs.

## Where it is still not all-or-nothing, and this is a real finding

Two error paths swallow their result, and both are `let _ = fs::remove_file(...)`:

**In `rollback` (line 180).** A destination copy that cannot be deleted stays. The Reviewer is
correctly told nothing moved, and a stray file sits in the target folder pointing at nothing. It is
harmless to the Library and it is invisible — the orphan report (`FR-15`) scans the *current* Vault,
which after a failed move is still the old one, so it will never see it.

**On the success path (line 141).** After every copy is verified, the sources are removed with the
same swallowed result. A source file that cannot be deleted leaves a **duplicate**: the image exists
in both folders, the Setting now names the new one, and the old copy is unreferenced and unreported.
On a Vault holding personal data, an unreported leftover copy is the failure mode that matters, and
the product currently reports the move as fully successful.

Neither is a `[NEEDS CONFIRMATION]` — both were read directly. Both are dispositioned as planned work,
not as a bug in what was specified: the specification never said what happens to a source that will
not delete.

## What is genuinely NOT guaranteed

**Nothing, on the source side.** That is the strength of the ordering, and it is worth stating plainly
because the earlier draft of this scenario assumed a move-file-by-file implementation and spent a
paragraph on a rollback-of-the-rollback that this design makes impossible.

## Tests this scenario names

- `settings::vault_move_failing_at_file_n_leaves_every_source_file_in_place` — exists as
  `migration_rollback_on_failure_leaves_source_intact` (`vault_migration.rs:249`)
- `settings::vault_move_failing_leaves_the_location_setting_unchanged`
- `settings::vault_move_failing_leaves_every_record_resolving`
- `settings::vault_move_reports_the_file_that_stopped_it`
- `settings::vault_move_reports_a_destination_copy_it_could_not_clean_up` — **does not exist**
- `settings::vault_move_reports_a_source_file_it_could_not_remove` — **does not exist**, and this is
  the one that matters: it is the duplicate case
