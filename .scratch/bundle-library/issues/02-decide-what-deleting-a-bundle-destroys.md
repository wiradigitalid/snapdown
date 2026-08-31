# 02: Decide what deleting a Bundle destroys

**Type:** grilling
**Status:** open
**Blocked by:** None (can start immediately)

## Question

`Delete` is one of the four Library row actions. Decide precisely what it removes, and what the
Reviewer is told before it happens.

- **Blast radius.** `delete_bundle()` cascades to `bundle_item` rows in SQLite. But the Bundle also
  owns files in the Vault: `bundles/<id>/bundle.md` and one `finding_N_burned.png` per item. Are
  those deleted too, or left orphaned on disk? The archived Tauri implementation deleted both
  (`archive/desktop-tauri/src-tauri/src/commands/bundle.rs`, `delete_bundle_impl`).
- **What survives.** Confirm and state plainly that the source Findings are untouched. This is
  already true structurally: `bundle_item.finding_id` has carried **no foreign key** since migration
  v6 (`crates/snapdown-store/src/sqlite/migrations.rs:114-135`), and each `BundleItem` holds its own
  burned image copy — a Bundle already survives its source Finding being deleted, by design.
- **Confirmation copy.** What the dialog says, naming the Bundle, and whether it states that the
  Findings remain.
- **Reversibility.** Is this recoverable in any way, or permanent? If permanent, say so in the
  dialog.
- **Failure.** What the Reviewer sees when the DB row is deleted but a blob delete fails, and
  whether that leaves a half-deleted Bundle. Note the standing repo rule: `let _ =` on a Result an
  invariant depends on is a defect — `bundle.rs` has swallowed blob-delete errors before.

## Owner's answer, 2026-08-31 — narrows this ticket

**Delete removes the Bundle's whole folder: `<vault>/bundles/<id>/`, Markdown and images together.**

Tested and it holds: that folder contains exactly `bundle.md` plus this Bundle's own
`finding_N_burned.png` copies and nothing else, so a folder-level delete removes precisely what the
Bundle owns. Simpler than deleting blob by blob.

Two implementation constraints this creates:

- The Vault exposes `delete_blob` (single file) only — no directory delete. A new capability is
  needed and it must keep `resolve_path`'s traversal guard, which is the thing standing between a
  bad id and a recursive delete outside the Vault.
- Errors must not be swallowed. `AGENTS.md` records `bundle.rs` having done exactly this before with
  `let _ =` on blob deletes; a delete that half-fails silently is the worst outcome here.

**What is still open:** the failure ordering. Delete the files first or the DB row first? The
recommendation on the table is **files first, then the row** — if a file delete fails, the row
survives and the Reviewer can retry, whereas the reverse leaves invisible orphans nobody can find or
clean up. Also still open: confirmation copy, and whether the dialog states that the source Findings
remain.

**No unpublish step is needed.** A Bundle can never currently reach a published state: the
`Publication` / `SqlitePublicationStore` path is reachable from no CLI, MCP tool, or app code —
verified while charting. If Publish later ships, delete's interaction with a live Publication becomes
a question for *that* stage, not this one.
