---
type: structure
scope: document
verified: "2026-08-22"
commit: 72bf291
---

# Document Structure

Written and refreshed only by `wdi-init` intent `structure`, never by hand. Rules live in
`.constitution/method/structure-guide.md`; which layer owns a file is
`.constitution/method/document/corpus-guide.md`, and the placement test is stated there — it is not
restated here.

## Verified

Derived from the tree on disk on **2026-08-22**, at commit **72bf291**.

## Product Component folders

Five Product Components, each present on both sides. No one-sided folder, so no drift.

| Product Component | `.what/` | `.how/` | `mode` | Slots split out of the kernel |
| --- | --- | --- | --- | --- |
| `finding` | SRS + `03-domain/` | SDD skeleton | `guarded` | none — `02-rules/` and `04-usecases/` exist and are empty, awaiting G4 |
| `bundle` | SRS + `03-domain/` | SDD skeleton | `outline` | none — same |
| `settings` | SRS + `03-domain/` | SDD skeleton | `catalog` | none, and none is coming: at `catalog` the empty slots are the finished state |
| `agent-access` | SRS + `03-domain/` | SDD skeleton | `guarded` | none — awaiting G4 |
| `sharing` | SRS + `03-domain/` | SDD skeleton | `guarded` | none — awaiting G4 |

## `.constitution/`

```text
.constitution/
  method/            from the WDI Method package, overwritten in full by `update`
    constitution.md      ★ Articles 3, 4, 6, 7
    repo-guide.md        ★ does this belong in the repository at all
    structure-guide.md   ★ the rules these two maps obey
    language-guide.md    which language a name is written in
    method-glossary.md   method vocabulary
    README.md            the method in five minutes
    document/            one guide per document kind, plus templates/ (27 of them)
    why/                 status: Reference. Explains, never binds
    scripts/             validate.py, inventory.py, timeline.py
  project/           ours, never overwritten, never published
    constitution.md      ★ Articles 1, 2, 5 — scope, content boundary, method ownership
    codebase-stack-guide.md        status: Draft
    codebase-conventions-guide.md  status: Draft
    codebase-brownfield-guide.md   status: Draft
    inventory-readers.py           still the shipped skeleton
    README.md            what may live in this room
```

## `.control/`

```text
.control/
  registry/
    index.yaml         ★ product identity, global `mode: outline`, gates, touches vocabulary
    requirements.yaml  ★ BG-1..6 · CAP-1..8 · FR-1..26 · NFR-1..15 · UJ-1..6
    usecases.yaml      ★ UC-1..23, four marked critical
    components.yaml    ★ five Product Components, four containers, no LC yet
    decisions.yaml     empty — no DEC- recorded yet
    waves.yaml         empty until G5
    risks.yaml         empty
    defects.yaml       empty
  questions/
    assumptions.md     OQ-1..12, the default class
    external.md        OQ-13..15, go-live only
    blocking.md        empty, and stated as empty
    answered.md        empty
  decisions/           empty
  meetings/            empty
  memlog/
    brief.md               G1
    prd-capture-to-markdown.md   G2
    prd-agent-handoff.md         G2
    spine.md               G3
  generated/           output of validate.py --generate
  product-glossary.md  ★ 18 entries, the product's vocabulary
  project-non-technical-log.md   empty
  structure-codebase.md  this map's sibling
  structure-document.md  this file
  wdi-method.yaml      the install trace: wdi-method 0.5.13, bmad-method 6.11.0
```

## `.what/`

Product Component folders are in the table above and are deliberately not expanded here.

```text
.what/
  _product-brief/
    brief.md         ★ G1. One problem, one primary user, one measure
    addendum.md      rejected alternatives, options weighed, sizing
  _prd/
    capture-to-markdown/   ★ G2. The desktop loop: CAP-1..6, FR-1..18
      prd.md
      addendum.md
    agent-handoff/         ★ G2. MCP and web publishing: CAP-7..8, FR-19..26
      prd.md
      addendum.md
  business-rules.md  ★ G3. BR-1..30, every one binding more than one component
  <pc>/              per Product Component, see the table above
    SRS-<pc>.md      ★ actor register + UC catalogue. Exists at every mode
    02-rules/        empty until G4, from mode: outline
    03-domain/
      domain-model.md   ★ conceptual. G3, exists at every mode
    04-usecases/     empty until G4. At most three flows at outline and guarded
```

## `.how/`

```text
.how/
  _platform/
    ARCHITECTURE-SPINE.md    ★ AD-1..9. Invariants only; stack and tree are marked as seeds
    c4-l1-system-context.md  who uses it and what it talks to
    c4-l2-containers.md      ★ owns the container list and the PC x container matrix
    c4-l3-desktop-app.md     the one container holding more than one Product Component
    inventory-db.md          ★ 12 tables across two stores, derived_from: plan
    inventory-api.md         ★ 14 endpoints across three surfaces, derived_from: plan
    inventory-screen.md      ★ 15 screens across two front ends, derived_from: plan
    cross-cutting.md         ★ the error envelope, the catalogue, and six product-level agreements
  <pc>/              per Product Component, see the table above
    SDD-<pc>.md      skeleton at G3; filled at G4 except for `settings`
```

`design-system.md` does not exist in `_platform/`. It is optional at every `mode` and belongs to
`wdi-ux`; two React front ends make it worth having, and it is written when `wdi-ux` runs.

## `_bmad-output/` and `.work/`

Both exist and both are **empty**. No skill run has produced a workspace, and no scratch is open.
That is a legitimate state and it is recorded rather than left to be inferred.

---

`★` marks an entry point, the single place a fact is stated for everything below it, or a file an
agent working in that folder would have to open first.
