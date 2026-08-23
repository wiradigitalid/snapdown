---
topic: Snapdown — depth raised for the three desktop components
artifact: .control/registry/components.yaml
updated: 2026-08-23T21:00
---

# Memlog — depth raised for the three desktop components

## The disclosure, made before the proposal

`wdi-init` discloses what a component touches before proposing its depth. What each of the five
touches, read from the `FR` that fall to it:

| Component | Money | Personal data | Irreversible act | Contract to an outside party | Unrollbackable 3P integration |
|---|---|---|---|---|---|
| `finding` | no | **yes** — a Capture may hold anything on screen | **yes** — delete removes the file from disk | no | no |
| `bundle` | no | **yes** — carries image copies | **yes** — delete removes files | no | no |
| `settings` | no | no | no | no | no |
| `agent-access` | no | **yes** — across a process boundary the product does not control | **yes** — disclosure cannot be recalled | no | no |
| `sharing` | no | **yes** — onto the public internet | **yes** — an unpublish cannot recall what was fetched | no | no |

The forced floor was checked and does not apply. `product.client` is `Wira Digital Indonesia`, the
owner's own agency; there is no regulator, auditor, or contracted client who will demand these
artifacts as a deliverable. Nothing here is forced to `deep`/`low` against the owner's preference.

`risk_accepted` was **not** touched. Depth and review intensity are separate fields and this intent
moved only depth. `settings` stays at `medium` while going to `deep` — thin review, thick documents —
and that combination is exactly what the split exists to make sayable.

## What changed

| Component | Was | Now | `g4_passed` |
|---|---|---|---|
| `finding` | `guarded` | `deep` | `true` → **`false`** |
| `bundle` | inherited `outline` | `deep` | `true` → **`false`** |
| `settings` | `catalog` | `deep` | stays `false`, now meaningful |
| `agent-access` | `guarded` | unchanged | `true` |
| `sharing` | `guarded` | unchanged | `true` |

Global `mode: outline` in `index.yaml` is untouched. It is now only a floor for anything unlisted;
every registered component names its own.

## The two things worth remembering

**`settings` was skipping G4 entirely.** That is the finding of this pass. At `mode: catalog` a
component gets a use-case row and nothing else — no flow, no state, no screen specification, no
Failure Behaviour. Every one of the owner's Settings complaints (unreadable labels, a screen longer
than its four choices warrant, a control made of two unanswerable numbers) is a question the corpus
had no slot to answer, because the gate that would have answered it was configured off. The four
choices `settings` holds looked small enough to leave thin. They are the choices the Reviewer meets
first, and thinness there is what the owner actually saw.

**`g4_passed` was reset on two components that genuinely passed.** `finding` and `bundle` passed G4
at their old depth. The depth moved under them, so the flag was lowered rather than left standing.
Leaving it true would have claimed a gate that never ran at this depth — and the whole reason the
flag exists is to stop exactly that claim. This is not a criticism of the earlier waves; it is the
mechanical consequence of raising depth on shipped work.

## What raising depth does NOT mean here

All three components already run in production. `wdi-init`'s own rule governs what comes next: what
G4 produces on a component whose code already runs is an **as-built record**, not a design. It is
written under the evidence labels in `sdd-guide.md`, and it describes what is there. Where the record
and the intent disagree, the disagreement is a defect to file — not a licence to document the code as
though it were the decision.

Nothing was deleted. Lowering depth deletes nothing and raising it removes nothing either; the
existing `outline`- and `guarded`-depth documents stay and are extended.
