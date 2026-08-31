# Answered

**Loaded when:** looking for a decision that was reached through a question, not through a `DEC-`.

Rows move here from the other three lists. MUST NOT be deleted.

## Answered

| id | Question | Answer | Closed |
|---|---|---|---|
| OQ-29 | Seven behaviours the owner asked for on 2026-08-28 had no `FR-` under any capability: canvas **zoom**, user **resize** of a stored image, **paste** from the clipboard, **copy** a burned image, a **context menu**, **crop**, and **undo/redo** | **Four became requirements, one already was one, two are refused.** `FR-34` zoom, `FR-35` paste, `FR-36` copy burned image and `FR-37` context menu are now in `requirements.yaml` and in PRD §4.9. `undo`/`redo` needed nothing: `FR-33`'s consequences already say *"Redo/Undo history is supported for canvas additions, moves, edits, and deletions"* - the assumption was written from a defect row that had misread it, and the code now implements it. `crop` and destructive `resize` are REFUSED: both are named non-goals in `SRS-finding.md` and in the Product Brief, and the refusal is load-bearing rather than tidy - `AD-9` promises byte identity and the Vault keeps no second copy. Reversing that is a `DEC-`, not a story. The Crop toolbar button stays inert on purpose | 2026-08-28 |
| OQ-12 | Recomposing a bundle is acceptable in place of editing its written Markdown | **No — the assumption turned out false, and `FR-40` is what replaced it.** A Reviewer can now correct a composed Bundle's title, its Bundle notes, and the note text on the Findings inside it, saving as one act; recomposing is no longer what anyone does to fix a typo. Three things settled it, in order: the owner asked for it through the Bundle Library map, so `wdi-product` intent `update` landed `FR-40`; `BR-11` was narrowed the same day from *"A Bundle is never edited in place"* to *"changed only by the composer writing it again over the Bundle's own copy"*, with the old wording preserved in `.what/business-rules.md` § Amended; and `DEC-012` recorded the reading of `AD-9` that makes both legal — settled by `AD-9`'s own **Prevents**, not by a new argument. **The cost this row named did not materialise, and why is worth keeping:** it feared Bundles drifting from the Library that produced them. That drift is still guarded — `BR-10` keeps a Finding's edits out of a Bundle already holding it, and `FR-40` writes only the Bundle's own stored copy and never reads a Finding. The assumption's mistake was concluding that forbidding *all* editing was the only way to prevent it; the narrower rule prevents it by construction | 2026-08-31 |

A row arrives here by moving from `blocking.md`, `assumptions.md`, or `external.md`, keeping its
id. First arrival: `OQ-29`, 2026-08-28.

`OQ-12` arrived 2026-08-31 from `assumptions.md`. It went here rather than into that file's own
§ Answered table, which holds `OQ-21`: that row was kept beside the open assumptions because its answer
was a decision **not** to act and the reasoning had nowhere else to live. `OQ-12`'s answer is a decision
to act, and its reasoning is already held by `DEC-012`, `prd.md` § 4.4, `SRS-bundle.md` § Assumptions,
and `business-rules.md` § Amended — so the archive is the right home and no reasoning is lost by it.

**A note on `OQ-29` above, added 2026-08-31.** Its answer refuses `crop` and destructive `resize` partly
on the grounds that *"`AD-9` promises byte identity and the Vault keeps no second copy"*. `DEC-012`
narrowed `AD-9` on 2026-08-31, and a reader who checks `AD-9` now will not find the phrase. **The
refusal is unaffected and stands.** The byte identity it relies on is between *a Bundle's image copy and
the Finding's image*, which `DEC-012` does not touch — that decision is about the **document's image
links** on a handoff path. This note exists because the row is an archive that MUST NOT be rewritten,
and because the refusal it records is load-bearing.
