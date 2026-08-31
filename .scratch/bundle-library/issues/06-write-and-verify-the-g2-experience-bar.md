# 06: Write and verify the G2 experience bar

**Type:** task
**Status:** open
**Blocked by:** 01, 05 - both resolved 2026-08-31, so this is UNBLOCKED

## Question

This ticket is the gate between the Library and cloud **Publish**. It is not a design question — it
is the missing artifact that makes an existing decision checkable.

`DEC-005` freezes the `sharing` component: *"no new FR, no new use case, no UX pass, and no depth
above the `guarded` they already carry."* That wording covers **planning**, not merely implementing —
so a spec for Publish cannot legally be written while the freeze stands.

The freeze does not need to be fought. `DEC-005`'s own reversal trigger reads: *"The experience bar
from G2 is met and verified. **This decision lifts by its own terms; it does not need superseding.**"*

But `DEC-005` also indicts itself: *"**The bar is not yet written.** … Until `wdi-ux` produces it,
this decision names a condition nobody can check."*

So the work is:

- Run `wdi-ux` to produce the G2 experience bar — the written, checkable standard the desktop
  surfaces must meet.
- Assess the desktop experience against it and record the verdict.
- If the bar is met, `DEC-005` lifts by its own terms and the Publish patch in the map's
  **Not yet specified** graduates into tickets. If it is not met, the gap it names becomes the next
  work, and Publish stays where it is.

**Sequencing note.** This is blocked by the two prototype tickets so the bar judges an Editor that
includes the Library, rather than one missing it. Be honest about the limit of that: a bar can be
*written* against a design, but *verifying* it needs the Library actually shipped. If the Library has
not been built by the time this ticket is taken, split it — write the bar now, verify it later.

Do **not** write FR/UC/UX for `sharing` in this ticket. That is exactly what the freeze forbids, and
doing it here would defeat the point of lifting the freeze cleanly.
