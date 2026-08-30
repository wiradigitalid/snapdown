# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

> **This repo does not use the layout below.** `DEC-011` gives the documents to WDI Method, which states
> outright that it has no `docs/` layer for corpus or rules. Domain knowledge lives in `.what/`, design in
> `.how/`, the glossary at `.control/product-glossary.md`, and a decision is a `DEC-` in
> `.control/decisions/` — not an ADR under `docs/adr/`. Neither `CONTEXT.md` nor `docs/adr/` has ever
> existed here and **neither MUST be created**; a second home for the same facts is drift, not tidiness.
> Read the rest of this file for its reasoning about single-context repos, and take the locations from
> `AGENTS.md` § `## The thing in your hand → its folder`. Where this file and `AGENTS.md` disagree,
> `AGENTS.md` wins.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root, or
- **`CONTEXT-MAP.md`** at the repo root if it exists: it points at one `CONTEXT.md` per context. Read each one relevant to the topic.
- **`docs/adr/`**: read ADRs that touch the area you're about to work in. In multi-context repos, also check `src/<context>/docs/adr/` for context-scoped decisions.

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`) creates them lazily when terms or decisions actually get resolved.

## File structure

Single-context repo (most repos):

```
/
├── CONTEXT.md
├── docs/adr/
│   ├── 0001-event-sourced-orders.md
│   └── 0002-postgres-for-write-model.md
└── src/
```

Multi-context repo (presence of `CONTEXT-MAP.md` at the root):

```
/
├── CONTEXT-MAP.md
├── docs/adr/                          ← system-wide decisions
└── src/
    ├── ordering/
    │   ├── CONTEXT.md
    │   └── docs/adr/                  ← context-specific decisions
    └── billing/
        ├── CONTEXT.md
        └── docs/adr/
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal: either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders), but worth reopening because…_
