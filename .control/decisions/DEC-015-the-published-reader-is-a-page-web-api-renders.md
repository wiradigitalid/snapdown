---
type: decision
id: DEC-015
status: accepted
serves: [CAP-8]
touches:
  - .control/registry/components.yaml
  - .control/registry/defects.yaml
  - .control/structure-codebase.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l2-containers.md
  - .how/_platform/cross-cutting.md
  - .how/_platform/inventory-api.md
  - .how/_platform/inventory-screen.md
  - .how/sharing/SDD-sharing.md
supersedes: null
superseded_by: null
created: "2026-09-01"
---

# DEC-015 — The published Bundle reader is a page `web-api` renders, not a container of its own

> **Raised on the owner's instruction of 2026-09-01** — *"bereskan ini"*, against the drift reported
> the same day: `web-ui` registered as a `built: true` container with nothing inside it.

## Decision

The `web-ui` container is **withdrawn from the registry, the C4 L2, and the codebase map.** It was
registered as *"React + Vite single-page application, served as static assets by web-api"*, and no
part of that description was ever true.

`LC-027 bundle-reader` is **not** retired. It moves to `web-api`, because that is where it actually
runs: the published Bundle page is server-rendered HTML, built in Go.

**This decision does NOT answer `OQ-22`, and MUST NOT be read as answering it.** That question — is a
bare `<pre>` sufficient, or should the reader become three real screens — stays open and stays the
owner's. What is settled here is only *where the reader lives today*, which is a fact about deployment,
not a promise about design. If `OQ-22` is later answered "build the screens", they can be built inside
`web-api` or a container can be born for them then; **birthing a container is cheap, and claiming one
exists is what has been expensive.**

## Why

**Three artifacts described a deployable container that was never built.** `BUG-8` established this on
2026-08-23 and it has been open since: `components.yaml` claimed a fourth built container, the C4 L2
drew it and argued at length for why it *is* a container rather than static assets, and
`structure-codebase.md` carried the `### web-ui` heading that `V25` requires of a `built: true`
container.

**The name collision is what hid it, and it is worth naming.** `web-ui` the container and `web/ui` the
package read as the same thing in every document, and only one of them was real — so the structure
map's heading was satisfied by describing a React component *library* while the registry meant a
deployable *application*. `V25` passed throughout, correctly: it can ask whether a heading exists, not
whether the heading describes the thing it is named after.

**The code has moved twice since, and both moves point the same way.** `DEC-007` removed React from the
product entirely, so a React SPA is no longer a thing left unbuilt — it is a thing the architecture does
not have. `OQ-27` then deleted `web/ui` on 2026-09-01, taking the last artifact anyone could mistake for
the container. Meanwhile `apps/web-service` has been rendering the page itself the whole time:
`server.go` builds an HTML document inline with `fmt.Sprintf` and serves it as `text/html`, and there
is no `FileServer`, no `embed`, and no `http.Dir` anywhere in it. The reader was never missing. It was
in Go.

**`DEC-005` does not block this, and the belief that it did is a recorded defect pattern.** `BUG-8`
carried `blocked_by: DEC-005, which freezes sharing`. That decision says, in its own words: *"This
decision does not forbid a fix. It forbids new work."* `AGENTS.md` already names this exact
misquotation as the reason `BUG-3` and `BUG-10` sat unfixed — `BUG-3` being a public,
unauthenticated HTML-injection path. **`BUG-8` is the third instance of one register field
misreading one decision**, and the pattern is now three for three: every time, the field said blocked
and the decision said otherwise.

Nor does this remove anything `DEC-005` protects. Its guarantee is *"nothing built for them is
removed"*, and nothing was built.

## Cost

**`AD-3` and `AD-5` lose a name from their `Binds`, and that is the part to be careful about.** `AD-5`
— *every surface outside the desktop process is read-only* — listed `web-ui` among the surfaces its
Rule binds. Dropping the name does not weaken the invariant: `web-api` is still bound, and it is the
process that actually serves the page, so the read-only guarantee over the published surface is intact
and is now attached to the thing that can break it. But an `AD-N` with a shorter Binds list is a
smaller net, and if a browser client is ever built it MUST be added back to both — that is not
automatic and nobody will be reminded.

**The C4 L2 loses a container and its argument.** `c4-l2-containers.md:53` reasons explicitly that the
reader *is* a container because it runs in the reader's browser as its own process. That reasoning is
sound and would be correct again the day such a client exists. It is deleted rather than kept as a
comment, and this paragraph is the only surviving record of it.

**`inventory-screen.md` rows 14 and 15 keep describing screens that do not exist.** They are `BUG-2`
and they stay, because withdrawing them is `OQ-22`'s answer and not this decision's. So the corpus
still promises a surface nobody has built — the promise is simply no longer accompanied by a false
claim about a deployed container to serve it from.

## Alternatives

**Set `built: false` instead of withdrawing.** Rejected: `built: false` means *we deploy someone
else's implementation*, and nobody deploys this one. The method is explicit that something whose
runtime we do not deploy is an external system belonging at C4 L1, and that its absence from the
registry is the check.

**Retire `LC-027` along with the container.** Rejected, and this is the substantive choice inside this
decision. Retiring it would say the published Bundle has no reader, which is false — one exists and
people can open it. It would also quietly answer `OQ-22` in the negative by deleting the thing that
question is about.

**Wait for `OQ-22`, and for ticket 06 to lift `DEC-005`.** This is what was done for nine days, and it
is what this decision reverses. The wait was founded on the `blocked_by` misquote above; there was
never anything to wait for, and in the meantime the registry asserted a deployed artifact that does
not exist.

## Reversal trigger

- **A browser client for a Publication is built.** Then `web-ui` — or whatever it is named, and it
  SHOULD be named something that cannot be confused with a package path — is born as a container
  through `wdi-blueprint` intent `platform`, `LC-027` moves into it, and it goes back into `AD-3` and
  `AD-5`'s `Binds`.
- **`OQ-22` is answered "build the three screens" and they are built inside `web-api`.** Then nothing
  about this decision reverses; only `inventory-screen.md` rows 14 and 15 stop being `BUG-2`.

## Trace

`BUG-8`, open 2026-08-23 to 2026-09-01, closed by this decision. `OQ-22` deliberately left open.
`BUG-2` unchanged — its screens are still promised and still absent.
