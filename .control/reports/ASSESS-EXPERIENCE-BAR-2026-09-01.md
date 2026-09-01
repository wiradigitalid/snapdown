---
type: assessment
subject: DEC-005 — the G2 experience bar
status: Reference
created: "2026-09-01"
read_only: true
---

# Is the G2 experience bar met?

`DEC-005` freezes `sharing` and `agent-access` and lifts *by its own terms* on one condition: **"The
experience bar from G2 is met and verified."** It then indicts itself — *"The bar is not yet written.
… Until `wdi-ux` produces it, this decision names a condition nobody can check."*

This report answers the condition. **It changes nothing and decides nothing.** It states where the
bar already lives, checks the shipped desktop against each part of it, and names what the failures
become.

**Answer: the bar is not met. `DEC-005` does not lift.** Four of the six checkable items fail, one of
them on a defect that is open, high, and measured. A seventh part of the bar cannot be settled by any
test and has never been observed.

## First finding: the bar was written on 2026-08-23, and nobody assembled it

`DEC-005`'s Cost section is correct for the day it was written and has been stale since **the same
day**. Three things landed on 2026-08-23, after it:

- `.what/_product-brief/brief.md` gained `BG-7` and **four Success Criteria** under it, written to be
  gathered by watching a first encounter rather than by timing a loop. The brief says outright that
  *"These four are how the condition in DEC-005 … is actually checked."*
- `.what/_prd/capture-to-markdown/prd.md` § 4.7 sharpened them into `CAP-9` with `FR-27`, `FR-28`,
  `FR-29`, plus `NFR-16` and `NFR-17`. Its own description says these *"are not features. They are
  the conditions under which every other feature in this document is actually reachable."*
- `wdi-ux` ran, for the first time in the product's life, and landed `EXPERIENCE.md` and `DESIGN.md`
  for `finding`, `bundle` and `settings`, plus `.how/_platform/design-system.md`.

So ticket 06's first bullet — *"Run `wdi-ux` to produce the G2 experience bar"* — was already done
before the ticket was written. **A second `wdi-ux` run is not what this needed.** What was missing is
the thing below: the bar stated as one checkable list, and a verdict against the code.

That is the shape of `BUG-8` again, and of `BUG-12` before it: a register field naming work that had
already happened, believed for days because nobody read past it.

## Second finding: nothing here is a new promise

Everything in the table below is quoted or derived from `BG-7`'s measure, its four Success Criteria,
`FR-27`–`FR-29`, `NFR-16` and `NFR-17`. **No item was invented for this assessment.** A bar written
here would be a fourth copy of one fact, and it would drift from the three that already exist.

## The bar, and the verdict

| | Item | Where it is promised | Verdict |
|---|---|---|---|
| **B1** | The Reviewer can name the application and which of its two personas is on screen | `FR-27` | **FAIL** |
| **B2** | Every primary surface is reachable from every other, without knowing it exists beforehand | `FR-28` | **FAIL** |
| **B3** | A primary surface fits its window; nothing is discovered only by scrolling | `FR-29` | **FAIL**, and the corpus disagrees with itself about it |
| **B4** | Every text element meets WCAG AA in both Windows themes | `NFR-16`, `BG-7`'s measure | **FAIL** |
| **B5** | No colour is defined for one theme only | `NFR-17` | **PASS**, with a stated and reasoned exception |
| **B6** | Every setting is answerable from its own screen; no screen asks for a number the Reviewer cannot judge | `BG-7`, `FR-5` / `DEC-004` | **PASS** |
| **B7** | A Reviewer who has never seen Snapdown reaches their first handed-over Bundle without being told how | `BG-7`'s measure, brief criterion 1 | **NOT VERIFIED — and no test can verify it** |

### B1 — Naming the persona · FAIL

`FR-27`: *"the tray and the installed executable are **Snapdown**, and the workspace window titles
itself **Snapdown Editor**."*

The shipped window titles itself `Snapdown` in both places it says a name — the `Window`'s own
`title:` and the custom titlebar's label, since the window is `no-frame` and draws its own chrome.
The tray tooltip is `Snapdown` and the executable is `Snapdown`, which is correct. So all three agree,
and that is exactly the failure: `FR-27` asks the Editor to be **distinguishable** from the tray, and
a Reviewer looking at the window cannot tell which part of the product they are in.

**This is a regression, and the register already holds the proof.** `BUG-11`'s note records the one
thing verified from the Tauri build before it was retired: *"the window title read 'Snapdown Editor'.
W6-S2's persona naming (DEC-003, FR-27) is correct in a real running window."* `DEC-007` rewrote the
front end in Slint and the name did not travel.

`FR-27`'s own consequence promises *"A test asserts the three against one source."* One of the three
is asserted — the executable's name, in `test_executable_identity.rs`, which reads the crate's
`[[bin]]` table. Nothing asserts the tray tooltip or either window label, which is why a rewrite could
drop one of them silently.

### B2 — Reaching every surface · FAIL

`FR-28` names four primary surfaces — Findings, Bundles, Settings, Agent access — and requires each to
be reachable from each. Shipped:

| Surface | State |
|---|---|
| Findings | **Present.** It is the Editor window itself: filmstrip, canvas, note pane |
| Settings | **Present**, as a modal over the Editor, reachable from the toolbar and from the tray |
| Bundles | **Absent.** `library-clicked` is a handler whose whole body is a `println!`, recorded in `KNOWN_STUBS` as *"there is no Library screen in Slint yet — only the filmstrip"* |
| Agent access | **Absent as a primary surface.** It exists as a read-only `Agent Bridge` tab inside Settings, showing a status string and nothing else |

Two of four. The Bundles gap is the whole subject of `.scratch/bundle-library/`, so it is already
owned; the Agent-access gap is `BUG-59` and is frozen by `DEC-005` itself. **That circularity is worth
naming: `DEC-005` freezes the component whose absence is one of the reasons its own bar fails.** It is
not a deadlock — `FR-28` is met by *listing* Agent access as a surface, not by giving it behaviour,
and `.what/settings/04-usecases/EXPERIENCE.md` already says so: *"it shows its current state and gains
no behaviour; it stays listed because `FR-28` and `BR-120` require it."*

### B3 — Fitting the window · FAIL, and the corpus contradicts itself

The window is `min-width: 1024px; min-height: 720px`, which is the minimum supported size the
requirement is written against. That half is right.

`FR-29`'s consequence is not met: *"Settings presents its four groups — startup, Vault folder, Quality
Budget, hotkeys — within the window at its minimum supported size."* The shipped Settings is a
**four-tab screen** — `General & Quality`, `Hotkeys`, `Agent Bridge`, `About` — so three of the four
groups sit behind tab 0 and hotkeys behind tab 1. At minimum size the Reviewer sees four tab labels,
not four groups.

**The code is not wrong by accident here, and that is the finding.** The tabs are the landed design:
`.how/settings/01-ux/DESIGN.md` carries four HTML assets, one per tab, including
`.how/settings/01-ux/assets/06c-settings-agent-bridge.html`; `.how/_platform/inventory-screen.md`
row 12 is *"Settings — General & Quality"* and row 13 is *"Settings — Agent access"* at
`/settings/agent-access`; and `test_design_system.rs` asserts the horizontal tab row on purpose,
citing `06a-settings-general.html`. Three artifacts and a test agree with the code.

What disagrees is `.what/settings/04-usecases/EXPERIENCE.md`, landed from the same `wdi-ux` run on the
same day, which states the opposite twice: *"Settings itself holds four groups and **is not a second
level of navigation**"*, and *"**Agent access is not a fifth group.** It is a primary surface of its
own, listed in the rail beside Findings, Bundles and Settings."* `.control/memlog/ux.md` records the
reasoning that produced that sentence — sub-nav *"would have satisfied `FR-29` by hiding four groups
behind a click, which is the letter of the requirement and the opposite of its intent."*

So the UX run landed a rejection of tabs into `.what/` and a design made of tabs into `.how/`, on the
same day, and nothing since has read them together. `FR-29`'s consequence text then cites row 13 as
evidence that Agent access is a primary surface — but row 13's route is `/settings/agent-access`, a
child of Settings, so the citation contradicts the sentence it is supporting.

**This one is not mine to resolve and it is not a code fix.** Either the promise is right and the
design is wrong, or the reverse; `EXPERIENCE.md` is `bmad-ux`'s to write and `FR-29` is
`wdi-product`'s. What is certain is that the bar cannot be *"met and verified"* while two halves of
its own definition say different things.

### B4 — WCAG AA contrast · FAIL

This is the only part of the bar the brief deliberately gave a number to, *"so that at least one part
of the bar is settled by a test instead of by taste."* The test exists — `test_theme_contrast.rs`,
over `theme.slint`, which is the palette that actually ships — and it reports **six pairings below
AA**, each recorded with its measured ratio and owned by `BUG-54`, which is open at `high`:

```
DARK   text-on-accent  on accent-primary   3.20   needs 4.5   every primary button label
DARK   text-on-accent  on accent-hover     2.59   needs 4.5   ...under the pointer
LIGHT  text-muted      on bg-app           4.34   needs 4.5
LIGHT  text-dim        on bg-app           2.34   needs 3.0
DARK   text-dim        on bg-card          2.99   needs 3.0
LIGHT  text-dim        on bg-card          2.56   needs 3.0
```

**The suite is green and the requirement is unmet, and both are true at once.** The test is a ratchet,
not a pass mark: it holds the six at their measured values, fails if any degrades, and fails again if
`BUG-54` is closed without emptying the list. Reading its green as evidence the bar is met is exactly
the mistake the file's own header warns about.

`NFR-16` says *every text element*, and this measures *tokens*. That is the stronger reading available
today and it is still a partial one — a literal colour outside `theme.slint` would not be seen except
in the capture overlay, where `test_capture_interaction.rs` refuses one.

### B5 — Both themes · PASS, with a reasoned exception

`theme.slint` is the single palette and defines both columns. Eight overlay tokens carry no `is-dark`
branch, and that is deliberate rather than an omission: the overlay is drawn over a frozen screenshot
of the operator's own desktop, not over app chrome, so a light scrim over a dark capture is invisible.
The reason sits beside them in the file, `DEC-009` carries the study, and `BUG-37`'s note records the
mechanical half as done.

`NFR-17`'s `enforced_by` was corrected on 2026-09-01 to name the two Rust guards that exist rather
than a lint over a package that was deleted.

### B6 — Answerable settings · PASS

`BG-7` ends *"no screen asks the Reviewer for a number they have no way to judge."* Settings offers the
Quality Budget as named presets (`budget-chosen`), with the raw long-edge, quality and colour values
reachable only behind an `Advanced` disclosure that is closed by default (`budget-advanced`). That is
`DEC-004`'s shape and it holds in the shipped screen.

### B7 — The first encounter · never observed, and no test can settle it

`BG-7`'s measure and the brief's first criterion both read: *"A Reviewer who has never seen Snapdown
reaches their first handed-over Bundle without being told how. No screen is explained to them, and no
control is defended."*

**Nothing in this repository can answer that, and nothing here has tried.** It is gathered by watching
a person meet the product, which the brief says in as many words — the criteria are *"gathered
differently: by watching a first encounter, not by timing a loop."* No such session is recorded
anywhere in `.control/`.

Two things follow, and the second matters more:

1. Even if `B1`–`B6` all passed, `DEC-005` would still not lift, because its trigger says **met and
   verified** and `B7` would be unverified.
2. `B7` cannot be met today for a reason that needs no observer: it requires reaching a **handed-over
   Bundle**, and the Library screen that lists Bundles does not exist (`B2`). The route the criterion
   describes is not there to be walked.

## What the failures become

Nothing here needs a new decision. Every gap lands on work that is already owned:

| Gap | Where it goes |
|---|---|
| **B1** — the Editor does not name its persona, and two of `FR-27`'s three names are unasserted | A new defect row. It is a small, self-contained code fix plus the test `FR-27` already promised |
| **B2** — no Bundles surface | `.scratch/bundle-library/`, which exists for exactly this. Ticket 06 was blocked on it and the dependency is the right way round |
| **B2** — no Agent-access surface | `BUG-59`, critical, frozen by `DEC-005`. Listing it is permitted by the freeze; building it is not |
| **B3** — `EXPERIENCE.md` and the design disagree about tabs | `wdi-ux` for `EXPERIENCE.md`, `wdi-product` if `FR-29`'s consequence is what is wrong. **The owner's call which** |
| **B4** — six pairings below AA | `BUG-54`, open, high, measured. The single cheapest item on this list and the only one blocking a numeric criterion |
| **B7** — never observed | A first-encounter session with a Reviewer, after `B2` gives them a Bundle to reach |

**The order that gets the bar closest, cheapest:** `B4` first — it is a palette change with a test
already waiting for it, and it converts a hard failure into a pass. Then `B1`, which is two string
literals and one assertion. `B2` is the Library, which is a whole effort and already mapped. `B3` is a
conversation, not work. `B7` waits on `B2`.

## What this assessment did not do

- **It did not run the application.** Every verdict above was read out of the shipped source and its
  tests, not observed in `Snapdown.exe`. That is sufficient for `B1`, `B4`, `B5` and `B6`, which are
  literals and measurements; it is weaker evidence for `B2` and `B3`, where a running window could
  show something the source does not, and it is no evidence at all for `B7`.
- **It did not re-run `wdi-ux`.** The 2026-08-23 run is landed and the bar does not depend on a second
  one. Worth knowing separately: that run predates `CAP-10`, `CAP-11` and `CAP-12`, so the UX
  documents are behind the PRD by three capabilities. That is ordinary lag, it is not part of this
  bar, and it is not a reason to hold `DEC-005`.
- **It edited no promise, no design, and no decision.** `DEC-005` stands unchanged and needs no
  amendment; it lifts on its own terms when the bar is met, and today it is not.
