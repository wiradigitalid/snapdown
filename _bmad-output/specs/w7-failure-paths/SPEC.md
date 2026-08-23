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

# W7 — The panic surface, and the two fixes the freeze allows

## Why

**A pain to solve, and the same pain three times.** Three code paths decline to report a failure they
have already detected. None of the three is a missing feature; each is a promise the product already
made and does not keep.

The severity is not evenly spread, and the ordering follows it. `BUG-12` can make the entire product
appear not to exist: five store opens are `.expect()`, a Tauri release binary on Windows has no
console, and `AD-11` puts the tray, the hotkeys, the overlay and the Editor in that one process — so
a corrupt `library.db` means the Reviewer double-clicks `Snapdown.exe` and **nothing happens at all**.
`DEC-003` predicted exactly this class in writing and nobody then went and looked at the unwraps.
`BUG-3` interpolates the Reviewer's own Note into HTML with no escaping, on a public unauthenticated
endpoint. `BUG-10` can hand an agent an error whose message is the empty string.

Two of the three components are frozen by `DEC-005`, and this wave is permitted by that decision's
own sentence: *"This decision does not forbid a fix. It forbids new work."*

## Capabilities

- **CAP-6** — Keep the tool out of the way (the startup half)
  - **intent:** The Reviewer learns what went wrong when Snapdown cannot open its own store, instead
    of watching the application fail to appear.
  - **success:** With a `library.db` that cannot be opened, launching the product produces a visible
    report naming the path of the file that failed, and the unreadable file is still on disk
    unmodified with no fresh store beside it (`BR-118`). With a readable store, startup is unchanged.

- **CAP-8** — Let an agent on another host read a Bundle over HTTPS (the rendering half)
  - **intent:** A published Bundle's page presents the Reviewer's Note as text, so that whoever is
    handed the URL reads what was written rather than running it.
  - **success:** A Note whose text closes the surrounding block and opens a script tag is served back
    escaped: it appears in the page as the characters the Reviewer typed, and reaches the browser as
    text rather than as markup. The unknown-slug refusal is byte-for-byte what it was before.

- **CAP-7** — Let an agent on this machine read a Bundle (the refusal half)
  - **intent:** Every refusal the bridge returns to an agent carries a message that says something.
  - **success:** When the error body cannot be read, the agent receives a message naming the status
    code and stating that the error body could not be read. No path through
    `parse_error_response` can return the empty string.

## Constraints

- **`BR-118` MUST NOT be weakened to make the report easier.** The settings store is opened, never
  created over; a store that cannot be read is reported with its path and no fresh one is started
  beside it. The current panic keeps this half — a fix that starts a replacement store to stay
  running trades a visible failure for silent data loss and is worse than the defect.
- **`DEC-005` permits a fix and forbids new work.** `W7-S2` and `W7-S3` MUST NOT widen beyond their
  defect: no new FR, no new use case, no UX pass, and no depth above the `guarded` those two
  components already carry.
- **`AD-11` leaves no surviving surface to report into.** One process owns the tray, the hotkeys, the
  overlay and the Editor, so `W7-S1`'s report cannot assume a window already exists.
- **`AD-7` binds the bridge's error shape.** Every failure crossing a process boundary is returned in
  the `cross-cutting.md` envelope with a code from that catalogue; `W7-S3` changes the message, never
  the envelope.
- **`NFR-15` MUST survive `W7-S2` unchanged.** The service still exposes no route that lists,
  searches, or enumerates Publications, and an unknown slug and a revoked one still get the same
  refusal. Escaping is a change to one render, not to the route table.
- **A test that asserts a literal instead of the behaviour it claims to cover is a defect**, not a
  style choice. This repository has landed that mistake three times.
- **Never commit a captured screenshot.** The repository is public and the brief forbids it.

## Non-goals

- **Rewriting the published page.** `W7-S2` escapes what is already rendered. `inventory-screen`
  row 14 describes an SPA that does not exist, and building it is not in this wave.
- **A recovery, repair, or migration path for a corrupt store.** `W7-S1` reports; it does not mend.
  Anything that writes to a store it could not read is forbidden by `BR-118`.
- **Sweeping the remaining `unwrap`/`expect` calls.** `BUG-12`'s own register entry already swept and
  deliberately did not register three groups — 26 infallible `Header::from_bytes` calls, the Tauri
  `run` call, and two bridge serialisations. Re-raising them is not this wave's work.
- **Any new promise for `sharing` or `agent-access`.** Forbidden by `DEC-005`.
- **Closing the six use cases V3 reports as unscheduled**, or the seven more this wave adds. That is
  the `OQ-21` record gap, decided in advance and recorded in `waves.yaml`.

## Success signal

A Reviewer whose `library.db` has been corrupted launches Snapdown and is told which file is at
fault, rather than seeing nothing happen — and the corrupt file is still there afterwards, untouched.
A Bundle whose Note contains angle brackets is handed to someone as a URL, and they read the
brackets. An agent that hits a failed call learns what failed.

## Assumptions

- `AD-7` and `BR-17` are the exact anchor for `W7-S3`: `BR-17` says a refusal is always
  distinguishable from an empty result, and an error whose message is the empty string is not.

## Open Questions

- **No requirement covers output encoding on the published page.** `BUG-3` records
  `contradicts: [NFR-15]`, but `NFR-15` is about enumeration and identical refusals, not about
  rendering a Note as text rather than as markup. All six `sharing` NFRs (`NFR-10`–`NFR-15`) were
  read and none fits. The fix is unambiguous regardless, so this does not block `W7-S2`; what is
  missing is the promise it restores. **Reported upstream to `wdi-product`; MUST NOT be invented
  here.**
