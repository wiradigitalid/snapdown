# 02: Decide what deleting a Bundle destroys

**Type:** grilling
**Status:** resolved
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

## Settled 2026-08-31 — and the question turned out to be bigger than deletion

Design canvas (row states, both menus, all four dialogs, Reclaim space and its empty state):
https://claude.ai/code/artifact/6f798d70-77a0-491e-973b-92e7a2641a2f

**The artboard source is in the repo** at `.scratch/bundle-library/design/`, so whoever builds this
reads exact hex, px and weights rather than measuring them off a picture. See its `README.md` for
what each file shows and which running-app component each value came from.

**A premise had to be corrected first.** The owner's model was that assembling *moves* a Finding's
image into the Bundle. It does not: `plan_bundle` only **reads** the Finding's image, burns markers
into a copy, and writes a new blob under `bundles/<id>/`. The only code that deletes a Finding's
image anywhere in the app is `delete_finding_everywhere` (`main.rs:1430`), reachable solely from the
explicit Delete Finding dialog. What looks like the Finding disappearing after assembly is a **view
filter**: `load_findings_into_window` hides any Finding a Bundle holds, computed live from
`list_bundles()`. Nothing is destroyed, and nothing is moved.

That matters because it means **deleting a Bundle already returns its Findings to the filmstrip** —
the `bundle_item` rows were the only thing hiding them. So "Delete" and a hypothetical "Disassemble"
would have been two names for one operation.

**A Bundle therefore has two states, and the schema already anticipated both.** Migration v6
(`migrations.rs:114-135`), titled *"drop finding_id foreign key constraint from bundle_item table"*,
deliberately removed the FK so a Bundle survives its source Findings being deleted. A Bundle whose
originals are gone is not a broken state to guard against; it is the state that change was made to
allow. No new column is needed to tell them apart — check whether the item's `finding_id`s still
resolve.

| State | Actions |
|---|---|
| Still holds its Findings | `Disassemble…` · `Discard originals…` · `Delete…` |
| Originals discarded | `Delete…` |

While a Bundle still holds its Findings, three outcomes are possible, and each gets its own named
action. The question a Reviewer is really answering is **what do you want to keep**:

| Action | Bundle | Original captures |
|---|---|---|
| `Disassemble…` | goes | **stay** |
| `Discard originals…` | **stays** | go |
| `Delete…` | goes | go |

- **Disassemble** — the Bundle's folder goes; its Findings become assemblable again. Named for what
  it does rather than "Delete", at the owner's direction: the name must match the behaviour.
- **Discard originals** — the source Findings go (rows, notes, markers, annotations, original image
  blobs); the Bundle keeps its own burned copies and stays readable, sealed. Named for what it
  destroys, following the same rule that chose Copy/Export/Publish over "Share".
- **Delete** destroys both. It was first left out on the grounds that the most destructive act should
  not sit one click away beside harmless ones — the owner corrected this, rightly. **All three of
  these actions are destructive and irreversible**; the harmless ones (Edit, Copy Markdown, Open file
  location, Export PDF) are already separated by a divider. The guard is the confirmation dialog, not
  the absence of the menu entry. Forcing a two-step for a common intent also made the safer-looking
  path the *less* clear one: it means two dialogs, and a Reviewer who has to understand the sealed
  state to know they are on the right route.
- **Menu order runs least to most destructive** — Disassemble, Discard originals, Delete — so the
  ordering itself carries a warning.
- **The dangerous Delete carries a different button label: `Delete both`, not `Delete`.** A Reviewer
  who has deleted several sealed Bundles learns the gesture *menu → Delete → Delete*. That muscle
  memory has to break at the one moment where the identical gesture also destroys captures, and the
  button is where it breaks — a heading nobody re-reads will not do it.
- **No danger colouring on the menu entries.** `semantic-error` exists, but `SdContextMenu` has no
  destructive variant today, and the design-system guide requires a new visual treatment to become a
  component before it is used twice. Plain `text-primary` for all three; the dialogs carry the weight.

**UI decisions, drawn on the canvas:**

- Only the exception state is marked: a `radius-pill` chip reading `Originals discarded`
  (`bg-subtle`, `text-muted`, 10px). Pill radius is the one shape the design-system guide reserves
  for a status chip, so this is the sanctioned use. Normal rows carry nothing, keeping the list quiet.
- On a sealed row `Disassemble…` stays **visible but disabled**, with `no originals` in
  `SdContextMenu`'s existing hint column. A menu that changes shape between rows confuses more than a
  greyed row that explains itself. This is deliberately unlike `Publish — soon`, where the feature
  does not exist at all.
- All three dialogs follow the existing Delete-Finding dialog exactly: 20px padding, 14px spacing,
  11px/800 uppercase question heading, 12px body, 30px buttons 8px apart, every one closing on
  *"This cannot be undone."* The cancel verb follows the object — `Keep it` for a Bundle,
  `Keep them` for captures.

**Bulk: a `Reclaim space` screen, two doors.** Chosen over row multi-select and over a single
"discard all" button. It lists the Bundles still holding originals with each one's size and a running
total — the actual input to the decision, which a row list cannot show — and is reached from the
Library header *and* from Settings' Vault section. Settings is only a door: the destroying happens
where the Bundles are visible, never behind a number.

**This grew the scope.** `Discard originals` and `Reclaim space` are new promises with no `FR-`, and
are now folded into
[Grow the promises this map needs](08-grow-pdf-export-and-bundle-rename-into-scope.md).

## Failure ordering — settled

**Files first, then the database row.** Owner's call, 2026-08-31.

The property that matters is not atomicity, it is that a failure stays **visible and retryable**. If
a file delete fails with this ordering, the row survives: the Bundle is still listed, possibly
broken, and deleting it again finishes the job. The reverse ordering — row first — leaves orphaned
files that nothing in the product can see, find, or clean up.

A SQLite transaction wrapping the file deletes was considered and rejected: **filesystem deletes do
not roll back**, so the transaction would produce the identical half-deleted folder while adding
machinery that only looks safer. A staging rename (`bundles/<id>` → a hidden name, delete the row,
then remove) genuinely is atomic and was also rejected, for cost rather than correctness: it buys
that guarantee by inventing a hidden-folder category that then needs its own sweeper, which nobody
would write. Reach for it only if half-deletes prove real in practice.

The same ordering governs `Discard originals`: the Finding blobs and rows go before the Bundle is
marked sealed, so a failure leaves the originals still present and the operation repeatable.

Errors must not be swallowed at any step. `AGENTS.md` records `bundle.rs` doing exactly that with
`let _ =` on blob deletes, and a delete that half-fails in silence is the worst outcome available
here.

## Resolved

All questions answered. What deleting destroys, the two-state lifecycle that replaced a plain
delete, the naming, the dialogs, the bulk surface, and the failure ordering are all settled above.
Two of the actions it produced are new promises and moved to
[Grow the promises this map needs](08-grow-pdf-export-and-bundle-rename-into-scope.md).
