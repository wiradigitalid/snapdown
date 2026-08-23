# Agent Rules — Snapdown

<!-- BEGIN:wdi-method -->
This repo uses **WDI Method**. It wraps BMad; it does not replace it. This marked
block is owned by the WDI Method package and is **replaced on every update**.
Product rules belong **outside** it (extra boundaries, `## Code`, stack notes).
A fact written inside this block will be overwritten.

Product identity lives in `.control/registry/index.yaml` (`product.name`, optional
`product.client`). G1 confirms it. This file MUST NOT become a second source of the name.

## Install and update

BMad first, then WDI Method. In the product repo:

```bash
npx bmad-method install
npx wdi-method
```

No subcommand opens the installer TUI. It detects an existing install and offers
**update**. Non-interactive:

```bash
npx wdi-method install --yes
npx wdi-method update --yes
```

BMad: https://github.com/bmad-code-org/BMAD-METHOD
WDI Method: https://github.com/wiradigitalid/wdi-method

A method file MUST NOT be invented here. If a rule is wrong, fix it in the WDI
Method package, then update.

This file is loaded every session; everything else is loaded **lazily**, only when
the task matches.

## Language

**Two settings decide this, and they live in `.control/registry/index.yaml` under `policy:`.** Both are
free text and both default to English:

| Setting | Governs |
|---|---|
| `doc_language` | The prose of working documents in `.what/` · `.how/` · `.control/` |
| `doc_filename_language` | The slug part of a document filename |

Read those two before writing a document. A technical term the industry writes in English MUST be left in
English whatever the setting says — an equivalent MUST NOT be invented for it.

**These files are always English, whatever the settings say:** `AGENTS.md`, `CLAUDE.md`, and everything
under `.constitution/`. They are agent instructions, and they travel to every repo through the
`wdi-method` package. The one exception is `.constitution/project/`, which is this product's own room.

**Always English and never a setting**, because a script matches them:

- method terminology — `DEC` `SRS` `SDD` `UC` `FR` `AD`, the gate names, the values of `mode` and
  `risk_accepted`
- document code prefixes — `UC-` `DEC-` `SRS-`; only the slug after them follows `doc_filename_language`
- markers — `[NEEDS CONFIRMATION]` `[MISSING]` `[ASSUMED]` `[PARTIAL]`, and `yes`/`no` in a `critical`
  column
- registry values — `mode: catalog`, `status: applied`, `risk_accepted: low`. Used as written in prose
  too: one thing, one name
- code identifiers, database columns, config keys — `language-guide.md` owns these

**A corpus written before these settings existed MUST NOT be migrated for them.** The readers accept more
than one language, so existing documents keep working and only new writing follows the setting.

## The thing in your hand → its folder

Read this instead of reasoning about what `.what/` and `.how/` mean.

| The thing in your hand | Its folder |
|---|---|
| A rule, a guide, a template — how we work | `.constitution/method/` — **overwritten in full by `update`** |
| A rule that binds **only this product** | `.constitution/project/` — `update` never writes over it, `promote` never publishes it |
| The explanation of a rule, never a rule itself | `.constitution/method/why/` |
| A decision, an open question, a registry, a structure map, minutes | `.control/` |
| The brief, a PRD, a use case, a business rule — what is promised | `.what/` |
| The spine, C4, an inventory, an SDD, a contract — how it is built | `.how/` |
| A skill run's working output, and documents that predate the method | `_bmad-output/` |
| Scratch that empties when the task closes | `.work/` |
| The application | named under `## Code` below |

## Layer boundaries

| Layer | Answers | MUST NOT hold |
|---|---|---|
| `.constitution/` | How we work | State, decisions, product content |
| `.control/` | What currently holds and what has been decided | Rules |
| `.what/` | What is promised | Solution shape — tables, endpoints, technology |
| `.how/` | How it is built | Promises to the user |
| `_bmad-output/` | Work in progress; committed, not curated | Anything still correct after its wave has passed |
| `.work/` | Scratch; emptied when a task closes | Secrets, commercial figures, anything meant as authority |

The placement test: **is this file still correct after its wave has passed?** Yes → the corpus. No →
`_bmad-output/`. In doubt → `.constitution/method/document/corpus-guide.md`.

The method does not use a `docs/` layer for corpus or rules. A leftover `docs/` folder is inventory
to sort, not a second home.

## Depth and review intensity — two fields, never merged

| Field | Where | Controls |
|---|---|---|
| `mode` | `index.yaml` globally, `components.yaml` per component | **Document depth**, and only that. `catalog` · `outline` · `guarded` · `deep`; default `catalog` |
| `risk_accepted` | `components.yaml` per component | **Review intensity**, and only that. `low` · `medium` · `high` |

Per-component `mode` wins over global, and there is no third scope — `mode` MUST NOT be overridden per
wave or per `SPEC.md`. A component at `mode: catalog` **skips G4 entirely**. Neither field MUST be
derived from the other: one component MAY be thin on purpose and reviewed the hardest.
`.constitution/method/document/delivery-flow-guide.md` owns both;
`.constitution/method/why/rationale.md` says why they are separate.

## The five gates and the fifteen skills

| Gate | Decides | Skill |
|---|---|---|
| **G1 Problem** | What the problem is, whose it is, why it earns work | `wdi-problem` |
| **G2 Product** | What is built, and how it feels to use | `wdi-product` · optional `wdi-ux` |
| **G3 Blueprint** | The whole portrait, once per product | `wdi-blueprint` |
| **G4 Component** | How one component is built — **skipped at `catalog`** | `wdi-component` |
| **G5 Release** | Whether it is done and proven | `wdi-build` |

Before G1 and at the tail of G2: `wdi-init`, five intents — `setup` · `component` · `mode` · `risk` ·
`structure`.

Any time: `wdi-decision` · `wdi-question` · `wdi-log` · `wdi-help` · `wdi-reconcile` · `wdi-review` ·
`wdi-report` · `wdi-systematic-debugging`.

**No BMad skill is invoked directly.** Each has a wrapper, and the wrapper is what checks position,
verifies the result, and lands the memlog.

## What MUST NOT be done

- A method file MUST NOT be invented or patched here to improve the method. If a rule is wrong, it is
  fixed in the WDI Method package, then brought here with `npx wdi-method update`.
- A file in `_bmad-output/prior-knowledge/` MUST NOT be copied into `.what/` or `.how/`. It enters
  the corpus only through the skill that owns the slot.
- `.control/generated/` MUST NOT be written by hand — it is the output of `validate.py` and
  `timeline.py`.
- The two structure maps in `.control/` MUST NOT be edited by hand — `wdi-init` intent `structure`
  re-derives them.
- A `DEC-` with status `applied` MUST NOT be edited, except to record its supersession — status moves
  to `superseded` and names its replacement. A change of mind produces a new `DEC-`.
- A file in `.constitution/method/why/` MUST NOT be cited as the reason to reject a change. It is
  `status: Reference` — it explains, it does not bind, and where it disagrees with a guide the guide
  wins and the disagreement is a defect. This covers `why/` ONLY: a guide in
  `.constitution/method/document/` is `status: Accepted` and it binds.
- More than the component's `mode` demands MUST NOT be written. Exceeding the depth the owner set is
  not diligence.
- `.claude/skills/bmad-*/customize.toml` MUST NOT be edited — it is overwritten on every BMad update;
  customise through `_bmad/custom/`.

## Routing — load a guide when the task matches

| Task | Load |
|---|---|
| Wanting the whole method in five minutes | `.constitution/method/why/README.md` |
| About to change a rule, and needing to know what breaks | `.constitution/method/why/rationale.md` |
| Asking whether a document exists at this `mode`, or where a file goes | `.constitution/method/why/artifact-map.md` |
| Unsure whether a file may exist in this repo | `.constitution/method/repo-guide.md` |
| Unsure where a file lives | `.constitution/method/document/corpus-guide.md` |
| Unsure what a method term means | `.constitution/method/method-glossary.md` |
| Unsure about a domain term | `.control/product-glossary.md` |
| Looking for a non-technical fact — a domain, an account, a legal entity, a locked date | `.control/project-non-technical-log.md` |
| Naming anything — a code identifier, a file, a database column | `.constitution/method/language-guide.md` |
| Asking "which gate now, what next" | `.constitution/method/document/delivery-flow-guide.md` · skill `wdi-help` |
| Setting or changing `mode` or `risk_accepted` | `.constitution/method/document/delivery-flow-guide.md` · skill `wdi-init` |
| Invoking a BMad skill | `.constitution/method/document/bmad-guide.md` · `.constitution/method/document/bmad-skill-register.md` |
| Writing or reviewing a product brief | `.constitution/method/document/brief-guide.md` |
| Writing or reviewing a PRD | `.constitution/method/document/prd-guide.md` |
| Writing or reviewing UX | `.constitution/method/document/ux-guide.md` |
| Writing or reviewing an SRS | `.constitution/method/document/srs-guide.md` |
| Writing or reviewing an SDD | `.constitution/method/document/sdd-guide.md` |
| Writing the spine, an `AD-N`, C4, or one of the three inventories | `.constitution/method/document/architecture-guide.md` |
| Opening, accepting, or applying a `DEC-` | `.constitution/method/document/decision-guide.md` |
| Writing or reading a structure map | `.constitution/method/structure-guide.md` |
| Looking for where code lives, or placing new code | `.control/structure-codebase.md` |
| Looking for where a document lives | `.control/structure-document.md` |
| Writing or reviewing code | `.constitution/project/codebase-stack-guide.md` · `.constitution/project/codebase-conventions-guide.md` · `.constitution/project/codebase-brownfield-guide.md` |

All three `.constitution/project/codebase-*-guide.md` start as `status: Draft`. While they are, their contents MAY be read
as guidance but MUST NOT be used to reject a change.

The two structure maps MUST NOT be installed as `doc_standards` — they are facts, not standards. Nor
MUST anything in `.constitution/method/why/`; `status: Reference` forbids it. A guide in
`.constitution/method/document/` MAY be installed that way, and several already are — see
`_bmad/custom/bmad-prd.toml`.

## Bugs, decisions, questions

- A bug, a failing test, or unexpected behaviour → skill `wdi-systematic-debugging`, **before** any
  fix is proposed.
- A decision worth remembering → skill `wdi-decision` → `.control/decisions/`. Recording is **not
  mandatory**: if the answer to *why is it like this* is readable from the code, it MUST NOT be
  recorded. One case is mandatory — contradicting an `AD-N`.
- Something that cannot be decided now → skill `wdi-question` → `.control/questions/`. The default
  class is `assumptions.md`, not `blocking.md`; filing something as blocking "to be safe" is the
  habit that produced unreadable question lists.
- A non-technical fact that constrains the build → skill `wdi-log` intent `fact` →
  `.control/project-non-technical-log.md`.

## Method policy

- A skill MUST NOT be invoked automatically. Name the one that fits and wait for the owner's
  go-ahead — this holds even when the skill's own description says it must be used. Reading a
  skill as reference is fine.
- `.work/` is not production code. It MUST NOT be imported by the application, and MUST be
  excluded when searching for code.
<!-- END:wdi-method -->

## Code

Tauri v2 desktop app. Rust workspace (`crates/snapdown-core` pure domain, `crates/snapdown-store`
adapters, `crates/snapdown-bridge` the MCP executable), a React + Vite webview in `apps/desktop`, a
shared UI package in `web/ui` consumed as `@snapdown/ui`, and a Go service in `apps/web-service`.
`.control/structure-codebase.md` is the map; this section does not duplicate it.

### Verification — run all of it, from the repo root

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix web/ui run typecheck && npm --prefix web/ui run lint && npm --prefix web/ui run test
npm --prefix apps/desktop run typecheck && npm --prefix apps/desktop run lint
npm --prefix apps/desktop run test && npm --prefix apps/desktop run build
```

Three CI jobs cover these: `rust-check`, `web-check`, and `web-service`. A green `korpus.yml` is
**not** proof the code compiles — it validates the corpus, and they answer different questions.

**`cargo build` does NOT build this application.** A Tauri app needs the Tauri CLI; without it the
release binary requests `devUrl` from `tauri.conf.json` and shows `ERR_CONNECTION_REFUSED` instead of
the frontend. The CLI is currently absent from this repository entirely — see `BUG-11`. Until that is
fixed, **a locally built `Snapdown.exe` is not the application**, and any UI finding taken from one is
a finding about the build.

**Four ways a verification run lies, all hit on 2026-08-23:**

- **`cmd | tail` reports the exit code of `tail`, not of `cmd`.** A `cargo build` that failed with
  *package ID specification did not match any packages* was reported as exit 0 because it was piped.
  Check `${PIPESTATUS[0]}`, or redirect to `/dev/null` and read `$?`.
- **The coordinator's own worktree goes stale the moment a story adds a dependency.** `web/ui`
  typecheck failed locally on missing `@types/node` while CI was green: CI runs `npm ci` from the
  lockfile, a long-lived worktree does not. Run `npm --prefix <pkg> ci` before believing a local red.
- **`cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did.** The script's exit code is
  `echo`'s, and `echo` always succeeds — so the background-task notification says *exit code 0* while
  the echoed line says `EXIT=1`. A `tauri build` that died on *Access is denied* was reported as a
  success this way. Read the echoed value, never the notification's code.

### Pitfalls

**A green unit test does not mean the component is reachable.** This is the most expensive mistake
this repository has made. On 2026-08-23 a sweep found **four** components built, unit-tested, and
mounted nowhere: `CaptureOverlay` (the capture path — `BUG-4`), `MarkerLayer` (marker annotation —
`BUG-5`), `OrphanReportView` (`BUG-6`), and `EmptyState`. Three requirements — `FR-1`/`FR-2`, `FR-8`,
`FR-15` — were unmet in a build whose tests all passed, for four waves.

There is no composition test class here yet (`OQ-23`). Until there is, **before closing any story
that adds a component, grep for `<ComponentName` across `apps/desktop/src` and `web/ui/src`,
excluding its own file and its tests.** No hit means nobody can reach it. `V12` will not catch this:
it checks that an `LC` is *registered*, not that it is *reached*.

**A panic in the desktop process takes the whole product with it.** `AD-11` puts the tray, the
hotkeys, the capture overlay and the Editor in one process, and `DEC-003` accepted that cost in
writing: *"a panic in the editor's Tauri commands kills the tray, the hotkeys, and the overlay with
it."* A Tauri release binary on Windows has no console, so a panic in the setup hook means the
Reviewer double-clicks the exe and **nothing happens at all** — see `BUG-12`, five `.expect()` calls
on store opens. Before writing `unwrap`/`expect` outside a test, ask what the Reviewer sees when it
fires. Genuinely infallible cases exist and are fine — `Header::from_bytes` over a compile-time byte
constant is one — but they are rarer than they look.

**`let _ =` on a Result an invariant depends on is a defect, not a style.** Five instances found on
2026-08-23 across two files: `vault_migration.rs` swallowed both `fs::remove_file` results, and
`bundle.rs` swallowed the Markdown write, the Vault open, the unpublish, and two blob deletes. The
worst of them leaves a published Bundle **live on the internet** after the Reviewer deletes it. It
reads as deliberate, clippy ignores it at default levels, and no test catches it because every store
test uses a writable temp directory. Before writing one, ask what invariant the call is holding up.

**Colour lives in exactly one file.** `web/ui/src/styles/tokens.css`, defined for both themes
(`AD-10`). A lint rule refuses a colour literal anywhere else. The four deliberately theme-invariant
groups — `--color-marker*`, `--color-overlay-scrim`, `--color-overlay-ring`, `--canvas-checker` —
are the one exception and they live in that file too, each with a comment saying why.

**A test that asserts a literal is a test that cannot fail.** `contrast.test.ts` originally hardcoded
its own copy of the token values; changing a token to a 2:1 ratio left it green. It now parses
`tokens.css` and was verified by mutation. Assert the behaviour, not a copy of the input.

**Two SQLite stores, not one.** `library.db` (Rust, `crates/snapdown-store/src/sqlite/migrations.rs`)
and the web service's own (Go, `apps/web-service/internal/store/store.go`). A reader that looks at
only the first will report the second's tables as missing — that is what
`.constitution/project/inventory-readers.py` did for two waves.

**Never commit a captured screenshot.** This repository is public and the product brief forbids it.
`.gitignore` covers every image outside the app icon set, and `korpus.yml` now refuses tracked images,
raw accessibility-tree dumps, and operator home-directory paths.

**Scrubbing a leak from a branch is not finished when the branch is rewritten.** `git filter-branch`
leaves its own backup at `refs/original/refs/heads/<branch>`, and the old objects stay reachable
through it. Delete that ref, `git reflog expire --expire=now --all`, then `git gc --prune=now`. Verify
with `git log --all --oneline -- <path>` returning nothing. This was missed once, on 2026-08-23, and
found only because the tag cleanup prompted a second look.

**Stale binaries mislead.** Renaming the product left `desktop.exe` beside `Snapdown.exe` in
`target/release/`, the owner ran the old one, and reported four defects that did not exist. `FR-27`
now makes a second desktop executable a build failure.

**A dispatched worktree branches from `main`, not from the branch you are on.** `worker-start
--worktree new-child` without `--base-branch` gave W6-S9's planner a checkout at `main`'s tip, so the
whole wave — `SPEC.md`, `stories.yaml`, every dispatch brief — was absent, and the worker rebuilt them
from scratch as new files. Pass `--base-branch` explicitly for any wave work, and take only the files
the brief asked for out of a worktree that got this wrong: its reconstructed registries are guesses,
and overwriting the real ones with them loses everything the wave has written.

**A leftover `Snapdown.exe` process locks its own file and fails the next build.** A binary launched
by an earlier UI audit was still running hours later; `tauri build` died with *failed to remove file
`Snapdown.exe`: Access is denied (os error 5)*, which reads like a permissions problem and is not.
`Get-Process -Name Snapdown` before rebuilding, and treat a still-running instance as cleanup the
same way a stale worktree is.

**`agent_prompt_stalled` does not mean the worktree failed.** Orca's `worker-start --worktree
new-child` creates the checkout first and injects the agent's prompt second, so a stall leaves a
perfectly good worktree behind at the right commit. Retrying with `new-child` makes another one;
`w6-s3-plan` and `w6-s3-plan2` were both born this way. Check
`D:/Developer/orca-workspaces/<repo>/` first and re-dispatch into the existing one with
`--worktree path:<path>`. Attaching a terminal in another worktree does not work either — the Run is
bound to the main worktree and refuses the handle.

**`orca terminal send --text ... --enter` does not submit to a TUI agent.** The text goes into the
input box and nothing happens. `--text` writes a **bracketed-paste** block — `ESC[200~`, the text,
`ESC[201~`, then the CR — and TUI input widgets deliberately swallow a CR that arrives inside a paste
so that multi-line pastes do not fire early. Byte counts show it: a 12-character message with
`--enter` writes 25 bytes, while `--enter` alone writes **1**. Send the text, then send `--enter` as a
**separate call** so the CR lands outside the paste block. This is only for driving a worker's TUI;
workers report back through `orca orchestration worker-done`, which is a CLI call and unaffected.
