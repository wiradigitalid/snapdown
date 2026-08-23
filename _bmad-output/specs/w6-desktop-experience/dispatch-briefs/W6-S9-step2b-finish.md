# W6-S9 · Step 2 (continued) — FINISH THE STORY

A previous worker wrote most of this story and stopped without completing it. Its spec frontmatter
still reads `status: ready-for-dev`. **Your job is to finish it, not to start it over.**

Read `AGENTS.md` first. Spec path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S9-a-bundle-survives-the-deletion-of-a-finding-it-holds-and-stops-swallowing-its-failures.md`

## What is already done, verified by the coordinator — do not redo it

**`BUG-1` is finished and proven.** `cargo test -p snapdown-store` exits **0**, and these pass:

```
migrations_v6_apply_cleanly_and_create_bundle_tables
deleting_a_finding_leaves_its_bundle_item_in_place
deleting_a_bundle_still_cascades_to_its_items
deleting_a_finding_leaves_the_bundle_markdown_byte_identical
deleting_a_finding_leaves_the_bundles_own_image_copy_in_the_vault
```

Migration v6 recreates `bundle_item` without the `finding_id` foreign key and keeps the `bundle_id`
cascade. That is correct. **Leave it alone.**

`BUG-9`'s production fix is also written: `bundle.rs:124` now reads
`unpublish_bundle(id.clone(), state.clone())?;`, so a failed unpublish aborts the delete. The code is
right. What is missing is proof.

## The one thing blocking this story

`cargo test --workspace` fails, and it fails before most of the suite even runs:

```
Running tests\test_bundle_failures.rs
error: test failed, to rerun pass `-p snapdown --test test_bundle_failures`
Caused by:
  process didn't exit successfully: ...test_bundle_failures-<hash>.exe
  (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)
```

The test binary **cannot start at all**. This is not an assertion failing — it is a load-time failure,
and no test in that file has ever executed.

The previous worker added `features = ["test"]` to the `tauri` dependency in the **workspace**
`Cargo.toml` to make `tauri::test` available, and added `NoopAutoStartBackend` to
`apps/desktop/src-tauri/src/startup/mod.rs` as a test double.

**Run `wdi-systematic-debugging` before proposing any fix.** The cause is a hypothesis, not a fact:
do not guess at it and do not try a fix you have not reasoned to.

## The steer, if debugging confirms the Tauri test harness cannot run here

Do not spend the story on it. `BR-20`'s guarantee does not depend on Tauri being testable — it depends
on the decision to abort. **Extract that decision into a plain function that takes the publication
record and an unpublish closure, and test that function directly**, the same way the store tests
prove the cascade. A test that runs is worth more than a test that describes a harness.

If you take that route, `test_bundle_failures.rs` should stop needing `tauri::test`, and then:

- **`features = ["test"]` must come back out of the workspace `Cargo.toml`.** It is currently on the
  production dependency, so the release binary carries the test harness. If the feature is still
  needed after your fix, it belongs in `[dev-dependencies]`, not where it is now.
- `NoopAutoStartBackend` may stay if a test still needs it; say so in your report either way.

## Both of these were changed outside the story's declared `files:` list

`Cargo.toml` and `apps/desktop/src-tauri/src/startup/mod.rs`. That is reported, not forbidden — they
serve the story. But whatever state you leave them in, **say so explicitly in your report**, because
the coordinator diffs the change set and anything unexplained comes back.

## The five tests in `test_bundle_failures.rs` that have never run

```
composition_that_cannot_open_the_vault_is_refused_not_silently_skipped
composition_that_cannot_write_its_markdown_writes_no_bundle_row
deleting_a_published_bundle_whose_unpublish_fails_aborts_and_reports
deleting_a_bundle_reports_an_image_copy_it_could_not_remove
a_bundle_whose_source_finding_is_gone_still_copies_the_same_bytes
```

The third is the one that matters most: it is the only proof that a Bundle whose unpublish fails is
not deleted locally while its published copy stays live on the public internet.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase** — and this story has already triggered it. A third
  failed fix attempt is the signal to escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed**, and four traps are recorded in `AGENTS.md`. The newest:
  `cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did — read the echoed value.
- **A test that cannot fail is a review finding.** So is a test file that cannot start.
- **Write UTF-8, no BOM.** No scratch files in the commit. **Do not push.**

## Done means

The spec's frontmatter reads `status: done`, and `cargo test --workspace` exits **0** with every one
of the story's named tests actually executing.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
