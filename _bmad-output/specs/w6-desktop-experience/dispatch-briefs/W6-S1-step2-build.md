# W6-S1 · Step 2 — BUILD

Implement the story. Commit locally. **Never push** — the coordinator is the only hand that pushes.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 2. Read `AGENTS.md` first.

Run `bmad-build-auto` with the spec file path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S1-every-colour-through-tokens-both-themes-enforced-by-a-lint-rule.md`

Its frontmatter reads `status: ready-for-dev`. This step ends when it reads `status: done`.

## The contract

The story spec is the contract. Above it sit `SPEC.md` and its `companions:` list — read
`.how/_platform/design-system.md` and **AD-10** in `.how/_platform/ARCHITECTURE-SPINE.md` before
writing a line.

## Three things that are easy to get wrong

**Three token groups are theme-invariant ON PURPOSE** and must keep literal values while living in
the token file, each with a comment saying why: `--color-marker*`, the capture overlay's scrim and
region ring, `--canvas-checker`. They are drawn over the Reviewer's own screen content or over an
exported image read on another machine under another theme, so this machine's theme is the wrong
reference. A lint rule that blindly refuses every literal will fight them — scope the rule.

**The contrast assertion checks every text element against ITS OWN background**, not against the page
background. Checking against the page is exactly what would have passed the shipped build: the shell
is dark, the text is white, and the white panel in between is what nobody looked at.

**`Toggle`'s indeterminate state is load-bearing.** W6-S5 needs it: `FR-18` requires the startup
control to reflect the real Windows registration and never a remembered intention; reading that is
asynchronous; without a third state the control must guess, and the shipped build guesses `true` then
repaints to `false`.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation from an SDD or an `AD-N` is **reported** and becomes a `DEC-`, never absorbed as a patch.
- **Verification is run, not assumed.** From `.constitution/project/codebase-stack-guide.md`:

  ```
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  npm --prefix web/ui run typecheck && npm --prefix web/ui run lint && npm --prefix web/ui run test
  npm --prefix apps/desktop run typecheck && npm --prefix apps/desktop run lint
  npm --prefix apps/desktop run test && npm --prefix apps/desktop run build
  ```

  A green `korpus.yml` is **not** proof the code compiles; they answer different questions.
- **Never commit a captured screenshot.** This repository is public and the brief forbids it.
- **Do not push.** Commit locally and report.

## Done means

The story spec's frontmatter reads `status: done`, every command above is green, and the work is
committed on this worktree's branch.

Report `worker_done` with `--outcome succeeded` and `--files-modified`, or `--outcome failed` with the
blocking reason. Do not encode failure only in prose.
