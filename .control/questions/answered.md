# Answered

**Loaded when:** looking for a decision that was reached through a question, not through a `DEC-`.

Rows move here from the other three lists. MUST NOT be deleted.

## Answered

| id | Question | Answer | Closed |
|---|---|---|---|
| OQ-29 | Seven behaviours the owner asked for on 2026-08-28 had no `FR-` under any capability: canvas **zoom**, user **resize** of a stored image, **paste** from the clipboard, **copy** a burned image, a **context menu**, **crop**, and **undo/redo** | **Four became requirements, one already was one, two are refused.** `FR-34` zoom, `FR-35` paste, `FR-36` copy burned image and `FR-37` context menu are now in `requirements.yaml` and in PRD §4.9. `undo`/`redo` needed nothing: `FR-33`'s consequences already say *"Redo/Undo history is supported for canvas additions, moves, edits, and deletions"* - the assumption was written from a defect row that had misread it, and the code now implements it. `crop` and destructive `resize` are REFUSED: both are named non-goals in `SRS-finding.md` and in the Product Brief, and the refusal is load-bearing rather than tidy - `AD-9` promises byte identity and the Vault keeps no second copy. Reversing that is a `DEC-`, not a story. The Crop toolbar button stays inert on purpose | 2026-08-28 |

A row arrives here by moving from `blocking.md`, `assumptions.md`, or `external.md`, keeping its
id. First arrival: `OQ-29`, 2026-08-28.
