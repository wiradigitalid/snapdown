---
type: decision
id: DEC-020
status: applied
serves: [CAP-8]
touches:
  - .control/registry/defects.yaml
  - .control/registry/components.yaml
  - .how/_platform/inventory-screen.md
  - .how/sharing/SDD-sharing.md
  - .how/bundle/01-ux/DESIGN.md
  - .how/bundle/SDD-bundle.md
  - .control/questions/assumptions.md
  - .control/questions/answered.md
supersedes: null
superseded_by: null
created: "2026-09-04"
accepted_by: DEC-019
date: "2026-09-04"
---

# DEC-020 — The publish dialog and the reader's not-found page are withdrawn; the bare reader `DEC-015` already found stands, corrected in wording

## Decision

Of the three things `BUG-2` named as delivered in W5 and never built, two are withdrawn outright: row
11 of `inventory-screen.md` (`/bundles/:id/publish`, the publish confirmation dialog, `LC-022`
`publish-dialog`) and row 15 (`/b/:slug`, the refused state, `PublicationNotFound.tsx`). Neither has
ever had a line of code behind it, and neither will while `DEC-005` holds.

Row 14 (the published Bundle reader) is **corrected, not withdrawn**. `DEC-015` already found, on
2026-09-01, that `LC-027 bundle-reader` is real: `web-api` serves `GET /b/{slug}` as a server-rendered
HTML page today (`apps/web-service/internal/server/server.go:132-159`). What row 14 promised —
`PublishedBundleReader.tsx`, matching the mockup at `04-bundle-assembly-modal.html` — is what is
withdrawn. What actually exists, a bare `<pre>` dump with no stylesheet and no rendered images, stands
and is now described as what it is rather than as a stand-in for the file that was never written.

This is `OQ-22`'s answer — *"Serving a published Bundle as raw Markdown in a bare `<pre>` is
sufficient, because Snapdown's reader is a machine and the human reader those inventory rows promise is
not actually wanted"* — and the answer is: sufficient, for now. Nothing here is a permanent
renunciation. If `DEC-005` lifts and the owner wants the three screens built, they can be proposed again
through `wdi-product`.

## Why

`BUG-2`'s own `fix:` field names the choice plainly: *"Two honest options, and this is a decision rather
than a task. Either build the reader SPA and the publish dialog, or withdraw rows 11, 14, 15 and LC-027
and state that the published surface is machine-facing only. The second is cheaper and may be correct:
Snapdown's reader is a machine."*

The choice is forced, not open, while `DEC-005` stands. Its own text: *"This decision does not forbid a
fix. It forbids new work."* A publish confirmation dialog and a dedicated not-found page do not exist in
any form — building either from nothing is new work by any reading of that line, so `DEC-005` forbids
it. Withdrawing an unbuilt promise is a document correction: it changes what the corpus claims, not what
runs, so it is compliant with the freeze while the freeze holds.

`LC-027` is the one place `BUG-2`'s own evidence is now stale, and it is worth saying so rather than
carrying the misquote forward into a second decision. `BUG-2` was filed 2026-08-23 and reads *"LC-027
`bundle-reader` is registered in `components.yaml` with nothing implementing it."* `DEC-015`,
2026-09-01, corrected exactly that: the reader was never missing, it was registered against the wrong
container. `web-api` has been rendering it in Go the whole time. Retiring `LC-027` now, as `BUG-2`'s
literal fix text would have it, would re-introduce the claim `DEC-015` already spent a decision
correcting — *"Retire `LC-027` along with the container. Rejected... Retiring it would say the published
Bundle has no reader, which is false — one exists and people can open it."* This decision does not
retire `LC-027`, and does not reopen `DEC-015` to do it. What is withdrawn is the promise of a screen
matching a mockup; the screen `LC-027` actually renders is not that, and is not going away.

Row 15's withdrawal rests on a second, independent finding, not just `BUG-2`'s original evidence:
`writeIdentical404` (`server.go:195-205`) returns a JSON error envelope on every path, never HTML. There
is no bare page standing in for `PublicationNotFound.tsx` the way there is for row 14 — the refused
state has never rendered anything a person looks at, only a body a machine parses. Row 15 is withdrawn
outright for that reason, not corrected the way row 14 is.

## Cost

- **A Reviewer composing a Bundle gets no in-product publish confirmation beyond whatever `LC-020`'s own
  client already surfaces.** The dialog row 11 promised does not exist and is not scheduled while
  `DEC-005` holds.
- **A person following the published URL still gets a bare, unstyled Markdown dump**, not a designed
  reading surface, and a refused slug gets a JSON body, not a page. Both were already true; they are now
  the documented state rather than a gap nobody closed.
- **`LC-022 publish-dialog` is retired**, not merely re-scoped — no replacement stands in `desktop-app`.
  `bundle`'s own `01-ux/DESIGN.md` and `SDD-bundle.md` named it as a live cross-component dependency;
  both are corrected here so neither points at a component that no longer exists.
- **`OQ-22` closes** on "sufficient, for now" rather than staying open — a real choice, not a
  formality: the owner's want for the three screens is settled as "not currently wanted" rather than
  left unformed. If `DEC-005` lifts and that reading turns out wrong, the promise can be reopened through
  `wdi-product`; this decision does not need superseding to allow that, because it never claimed the
  withdrawal was permanent.
- **A coding agent's path is unaffected.** `GET /b/{slug}` with `Accept: text/markdown` already returned
  raw Markdown before this decision and does after it — `FR-24`'s statement and proof were already
  scoped to the machine-facing path and needed no correction; they were checked and left as they stood.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | `OQ-22` — closed by this decision, answer: sufficient for now |
| Source material | `BUG-2` (`.control/registry/defects.yaml`); `DEC-005`; `DEC-015`; `BUG-23`'s rescoped note; `apps/web-service/internal/server/server.go:132-205` |
