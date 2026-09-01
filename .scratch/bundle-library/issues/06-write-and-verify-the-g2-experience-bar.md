# 06: Write and verify the G2 experience bar

**Type:** task
**Status:** resolved 2026-09-01
**Blocked by:** 01, 05 - both resolved 2026-08-31, so this was UNBLOCKED

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

---

## Answer

**The bar is not met. `DEC-005` does not lift, and Publish stays where it is.**

The full assessment, with every citation, is
[`.control/reports/ASSESS-EXPERIENCE-BAR-2026-09-01.md`](../../../.control/reports/ASSESS-EXPERIENCE-BAR-2026-09-01.md).
What follows is what a reader of this map needs and no more.

### The first bullet was already done, and this ticket's premise was stale

**`wdi-ux` ran on 2026-08-23, and the bar was written the same day `DEC-005` said it had not been.**
Three things landed after the decision, on the decision's own date:

- the brief gained `BG-7` plus **four Success Criteria**, and says in its own text that *"These four
  are how the condition in DEC-005 … is actually checked"*;
- the PRD's § 4.7 sharpened them into `CAP-9` — `FR-27`, `FR-28`, `FR-29` — plus `NFR-16` and
  `NFR-17`;
- `wdi-ux` landed `EXPERIENCE.md` and `DESIGN.md` for `finding`, `bundle` and `settings`, and
  `.how/_platform/design-system.md`.

`DEC-005`'s Cost section was never corrected, and this ticket copied it. **A second `wdi-ux` run was
not needed and was not made.** What was missing is the thing below: the bar as one checkable list,
and a verdict against the code. Same shape as `BUG-8` and `BUG-12` — a field naming work that had
already happened, believed because nobody read past it.

### The bar, and the verdict

Nothing here was invented. Every row is `BG-7`'s measure, one of its four Success Criteria, or an
`FR`/`NFR` that already sharpens one.

| | Item | Promised by | Verdict |
|---|---|---|---|
| **B1** | The Reviewer can name the app and which persona is on screen | `FR-27` | **FAIL** |
| **B2** | Every primary surface reachable from every other | `FR-28` | **FAIL** — 2 of 4 exist |
| **B3** | A primary surface fits its window; nothing found only by scrolling | `FR-29` | **FAIL**, and the corpus disagrees with itself |
| **B4** | Every text element meets WCAG AA in both themes | `NFR-16` | **FAIL** — six pairings below AA |
| **B5** | No colour defined for one theme only | `NFR-17` | **PASS**, with a reasoned exception |
| **B6** | No screen asks for a number the Reviewer cannot judge | `BG-7`, `FR-5`/`DEC-004` | **PASS** |
| **B7** | A first-time Reviewer reaches their first handed-over Bundle unaided | `BG-7`'s measure | **NEVER OBSERVED** |

The four that matter to this map:

- **B4 is the decisive one and the cheapest.** `test_theme_contrast.rs` reports six token pairings
  below WCAG AA, including *every primary button label in dark mode* at 3.20 and 2.59 under the
  pointer. `BUG-54`, open, high, measured. **The suite is green and the requirement is unmet at the
  same time** — the test is a ratchet holding six known failures at their measured values, not a pass
  mark. Reading its green as "the bar is met" is the exact mistake its own header warns about.
- **B2 is this map's own subject.** Of `FR-28`'s four primary surfaces, Findings and Settings exist;
  **Bundles does not** (`library-clicked` is a `println!` recorded in `KNOWN_STUBS`), and Agent access
  exists only as a read-only tab inside Settings. `.scratch/bundle-library/` is the effort that closes
  half of that, which means **the dependency runs the other way from what this ticket assumed**: the
  Library is not blocked by the bar, the bar is blocked by the Library.
- **B7 cannot be met until B2 is, and then it still needs a person.** The criterion is *reach your
  first handed-over Bundle* — there is no screen that lists Bundles, so the route does not exist to be
  walked. And even once it does, this one is gathered by watching a first encounter, which no test can
  do and which has never been run.
- **B3 is not a code fix and is not mine.** The shipped tabbed Settings matches `DESIGN.md`, its four
  HTML assets, `inventory-screen.md` rows 12 and 13, and a test that asserts the tab row on purpose.
  What disagrees is `EXPERIENCE.md`, landed from the same run on the same day, which says Settings
  *"is not a second level of navigation"* and that Agent access is *"a primary surface of its own."*
  Two halves of the bar's own definition contradict each other. **Owner's call which half is wrong**;
  `EXPERIENCE.md` is `bmad-ux`'s to rewrite and `FR-29`'s consequence is `wdi-product`'s.

### What landed

- `.control/reports/ASSESS-EXPERIENCE-BAR-2026-09-01.md` — the assessment, read-only, changes nothing.
- `BUG-89` — **new.** The Editor window titles itself `Snapdown`, not `Snapdown Editor`, in both
  places it says a name. A regression from `DEC-007`'s Slint rewrite: `BUG-11`'s note records that the
  Tauri build had it right. `FR-27` promises *"A test asserts the three against one source"* and only
  one of the three is asserted, which is how a rewrite dropped it silently. Two string literals and
  one test.
- `BUG-57` and `BUG-61` — **re-counted against the tree and corrected.** `BUG-57` is now two dead
  buttons, not three (`settings-clicked` was wired). `BUG-61` listed nine absent surfaces; four have
  since landed. Neither row was wrong when written; both had gone stale in silence, which is the
  hazard `AGENTS.md` names.
- `test_theme_contrast.rs`'s own header said *"Four pairings fail AA today"* while `EXCEPTIONS` has
  held six since the day it was written. Corrected, with the reason it matters: a green run there is
  not evidence that `NFR-16` is met.

### What this does to the map

**Publish stays in Not yet specified.** No ticket graduates out of it.

The order that gets the bar closest, cheapest — and none of it belongs to this map except the second
item:

1. **`BUG-54`** — the palette. A test is already waiting for it, and it converts the one numeric
   criterion from a hard fail to a pass.
2. **`BUG-89`** — two literals and one assertion.
3. **The Library** — this map, which closes half of B2.
4. **B3** — a conversation with the owner about which half of the corpus is wrong.
5. **B7** — a first-encounter session, after 3.

**This ticket does not need re-taking when those land.** It is the bar plus a dated verdict; re-running
the verdict is a fresh assessment, not a reopening of this ticket.
