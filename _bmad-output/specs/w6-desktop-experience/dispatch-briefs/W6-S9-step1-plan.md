# W6-S9 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first — the
second pitfall in its `## Code` section is what half this story is about.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S9`

## Two defects, both `bundle`, both about telling the truth when something fails

### `BUG-1` — deleting a Finding silently guts every Bundle that holds it

`crates/snapdown-store/src/sqlite/migrations.rs`, migration v3:

```sql
FOREIGN KEY(finding_id) REFERENCES finding(id) ON DELETE CASCADE
```

and `finding_store.rs:29` sets `PRAGMA foreign_keys = ON`. So deleting a Finding deletes its
`bundle_item` rows.

`FR-13`'s third consequence has been `active` since G2 and says the opposite:

> *A Finding that belongs to a Bundle can still be deleted; the Bundle keeps its own copy of the
> image and stays readable.*

`bundle.markdown` is a column, so `AD-9` holds and the document still reads correctly and still
copies the same bytes. **Only the item list loses a row.** The delivered document and the record of
the delivered document disagree, and nothing reports it.

**Prefer dropping the foreign key entirely, not just its cascade.** A `BundleItem` legitimately
refers to a Finding **that may no longer exist**, which is the precise condition a foreign key exists
to forbid. `KEEP` the `bundle_id` cascade — deleting a Bundle *does* delete its items (`FR-14`), and
`bundle_store.rs:277` asserts it.

Rows already lost are **not recoverable**. The migration must not pretend otherwise.

### `BUG-9` — `bundle.rs` swallows every failure its invariants depend on

Three paths in `apps/desktop/src-tauri/src/commands/bundle.rs`, and they are not equally bad:

| Line | What is swallowed | Why it matters |
|---|---|---|
| `:72-74` | The Markdown write **and** the `VaultBlobStore::new` open | The Bundle row is committed anyway, with `markdown_path` naming a file that does not exist. `FR-10` is all-or-nothing; `AD-2` forbids committing a record before its files exist |
| **`:116`** | **`unpublish_bundle`** | **The serious one — see below** |
| `:135`, `:137` | The Markdown file and image-copy deletes | Orphaned files, unreported. `FR-14`, `NFR-5` |

**`:116` is the one to plan first.** `delete_bundle` swallows the unpublish. The unpublish fails, the
deletion proceeds, the local record is gone, and **the published copy stays live on the public
internet with nothing pointing at it.** The Reviewer believes they deleted it.

`BR-20` was written to forbid exactly that outcome:

> *An unpublish that fails leaves the Bundle marked published. The Reviewer is never told something
> is private when it may not be.*

The fix follows from the rule: on a failed unpublish, **abort the delete and report it.** Never delete
a local record whose remote copy may still be live.

Note that `:72` also swallows the `VaultBlobStore::new` failure — the `if let Ok(...)` simply does not
run and composition continues as though the file had been written. That is a second swallow hiding
inside the first.

## Migration numbering — already settled, do not re-derive it

The highest existing version is **v5** (`publication`). This story takes **v6**. `W6-S4` adds the
Quality Budget migration later in the chain and takes v7. The wave was sequenced so that these two
never race; you do not need to coordinate with `W6-S4`.

## Why this story is fifth

The wave was resequenced risk-first on 2026-08-23. `BUG-1` corrupts the record of what was handed
over and `BUG-9` can leave a deleted Bundle live on the internet — both outranked six stories of UI
work that were originally ahead of them purely because of how the dependency graph had been
linearised. Nobody had chosen that order.

## Not frozen work, and the reasoning matters

`BUG-9`'s second path reaches `sharing` behaviour, and `DEC-005` freezes `sharing`. **This is still
not frozen work:** the swallow is in `bundle.rs`, the fix is in `bundle.rs`, and `DEC-005` says in its
own text that it does not forbid fixing a defect in what already shipped. Do not widen the change into
`apps/web-service` or the publish client.

## The tests that matter

`waves.yaml` records nine for this story. The shape to aim for: **a test that a cascade does *not*
fire**, and **tests that exercise a failing filesystem and a failing unpublish**. Neither class exists
in this repository — every store test uses a writable temp directory and a live path, which is exactly
why five waves passed over both defects.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed**, and two traps were caught on 2026-08-23: `cmd | tail` reports
  `tail`'s exit code, and a long-lived worktree goes stale the moment a story adds a dependency.
- **Write UTF-8.** A story spec came back in cp1252 on `W6-S2`.
- **No scratch files in the commit**, never a captured screenshot, and **do not push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S9-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
