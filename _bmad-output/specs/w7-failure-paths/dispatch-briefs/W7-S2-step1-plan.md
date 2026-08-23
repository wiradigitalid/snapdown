# W7-S2 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W7**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w7-failure-paths/`
- `story_id`: `W7-S2`

## `BUG-3` — the published page interpolates the Reviewer's Note without escaping it

`apps/web-service/internal/server/server.go:145`:

```go
html := fmt.Sprintf(`<!DOCTYPE html>...<body><pre>%s</pre></body>...`, slug, b.Markdown)
```

`b.Markdown` is the Reviewer's own Note text, published verbatim. `%s` performs **no escaping**, so a
Note that closes the `pre` block and opens a script tag leaves the block and executes in the browser
of whoever opens the URL. The endpoint is **public and unauthenticated**.

**Who is exposed, stated accurately.** The author and the publisher are the same person, so this is
not third-party injection. The exposure is to **whoever the Reviewer hands the URL to** — and a Note
is free text that may be pasted out of the page being reviewed, which is precisely where a hostile
string would come from.

## The fix, and one part of it that looks unnecessary and is not

`html/template`, or `html.EscapeString` on `b.Markdown` before interpolation.

**Escape the slug too.** It is a 160-bit CSPRNG value today, and that is a property of the
*generator*, not of this render. A renderer that is only safe because of what its caller happens to
pass is one refactor away from not being safe.

## `DEC-005` permits this and forbids widening it

`sharing` is frozen. This story is allowed by the decision's own sentence:

> *"This decision does not forbid a fix. It forbids new work."*

So: **no new route, no new promise, no UX pass, and no depth above the `guarded` the component
already carries.** In particular, do **not** build the reader SPA that `inventory-screen` row 14
describes and that does not exist — that is `BUG-8` and `OQ-22`, and it is the owner's decision, not
this story's.

## `NFR-15` MUST survive unchanged, and you should pin it

> The web service exposes no route that lists, searches, or enumerates Publications, and returns the
> same refusal for an unknown slug as for a revoked one.

Escaping is a change to one render, not to the route table. Plan a test that **pins** that rather
than assuming it — `an_unknown_slug_still_returns_the_identical_refusal` is in the test list for
exactly this reason.

## Why nothing caught it, which tells you what to write

The existing route tests assert the `NFR-15` identical-404 and the response codes. **Nothing asserts
what the HTML body does with hostile content.** The endpoint also predates any document describing
what it renders.

And there is a second reason worth carrying: **no workflow touched Go at all until 2026-08-23.**
`apps/web-service` was never built, vetted, or tested by CI, so this shipped into a container with
nothing behind it and a regression would have been equally invisible. A `web-service` job was added
to `desktop-ci.yml` that day — `go build`, `go vet`, `go test`. That is regression protection, not a
fix.

## A gap reported upstream — read this so you do not try to close it

`BUG-3` records `contradicts: [NFR-15]`, and that fit is loose: `NFR-15` is about enumeration and
identical refusals, not about rendering a Note as text rather than as markup. All six `sharing` NFRs
(`NFR-10`–`NFR-15`) were read and **none of them promises output encoding on the published page.**

The fix is unambiguous regardless, so this does not block you. What is missing is the *promise* the
fix restores, and that is a corpus gap already reported upstream in `SPEC.md` § Open Questions.
**MUST NOT invent a requirement to fill it**, and MUST NOT edit `.what/` or `.how/`.

## The tests that matter

`waves.yaml` records four, carried through verbatim:

```
go::a_note_containing_markup_is_escaped_in_the_rendered_page
go::a_note_that_closes_the_pre_block_cannot_reach_the_browser_as_markup
go::the_slug_is_escaped_in_the_rendered_page
go::an_unknown_slug_still_returns_the_identical_refusal
```

**Assert the behaviour, not a literal.** A test that hardcodes the expected escaped string beside the
implementation's own escaping is a test that cannot fail; this repository has landed that mistake
three times, and `contrast.test.ts` is the worked example of the fix — it parses the real input and
was verified by mutation.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` before proposing any fix. A third failed fix attempt is the signal to
  escalate.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed** — every command in `AGENTS.md` § Code, plus `go build`,
  `go vet`, `go test` from `apps/web-service`. Note that `cmd | tail` reports `tail`'s exit code, and
  `cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did.
- **Write UTF-8, and no BOM.**
- **Never commit a captured screenshot.** No scratch files in the commit. **Do not push.**

## Done means

`_bmad-output/specs/w7-failure-paths/stories/W7-S2-*.md` exists, carries an `<intent-contract>`, and
its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
