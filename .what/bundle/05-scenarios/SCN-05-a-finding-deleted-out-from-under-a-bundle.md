---
type: scenario
id: SCN-05
component: bundle
branches_from: UC-12
created: "2026-08-23"
---

# SCN-05 — A Finding deleted out from under a Bundle

Branches from `UC-12`'s neighbourhood rather than its flow: it is the case where the *other*
component's deletion reaches this one. It is written because the as-built pass found the code and the
promise disagreeing here, and the disagreement is silent.

## The promise

`FR-13`, third consequence, verbatim:

> A Finding that belongs to a Bundle can still be deleted; the Bundle keeps its own copy of the image
> and stays readable.

`EXPERIENCE-bundle.md` states the same thing from the Reviewer's side, and `EXPERIENCE-finding.md`
requires the deletion confirmation to *say so*, because a Reviewer will otherwise assume the Bundle
broke.

## The setup

`checkout-pass-1` holds five Findings. The Reviewer deletes Finding 3 from the Findings surface,
confirming once. They then open Bundles.

## What must happen

1. The Bundle still exists and still opens.
2. Its Markdown is unchanged — five sections. It was composed once and stored (`AD-9`), and nothing
   recomposes it.
3. Its own copy of Finding 3's image is still in the Vault and still referenced.
4. The item list still shows five items, including the one whose source Finding is gone.
5. Copy Markdown still produces the same bytes it produced yesterday.
6. Publishing still works, and a published copy is unaffected.

Point 4 is the one worth arguing about, and it is right: a `BundleItem` is the **membership** of a
Finding in a Bundle, holding the position and the image copy that was written for it. The Finding
going away does not unmake the membership; the Bundle is a record of what was handed over, and
rewriting history because a source was tidied up would make a delivered document disagree with what
was delivered.

## What the code actually does — a defect

`crates/snapdown-store/src/sqlite/migrations.rs`, migration v3:

```sql
CREATE TABLE IF NOT EXISTS bundle_item (
    ...
    FOREIGN KEY(bundle_id)  REFERENCES bundle(id)  ON DELETE CASCADE,
    FOREIGN KEY(finding_id) REFERENCES finding(id) ON DELETE CASCADE,
    ...
);
```

The second cascade is the problem, and it is live: `finding_store.rs:29` sets
`PRAGMA foreign_keys = ON`.

**Deleting a Finding deletes its `bundle_item` rows.** Points 4 and 5 above fail. The Bundle's stored
Markdown survives — `bundle.markdown` is a column, so `AD-9` still holds — and its item list silently
loses a row, so the document and the record of the document disagree.

Nothing reports it. No test covers it: `finding_store.rs:362` asserts the cascade to `note` and
`marker`, which is correct, and nothing asserts the *absence* of one to `bundle_item`.

The first cascade, `bundle_id`, is correct and should stay — deleting a Bundle does delete its items
(`FR-14`).

## Disposition

A `BUG-`, not planned work. The promise is older than the schema and was never withdrawn; this is code
disagreeing with a requirement that has been `active` since G2, not a requirement written after the
code.

The fix is to drop `ON DELETE CASCADE` from `bundle_item.finding_id` — the reference becomes a
historical one — and to decide whether the column keeps its foreign key at all. It probably should
not: a `BundleItem` refers to a Finding **that may no longer exist**, which is precisely what a
foreign key exists to forbid.

## Tests this scenario names

- `bundle::deleting_a_finding_leaves_its_bundle_item_in_place`
- `bundle::deleting_a_finding_leaves_the_bundle_markdown_byte_identical`
- `bundle::deleting_a_finding_leaves_the_bundles_own_image_copy_in_the_vault`
- `bundle::a_bundle_whose_source_finding_is_gone_still_copies_the_same_bytes`
- `bundle::deleting_a_bundle_still_cascades_to_its_items`
