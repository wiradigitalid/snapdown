---
topic: Capture to Markdown — the desktop review loop
artifact: .what/_prd/capture-to-markdown/prd.md
updated: 2026-08-22T22:42
---

- (event) headless run, intent create. Two initiatives split by the reader test: someone looking for MCP or web publishing would not open this document
- (decision) CAP-1..CAP-6, FR-1..FR-18, NFR-1..NFR-8, UJ-1..UJ-4 allocated from requirements.yaml; the sequence continues into the agent-handoff PRD
- (decision) Note is written at capture time, inline at the region. Wins on every criterion set — recorded in the addendum because a non-trade-off should not be revisited as one
- (decision) editor does not auto-open after a capture; setting exists, default off
- (decision) image reduction happens once on the way in; the unreduced capture is not retained
- (decision) Marker coordinates are stored, not burned into the Finding image, because FR-8 requires repositioning and renumbering
- (decision) selection order is the only ordering inside a Bundle; no reorder step in r1
- (decision) hard deletion only. NFR-5 states the no-orphan property as an invariant, and FR-15 reports violations
- (assumption) not auto-opening the editor is what the Reviewer wants — OQ-9
- (assumption) reading cost tracks pixel area, so the long-edge cap is the dominant lever — OQ-2
- (assumption) recomposing is acceptable in place of editing a Bundle's Markdown — OQ-12
- (gap) default long edge of 1600px is a working answer, unmeasured against a real agent's reading cost — OQ-3
- (change) stack answered mid-run by the owner: Tauri v2 + Rust, desktop UI React + Vite + TypeScript (Svelte was the earlier plan). Lands as AD-N at G3, not in this PRD
- (event) PRD and addendum written; review lenses structure+prose applied at write time

---

## 2026-08-23 — update: § 4.7, the surface itself

**change** — Added `CAP-9` and `FR-27`–`FR-29` (name the surface · reach every surface · fit the
window), `NFR-16`–`NFR-18`, rewrote `FR-5`, amended `FR-18`. Target release `r3`.

**decision** — `CAP-9` is administered by `settings` even though `FR-28` and `FR-29` govern surfaces
`finding` and `bundle` own. `settings` already holds the container-level Logical Components — the
startup registrar, the hotkey registrar, the settings store — which are the app's own machinery
rather than any one screen's, and the window shell is machinery of the same kind. The alternative,
three copies of the same requirement under three components, is how a shell drifts. The defence is
written into § 4.7 itself so a later reader does not have to reconstruct it.

**decision** — `FR-5` was **amended in place, not replaced**. Its promise — "defaults the Reviewer
never has to change" — was already right. What failed was its presentation: two raw numbers a
Reviewer can accept and cannot judge. Retiring `FR-5` and issuing a new number would have implied the
original promise was wrong, and it was not. `DEC-004` is cited from the requirement text.

**change** — `FR-5` now carries a consequence that is deliberately awkward to satisfy: *Auto resolves
different parameters for a small region than for a full-screen capture, and a test finding them
identical is a failing test.* Without it "Auto" can be implemented as the old constant wearing a new
label, and every other consequence would still pass.

**decision** — `FR-18`'s new default (on after first run) is written so it applies to a **first run
nobody configured**, never to a Reviewer's decision. A default that re-asserts itself over a Reviewer
who turned it off is a bug wearing a default's clothes, and the requirement says so rather than
leaving it to the implementer.

**change** — `NFR-18` (store the resolved budget with each Finding) exists only because `FR-5` forbids
re-encoding an existing Finding. That interaction is not obvious: taken together they mean two
Findings captured a month apart on "the same" Auto setting can legitimately differ, and without the
stored record nothing can explain why. It was found by reading `DEC-004`'s Cost section, not by
reading the requirement.

**event** — ID collision caught and corrected during this update. The new requirements were first
numbered `FR-19`–`FR-21`, which are already held by `CAP-7` in the `agent-handoff` PRD. FR numbering
is global to the product and does not restart per PRD; the highest allocated was `FR-26`. They are
now `FR-27`–`FR-29`. `NFR-16`–`NFR-18` were checked against the same rule and are clean.

**assumption** — `OQ-3` is restated rather than closed by `DEC-004`, and § 8 says so in the document
rather than quietly dropping the old question. `OQ-18` and `OQ-20` added to § 8.

### Not done, and why

- **No use cases written for `FR-27`–`FR-29`.** The UC catalogue belongs to `wdi-blueprint` intent
  `catalog`, and these three components now sit at `mode: deep`, so the catalogue is being re-derived
  there rather than extended here.
- **No screen specifications.** `wdi-ux` owns those. § 4.7 states what must be true of a surface and
  deliberately does not say what it should look like — `FR-29` names a condition (nothing discovered
  only by scrolling), not a layout.
- **`agent-handoff` PRD untouched.** `DEC-005` freezes `sharing` and `agent-access`; editing their
  PRD would be new work on them.
- **`NFR-3` left alone.** It still names "the shipped default" for a 4K capture under 250 KB. Under
  Auto there is no single shipped default, so the wording is now loose. It is flagged here rather
  than changed, because tightening it needs the derivation to exist first — a finding for
  `wdi-reconcile`, not a silent edit.
