# 14: Edit and Save a Bundle

**What to build:** In Review & Update, **Edit** unlocks exactly four kinds of field — the Bundle's
title, its notes, each Finding's note, each Marker's note — and the badge flips to **Editing**. Images
are frozen (no add, remove, reorder or replace) and carry a small **Fixed at compose** chip in edit
mode only. Edits stay in a buffer; nothing touches a Finding (`BR-10`, `BR-11`). **Save** is always
clickable once editing. On Save the edited blocks go back to the composer, which writes the document
again (`AD-9`), and the Bundle's title and document are stored together under `BR-5`: the file is
written first, atomically (a temporary file beside it, then a rename), then the row; if the row write
fails, the previous file content — held in memory for exactly this — is written back, the Reviewer is
told which part refused, and the buffer survives so Save can be tried again. A Save whose blocks
serialise to the stored document with the title unchanged writes nothing and toasts **"Saved.
Nothing had changed."**; otherwise **"Saved."** and the window returns to locked showing the new text.
**Cancel** returns to locked at once when the buffer equals the stored document, and confirms first
when it does not. The store gains an update operation that writes name and document together; the
document-only update nothing calls today is subsumed by it.

**Blocked by:** 13

**Status:** ready-for-agent

- [ ] Edit unlocks the four field kinds and only those; the badge reads Editing; each image carries the
      Fixed at compose chip in edit mode and none in locked mode
- [ ] Editing a Finding's note in the Bundle leaves that Finding's own note, and every other Bundle
      holding it, byte-identical — asserted at the store seam
- [ ] Save with a changed title and a changed note produces a stored document whose heading and that
      note read the new text and whose images and every other line are unchanged (`FR-40`'s proof);
      the file on disk and the row hold the same document
- [ ] Save with nothing changed writes neither file nor row (asserted by modification time and a
      byte comparison) and toasts "Saved. Nothing had changed."
- [ ] Write ordering: with a failure injected after the file rename and before the row write, the file
      is restored to its previous content, the row is unchanged, the toast names the part that refused,
      and the edited text is still in the fields
- [ ] Cancel with an untouched buffer returns to locked immediately; Cancel after typing asks first
- [ ] Editing and saving a sealed Bundle works identically
- [ ] Each guard seen red first: the ordering test with the restore step removed, the no-op test with
      the comparison removed
- [ ] Callback-reachability and component-wiring tests cover the new callbacks
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** fix a typo in a Bundle's note and Save; open the Markdown file and confirm the
      change; open the original Finding in the Editor and confirm its note did not change. Press Save
      twice — the second toast must say nothing had changed. Type, then Cancel — it must ask
