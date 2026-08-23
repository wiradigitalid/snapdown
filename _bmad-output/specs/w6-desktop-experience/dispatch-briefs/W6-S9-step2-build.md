# W6-S9 · Step 2 — BUILD

The plan is done and approved. Implement it.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 2. Read `AGENTS.md` first — its
`## Code` section carries the verification commands and every pitfall this repo has paid for.

Run `bmad-build-auto` with the spec path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S9-a-bundle-survives-the-deletion-of-a-finding-it-holds-and-stops-swallowing-its-failures.md`

The spec is complete and its `<intent-contract>` is the owner's. **Do not edit anything inside it.**
Everything you need — the migration shape, the abort semantics, the test list, the edge-case matrix —
is already written there. This step ends when its frontmatter reads `status: done`.

## The two defects, in the order they must land

**`BUG-9`'s `:116` first.** `delete_bundle` swallows `unpublish_bundle`. The unpublish fails, the
deletion proceeds, the local record is gone, and the published copy stays live on the public internet
with nothing pointing at it. The Reviewer believes they deleted it. `BR-20` was written to forbid
exactly that outcome, so the fix follows from the rule: **abort the delete and report it.** Never
delete a local record whose remote copy may still be live.

**Then `BUG-1`.** Migration **v6**, dropping the `finding_id` foreign key entirely — not merely its
cascade. A `BundleItem` legitimately refers to a Finding **that may no longer exist**, which is the
precise condition a foreign key exists to forbid. `KEEP` the `bundle_id` cascade; `bundle_store.rs:277`
asserts it and `FR-14` requires it.

Rows already lost are **not recoverable**, and the migration must not pretend otherwise.

**Then the remaining swallows** at `:72-74`, `:135` and `:137`. Note that `:72` hides a second swallow
inside the first: the `if let Ok(...)` on `VaultBlobStore::new` simply does not run, and composition
continues as though the Markdown had been written.

## The tests are the point of this story

Nine are named in the spec. Two classes of them **do not exist anywhere in this repository**, and their
absence is exactly why five waves passed over both defects:

- a test that a cascade **does not** fire
- tests that exercise a **failing** filesystem and a **failing** unpublish

Every store test in this repo today uses a writable temp directory and a live path. A test that only
ever sees the happy path cannot catch a swallowed failure.

**A test that cannot fail is a review finding.** If you assert a literal that the production code also
computes from the same constant, you have written a tautology. Assert the behaviour.

## Migration numbering — settled, do not re-derive

Highest existing version is **v5** (`publication`). This story takes **v6**. `W6-S4` takes v7 later in
the chain; the wave was sequenced so they never race.

## Not frozen work, and the reasoning matters

`BUG-9`'s second path reaches `sharing` behaviour and `DEC-005` freezes `sharing`. **This is still not
frozen work:** the swallow is in `bundle.rs`, the fix is in `bundle.rs`, and `DEC-005` says in its own
text that it does not forbid fixing a defect in what already shipped. Do not widen the change into
`apps/web-service` or the publish client.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed**, and three traps have been caught on this repo: `cmd | tail`
  reports `tail`'s exit code and hid a failing `cargo build`; a long-lived worktree goes stale the
  moment a story adds a dependency; and `cargo build` is not `tauri build`.
- **Write UTF-8, no BOM.** A story spec came back cp1252 on `W6-S2` and BOM-prefixed on `W6-S9`.
- **No scratch files in the commit**, never a captured screenshot, and **do not push.**

## Done means

The spec's frontmatter reads `status: done`, the nine named tests exist and pass, and
`cargo test --workspace` is green.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
