---
type: component
lc: LC-013
name: bundle-store
component: bundle
container: desktop-app
created: "2026-08-23"
---

# LC-013 — bundle-store

The store over `bundle` and `bundle_item`, and the object `BUG-1` lives in.

## Responsibility

Read and write Bundles and their memberships, in units of work that satisfy `AD-2`, and hold the one
set of Markdown bytes that `AD-9` makes authoritative.

## The invariant this store owns, and currently breaks

> A `BundleItem` is a **membership**, not a pointer. It survives the deletion of the Finding it
> recorded.

That follows from `FR-13` and from what a Bundle is for: a record of what was handed over. The store
enforces the opposite — `bundle_item.finding_id` carries `ON DELETE CASCADE`, and
`finding_store.rs:29` sets `PRAGMA foreign_keys = ON`, so the membership is deleted with its source.

The fix is a migration dropping the constraint, most likely the whole foreign key rather than only its
cascade: this column refers to a row that may legitimately be absent, which is the exact condition a
foreign key exists to forbid. Rows already lost are not recoverable.

## What it must not do

- **Regenerate Markdown.** `bundle.markdown` is the truth; `MarkdownWriter` produced it once.
- **Reorder items.** `position` is fixed at composition (`BR-58`).
- **Commit a row before its files exist.** `AD-2`.
- **Delete a Bundle's image copies while another Bundle references them.** Each Bundle has its own
  copies, so this cannot arise — but it is the assumption the deletion path rests on and it is worth
  writing down.

## Boundaries

| Direction | With | Contract |
|---|---|---|
| in | `LC-010 bundle-composer` | Everything a composition writes, as one unit |
| in | `LC-014 bundles-editor` | Reads |
| in | `LC-017 local-api-server` | Reads only (`AD-5`) |
| in | `LC-020 publish-client` | Reads, plus the unpublish cascade (`BR-23`) |
| out | `library.db`, `LC-005 vault-blobs` | |

Four readers and one writer. The read-only ones matter: `AD-5` says every surface outside the desktop
process is read-only, and this store is where that is actually true or not.

## Tests it owns

Existing: `bundle_store.rs:277` asserts that deleting a Bundle cascades to its items — correct, and it
should stay.

Missing, and this is the shape of the gap: nothing asserts that deleting a **Finding** does *not*
cascade here. A test that a cascade does **not** fire is one nobody writes unless a document says the
cascade must not exist. That document did not exist until this gate.
