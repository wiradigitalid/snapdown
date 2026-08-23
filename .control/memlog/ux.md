---
topic: Snapdown — UX run and landing, scope capture-to-markdown
artifact: .how/_platform/design-system.md
updated: 2026-08-23T21:00
---

# Memlog — UX

## 2026-08-23 — run and land, scope `capture-to-markdown`

**event** — First UX run in this product's life. `.how/_platform/design-system.md` existed as a
skeleton; no `DESIGN.md` or `EXPERIENCE.md` had ever been written. That absence is the root cause of
everything the owner reported, and naming it is more useful than the individual defects: five waves of
capability were built with no document anywhere saying what a screen owes.

**event** — Grounded on the shipped build rather than on the corpus. A UI worker drove the real
`Snapdown.exe` and captured every primary surface to `.work/ux-audit/`. Every defect designed against
below was seen, not inferred.

### Decisions

**decision — the white-on-white has one cause, and it is architectural.** The owner reported it on the
Hotkeys labels. The screenshots show it on **Findings and Bundles**: those two components paint
`#ffffff` and `#f8fafc` panels unconditionally, while the token file defines a dark theme under
`prefers-color-scheme` and the shell paints `--color-text` white over them. 23 distinct hex literals
live outside the token file across `apps/desktop/src`. The fix in the design system is not a palette
change: it is that a literal colour in a component is a defect, enforced by a lint rule (`NFR-17`).
Patching the specific panels would have left the mechanism intact.

**decision — top tabs replaced by a left navigation rail.** Not taste. `FR-29` requires Settings to fit
1024×720 without scrolling, vertical space is the scarce resource on this window, and a top tab row
costs ~64 px of height on every surface. A rail costs width, which Findings and Bundles were already
spending on a list column. The rail also gives the Capture action a permanent home, which the tab row
had nowhere to put.

**decision — Settings stays ONE surface; it does not become a second level of navigation.** The obvious
move was the Windows 11 Settings pattern: a sub-nav of five groups. Rejected. The owner's words were
"settingannya bisa padat-padat" — dense, not paginated. Ten controls across five groups genuinely fit
one screen once groups stop being stretched to match each other. Sub-nav would have satisfied `FR-29`
by hiding four groups behind a click, which is the letter of the requirement and the opposite of its
intent.

**decision — two columns packed by content height, never stretched.** This is the single change that
recovers the wasted third of the Settings window. The shipped build puts a one-checkbox group and a
four-control group in equal grid columns; the checkbox group inherits the tall one's height.

**decision — the `Toggle` gains an indeterminate state, and it is load-bearing.** `FR-18` requires the
startup control to reflect the real Windows registration and never a remembered intention. Reading it
is asynchronous. Without a third state the control must guess during the read, and the shipped build
guesses `true` then repaints to `false` — the Reviewer watches the product change its mind about its
own state. Indeterminate is what makes the requirement satisfiable rather than merely stated.

**decision — Findings is Snagit's three-region reading with Cobalt's third column.** Rail of recent
captures, canvas, and — where Snagit puts a tool inspector — the Note. That substitution is the whole
argument: Snapdown's third column is the Note because Snapdown's reader is a machine that reads text,
where Snagit's is a person who reads a drawing.

**decision — no tool palette, and the absence is documented as a design position rather than a gap.**
Arrows, callouts, blur and effects are a PRD Non-Goal. The new evidence is that Cobalt Capture, built
for the same audience, independently reached the same place: no markup tools, a crop at capture time,
an editable paragraph beside the screenshot. Recorded in both `DESIGN-finding.md` and
`EXPERIENCE-finding.md` so that a later reader does not "fix" the omission.

**decision — the Markdown preview on Bundles has no cursor.** `FR-11` and the Non-Goals say a Bundle is
recomposed, never patched. An editable preview would quietly break that and the drift would be
invisible. It is rendered as a read-only *region*, not a disabled input — a disabled control announces
as unavailable, and the content is available; it is the editing that does not exist.

**decision — Publish and Agent access stay visible while frozen.** `DEC-005` holds both components
still. Hiding their surfaces would break `FR-28`, which requires every primary surface to be reachable
from every other. They show current state and gain no behaviour.

**decision — three token groups are theme-invariant on purpose.** `--color-marker*`, `--overlay-*`, and
`--canvas-checker`. Each is drawn over the Reviewer's own screen content or over an exported image that
will be read on someone else's machine under someone else's theme. The app's theme is the wrong
reference for them, and this is written down so a later pass does not "fix" the inconsistency.

**change** — `EmptyState` and `ErrorState` promoted to base elements rather than written per screen.
Every screen owes both, and eight hand-written empties drift into eight voices.

**change** — Four `LC` born and registered in the same act as landing: `LC-028` `editor-shell`,
`LC-029` `capture-note-field`, `LC-030` `orphan-report`, `LC-031` `compose-bundle-dialog`. Three of
them are screens `inventory-screen.md` already named — rows 2, 7, 9 — that no build unit carried.
`LC-028` is the more interesting one: `FR-27` and `FR-28` are promises about the window *frame*, and
in the shipped build the frame is inline JSX at the top of `App.tsx`, owned by nothing.

### Verification, all nine checks

| # | Check | Result |
|---|---|---|
| 1 | Landing zone | **Pass.** The run wrote only to `_bmad-output/ux/capture-to-markdown/` |
| 2 | Split correctly | **Pass.** Layout, tokens, and components in `DESIGN`; IA, states, journeys, accessibility in `EXPERIENCE` |
| 3 | Journeys reference `UJ-N` | **Pass.** `UJ-1`, `UJ-2`, `UJ-3`, `UJ-4` referenced, not restated |
| 4 | Every screen has empty and error | **Pass**, with two stated exceptions that are answers rather than omissions: Settings has no empty state (every setting always has a value) and the Capture Overlay has neither empty nor loading (it is armed or absent) |
| 5 | Every user-facing noun in the glossary | **FAIL — routed, not fixed.** See Impact below |
| 6 | No new capability | **Pass.** Every screen traces to an `FR` |
| 7 | Every `[ASSUMPTION]` filed | **Pass.** No unfiled assumption; `OQ-18` and `OQ-20` already carry the two this pass rests on |
| 8 | Memlog location | **Pass.** This file. No `.memlog.md` inside the corpus |
| 9 | `bmad-review` structure + prose | **Not run.** See Not done below |

### Impact — found here, routed elsewhere, edited nowhere

- **The glossary's `Quality Budget` entry is stale.** It reads "the Reviewer's setting pair: a maximum
  long edge in pixels, and an encoder quality." `DEC-004` makes it a named intent with `Auto` the
  default and the numbers behind Advanced. The glossary is `wdi-blueprint`'s, so this is routed to G3
  and deliberately not patched here. This is check 5's failure and it is a real one.
- **`Auto`, `Sharp`, `Balanced`, `Small`, `Custom` are new user-facing nouns** with no glossary entry.
  Same route, same reason.
- **`Orphan report` is a user-facing noun with no glossary entry**, and it predates this pass — it has
  been inventory row 7 since G3. Same route.
- **`inventory-screen.md` needs four rows and a correction.** Rows for the editor shell, and rows 2, 7
  and 9 now have `LC` behind them. `wdi-blueprint` intent `platform` owns that file.
- **`NFR-3` reads loosely under `DEC-004`.** It names "the shipped default" for a 4K capture under
  250 KB; under `Auto` there is no single shipped default. Already flagged in the PRD memlog; repeated
  here because the design makes it concrete.

### Not done, and why

- **`bmad-review` did not run (check 9).** The gate rule is that G2 MUST NOT open on UX that has not
  been through it. `wdi-review` is the owner of that pass and it is the next step for these four
  documents, before the G2 stamp. Recording this as a pass would have been the one failure the whole
  check list exists to prevent.
- **No key-screen HTML mocks.** The ASCII layouts in the `DESIGN` documents carry the structural
  decisions, and the decisions here are about density and hierarchy rather than visual language. A mock
  would have added fidelity the decisions do not have yet.
- **No UX for `sharing` or `agent-access`.** `DEC-005` freezes both. Their existing screens are
  referenced where `FR-28` requires them to stay reachable, and nothing about them was redesigned.
- **The design system does not restate values.** It names tokens and where they resolve.
  `web/ui/src/styles/tokens.css` stays the single source for every hex code, which is the entire point
  of `NFR-17`.
