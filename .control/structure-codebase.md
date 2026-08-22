---
type: structure
scope: codebase
verified: "2026-08-22"
commit: 72bf291
---

# Codebase Structure

Written and refreshed only by `wdi-init` intent `structure`, never by hand. Rules live in
`.constitution/method/structure-guide.md`. Naming belongs to
`.constitution/project/codebase-conventions-guide.md`, versions to
`.constitution/project/codebase-stack-guide.md`.

## Verified

Derived from the tree on disk on **2026-08-22**, at commit **72bf291**.

## Top level

```text
snapdown/
  .agent/            ★ generated agent skills — Antigravity/Gemini copy, written by `npx bmad-method install`
  .agents/           ★ generated agent skills + AGENTS.md — Antigravity/Gemini copy, written by the installers
  .claude/           ★ generated agent skills — Claude Code copy, written by the installers
  .constitution/     rules: method/ from the WDI Method package, project/ ours
  .control/          what currently holds — registries, decisions, questions, memlog, maps
  .how/              how it is built — the platform blueprint and one folder per Product Component
  .opencode/         generated slash commands — OpenCode copy, written by the installers
  .what/             what is promised — the brief, the PRDs, one folder per Product Component
  .work/             scratch, committed and emptied when a task closes. Empty
  _bmad/             the BMad installation: core, the bmm module, config, and our overrides in custom/
  _bmad-output/      skill-run workspace, committed and never curated. Empty
  .cursorrules       generated agent rules — Cursor copy
  .gitignore         Rust and Cargo ignores, from the initial commit
  AGENTS.md          ★ agent rules, source of truth for the project-scoped set
  CLAUDE.md          ★ agent rules for Claude Code; includes AGENTS.md
  LICENSE            the repository licence
  README.md          one-paragraph description of the product
```

## Containers

**None yet.** No application code has been written, so no container has a folder on disk, and this
map MUST NOT list a folder the architecture implies but no file has created.

Four containers are registered in `.control/registry/components.yaml` with `built: true` —
`desktop-app`, `mcp-bridge`, `web-api`, `web-ui` — and `.how/_platform/c4-l2-containers.md` describes
them. **V25 reports four findings against this section until their code exists**, and that is the
correct state at G3: the registry records the plan and this map records the tree, and the difference
between them is exactly what the validator is for.

The planned shape is in `.how/_platform/ARCHITECTURE-SPINE.md` § Structural Seed, marked there as a
seed. This section is re-derived at the close of the first wave, and the four findings close with it.

## Libraries

**None yet**, for the same reason. The spine's seed names four Rust crates —
`snapdown-core`, `snapdown-store`, `snapdown-capture`, `snapdown-mcp` — of which the last is a
container's entry point rather than a library. Nothing is on disk.

## Tooling

```text
.constitution/method/scripts/
  validate.py        ★ V1..V27 plus the .control/generated/ generator. Run with --generate
  inventory.py       derives the three inventories from code, using the readers below
  timeline.py        renders .control/reports/<period>.md
.constitution/project/
  inventory-readers.py   ★ how THIS product's code is read, for inventory.py. Still a skeleton
_bmad/scripts/
  memlog.py          ★ append-only memlog writer, used by every skill run
  resolve_customization.py   merges skill customize.toml with _bmad/custom overrides
  config_utils.py    the three-layer TOML merge behind it
```

## Generated

```text
.control/generated/          output of validate.py --generate. Empty until first generated
.agent/  .agents/  .claude/  .opencode/  .cursorrules
                             output of `npx bmad-method install` and `npx wdi-method`.
                             A hand edit here is overwritten on the next update
_bmad/                       same, except _bmad/custom/*.user.toml which is ours
.constitution/method/        same. `.constitution/project/` is never overwritten
```

## Unclaimed

None. Every base folder above has a stated purpose.

---

`★` marks an entry point, a composition root, the single place a rule is enforced for the tree below
it, or a file an agent changing that folder would have to open first.
