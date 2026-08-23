---
type: reconcile-report
created: "2026-08-23"
scope: full sweep, after the G1–G4 re-run, the UX gate, W6 opening, and W6-S1 landing
read_only: true
---

# Reconcile — 2026-08-23

**Scope.** Everything. G1–G5 are all in play: W6 is open and W6-S1 has landed. `finding`, `bundle`
and `settings` are at `mode: deep`; `agent-access` and `sharing` at `guarded`, and both are frozen by
`DEC-005`, so nothing is expected to have moved in them.

**Excluded, deliberately:** nothing. A narrower scope was available and not taken — this sweep follows
the largest batch of cross-layer edits this product has had, which is exactly the condition the pass
exists for.

**This report edits nothing.** Every fix below names the skill that owns it.

## Carried from the validators

`validate.py` — 16 findings, and all 16 are already justified line by line in
`.github/validate-baseline.README.md`: six `V3` that are permanent and correct, eight `V18` that leave
as each W6 story is planned, two `V24` inside a BMad package file. Not re-derived here.

`inventory.py` — 5 plan-versus-code gaps, all true. Three are `BUG-2`; two are W6 work. Not
re-derived here.

`.control/generated/` is current — regenerated after the last registry change.

---

## DRIFT — a right side exists, and it needs carrying across

### D1 · Three decisions are `accepted` and behave as `applied`

| | |
|---|---|
| **What** | `DEC-003`, `DEC-004` and `DEC-005` all read `status: accepted` with `touches: []`. Their content has already been carried into the brief, the PRD, the glossary, the spine, the C4 set, `business-rules.md`, three SDDs, three `EXPERIENCE.md`, three `DESIGN.md`, `SPEC.md` and `waves.yaml`. |
| **Where** | `.control/registry/decisions.yaml` and the three files in `.control/decisions/` on one side; roughly thirty corpus files on the other |
| **Which is right** | **Reality.** They are applied. The registry is the side that is wrong, and it is wrong in the direction that matters: `wdi-decision` says *applying is what freezes a decision, not accepting*, so three decisions that everything now depends on are still formally editable. `V8` cannot see this — it checks that an `applied` decision names a non-empty `touches`, not that an `accepted` one which has plainly been applied was ever raised. |
| **Who fixes it** | `wdi-decision` intent `apply` — which is also the step that was skipped. It lists the targets, fills `touches` with the files actually changed, and raises `status: applied`. |

The honest reading of how this happened: the decisions were written, accepted, and then carried into
every layer by the gate skills as they ran, so the *work* of applying was done continuously and the
*act* of recording it never was.

### D2 · Thirteen documents assert colour literals that no longer exist

| | |
|---|---|
| **What** | Thirteen documents state, as present-tense fact, that components carry hard-coded hex values — "23 distinct hex literals live outside the token file", "`BundleView.tsx` carries `#f8fafc` (line 93)", "`HotkeySection.tsx` carries `#dcfce7`, `#166534`…". **W6-S1 removed every one of them.** A grep for a hex literal across `apps/desktop/src` now returns nothing, and `BundleView.tsx:93` reads `backgroundColor: 'var(--color-bg)'`. |
| **Where** | `.how/_platform/design-system.md` · `.how/{bundle,finding,settings}/01-ux/DESIGN.md` · `.how/{bundle,settings}/SDD-*.md` · `.how/settings/03-integrations/windows-shell.md` · `.what/bundle/04-usecases/EXPERIENCE.md` · five files in `_bmad-output/ux/capture-to-markdown/` |
| **Which is right** | **The code.** The documents were true when written this morning and the fix is what made them false — which is the good direction, and still drift. The `[MISSING]` evidence rows against `AD-10` in `SDD-settings.md`, `SDD-finding.md` and `SDD-bundle.md` are now **resolved**, and `sdd-guide.md`'s evidence ladder governs how a resolved claim is recorded rather than deleted. |
| **Who fixes it** | `wdi-component` for the three SDDs — the evidence labels are its slot. `wdi-ux` for `design-system.md` and the three `DESIGN.md`. |

**This is the check no validator can perform**, and it is the one this pass exists for: nothing in the
ID chain moved, every citation still resolves to a real file and a real line, and the sentences are
simply no longer true.

**Not a finding, and worth saying so:** `.work/ux-audit/AUDIT.md` also asserts them. It is a dated
record of what an audit saw on 2026-08-23, and a historical record that stayed accurate to its moment
is correct. It should not be "fixed".

**Still true, and checked:** `FindingsEditor.tsx:137` really does still render
`{f.markers.length} markers` as text. `BUG-5` is live.

### D3 · The run folder and the landed UX have diverged again

| | |
|---|---|
| **What** | `_bmad-output/ux/capture-to-markdown/` was resynced after the SPEC review's corrections, and has drifted again since — it carries the pre-W6-S1 literal claims that D2 describes, and `DESIGN-bundle.md` and `EXPERIENCE-bundle.md` were never resynced at all. |
| **Where** | `_bmad-output/ux/capture-to-markdown/` versus `.how/*/01-ux/` and `.what/*/04-usecases/` |
| **Which is right** | **The landed copies.** `wdi-ux` says the run folder must not be deleted because intent `update` reads it again — which is exactly why a stale run folder is a trap rather than a harmless copy. |
| **Who fixes it** | `wdi-ux`, in the same pass that fixes D2. |

---

## CONFLICT — no clearly right side; someone has to decide

### C1 · Three `sharing` screens: build them, or withdraw the promise

`BUG-2`. `inventory-screen.md` rows 11, 14 and 15 and `LC-027` promise a publish dialog and a reader
SPA. None exists. `GET /b/{slug}` returns the stored Markdown inside a bare `<pre>`.

For the **primary** audience — an agent fetching the URL — that is arguably sufficient. For the human
reader those rows promise, it is not. Building three screens and withdrawing three rows are both
defensible, and they are opposite answers.

**Who decides:** the owner, through `wdi-product` if the answer is *withdraw* — it changes a promise
and is not a patch. Filed as `OQ-22`. Blocked by `DEC-005` either way.

### C2 · Whether a composition test becomes a standing convention

`OQ-23`. Three defects — `BUG-4`, `BUG-5`, `BUG-6` — are one shape: a component built, unit-tested,
and mounted nowhere. W6-S2 and W6-S7 fix the three instances. Whether every future story pays a small
tax to prevent a fourth is a convention question, not a story.

**Who decides:** the owner. The interim control is the grep now written as the first pitfall in
`AGENTS.md` § Code.

### C3 · Whether `main`'s published history is rewritten

`BUG-7`. Twelve screenshots of the running product have been fetchable from `main` since W1. They are
out of the working tree and CI now refuses their recurrence; neither un-publishes them.

**Who decides:** the owner, and only the owner. Rewriting published history on a public repository
affects every clone and fork.

---

## Checked and clean

| Check | Result |
|---|---|
| Chain `BG-7 → CAP-9 → FR-27/28/29 → UC-24/25/26 → W6-S2/S3 → tests` | Intact end to end |
| Depth — anything written beyond its component's `mode` | None. All three desktop components are at `deep`, which demands everything present |
| Vocabulary — a domain noun not in the glossary, or a synonym for one that is | None. `Persona`, `Advanced`, `Auto`, `Sharp`, `Balanced`, `Small`, `Custom` and `Orphan report` were all added at G3. `Editor shell`, `capture rail` and `note pane` are build-unit and layout names, not domain nouns |
| Registry — an `LC` with no prose in the slot its `type` names | None. `LC-028` has `04-components/`; `LC-029`, `LC-030`, `LC-031` are `ui-*` and live in `01-ux/`, which is where their type belongs |
| Registry — a container in the C4 set but not in `containers` | None |
| `owns:` — two components claiming one entity | None |
| Homeless output in `_bmad-output/` | None. Every folder maps to a row in `corpus-guide.md`'s ownership table |
| Promise appearing first in `.how/` | None found |

### One item considered and deliberately not raised

`.what/settings/04-usecases/EXPERIENCE.md:103` says *"Target size at least 24×24 px."* A pixel value in
`.what/` looks like solution shape leaking across a layer boundary.

It is not. 24×24 CSS px is the **unit WCAG 2.2 AA states the requirement in**, and `language-guide.md`
holds that a technical term the industry writes a particular way is left as it is. Restating it as
"large enough to hit reliably" would make a checkable promise uncheckable. Raised here so that a later
reader finds it already weighed rather than missed.

---

## Summary

| | Count |
|---|---|
| Drift — a right side exists | **3** |
| Conflict — needs a decision | **3**, all three already filed as `BUG-` or `OQ-` |
| Clean | 8 checks |

**D1 is the one to act on first.** Three decisions everything now depends on are formally still
editable, and the fix — `wdi-decision` intent `apply` — is also the step that was skipped.

**D2 is the most interesting.** It is drift caused by a fix, in thirteen documents, invisible to every
validator, and it will recur on every story in this wave. Each story that removes a defect makes the
document describing that defect false. The evidence ladder in `sdd-guide.md` is the existing answer;
what this sweep suggests is that it needs running at **each story's close**, not only at the wave's.

---

## Resolution log — appended 2026-08-23, after the report was written

The report above describes the state **before** anything moved, and it is not edited. What was done
about it:

| Finding | Action | By |
|---|---|---|
| **D1** — three decisions `accepted` behaving as `applied` | `touches` filled from citations (10, 17, 11 files) and all three raised to `applied`. `V8` green. Step 2 of intent `apply` had nothing to do and the memlog says so | `wdi-decision` intent `apply` |
| **D2** — thirteen documents asserting removed literals | Resolved, **not deleted**: each claim moved to the past tense and stamped `Resolved by W6-S1 at 420ecce`, per the evidence ladder in `sdd-guide.md`. Three `[MISSING]` rows against `AD-10` struck through and marked done | `wdi-component` for the SDDs, `wdi-ux` for `design-system.md` and the three `DESIGN.md` |
| **D3** — the run folder diverged | All seven files resynced from the landed copies | `wdi-ux` |
| **C1 · C2 · C3** | Untouched. Each needs a decision that is the owner's, and each is already filed — `OQ-22`, `OQ-23`, `BUG-7` | the owner |

**One finding was made while fixing D2**, which the sweep had missed: `.what/bundle/04-usecases/EXPERIENCE.md`
listed *a Finding in the Bundle was deleted afterwards → this is `FR-13` working, not a fault* with no
mention of `BUG-1`, which makes it untrue in the shipped product. The same row in
`.what/finding/04-usecases/EXPERIENCE.md` had already been flagged; this one had not. Now flagged.

**Kept in the past tense rather than deleted, everywhere.** `AD-10` reads as a preference until you
know it was written against 23 literals in one codebase. A document that erases what it cost keeps
being right and stops being persuasive.

**The observation D2 ends on stands, and this pass is its first instance.** Each story in this wave
that removes a defect makes the document describing that defect false. Nine stories remain.

---

## Addendum — one check this sweep got wrong, found 2026-08-23 after the report

The **Registry** row of *Checked and clean* reads *"a container in the C4 set but not in
`containers`" — none*. That was answered in one direction only, and the other direction is where the
finding was.

`web-ui` is in **both** the C4 set and `containers`, so the check passed. Neither of them is in the
tree. It is registered `built: true` as the *Published Bundle Reader*, a React SPA served as static
assets by `web-api` and running in a reader's browser. There is no `index.html`, no `src/main.tsx`,
and `apps/web-service` serves no static assets at all — a grep for `FileServer`, `StaticFS`, `embed`,
`http.Dir` and `ServeFile` returns nothing. What lives at `web/ui/` is `@snapdown/ui`, a component
**library** consumed by the desktop webview, which deploys nowhere on its own.

**The name collision is what hid it.** `web-ui` the container and `web/ui` the package read as the
same thing in every document, and one of them is real.

**And `V25` passes on it.** A `built: true` container must have a heading in the code map; `web-ui`
has one, and the heading describes the library. The validator is not wrong — it asks *does this
heading exist*, and it cannot ask *does this heading describe the thing it is named after*.

Registered as `BUG-8`. The fix is the same decision as `OQ-22`, and this sweep's Registry check
should have been two questions rather than one:

1. Is every container in the C4 set registered in `containers`? — the direction that was asked
2. **Does every registered container exist in the tree?** — the direction that was not
