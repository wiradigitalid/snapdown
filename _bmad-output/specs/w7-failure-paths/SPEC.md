---
id: SPEC-w7-failure-paths
companions:
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .what/business-rules.md
  - .what/settings/02-rules/rules-settings.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/cross-cutting.md
  - .how/settings/SDD-settings.md
  - .how/sharing/SDD-sharing.md
  - .how/agent-access/SDD-agent-access.md
  - .control/decisions/DEC-003-one-process-two-windows.md
  - .control/decisions/DEC-005-desktop-first-ordering.md
  - .constitution/project/codebase-stack-guide.md
sources:
  - .control/registry/requirements.yaml
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete,
> preservation-validated contract for what to build, test, and validate. Source documents listed in
> frontmatter are for traceability — consult them only if you need narrative rationale or prose
> colour this contract intentionally omits.

# W7 — Three failures the code declines to report

## Why

**A pain to solve, and the same pain three times.** Three code paths detect a failure and then
decline to report it honestly. None is a missing feature; each is a promise the product already made
and does not keep.

**`W7-S1` was re-scoped on 2026-08-24 and the reason matters more than the re-scope.** This wave
opened against `BUG-12` — five `.expect()` store opens that made a corrupt `library.db` end the
process with no message at all. `BUG-12` was **already fixed**, by `W6-S5` (commit `aa30434`), which
was not scoped to it and closed it as a side effect of needing a fallible startup path. The register
row still read `open`, a planner was dispatched against it, and it wrote a full implementation plan
for code that already existed. The SPEC review caught it by reading the code instead of the register.

What `W6-S5` did **not** close is real, and it is what `W7-S1` now carries. The sharpest is
`BUG-15`: all five stores switch the database to WAL **before** running `quick_check`, so a file with
a valid header and corrupt pages **is written to** and `-wal`/`-shm` **are created beside it** — and
the Reviewer is then shown a message saying nothing was touched. `BR-118` is broken by the code path
that reports it. The existing byte-identity test cannot see this, because the only corrupt fixture
anyone has used is garbage bytes, which SQLite rejects before a single pragma runs.

`BUG-3` interpolates the Reviewer's own Note into HTML with no escaping, on a public unauthenticated
endpoint. `BUG-10` can hand an agent an error whose message is the empty string.

**Why a wave, when `DEC-005` says this should not be one.** Two of the three components are frozen.
`DEC-005` permits the work — *"This decision does not forbid a fix. It forbids new work."* — but its
Cost section also names the vehicle: *"A defect in the frozen components has an awkward home. It is a
fix, not a wave, and the method has no third thing. It lands as a defect row and a patch release, and
that path is thinner than a wave's."* A wave is used anyway because **`BUG-3` reaches a requirement**,
and Fast Path work that turns out to touch one must stop and be raised to a wave regardless. That is
the whole argument; it is not a claim that `DEC-005` prefers a wave.

## Capabilities

- **CAP-6** — Keep the tool out of the way (the startup half)
  - **intent:** When Snapdown cannot open its own store, the Reviewer is told which file is at fault
    and can trust that nothing was written to it.
  - **success:** With a `library.db` whose header is valid and whose pages are corrupt — not merely
    a file of garbage bytes — launching the product produces a report naming the path that failed,
    and afterwards the file is **byte-identical** to its pre-launch state with **no `-wal` or `-shm`
    beside it**. With a readable store, startup is unchanged. The process exits without panicking.

- **CAP-8** — Let an agent on another host read a Bundle over HTTPS (the rendering half)
  - **intent:** A published Bundle's page presents the Reviewer's Note as text, so that whoever is
    handed the URL reads what was written rather than running it.
  - **success:** A Note whose text closes the surrounding block and opens a script tag is served back
    escaped: it reaches the browser as text, not as markup. The raw-Markdown paths still serve
    verbatim bytes. The unknown-slug refusal is byte-for-byte what it was before.

- **CAP-7** — Let an agent on this machine read a Bundle (the refusal half)
  - **intent:** Every refusal the bridge returns to an agent carries a message that says something,
    and a code the agent can act on.
  - **success:** When the error body cannot be read, or reads back empty, the agent receives a
    message naming the status code and what went wrong, **carrying a code from the `cross-cutting.md`
    catalogue**. No path through `parse_error_response` returns the empty string.

## Constraints

- **`BR-118` MUST NOT be weakened, and it is currently broken.** The store is opened, never created
  over; a store that cannot be read is reported with its path and no fresh one is started beside it.
  **A writing pragma is a write.** `journal_mode = WAL` mutates page 1 and creates two files, so it
  MUST NOT run before the integrity check. `SQLITE_OPEN_CREATE` is **not** the defect and MUST stay —
  creating a store that is absent is first run, and `BR-118` forbids creating one *over* a corrupt
  file, which that flag does not do.
- **A corrupt store MUST NOT be repaired, migrated, or replaced.** Reporting is the whole behaviour.
- **`AD-11` leaves no surviving surface to report into.** One process owns the tray, the hotkeys, the
  overlay and the Editor, so the report cannot assume a window exists. A native dialog is the answer,
  and it MUST be raised with `MB_SETFOREGROUND | MB_TOPMOST`: an unowned message box from a process
  with no foreground activation can open **behind** other windows, which is the "nothing happens at
  all" symptom this work exists to end.
- **`AD-7` binds the error envelope, and the bridge MUST NOT invent an error.**
  `cross-cutting.md` states it directly: *"The MCP Bridge does not invent its own errors. It maps a
  Local API envelope onto an MCP tool error, preserving `code` and `message` verbatim."* Where the
  bridge has no envelope to map — the body could not be read — it MUST synthesise a message carrying
  a **catalogue code**, not a free-form string. `AD-7`'s own rule is what makes this the point: *"A
  refusal MUST be distinguishable from an empty result **by its code**, never only by its body being
  empty."*
- **`DEC-005` permits a fix and forbids new work.** `W7-S2` and `W7-S3` MUST NOT widen beyond their
  defect: no new FR, no new use case, no UX pass, no depth above `guarded`.
- **`NFR-15` MUST survive `W7-S2` unchanged.** No route lists, searches, or enumerates Publications,
  and an unknown slug and a revoked one still get the same refusal.
- **The raw-Markdown paths MUST stay unescaped.** `GET /b/{slug}/raw.md`, and `GET /b/{slug}` under
  `Accept: text/markdown` or `text/plain`, serve verbatim bytes — an agent consuming a Bundle needs
  pristine Markdown. Escaping is scoped to the HTML render alone.
- **A test that asserts a literal instead of the behaviour it claims to cover is a defect**, not a
  style choice. This repository has landed that mistake three times. An expected escaped string
  hardcoded beside the implementation's own escaping is the same mistake in a new place.
- **A test fixture MUST be legal on Windows.** This repository's primary platform is Windows and its
  Go job runs on Linux; a fixture that only Linux accepts is a green CI over a red developer machine.
- **Never commit a captured screenshot.** The repository is public and the brief forbids it.

## Non-goals

- **Rewriting the published page.** `W7-S2` escapes what is already rendered. `inventory-screen`
  row 14 describes an SPA that does not exist; building it is `BUG-2`/`OQ-22` and the owner's call.
- **A recovery, repair, or migration path for a corrupt store.** `W7-S1` reports; it does not mend.
- **A general `unwrap`/`expect` sweep.** `BUG-12`'s sweep deliberately left three groups unregistered
  and two of those exclusions still hold — 26 infallible `Header::from_bytes` calls, and two bridge
  serialisations. **The third has expired** and is now `BUG-16`: `lib.rs:347` was excused because
  *"there is nothing left to report with"*, which was true only while every store open panicked
  before reaching it. That is in scope; the other two are not.
- **Path-traversal hardening on the publish slug.** `store.go` joins the slug into a filesystem path
  unfiltered. Real, out of scope here, and named so it is discovered-and-written rather than
  discovered-and-forgotten.
- **Proving the startup dialog is visible.** It cannot be asserted by `cargo test`, and `OQ-24`
  records that this project has no working way to run a UI verification.
- **Any new promise for `sharing` or `agent-access`.** Forbidden by `DEC-005`.
- **Closing the use cases V3 reports as unscheduled.** That is the `OQ-21` record gap.

## Success signal

A Reviewer whose `library.db` has corrupt pages launches Snapdown, is told which file is at fault,
and finds it afterwards byte-for-byte as it was, with nothing new beside it. A Bundle whose Note
contains angle brackets is handed to someone as a URL, and they read the brackets. An agent that hits
a failed call learns what failed and gets a code it can branch on.

## Assumptions

- `AD-7` and `BR-17` are the exact anchor for `W7-S3`: `BR-17` says a refusal is always
  distinguishable from an empty result, and an error whose message is the empty string is not.
- `internal` is the right catalogue code for a body that could not be read — *"something the producer
  did not anticipate"*. If the reader prefers `unavailable` for the dropped-connection reading, that
  is a refinement, not a contradiction.

## Open Questions

- **No requirement covers output encoding on the published page.** `BUG-3` records
  `contradicts: [NFR-15]`, but `NFR-15` is about enumeration and identical refusals, not about
  rendering a Note as text rather than as markup. All six `sharing` NFRs (`NFR-10`–`NFR-15`) were
  read and none fits. **Reported upstream to `wdi-product`; MUST NOT be invented here.**

  **This is not `DEC-005`'s reversal trigger firing, and the distinction is deliberate.** That
  trigger is *"a defect … that cannot be fixed as a patch — one that needs a new promise."* The
  **fix** here is a patch and needs no promise: escaping is unambiguous without one. What is missing
  is the promise the fix *restores*, which is a gap `wdi-product` closes on its own schedule. If that
  distinction turns out not to hold, the trigger has fired and this becomes a re-plan rather than a
  wave.

- **`OQ-25` — V11 forces this wave to state a dependency that does not exist.** The three stories
  touch disjoint trees. `waves.yaml` records the chain as intended order, not a build gate, and all
  three MAY be dispatched in parallel.
