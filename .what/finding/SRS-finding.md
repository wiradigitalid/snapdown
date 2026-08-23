---
type: srs
component: finding
status: draft
created: "2026-08-22"
updated: "2026-08-22"
satisfies: [FR-1, FR-2, FR-3, FR-4, FR-6, FR-7, FR-8, FR-9, FR-13, FR-15, NFR-1, NFR-2, NFR-3, NFR-4, NFR-5]
reviewed:
  date: '2026-08-23'
  sha: '783a561'
  lenses: [structure, prose, edge-case-hunter]
---

# SRS — finding

## Decision Summary · [G3]

This component is the whole life of one observation. It puts the Capture Overlay on the screen when
the Reviewer presses the hotkey, takes the region they drag, accepts the Note they type there, reduces
the image, and stores the three as one thing. Afterwards it is what the Editor shows: the list of
Findings, the Note that can be reworded, the numbered Markers that can be placed and moved, the
multi-select that makes bulk action possible, and the deletion that takes the image file with it.

Two promises decide almost every question here. The first is that a Note and its image are one object
from the moment of capture, and that a Marker's number and its Note line are one thing rather than two
kept in step. The second is that deleting is real: the file goes, nothing is archived, and the
Reviewer is told if a file refused to go rather than being told it succeeded.

It is set to `mode: guarded` and `risk_accepted: low`. A Capture can contain anything that was on the
screen, and deletion cannot be undone — so every boundary here gets an answer for what happens when
the other side is slow, absent, or lying, and the review is the hardest the method offers.

## Why · [G3]

Because the binding between an observation and its evidence is the product, and the binding is created
and destroyed here. No other component creates a Finding and no other component may write one. If this
boundary were drawn anywhere else — around "capture" on one side and "the library" on the other — the
Note would be born in one component and edited in another, and the one invariant that matters would
span a seam.

## Actor Register · [G3]

| Actor | Who they are | What they may do |
| --- | --- | --- |
| Reviewer | The person operating Snapdown. The only human actor and the only writer in the product | Press the Capture hotkey, drag a region, cancel a Capture, type and reword a Note, place, move and remove Markers, select Findings, delete Findings, act on an orphan report |

No second actor. An agent never reaches this component: `agent-access` and `sharing` read Bundles, and
BR-14 keeps an unbundled Finding invisible to both.

## UC Catalogue · [G3]

| id | Use case | Actor | Satisfies | critical |
| --- | --- | --- | --- | --- |
| UC-1 | I press a key, box the thing that is wrong, and say what is wrong with it | Reviewer | FR-1, FR-2 | no |
| UC-2 | I take five of these in a row without stopping | Reviewer | FR-3 | no |
| UC-3 | I look at everything I have captured so far | Reviewer | FR-6 | no |
| UC-4 | I reword a note now that I have read it back | Reviewer | FR-7 | no |
| UC-5 | I point at three separate spots inside one screenshot | Reviewer | FR-8 | no |
| UC-6 | I pick out several findings at once | Reviewer | FR-9 | no |
| UC-7 | I throw away the findings I am done with | Reviewer | FR-13 | yes |
| UC-8 | I find out what has gone missing or been left behind | Reviewer | FR-15 | no |

One of eight is `critical`, and it is UC-7: deleting a Finding removes a file from disk and cannot be
undone. FR-4 has no use case of its own and says so in the registry — image reduction has no actor and
no initiating step; it is a property of UC-1 and UC-2, asserted by NFR-3.

## Constraints · [G3]

| Constraint | Source |
| --- | --- |
| A Marker and its numbered Note line are one stored thing. No path writes one without the other | AD-1, BR-1 |
| A record and its files are created and removed in one unit of work; a partial failure leaves the prior state | AD-2, BR-5 |
| Marker positions are fractions of the image, never pixels | AD-3, BR-2 |
| The image is reduced once, at capture, and the unreduced pixels are not retained | AD-4, BR-8 |
| No network call may happen anywhere in this component | AD-6, NFR-4 |
| Windows 11 is the only capture platform, and the capture path may not be built against a cross-platform abstraction first | Product brief, Constraints |
| Hotkey registration and capture must work without administrator rights | NFR-7, OQ-5 |
| Nothing is soft-deleted | BR-7 |
| A change to the Quality Budget never re-encodes a stored image | BR-9 |
| The Quality Budget and the Vault location are read from `settings`; this component does not own either | `components.yaml` → `owns` |

## Non-Goals · [G3]

- **Composing anything.** Turning Findings into a document is `bundle`. This component does not know
  Bundles exist.
- **Owning the settings it obeys.** The Quality Budget, the Vault location, and the hotkey bindings
  belong to `settings`. This component reads them.
- **Exposing anything to an agent.** `agent-access` and `sharing` do that, and only for Bundles.
- **Annotation beyond numbered Markers.** No arrows, callouts, blur, redaction, or freehand. Not in
  this release and not later; it is a product Non-Goal.
- **Editing captured pixels.** No crop, rotate, or resize after capture.
- **Searching or filtering the Library.** Out of MVP scope, and it belongs to the store adapter when
  it arrives.
- **Deleting a Bundle.** That is `bundle`, per FR-14.

## Prerequisite · [G3]

- A Windows 11 machine with at least one monitor. Capture cannot be verified headlessly, which is why
  the capture tests are the one suite that must run on a real desktop session.
- A writable Vault location. `settings` supplies a default so that BR-28 holds — capture works before
  anything is configured.
- Nothing external. This component has no third-party dependency and no row in
  `.control/questions/external.md`.

## Success Signal · [G3]

Six Captures started from the application under review, in ninety seconds, produce six Findings with
six correct Notes, with no Snapdown window ever needing to be dismissed — and after deleting three of
them the Vault contains exactly three image files.

Measured: the overlay appears within 200 ms (NFR-1), the save returns focus within 500 ms (NFR-2), a
full-screen 4K capture is stored under 250 KB at the shipped default (NFR-3), and no deletion ever
leaves an orphan (NFR-5).

## Assumptions, Risks, and To Be Confirmed · [G3]

### Assumptions

- Windows global hotkeys register from a user-level process without administrator rights — OQ-5.
- A UI screenshot at a 1600 px long edge, lossily re-encoded, stays legible enough that the Reviewer
  does not reach for the original — OQ-3.
- Numbered Markers are sufficient annotation for a machine audience — OQ-4.
- Not auto-opening the Editor after a Capture is what the Reviewer wants — OQ-9.
- One Vault at a time is enough — OQ-11.

### Risks

- **Per-monitor DPI scaling.** The selected rectangle and the pixels it maps to can disagree when
  monitors have different scale factors. This is the most likely source of a wrong-region capture, and
  it is why the overlay is per monitor rather than one window over the virtual desktop.
- **Reduction on the save path.** NFR-2 gives the save 500 ms and NFR-3 requires real encoding work.
  If reduction is not moved off that path, one of the two numbers will be missed, and the Editor has
  to tolerate a Finding whose image is still being written.
- **Deletion atomicity on Windows.** A file held open by another process cannot be removed, and BR-5
  requires that nothing then be removed. Getting this wrong in the convenient direction produces
  exactly the orphans NFR-5 forbids.
- **Marker renumbering.** BR-2 requires no gaps, and the renumber touches Markers and Note lines at
  once. A partial renumber is a silent mis-attachment, which is the one defect the product cannot
  survive.

### To Be Confirmed

- Whether a Finding that belongs to a Bundle may still be deleted from the Library. FR-13 says yes and
  BR-12 makes it safe; the PRD lists it as open question 1.
- The shipped default long edge — OQ-3.

## Gate Checklist · [G3]

At `mode: guarded` the starred questions are answered and the rest are answered where they apply.

| Question | Answer |
| --- | --- |
| ★ Is every use case title a sentence a user would say? | Yes. All eight are written in the Reviewer's own voice, first person |
| ★ Any `FR` with no use case? | FR-4 only, and it carries `no_uc` with its reason. The validator confirms it |
| ★ Do the inventories and this catalogue describe one system? | Yes. Tables 1–3, screens 1–7, and no endpoint — this component has no interface of its own, which is correct |
| Actor list: is one missing, or are two the same person? | One actor. No agent reaches this component, per BR-14 |
| Does every `AD-N` here name a concrete failure, and would breaking it in one component break another? | AD-1, AD-2, AD-3, AD-4 all do, and all four are read or written by `bundle` as well |
| Which business rule am I not sure is right? | BR-12 — one Finding in several Bundles, each with its own image copy. It is what makes FR-13 safe, and it is also the reason a Bundle survives its source Finding |
| Is there a term I have to guess the meaning of? | No. Every noun is in `.control/product-glossary.md` |

## Design Reference · [G3]

Paired with `.how/finding/SDD-finding.md`.

Binding invariants: **AD-1** (Markers and Note lines are one sequence), **AD-2** (a record and its
files live or die together), **AD-3** (Marker coordinates normalised), **AD-4** (reduce once, at
capture), **AD-6** (nothing leaves the machine). No applied `DEC-` binds this component yet.

---

## Slots

`02-rules/rules-finding.md` — written at G4, `mode: guarded`.
`03-domain/domain-model.md` — written at G3, present.
`04-usecases/` — at most three full flows at `guarded`, written at G4.
`05-scenarios/` — not written below `mode: deep`.

## Open Items

- OQ-3 — the default long edge, unmeasured. `.control/questions/assumptions.md`.
- OQ-4 — numbered Markers as sufficient annotation. `.control/questions/assumptions.md`.
- OQ-5 — hotkeys without administrator rights. `.control/questions/assumptions.md`.
- OQ-9 — not auto-opening the Editor. `.control/questions/assumptions.md`.
- OQ-11 — one Vault at a time. `.control/questions/assumptions.md`.
