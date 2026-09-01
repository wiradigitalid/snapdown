---
topic: Snapdown — screenshot findings composed into Markdown for coding agents
artifact: .what/_product-brief/brief.md
updated: 2026-09-01T09:12
---

- (event) headless run, intent create — owner authorised writing and approval of G1..G5 in one pass without further questions
- (note) owner brain dump captured verbatim as the only source; no research subagents spawned, no _bmad-output run folder exists for this brief
- (decision) one problem: the note-to-image binding is lost when several visual findings are handed to a coding agent
- (decision) one primary user: the agent-assisted developer. The coding agent is a secondary consumer, not the primary
- (decision) one measure: median handoff time for a five-finding review under 120 seconds with zero mis-attached notes
- (decision) editor does NOT auto-open after a capture; default is a toast with an Open action. Owner delegated the call and the loop has to survive six runs in ninety seconds
- (decision) numbered markers are the only annotation. Arrows and callouts are invisible to the reader that matters
- (decision) three handoff paths ship because no single one covers all four criteria — see addendum, Options weighed
- (assumption) agent reading cost tracks pixel area, so downscaling is the dominant compression lever
- (assumption) a coding agent can open relative image paths from a Markdown file it is handed
- (assumption) Windows global hotkeys register from a user-level process without administrator rights
- (decision) delete is hard: a finding leaving takes its image file with it. No soft delete
- (decision) publishing is an act on a named bundle, never a sync. Captures may contain personal data
- (gap) no host and no domain yet for the web service — both filed as external prerequisites
- (note) commercial content deliberately excluded: repo is public, per repo-guide.md content boundary
- (event) brief and addendum written; G1 review lenses structure+prose applied at write time

---

## 2026-08-23 — update: the experience bar becomes a goal (BG-7)

**change** — Added `BG-7`: Snapdown's own surface costs the Reviewer no attention. Benchmark named as
Snagit and Cobalt Capture. `BG-1`..`BG-6` untouched and unrenumbered.

**decision** — `BG-7` was written as a separate goal rather than folded into `BG-6`, and the brief now
says why in its own text. They read as the same goal and are not: `BG-6` is about how *often* the
Reviewer has to touch the tool's own machinery; `BG-7` is about what each of those touches costs. The
shipped Settings screen satisfies `BG-6` completely — four choices, set once — and is exactly the
screen the owner could not read. Folding them would have made the failure unsayable.

**change** — Four Success Criteria added for `BG-7`, kept separate from the 120-second measure because
they are gathered differently: by watching a first encounter, not by timing a loop. One of them
carries a number on purpose (WCAG AA contrast in both Windows themes) so that at least one part of the
bar is settled by a test instead of by taste.

**decision** — Those four criteria are what makes `DEC-005` checkable. `DEC-005` defers `sharing` and
`agent-access` "until the experience bar set at G2 is met", and its own Cost section admits that bar
did not exist when it was written. It exists now in draft form here; G2 sharpens it into `FR`/`NFR`.

**change** — Two constraints added: the desktop-first ordering (`DEC-005`) and one-executable
(`DEC-003`). Both are written as forbids, matching the section's existing form.

**event** — Competitor found and read during this update: Cobalt Capture (`https://cobaltcapture.com/`),
which the owner named. It is a direct competitor, not a neighbouring tool — same problem, same named
list of coding agents, Markdown output.

**assumption** — The owner recommended Cobalt Capture *for its annotation*. Reading the product, it
deliberately does the opposite: no markup tools, a crop taken at capture time and an editable
paragraph beside each screenshot. The brief records what the product actually does rather than what it
was recommended for, because the finding is the more useful one either way — a second team building
for the same audience independently concluded that annotation-heavy markup is not what this audience
needs. That corroborates `OQ-4` (numbered markers are sufficient) instead of threatening it. Worth
raising with the owner: what they liked may have been Cobalt's *presentation* of the note beside the
image, which is a layout idea and lands in `wdi-ux`, not an annotation idea.

**event** — Cobalt's actual differentiator is voice dictation of the note. Recorded in the brief as
the first idea to reach for if the note field turns out to be where the loop slows. Not proposed for
scope; nothing measured says the note field is slow.

**assumption** — Filed `OQ-20`: Snagit and Cobalt Capture may be the wrong benchmark for a product
whose reader is a machine. Both are built for human readers. This is the assumption most likely to be
wrong in this update, and it sits underneath `BG-7` itself.

**event** — `OQ-17` (MCP want unformed) and `OQ-18` (four presets distinguishable) also added to the
Assumptions section, cross-referenced to `DEC-005` and `DEC-004`.

### Not done, and why

- **No Product Component list.** That belongs to `wdi-init` intent `component`, and it already exists.
- **No `bmad-deep-recon` run on the competitive landscape.** The owner named the benchmark directly and
  one page was read to ground it. A full teardown of Snagit and Cobalt is real work that `OQ-20` may
  eventually justify; it was not done here and the brief does not pretend otherwise.
- **The Problem, The Solution, Who This Serves, and Scope are unchanged.** Nothing the owner reported
  was a problem-statement failure. The problem was right; the surface carrying it was not.
- (change) BG-8 born 2026-09-01, closing OQ-31, and the number matters: the OQ row said 'a sixth goal' and BG-6 and BG-7 have both existed since G1, so the new goal is the EIGHTH. CAP-12 (Export PDF) had hung off BG-2 under a comment admitting the fit was known to be imperfect - BG-2's measure is a handoff TIME that cannot report on an exporter, and BG-2 promises 'no file management' while an export exists to produce a file a person manages, so the capability was serving a goal whose own words it contradicts. Owner chose a new goal over amending BG-2's measure, because stretching that phrase would weaken a promise the product holds elsewhere. BG-8 reads 'A review is readable by someone who does not have Snapdown', measured as zero Findings lost, reordered, or stripped of image or note between a Bundle's stored Markdown and the document exported from it. CAP-12's goal: re-pointed BG-2 -> BG-8. SECOND change in the same pass, from OQ-20: BG-7's bar is SPLIT. It used to end 'the bar is the experience of Snagit and of Cobalt Capture', applied to the whole product; the capture half keeps that benchmark, the handoff half is measured against BG-2 and BG-3 instead, because its reader is a machine. This lands on BG-7's own wording in the brief, which nobody had noticed was where that sentence lived - it was being treated as ticket 06's problem alone.
