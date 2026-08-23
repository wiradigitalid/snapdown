# W7 · SPEC review

**Read only. You MUST NOT edit any file.** Your whole output is a findings report.

## Method position

WDI Method, `wdi-review`. The artifact is a wave `SPEC`, and a `SPEC` **always** carries the
`edge-case-hunter` lens — it is the contract a builder works from, so a branch missed here surfaces
as a bug at G5 instead of as a review finding now.

Run `bmad-review` with:

- **content:** `_bmad-output/specs/w7-failure-paths/SPEC.md`, together with
  `_bmad-output/specs/w7-failure-paths/stories.yaml` and all three story files under
  `_bmad-output/specs/w7-failure-paths/stories/`. The stories are part of the artifact; reviewing the
  kernel alone misses where the branches live.
- **lenses:** `structure`, `prose`, `edge-case-hunter`.

## What this SPEC is

Wave W7 — three defect fixes that share one habit, a failure the code declines to report:

| Story | Defect | Sev | What it fixes |
|---|---|---|---|
| `W7-S1` | `BUG-12` | high | Five `.expect()` store opens; a corrupt `library.db` makes the product vanish with no message |
| `W7-S2` | `BUG-3` | medium | The published page interpolates the Reviewer's Note into HTML unescaped |
| `W7-S3` | `BUG-10` | low | The MCP bridge can return an error whose message is the empty string |

## What to weigh it against

- `.control/registry/defects.yaml` — `BUG-12`, `BUG-3`, `BUG-10`. **Every claim the SPEC makes about
  a defect must match its register entry.**
- `.control/decisions/DEC-005-desktop-first-ordering.md` — freezes `sharing` and `agent-access`. The
  SPEC claims this wave is permitted by the decision's own sentence. **Check that claim against the
  decision's actual text**, including its Cost section on where a defect in a frozen component lands.
- `.what/settings/02-rules/rules-settings.md` `BR-118`; `.what/business-rules.md` `BR-17`.
- `.how/_platform/ARCHITECTURE-SPINE.md` `AD-7`, `AD-11`.
- `.how/settings/SDD-settings.md` § Failure Behaviour, row `LC-025` → `library.db`.
- `.control/registry/waves.yaml` wave `W7` — **the test names in the story files must match
  `waves.yaml` verbatim.** A silent rename is a finding.

## Questions the report should answer, beyond the lenses' own output

1. **Does the SPEC introduce anything `.what/` or `.how/` does not say?** A wave SPEC is a
   *projection*, and inventing a promise here is the failure mode this check exists for.
2. **Is the `DEC-005` reading honest**, or is it stretched to license work the freeze forbids?
3. **The SPEC's one Open Question** says no requirement covers output encoding on the published page,
   and that all six `sharing` NFRs were read and none fits. **Verify that independently** against
   `.control/registry/requirements.yaml`. If a requirement does cover it, that is a significant
   finding.
4. **Are the Non-goals real boundaries or decoration?** In particular, is "do not sweep the remaining
   `unwrap`/`expect` calls" a defensible scope line given `BUG-12` is about exactly that class?
5. **Does any acceptance criterion assert a literal instead of the behaviour it claims to cover?**
   This repository has landed that mistake three times; `W7-S2`'s escaping tests are the obvious
   risk, since an expected escaped string is easy to hardcode beside the implementation's own
   escaping.
6. **`W7-S1` plans a native `MessageBoxW` and a new Windows dependency.** Is that justified by
   `AD-11`, and does the story acknowledge it is adding a dependency?

## Report

Findings by severity, each naming the file and section, what is wrong, and what would fix it. State
plainly where you found nothing — a clean lens is a result, not a gap. **Do not edit the artifacts;
fixing a finding is the author's act.**

Write the report to `.work/w7-spec-review.md` and report `worker_done` with that path.
