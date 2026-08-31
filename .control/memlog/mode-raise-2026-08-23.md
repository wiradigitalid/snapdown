---
topic: Snapdown — depth raised for the three desktop components
artifact: .control/registry/components.yaml
updated: 2026-08-31T19:18
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
- (decision) 2026-08-31, owner's instruction: 'set seluruh mode guarded. risk-accepted buat aja medium semuanya'. This partly reverses the 2026-08-23 raise this memlog records. GLOBAL mode outline -> guarded in index.yaml, and all five per-component mode overrides REMOVED rather than restated as 'guarded', because components.yaml's own header says mode is written ONLY where it must differ from global - so the global line governs again after governing nothing. finding/bundle/settings drop deep -> guarded; agent-access/sharing were already guarded and are unchanged in substance. All five risk_accepted set to medium: finding, bundle, agent-access and sharing move from low; settings was already medium. Disclosed before writing: the hard floor (an outside party demanding artifacts as a deliverable, which would pin deep+low) does NOT apply - product.client is Wira Digital Indonesia, the org that owns the product, and no regulator, auditor or contract appears in the brief or the non-technical log. No DEC- of type risk-acceptance is owed because that is required only at high, and nothing is at high. Every mode history comment was kept in place rather than deleted.
- (event) CONSEQUENCE the owner was told before choosing, recorded so it is not rediscovered: low -> medium changes NO document lens - both give structure + prose + edge-case-hunter. Its whole effect is dropping the two-reviewer CODE panel that low requires. That now includes sharing, whose own risk_note says it is 'the component where a mistake cannot be undone'. sharing is frozen by DEC-005 so the panel has nothing to review until that lifts, and agent-access's LC-017 has no implementation at all (BUG-59), so for two of the four the change is currently theoretical. For finding and bundle it is immediate.
- (decision) 2026-08-31, third setting of the day, owner's instruction verbatim: 'global: outline|high / finding: guarded|low / bundle: guarded|medium / settings: outline|high / agent-access: outline|high / sharing: guarded|high'. Applied: index.yaml mode guarded -> outline; finding gains an explicit mode: guarded and drops risk_accepted medium -> low; bundle gains an explicit mode: guarded and keeps medium; settings and agent-access carry NO mode row so they inherit outline (agent-access therefore genuinely DROPS guarded -> outline); sharing gains an explicit mode: guarded. settings, agent-access and sharing all move to risk_accepted: high with risk_accepted_by: DEC-013.
- (override) TWO PARTS OF THE INSTRUCTION WERE NOT APPLIED AS GIVEN, both reported to the owner rather than improvised. (1) The 'high' half of 'global: outline|high' has nowhere to land: risk_accepted has NO global scope - index.yaml carries no such field, and components.yaml's own header plus AGENTS.md both state it is per-component. Only the mode half was applied. Even if the field existed it would govern nothing, because all five components are set explicitly. (2) risk_accepted: high on settings, agent-access and sharing is NOT free - it requires a DEC- of type risk-acceptance with risk_accepted_by pointing at it. DEC-013 was written and registered at status: draft and the three rows point at it, which satisfies V23 (verified by reading the check: it requires the DEC- to EXIST in decisions.yaml, not to be accepted). The agent did NOT accept it. wdi-decision reserves accept to the Product Owner and forbids an agent accepting its own; that rule was already overstepped once today on DEC-012 and reported as not clean, and repeating it on a decision whose entire purpose is the owner's signature would be worse. Ratification is outstanding.
