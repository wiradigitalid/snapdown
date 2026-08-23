---
topic: Snapdown — settings component depth
artifact: .how/settings/SDD-settings.md
updated: 2026-08-23T22:00
---

- (event) 2026-08-23 G4 run for the FIRST time. This component sat at `mode: catalog` until today, and catalog skips G4 by design — so it had no local rules, no full flows, no state machines, no failure behaviour, no contracts, no data model. Every one of the owner's Settings complaints was a question the corpus had no slot to answer
- (change) behaviour: 02-rules born (BR-110..BR-121), three full flows (UC-13, UC-14, UC-15), 03-domain/state-machines.md, two scenarios (SCN-01, SCN-02)
- (decision) three flows and not seven. `deep` asks for at most 3 plus every `critical` UC, and this component has no `critical` UC at all. UC-16, UC-24, UC-25 and UC-26 deliberately have no full flow: UC-16's substance is in SCN-02 where the time dimension lives, and UC-24..26 are properties of a surface rather than sequences a Reviewer walks
- (event) BR NUMBERING COLLISION caught and fixed. The seven new cross-component rules were first written as BR-31..BR-37, which are already held by `finding`'s LOCAL rules. BR- is ONE global sequence that continues into every component's local file — max allocated was BR-102. They are now BR-103..BR-109, and this component's local rules start at BR-110
- (event) the first draft of the local rules file used an invented `LBR-S*` prefix. Wrong: this repo has no such prefix, local rules continue the same global BR- sequence. Rewritten
- (decision) BR-112 (the startup default) got a rule of its own, and the file argues why. A default that re-asserts itself is a bug wearing a default's clothes, and the distinction is invisible in code reading `if (!configured) enable()` — because *configured* is doing work nobody wrote down
- (change) BR-112 needs a tri-state — unset / on / off — which looks like it contradicts BR-111 (no unset state a caller handles) and does not: unset still READS as the default; it is additionally RECORDED, and only BR-112 ever looks at that
- (event) state-machines.md section 1 introduces `Unknown` as a RENDERED VALUE, not a spinner. That is what makes FR-18 satisfiable, and its absence is exactly the shipped defect: App.tsx initialises useState(true) and repaints to false
- (change) design: SDD taken to deep — Inherited Constraints (AD-2, AD-6, AD-10, AD-11 verbatim), Failure Behaviour over five boundaries, ABCE, plus contracts, data-model, the windows-shell integration, one flow, and LC-028
- (decision) the Failure Behaviour table admits, in one cell, that the product CANNOT detect a hotkey that registers and never fires. A periodic health check was rejected rather than deferred: it would be the only background task in the product, and NFR-6's 150 MB idle budget is written for a product that has none
- (event) SCN-01 was REWRITTEN after reading the code. The first draft assumed a move-file-by-file implementation and spent a paragraph on a rollback-of-the-rollback. The real implementation copies everything, verifies everything, and only then deletes the sources (vault_migration.rs:138) — so no file ever exists in neither place, and AD-2 is satisfied by ORDERING rather than by compensation. Considerably stronger than the document had assumed
- (event) but reading it found a REAL gap: both fs::remove_file calls swallow their result (:141 on the success path, :180 in the rollback). A source file that will not delete leaves an UNREPORTED DUPLICATE of an image that may hold personal data, and the move still reports success. Recorded as [MISSING] and as the highest-value item in the evidence table
- (note) NOT done: wdi-review has not run. The rule is that G4 MUST NOT open on depth that has not been through it
