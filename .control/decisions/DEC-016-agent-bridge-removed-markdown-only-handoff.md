---
type: decision
id: DEC-016
status: applied
touches:
  - .what/agent-access/SRS-agent-access.md
  - .what/agent-access/02-rules/rules-agent-access.md
  - .what/agent-access/03-domain/domain-model.md
  - .what/agent-access/04-usecases/UC-17-let-the-agent-in-front-of-me-read-my-reviews.md
  - .what/agent-access/04-usecases/UC-18-read-a-review-from-inside-my-agent.md
  - .how/agent-access/SDD-agent-access.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l1-system-context.md
  - .how/_platform/c4-l2-containers.md
  - .how/_platform/c4-l3-desktop-app.md
  - .how/_platform/cross-cutting.md
  - .how/_platform/inventory-api.md
  - .how/_platform/inventory-db.md
  - .how/_platform/inventory-screen.md
  - .control/registry/components.yaml
  - .control/registry/requirements.yaml
  - .control/registry/usecases.yaml
  - .control/structure-codebase.md
  - .control/structure-document.md
  - .control/product-glossary.md
  - .control/questions/assumptions.md
  - .control/questions/answered.md
  - .what/_prd/agent-handoff/prd.md
  - .what/_prd/agent-handoff/addendum.md
  - .what/_prd/capture-to-markdown/prd.md
  - .what/business-rules.md
  - .what/bundle/SRS-bundle.md
  - .how/bundle/SDD-bundle.md
  - .what/sharing/SRS-sharing.md
  - .how/sharing/SDD-sharing.md
  - .what/finding/SRS-finding.md
  - .what/settings/SRS-settings.md
  - .what/settings/03-domain/domain-model.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .how/settings/03-integrations/windows-shell.md
  - .how/settings/04-components/LC-028-editor-shell.md
  - .github/validate-baseline.txt
  - AGENTS.md
supersedes: DEC-002
superseded_by: null
created: "2026-09-04"
---

# DEC-016 — Snapdown hands a Bundle to an agent only as copied Markdown; the MCP bridge is removed

## Decision

Snapdown no longer opens a running channel into the Library for an agent. The `mcp-bridge` executable,
the Local API it fronted, and the Access Key ceremony are removed in full. The only way a Bundle reaches
an agent from here on is the Markdown a Reviewer copies — by hand with Copy Markdown, or automatically
on a successful Assemble & Save / Review & Update Save — and pastes themselves.

## Why

The owner gave this instruction directly, in chat, on 2026-09-04: *"Agent bridge hapus aja dari settings
dan fitur, MCP tidak relevan lagi harusnya, karena main sistem copy paste markdown saja sudah selesai
semuanya"* — remove the Agent bridge/MCP feature from Settings and from the product, because the
copy-Markdown-and-paste workflow now stands on its own and covers what the bridge used to be for.

`DEC-002` built the bridge to solve four constraints at once: work with an MCP client that can only
launch a stdio server, keep no secret on disk, end access in one action, and never hold a second copy of
the Library. All four were about doing that well — none of them is a reason to have a running channel
at all once the product's own copy/paste path is complete. This is a change of what the product should
offer, not a defect in `DEC-002`'s reasoning, which is why it supersedes rather than corrects it.

## Cost

- **`CAP-7` (Local agent access) is retired**, not merely re-implemented — there is no replacement
  running channel. A Reviewer who wants an agent to query the Library live, rather than read what was
  pasted to it, loses that option entirely.
- **`OQ-6`** (whether re-pasting the key every session is unwanted friction) is now moot — the friction
  it asked about no longer exists, because the thing that caused it is gone.
- **The `agent-access` Product Component becomes empty.** Its SRS/SDD, its rows in `components.yaml` and
  the two structure maps, and every `AD-N` Binds list that names it (`AD-1`, `AD-5`, `AD-7`, `AD-9`,
  `AD-10`) still describe a container that will no longer exist. Landing that — retiring the component,
  editing the spine's Binds lists, the PRD, and `SRS-agent-access.md` — is real work this decision does
  not itself do; `touches` stays empty until an `apply` pass lands it, and until then those documents are
  behind the decision the same way this repo already accepts a document trailing shipped code.
- **The removal itself is not this decision.** Deleting `crates/snapdown-bridge`, its Settings UI, and
  its tests is a separate `/to-spec` → `/to-tickets` → `/implement-spec` pass, which this decision only
  unblocks — it was blocked because `DEC-002` was `applied` and an applied decision cannot be edited
  except by a superseding one, and this is that one.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | `OQ-6` — closed by this decision; the friction it tracked no longer applies |
| Source material | the owner's chat instruction, 2026-09-04, quoted above |
