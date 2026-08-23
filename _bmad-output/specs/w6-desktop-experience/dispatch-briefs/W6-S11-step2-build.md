# W6-S11 · Step 2 — BUILD

Implement the story. Commit locally. **Never push** — the coordinator pushes.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 2. Read `AGENTS.md` first.

Run `bmad-build-auto` with the spec file path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S11-make-the-desktop-application-buildable-and-have-ci-build-it.md`

(If the filename differs, it is the only `W6-S11-*.md` in that folder.) Its frontmatter reads
`status: ready-for-dev`. This step ends when it reads `status: done`.

## Why this one matters out of proportion to its size

**Until it lands, nothing in this wave can be verified in the product.** `BUG-4` and `BUG-5` are
fixed and proven only in jsdom. Every remaining story writes UI. There is currently no reproducible
way to produce a running application to check any of it against.

## The resolved versions, so you do not have to dig

From `Cargo.lock`:

```
tauri                         2.11.5
tauri-build                   2.6.3
tauri-plugin-autostart        2.5.1
tauri-plugin-global-shortcut  2.3.2
tauri-plugin-single-instance  2.4.3
```

`Cargo.toml` says `tauri = "2.0"` — that is a caret requirement resolving to 2.11.5, **not** a pin.
The plan plumps for `@tauri-apps/cli` at `^2.0.0`, which resolves to the latest 2.x and is the right
call for the same reason: same major as the runtime.

## Two things to get right

**The manual check must stay manual.** The spec names
`manual::a_freshly_built_binary_loads_its_bundled_frontend_not_devurl` and the plan states it MUST
NOT be faked as an automated headless test. Honour that. This project has no reliable way to drive a
desktop UI from CI — `OQ-24` records three attempts producing three different failures in one day.
Write the procedure down; do not write a test that pretends to run it.

**CI builds and discards.** The plan chose that over uploading an artifact, and it is enough: it
proves the build works without paying storage on every PR. `BR-121` — *a build produces exactly one
desktop executable* — becomes checkable against a real build rather than against `Cargo.toml`.

## One exception to a standing rule, granted for this story only

`.constitution/project/codebase-stack-guide.md` is normally not yours to edit. **Here it is the
point.** Carrying `cargo tauri build` (or the npm script equivalent) into that guide, beside the
verification commands, is item 2 of the story. Nothing in `.what/` or `.how/` may be touched.

The reason is worth knowing: this exact command was already known to `W1`, whose reviewer wrote
*"vite build fails, therefore `cargo tauri build` fails, therefore `frontendDist` is never
produced"* — and it stayed in a closed wave's scratch folder where nobody would ever read it. Putting
it in the guide is the whole difference.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing build whose cause is unknown → run
  `wdi-systematic-debugging` before proposing any fix. A third failed fix attempt is the signal to
  escalate.
- **Verification is run, not assumed**, and two traps were caught on 2026-08-23: `cmd | tail`
  reports `tail`'s exit code — check `${PIPESTATUS[0]}` — and a long-lived worktree goes stale the
  moment a story adds a dependency, so run `npm ci` before believing a local red. This story **adds
  a dependency**, so that second one applies to you directly.
- **Write UTF-8.** **No scratch files in the commit.** Never a captured screenshot.
- **Do not push.**

## Done means

The spec's frontmatter reads `status: done`, every verification command in `AGENTS.md` § Code is
green, and the work is committed on this worktree's branch.

Report `worker_done` with `--outcome succeeded` and `--files-modified`, or `--outcome failed` with
the blocking reason.
