# W7 · Step 2 — BUILD. Rules that bind every story in this wave

Your story's spec file is the contract. Read it first, in full. This file carries only what is true
for all three stories and is not repeated in each spec.

Run `bmad-build-auto` given the spec file path. The step ends when the spec's frontmatter reads
`status: done`.

## The three rules every worker on this repo carries

- **Debugging is conditional, never a phase.** When a test or build fails and you do not know why,
  run `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the
  signal to escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation from the SDD or an `AD-N` is **reported** and becomes a `DEC-` — never absorbed as a code
  patch.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code, from the repo root.

## Four ways a verification run lies here, all of them hit on 2026-08-23

Read these before you trust a green result:

- **`cmd | tail` reports the exit code of `tail`, not of `cmd`.** A `cargo build` that failed was
  read as exit 0 because it was piped. Check `${PIPESTATUS[0]}`, or redirect and read `$?`.
- **`cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did**, because the script's exit
  code is `echo`'s. Read the echoed value, never the notification's code.
- **A long-lived worktree goes stale the moment a story adds a dependency.** CI runs `npm ci` from
  the lockfile; a worktree does not. Run `npm --prefix <pkg> ci` before believing a local red.
- **A leftover `Snapdown.exe` process locks its own file** and fails the next build with *Access is
  denied (os error 5)*, which reads like a permissions problem and is not. `Get-Process -Name
  Snapdown` before rebuilding.

## Tests

- **The test names in your spec come from `waves.yaml` and MUST be carried through verbatim.** If you
  must rename one, that is a finding to report, not a silent edit.
- **A test that asserts a literal instead of the behaviour it claims to cover is a defect.** This
  repository has landed that mistake three times. `contrast.test.ts` is the worked fix: it parses the
  real input rather than holding a copy of it, and it was verified by mutation. Where you can, prove
  your new test can fail — mutate the fix, watch it go red, put it back.
- **A green unit test does not mean the component is reachable.** Four components were once built,
  tested, and mounted nowhere. This wave is not UI work, so the analogue here is: prove the new error
  path is actually *reached* from the real call site, not only that the helper returns the right
  string.

## The seam for testing Tauri command logic

Extract an `_impl(&AppState)` function and have the `#[tauri::command]` wrapper delegate to it;
`commands/sharing.rs` is the worked example. **`tauri::test::mock_app` produced
`STATUS_ENTRYPOINT_NOT_FOUND` twice on `W6-S9` and MUST NOT be reached for again.** If you need
tauri's `test` feature, it MUST NOT land on the production workspace dependency — that would ship the
test harness in the release binary, which was caught and reverted once already.

## Hygiene

- **Write UTF-8, and no BOM.** Three story files arrived with one during W6; a BOM makes the
  frontmatter parser report the story as having no status, and the corpus guard rejects it.
- **Never commit a captured screenshot**, an accessibility-tree dump, or anything carrying an
  operator's home-directory path. The repository is public and the guard refuses them. A failure
  there is a finding about the content, never a reason to weaken the guard.
- **No scratch files in the commit.** A leftover `temp_*.test.rs` is a review finding.
- **Commit locally. DO NOT PUSH.** The coordinator is the only hand that pushes and opens PRs.

## Reporting

Report `worker_done` with `--outcome succeeded` and the spec path once its frontmatter reads
`status: done`, or `--outcome failed` with the blocking reason. Never encode failure only in prose.
