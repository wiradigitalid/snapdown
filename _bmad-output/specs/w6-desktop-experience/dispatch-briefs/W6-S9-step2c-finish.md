# W6-S9 · Step 2 (third attempt) — FINISH IT

Two workers have written most of this story and stopped without completing it. **This is the third
and last attempt before the coordinator escalates to the owner.** The remaining work is mechanical
and fully specified below. Do not redesign anything.

Read `AGENTS.md` first. Spec:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S9-a-bundle-survives-the-deletion-of-a-finding-it-holds-and-stops-swallowing-its-failures.md`

## Do not touch these — they are finished and verified

**`BUG-1` is proven.** `cargo test -p snapdown-store` exits 0. Migration v6 recreates `bundle_item`
without the `finding_id` foreign key and keeps the `bundle_id` cascade. Five tests pass, including
both directions of the cascade. **Leave `migrations.rs` and `test_sqlite_bundles.rs` alone.**

**The production fixes are written.** `bundle.rs` propagates the failed unpublish with `?`. The
workspace `Cargo.toml` no longer carries tauri's `test` feature. `unpublish_bundle_impl(&str,
&AppState)` already exists in `sharing.rs` with the `#[tauri::command]` as a two-line shim over it.

## The only thing left, and exactly how to do it

`apps/desktop/src-tauri/tests/test_bundle_failures.rs` **cannot start** — `STATUS_ENTRYPOINT_NOT_FOUND`
— because it builds a `tauri::test::mock_app()`. Its five tests have never executed once.

The seam that fixes it already exists; it is simply not used yet. Apply the **same extraction that
`unpublish_bundle` already has** to the three other commands the test file calls:

| Command | In | Extract to |
|---|---|---|
| `create_bundle` | `commands/bundle.rs` | `create_bundle_impl(input: CreateBundleInput, state: &AppState)` |
| `delete_bundle` | `commands/bundle.rs` | `delete_bundle_impl(id: &str, state: &AppState)` |
| `copy_bundle_to_clipboard` | `commands/bundle.rs` | `copy_bundle_to_clipboard_impl(id: &str, state: &AppState)` |

Each `#[tauri::command]` stays exactly where it is and becomes a one-line shim, precisely like:

```rust
#[tauri::command]
pub fn unpublish_bundle(bundle_id: String, state: State<AppState>) -> Result<(), String> {
    unpublish_bundle_impl(&bundle_id, &state)
}
```

**`delete_bundle_impl` must call `unpublish_bundle_impl`, not the command.** That is the whole point.

## Then the test file drops Tauri entirely

`AppState` is a plain struct with seven public `Arc` fields and no constructor —
`apps/desktop/src-tauri/src/state.rs`. It can be built directly. **`tauri::test` is needed for nothing
else in this file.**

Change `build_test_app` to return `AppState` instead of `tauri::App<MockRuntime>`: keep every line
that opens the five stores, sets `VaultPath` and `WebServiceAddress`, and builds the two registrars
(`NoopAutoStartBackend` is already in `startup/mod.rs` for this). Then **delete the `tauri::test::mock_app()`
and `app.manage(...)` lines** and return the `AppState` value. Every `state.clone()` at a call site
becomes `&state`.

Nothing about what the tests assert changes. They are already written and their assertions are right.

## The five tests that must actually run

```
composition_that_cannot_open_the_vault_is_refused_not_silently_skipped
composition_that_cannot_write_its_markdown_writes_no_bundle_row
deleting_a_published_bundle_whose_unpublish_fails_aborts_and_reports
deleting_a_bundle_reports_an_image_copy_it_could_not_remove
a_bundle_whose_source_finding_is_gone_still_copies_the_same_bytes
```

The third is the one that matters most: it is the only proof that a Bundle whose unpublish fails is
not deleted locally while its published copy stays live on the public internet. `BR-20` exists to
forbid exactly that.

If a test now fails on its **assertion**, that is a real finding — report it, and run
`wdi-systematic-debugging` before changing production code to satisfy it. Do not weaken an assertion
to get green. A test that cannot fail is a review finding.

## Report these explicitly, whatever you leave them as

`Cargo.toml` and `apps/desktop/src-tauri/src/startup/mod.rs` were changed outside the story's declared
`files:` list by earlier attempts. `NoopAutoStartBackend` is legitimate test infrastructure and may
stay. The coordinator diffs the change set and anything unexplained comes back.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** This story has already triggered it twice.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed.** Four traps are in `AGENTS.md`; the newest is that
  `cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did — read the echoed value.
- **Write UTF-8, no BOM, and keep the trailing newline.** No scratch files in the commit. **Do not push.**

## Done means

`cargo test --workspace` exits **0** with all five of those tests executing, and the spec's frontmatter
reads `status: done`. **Set the frontmatter — both previous attempts left it at `ready-for-dev` and
that is why they were judged incomplete.**

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
