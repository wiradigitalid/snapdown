---
type: decision
id: DEC-011
status: applied
touches:
  - AGENTS.md
  - .agents/AGENTS.md
  - .cursorrules
  - .github/validate-baseline.txt
  - _bmad/custom/bmad-correct-course.toml
  - docs/agents/domain.md
  - .claude/skills/
  - .agents/skills/
supersedes: null
superseded_by: null
created: "2026-08-30"
accepted: "2026-08-30"
applied: "2026-08-30"
---

# DEC-011 — WDI Method owns the documents, mattpocock/skills owns the code, and documents follow the code

## Decision

Both skill sets stay installed in this repository, and each owns one half of the work:

- **WDI Method owns the documents** — G1–G4 (`wdi-problem`, `wdi-product`, `wdi-blueprint`,
  `wdi-component`, `wdi-ux`), the corpus in `.what/` · `.how/` · `.control/`, and its upkeep
  (`wdi-decision`, `wdi-question`, `wdi-log`, `wdi-init`, `wdi-reconcile`, `wdi-review`, `wdi-report`).
- **mattpocock/skills owns the code and task management** — `/to-spec` → `/to-tickets` →
  `/implement-spec`, with tickets as markdown files under `.scratch/<feature-slug>/issues/`.

Three things follow, and they are the operative part:

1. **The epic / story / sprint / wave layer is retired.** `waves.yaml` and `stories.yaml` are frozen as
   history and are not extended. Tickets under `.scratch/` are the only task record.
2. **Documents follow the code.** The corpus is an input to a spec, never a gate on a change. Where a
   document and the code disagree, the code wins and the **document** is what gets corrected.
3. **Nothing is deleted.** All 15 `wdi-*` and 49 `bmad-*` skills stay installed beside the 37
   mattpocock ones. 17 of them — `wdi-build` and the BMad story/sprint/dev/build/code-review set —
   MUST NOT be run, and `AGENTS.md` names each with what to run instead.

`G5 Release` as a gate walked by `wdi-build` is therefore retired. The boundary is written in
`AGENTS.md` **outside** the `<!-- BEGIN:wdi-method -->` block, because that block is package-owned and
replaced in full by `npx wdi-method update`.

## Why

The owner evaluated `mattpocock/skills` on branch `eval/mattpocock-skills` against WDI Method, and the
evaluation produced a split rather than a winner: *"klo dokumen saya suka sih wdi-method, tapi kalau
coding saya suka mattpocock-skills"*, and on task management, *"soal epic, task dll gak relevan lagi di
wdi-method"*.

The split was not designed. It had already happened, unnoticed, in the work that ran during the
evaluation: the clipboard-only capture path was specified and ticketed under
`.scratch/clipboard-only-capture/` in mattpocock's shape, while `BUG-84` was filed into
`.control/registry/defects.yaml` and the reasoning cited `DEC-` ids. That produced good work, so this
decision records the practice rather than inventing one.

The direction of authority is the owner's own, stated twice. First, *"coding yang menang, karena coding
yang menentukan trial error mana yang berhasil"*, and then, on the day this was settled, *"coding lebih
terbaru ketimbang dokumen. bagi saya, dokumen sekarang lebih kepada left behind yang follow coding."*
Coding is where the trial and error happens, so the code is the record of which attempt actually
succeeded; a document is a claim about the code at a moment, and it goes stale silently. This repository
has already paid for the opposite assumption — a wave was opened, and a planner dispatched, against a
`BUG-12` row describing code that `W6-S5` had already fixed.

Deleting the WDI and BMad skills was considered and rejected by the owner in one line: *"boleh aja
mereka ada, tapi jangan dijalankan."* A wrapper's verification rules are often the clearest statement of
what a document has to satisfy, so the files are worth keeping as reference even where the flow is not.

## Cost

- **Three of the validator's checks stop measuring anything.** V3, V12 and V19 compare the corpus
  against `waves.yaml` / `stories.yaml`, and V13's `no reviewed trace` finding does the same for a wave.
  With the wave layer frozen they answer a question nobody asks here, which is why every line of
  `.github/validate-baseline.txt` is one of them. They are carried as fossils, and `validate.py` is not
  patched for it, because `.constitution/method/` is replaced on every update.
- **A green `korpus.yml` proves less than it looks.** V13's *other* half — a document changed after its
  last review — needs per-file git history, and `actions/checkout@v4` clones shallow, so it never
  appears on the runner. Four documents are stale by that measure right now
  (`ARCHITECTURE-SPINE.md`, `SDD-bundle.md`, `SDD-finding.md`, `SRS-finding.md`) and CI cannot see any of
  them. A local run is the stricter of the two, and the baseline holds the runner's visible set.
- **Two systems mean two sets of habits to keep straight**, and the `AGENTS.md` rule is all that keeps
  them apart. A skill listing still advertises every retired skill by name, and `wdi-help` still points
  at `wdi-build` in its own table; nothing mechanical prevents one being run.
- **Accepted document lag is easy to confuse with neglect.** "The document is behind the code" is now
  the expected state rather than a finding, so the thing actually worth catching — a claim that would
  steer the next reader toward the wrong repair — has to be found by reading, not by a validator.

## What would reopen this

A second person joining the code. This boundary leans on tickets in `.scratch/` being enough of a task
record, which holds for one owner and one agent working in sequence; it is not a claim about a team
needing to see who is doing what. Also: mattpocock/skills changing shape such that `/to-spec` no longer
reads an existing corpus, which would leave the documents with no consumer at all.
