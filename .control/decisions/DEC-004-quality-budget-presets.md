---
type: decision
id: DEC-004
status: applied
touches:
  - .control/product-glossary.md
  - .control/registry/waves.yaml
  - .how/finding/02-contracts/contract-inventory.md
  - .how/finding/04-components/LC-003-image-reducer.md
  - .how/finding/SDD-finding.md
  - .how/settings/01-ux/DESIGN.md
  - .how/settings/02-contracts/contract-inventory.md
  - .how/settings/05-model/data-model.md
  - .how/settings/SDD-settings.md
  - .what/_prd/capture-to-markdown/prd.md
  - .what/_product-brief/brief.md
  - .what/business-rules.md
  - .what/finding/05-scenarios/SCN-03-the-quality-budget-that-resolves-differently.md
  - .what/settings/02-rules/rules-settings.md
  - .what/settings/03-domain/domain-model.md
  - .what/settings/04-usecases/UC-13-decide-how-much-picture-quality-a-screenshot-is-worth.md
  - .what/settings/SRS-settings.md
supersedes: null
superseded_by: null
created: "2026-08-23"
---

# DEC-004 — The Quality Budget is chosen as a named intent, and Auto derives the numbers per capture

## Decision

The Reviewer chooses a Quality Budget by naming what they want — **Auto**, **Sharp**, **Balanced**,
or **Small** — and Auto is the shipped default. Auto derives the maximum long edge and the encoder
quality from the captured region itself rather than from a stored constant. The raw long-edge and
encoder-quality numbers remain settable, behind an **Advanced** disclosure, and setting either one
moves the budget to a fifth named state, **Custom**.

## Why

FR-5 already promised that "both values have defaults the Reviewer never has to change to get a
usable result." What shipped does not keep that promise: two numbered fields, `1600` and `75`, with
no unit the Reviewer can reason about and no way to tell whether either is right. The owner named
the shape they expected — TinyPNG, which asks nothing and returns a file that looks the same and
weighs less. The promise was correct; the presentation defeated it.

A fixed long edge is also the wrong instrument, and the PRD says so itself: OQ-3 records 1600 px as
"the working answer and it has not been measured." One constant cannot serve both of the two things
a Reviewer actually captures. A 300×120 button tooltip is already under any cap, so the cap does
nothing and the encoder quality decides everything — and at 75 the text in that tooltip is where JPEG
artefacts are most visible. A full 4K screen is 4× over the cap, so the cap decides everything and
the encoder barely matters. Deriving both numbers from the captured region is not a refinement of the
constant; it is the only way one setting can serve both cases.

Naming the intent rather than the numbers is what makes the setting answerable. "Sharp" and "Small"
are things the Reviewer can hold an opinion about. "1600" and "75" are things they can only accept.
That the assumption index already flags legibility at 1600 px as unmeasured (OQ-3) is the tell: if
the team cannot defend the number, the Reviewer certainly cannot.

Advanced stays because the numbers still exist and someone will one day need them — and because
hiding a control the product still honours is worse than showing it in the right place. Custom is a
named state rather than a silent condition so that the Settings screen can always answer *what
budget am I on* with a word.

## Cost

- **Auto's derivation is a promise that has to hold across an upgrade.** Two Findings captured a
  month apart on Auto will differ if the derivation changes, and nothing re-encodes an old Finding —
  FR-5 forbids it. The derivation is therefore versioned behaviour, not an implementation detail, and
  changing it is a decision rather than a tuning.
- **"Show the stored size of the latest Finding" gets harder to read.** FR-5 promises that feedback,
  and under Auto the number moves for a reason the Reviewer did not cause. The screen has to say what
  the budget resolved to for that capture, not just what the budget is named.
- **Four presets is four things to defend.** Each needs a stated intent and a measured effect, or
  they become three synonyms and a default. Balanced in particular has to differ from Auto in a way
  someone can state in one line.
- **Custom is a state the UI must never enter by accident.** Nudging an Advanced field must not
  silently abandon Auto without the Reviewer seeing that it happened.
- **OQ-3 is not closed by this.** It is moved: the open question stops being "what is the right
  constant" and becomes "is the derivation legible at its smallest output." That is a better
  question and still an unmeasured one.

## Alternatives

Required here: `finding` sits at `risk_accepted: low`, and it owns `image-encoding`.

| Option | Why not |
| --- | --- |
| Keep two raw numbers, improve the labels | The numbers are unanswerable however they are labelled; the Reviewer has no way to judge 1600 against 1400 |
| Auto only, no presets and no Advanced | Cleanest, and it removes the escape hatch for the one capture that must stay pixel-exact. Kept as the simplification to reach for if Advanced goes unused |
| Presets only, Advanced removed and the raw API dropped | Throws away a capability already built and tested, to buy tidiness in one screen |
| A file-size target — "keep it under 200 KB" | The honest goal, and it needs an encode-measure-re-encode loop the capture path cannot afford at hotkey latency. Deferred, not rejected |
| Per-capture choice at the overlay | Puts a decision in the one place the product promises not to: the capture must complete without the Reviewer thinking |
| PNG for screenshots, quality setting removed entirely | Right for flat UI, wrong for the screenshots that contain photos or gradients, and it abandons the size budget BG-3 exists for |

## Reversal trigger

- Advanced is measured as never opened across a real review period. Then Custom and the raw numbers
  are cost, and Auto-only wins.
- A Reviewer is observed reaching for the original screenshot because an Auto output was illegible.
  That reopens the derivation, and it is the observation OQ-3 is waiting for.
- Encode time at Auto is measured as visible at the moment of capture. That reopens whether the
  derivation may inspect the image at all, or must work from region dimensions alone.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | `OQ-3` — restated by this decision rather than closed |
| Source material | `.what/_prd/capture-to-markdown/prd.md` § FR-5 and § 8 item 2; owner's TinyPNG comparison of 2026-08-23 |
