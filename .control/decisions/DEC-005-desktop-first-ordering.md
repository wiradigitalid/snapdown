---
type: decision
id: DEC-005
status: accepted
touches: []
supersedes: null
superseded_by: null
created: "2026-08-23"
---

# DEC-005 — The desktop experience is finished before `sharing` and `agent-access` are touched again

## Decision

All work after W5 goes to the desktop surfaces — `finding`, `bundle`, `settings` — until the
experience bar set at G2 is met. `sharing` and `agent-access` keep the code they already have and
receive no new work in that period: no new FR, no new use case, no UX pass, and no depth above the
`guarded` they already carry. They are not cancelled and nothing built for them is removed.

## Why

Five waves shipped every capability the PRDs promised, and the owner's first sustained look at the
result produced a list of experience defects rather than a list of missing features. That is the
signal that the constraint has moved. Capability is no longer what is scarce; the surface that
carries it is.

The evidence is specific and it is all one kind. The owner could not tell which application they had
opened. They could not find the Editor, and reported seeing only Settings. They could not read the
hotkey labels against their background. They found Settings longer than it needed to be for the four
choices it holds. Not one of these is a missing promise — every one of them is a promise the surface
failed to deliver. Adding a sixth capability on top of that surface makes the same failure wider.

Ordering `sharing` and `agent-access` behind this is not a judgement about their value. It is a
judgement about sequence: both of them are reached *through* the desktop, from a Bundle the Reviewer
composed in the Editor. A publish dialog reached from a screen the Reviewer cannot navigate is a
feature nobody arrives at. Fixing the path first is cheaper than fixing it twice.

The owner also said plainly that they do not yet know what they want from MCP. That is the correct
reason to hold `agent-access` rather than a reason to advance it — a capability specified against an
unformed want is the most expensive kind to build. `wdi-question` holds that as an open question; it
does not need a wave.

There is a real risk on the other side and it should be named: r2 already shipped. Freezing two
components that are in the field means a defect found in them has no wave to land in. This decision
does not forbid a fix. It forbids new work.

## Cost

- **Two components go stale while the corpus moves.** G3's spine, the C4 set, and the three
  inventories are re-derived at `deep` for the desktop surfaces. `sharing` and `agent-access` will be
  described at the old depth in a document whose neighbours are deeper, and that asymmetry has to be
  visible rather than tidy.
- **`wdi-reconcile` will report drift that is deliberate.** Every scan until this lifts will flag the
  two frozen components against the new bar. Suppressing that would hide real drift; living with it
  costs a paragraph of explanation on every report.
- **A defect in the frozen components has an awkward home.** It is a fix, not a wave, and the method
  has no third thing. It lands as a defect row and a patch release, and that path is thinner than a
  wave's.
- **The web service keeps running against a client that is not being improved.** `apps/web-service`
  and the reader SPA are deployed artifacts. Freezing the desktop side of publishing does not freeze
  their operational reality — a dependency upgrade or a security fix is still work that has to happen
  outside this ordering.
- **The bar is not yet written.** "Until the experience bar set at G2 is met" is only as strong as
  that bar, and G2 has not been re-run yet. Until `wdi-ux` produces it, this decision names a
  condition nobody can check.

## Alternatives

Required here: both `sharing` and `agent-access` sit at `risk_accepted: low`.

| Option | Why not |
| --- | --- |
| Carry on to a sixth capability wave | Widens the surface that already failed, and makes the eventual rework touch more screens |
| Rework the desktop UX and advance `sharing` in parallel | Two waves writing the same Editor shell at once. The publish dialog is reached from a Bundle screen that is being redrawn underneath it |
| Cancel `sharing` and `agent-access` outright | Throws away shipped, tested, working code over a sequencing question. Nothing about them was judged wrong |
| Freeze the whole product and rewrite the front end | The Rust core, the stores, and the capture path are not what failed. Rewriting past the failure is how the failure gets re-earned |
| Fix only the four reported defects and move on | Treats the symptom list as the problem. The absence of any `wdi-ux` output is the cause; four patched screens leave it in place |

## Reversal trigger

- The experience bar from G2 is met and verified. This decision lifts by its own terms; it does not
  need superseding.
- A defect is found in `sharing` or `agent-access` that cannot be fixed as a patch — one that needs a
  new promise. That is a re-plan, and it reopens the ordering rather than bending it.
- The owner forms a concrete want for MCP. `agent-access` then has a reason to move that it does not
  have today, and the ordering is worth re-weighing against it.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | To be filed against MCP by `wdi-question`; the owner's want is unformed, not unstated |
| Source material | Owner's message of 2026-08-23; `.control/reports/RTR-W5.md`; `HANDOVER.md` § Wave Summary |
