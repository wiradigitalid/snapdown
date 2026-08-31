# 09: Decide whether an edited Bundle says so

**Type:** grilling
**Status:** open
**Blocked by:** -

## Question

A Bundle can now be edited (`FR-40`, and ticket 05 settled the window). Nothing anywhere records
that it was.

Verified in the code, 2026-08-31:

- The `bundle` table is `id / name / markdown / markdown_path / composed_at`
  (`crates/snapdown-store/src/sqlite/migrations.rs:66`). **No `updated_at`.** A `Finding` has one
  (`crates/snapdown-core/src/domain/finding.rs:50`); a Bundle does not.
- The Library lists `ORDER BY composed_at DESC`
  (`crates/snapdown-store/src/sqlite/bundle_store.rs:201`).
- The Library row's meta line and Review & Update's header both show composed time. The row's shape
  was settled by ticket 01, so changing it reopens that decision's surface, not its reasoning.

So a Reviewer who corrected a Finding's note an hour ago opens the Library and reads
*"composed yesterday"*. That is true and it is not the whole truth.

**Decide:**

- **Does a Bundle gain an `updated_at`?** That is a migration, and every existing Bundle needs a
  value - most cheaply `composed_at`, which then reads as "never edited" and is honest.
- **If it does, does the Library row show it?** Only when it differs from `composed_at`, or both, or
  keep the row as it is and surface it only inside the window.
- **Does it change the sort?** Newest-first by composed time is what ticket 01 settled. Sorting by
  edit time instead makes a typo fix jump a Bundle to the top, which may be exactly right or exactly
  wrong depending on what the Reviewer came to the Library for.

**The honest alternative, which must not be dismissed:** edits stay **silent**. `composed_at` is the
Bundle's identity, an edit is a correction to a document rather than a new event, and a product that
shows every timestamp it holds is not thereby clearer. Ticket 05's finding - that a no-op `Save`
cannot disturb anything a Reviewer can see - is true *because* there is no `updated_at`. Adding one
gives the always-clickable `Save` a visible side effect it does not have today, and that is a real
cost rather than a detail.

**Do not reopen:** ticket 01's row shape beyond its meta line, ticket 05's window, or the decision
that `Save` is always clickable.
