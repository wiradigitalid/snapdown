---
topic: Snapdown — bundle component depth
artifact: .how/bundle/SDD-bundle.md
updated: 2026-08-23T22:00
---

- (event) 2026-08-23 G4 re-run at `deep`, raised from an inherited `outline`. g4_passed reset for the same reason as `finding`
- (change) behaviour: 03-domain/state-machines.md and SCN-05. design: Inherited Constraints and Failure Behaviour written for the FIRST time — at `outline` the SDD stopped after section Structure — plus ABCE, contracts, data-model, and LC-013
- (event) **BUG-1 FOUND.** bundle_item.finding_id carries ON DELETE CASCADE to finding(id), and finding_store.rs:29 sets PRAGMA foreign_keys = ON. Deleting a Finding DELETES ITS BUNDLE_ITEM ROWS. FR-13's third consequence has been `active` since G2 and says the opposite: "A Finding that belongs to a Bundle can still be deleted; the Bundle keeps its own copy of the image and stays readable"
- (event) the damage is quiet, which is what makes it bad. bundle.markdown is a column, so AD-9 holds: the document still reads correctly and still copies the same bytes. Only the ITEM LIST loses a row. The delivered document and the record of the delivered document disagree, and nothing reports it
- (decision) registered as BUG-1, a DEFECT and not planned work. The requirement predates the schema and was never withdrawn — this is code disagreeing with something already `active`, which is exactly the line defects.yaml exists to hold. Planned work is where a requirement is NEW
- (event) why it went undetected for five waves: finding_store.rs:362 asserts the cascade to `note` and `marker`, which is correct. NOTHING asserts the ABSENCE of a cascade to bundle_item. A test that a cascade does NOT fire is one nobody writes unless a document says the cascade must not exist — and that document did not exist until this gate. This is the clearest argument in the whole pass for why raising `mode` was worth the cost
- (decision) the fix is probably to drop the FOREIGN KEY entirely, not only its cascade. A BundleItem legitimately refers to a Finding THAT MAY NO LONGER EXIST, which is the precise condition a foreign key exists to forbid. Rows already lost are not recoverable
- (decision) state-machines section 2 draws the BundleItem's relationship to its source, and drawing it is what surfaced the bug. `Orphaned` here means something DIFFERENT from `finding`'s `Orphaned`: there it is a fault to be repaired, here it is the normal, correct end state of a Bundle that outlived the Findings that fed it. The name collision is recorded rather than renamed away
- (decision) MarkdownWriter being PURE is the load-bearing choice behind AD-9. One function, called once, output stored in a column — three code paths cannot drift because there is nothing to keep in step. Storing both bundle.markdown and bundle.markdown_path is duplication on purpose: the column is the truth for every in-product path, the file is what a Reviewer or an agent opens from the Vault
- (note) [PARTIAL] filed: whether a composition failing partway leaves image copies in the Vault was not verified
- (note) NOT done: wdi-review has not run
