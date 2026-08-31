---
type: risk-acceptance
id: DEC-013
status: draft
serves: [CAP-6, CAP-7, CAP-8]
touches: []
supersedes: null
superseded_by: null
created: "2026-08-31"
---

# DEC-013 — Review intensity is lowered to `high` on `settings`, `agent-access` and `sharing`

> **This decision is `draft` and an agent MUST NOT accept it.** It exists because `risk_accepted: high`
> on a component that touches money, personal data, an irreversible action, a contractual promise, or
> an unrollbackable third-party integration requires the owner to accept that risk in writing, with
> `risk_accepted_by:` pointing here. V23 checks that the reference resolves; it cannot check that
> anyone read this file. The three components below are already set to `high` in `components.yaml` on
> the owner's instruction of 2026-08-31, so the setting is live and this ratification is outstanding.

## Decision

`settings`, `agent-access` and `sharing` are reviewed at `risk_accepted: high`. Their corpus documents
get **structure and prose only**; `edge-case-hunter` no longer runs on them, no two-reviewer code panel
is required, and V13 stops demanding a review trace on their SRS and SDD.

`finding` moves the other way, to `low`, and `bundle` stays at `medium`. Depth is a separate field and is
not part of this decision.

## Why

The owner set all five components' review intensity in one instruction on 2026-08-31, and `high` on
these three is the half that is not free to take. `risk_accepted` states **how much risk the owner is
willing to accept**, not how risky the component is — so `high` is a deliberate choice to look less
hard, and the method requires that choice to have a name against it rather than appearing as a registry
value nobody owns.

Nothing here argues the owner is wrong. The disclosure below is the whole point of the record: it says
what is being staked, so that a later reader asking *why is `sharing` reviewed the least* finds an
answer instead of an accident.

## What each of the three touches

Read from each component's own `risk_note`, not inferred.

**`sharing` — the sharpest of the three.** *"Publishing puts images that may contain personal data on
the public internet, and it is irreversible in the sense that matters — an unpublish cannot recall what
was already fetched. No money moves and there is no contractual promise, but this is the component
where a mistake cannot be undone."* Touches **personal data** and an **irreversible action**. This is
the component whose own note says a mistake cannot be undone, and it is now the least-reviewed of the
five.

**`agent-access`.** *"The key gates access to images that may contain personal data, across a process
boundary the product does not control. Revocation is the safety, so getting it wrong is an irreversible
disclosure."* Touches **personal data** and an **irreversible action**.

**`settings` — and this one is a genuine open question, not a formality.** Its `risk_note` says *"No
money, no personal data, no irreversible action. The worst outcome is a hotkey that does not register,
which is visible immediately."* On that reading `high` would be free. Two things cut against it:

- **V23 fires anyway, and correctly.** The check matches keywords against `risk_note` as prose, so a
  note that *denies* touching personal data trips the same marker as one that admits it. That is
  deliberate — the check's own comment says it *"leans toward disclosing more… it discloses, it does
  not judge."* Rewording the note to slip past it would be gaming the check, and the note is accurate
  as written.
- **The note may understate the component.** `settings` owns `FR-16`, choosing the Vault folder, and
  `BR-29` states *"Changing the Vault location either moves every existing file or moves none."*
  Moving the entire Vault touches every stored file on disk, and `vault_migration.rs` exists to do it.
  Whether that is "an irreversible action" is the owner's call. It was raised on 2026-08-31 and is
  unresolved.

## Cost

- **`edge-case-hunter` stops running on all three.** That is the lens that walks branches rather than
  reading prose, and it is the one that finds the unhandled path. On `sharing`, the unhandled paths are
  about content already on the public internet.
- **The two-reviewer code panel goes.** `low` requires it, `medium` and `high` do not, so `finding` is
  now the only component whose code must face it, and `bundle` and the three here must not.
- **V13 stops stamping these three.** Their SRS and SDD can drift from their last review with no
  validator saying so. One current finding disappears for exactly this reason —
  `.how/settings/SDD-settings.md`'s stale review stops being reported, and the staleness does not stop
  being real.
- **Two of the three are currently theoretical, and one is not.** `sharing` is frozen by `DEC-005`, so
  no new work reaches it until the experience bar lifts. `agent-access`'s `LC-017` has no
  implementation at all (`BUG-59`), so there is no code for a panel to review. `settings` is live, is
  at `outline` depth as of the same instruction, and is where the owner's own experience complaints of
  2026-08-23 landed.
- **`sharing` and `agent-access` are the two components whose documents describe surfaces that partly
  do not exist.** `DEC-012` established that of `AD-9`'s three handoff paths only the published page
  runs. Lowering the review intensity on the two whose descriptions are known to be ahead of their code
  removes a lens from the documents most likely to be wrong.

## Alternatives

| Option | Why not |
| --- | --- |
| Leave all five at `low` or `medium` | Ignores an explicit instruction. `risk_accepted` is the owner's field, and the method's own words are that the control is disclosure and not a veto |
| `high` on `sharing` and `agent-access` only, `settings` back to `medium` | Defensible, and it is the variant to reach for if the owner reads the `FR-16`/`BR-29` question above and decides the Vault move is irreversible. It is not chosen here because the instruction named `settings` explicitly |
| Reword `settings`' `risk_note` so V23 stops matching | Gaming the check. The note is accurate; the check is doing what its own comment says it is for |
| Keep `edge-case-hunter` on `sharing` while accepting `high` elsewhere | `risk_accepted` maps to one lens set; there is no per-lens override, and inventing one here would put a rule in a decision instead of in `delivery-flow-guide.md` |

## Reversal trigger

- **A defect is found in `sharing` or `agent-access` that `edge-case-hunter` would plausibly have
  caught** — an unhandled branch, not a wrong promise. That is this decision's cost arriving, and it
  reverses the setting on that component.
- **`DEC-005` lifts and `sharing` starts receiving work again.** The freeze is what makes `high` cheap
  there today; when work resumes, the trade is being made for real and is due a re-read.
- **`BUG-59` is fixed and the Local API exists.** Same reasoning for `agent-access`: there is currently
  no code for the panel to review, and that stops being true.
- **The owner answers the `FR-16`/`BR-29` question and finds the Vault move irreversible.** `settings`
  then has a live irreversible action and `high` on it is a different bet from the one described here.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | The `FR-16`/`BR-29` question above is unfiled; it belongs in `assumptions.md` through `wdi-question` and is named here so it is not lost. `OQ-31` is unrelated and already filed |
| Source material | The owner's instruction of 2026-08-31 setting all five components' `mode` and `risk_accepted`; `.control/memlog/mode-raise-2026-08-23.md`; `components.yaml` `risk_note` for each of the three |
| Checked | `validate.py`'s V23 implementation — it requires `risk_accepted_by` to name a `DEC-` present in `decisions.yaml`, and does not require that decision to be `accepted` or `applied`. So this file at `draft` satisfies the validator while leaving the ratification genuinely outstanding, which is the honest state |
