STEP 2 of 5 — BUILD, FIX ROUND 2 of 2. Snapdown wave W1, story W1-S1. THIS IS THE LAST ROUND.

You are a FRESH worker. You did not write this code, you did not review it, and you did not do round 1.
Everything you need is on disk. You are in the worktree
D:/Developer/orca-workspaces/snapdown/w1-settings, branch kodesh87/w1-settings, where commit `ae14a84`
holds the code under repair.

WHY THIS ROUND EXISTS
Round 1 fixed thirteen findings. The panel's second pass then found that **the repair itself
introduced three defects**, and every code gate is green on all three — `cargo fmt`, `cargo clippy -D
warnings`, `cargo test --workspace`, both front ends' typecheck/lint/test, and `npm run build` all
pass while the defects are present. That is why they were found by reading and measuring rather than by
running the suite.

The cap on return trips is two, and this is the second. If a third pass still finds a must-fix the
coordinator escalates to the owner and no PR opens. So: fix these three properly, not quickly.

WHAT TO DO
Invoke the skill `bmad-build-auto` with this spec file path:

  _bmad-output/specs/w1-settings/stories/W1-S1-cargo-workspace-tauri-v2-shell-react-webview-tray-and-ci.md

Read the `## Spec Change Log` entry dated **2026-08-23 — Return trip 2 of 2** FIRST. It is the work.
There is an earlier entry from return trip 1 above it; that one is already done — do not redo it. Then
read `_bmad-output/specs/w1-settings/SPEC.md`; it is the canonical contract and the files in its
`companions:` frontmatter are part of it. The `<intent-contract>` block is the owner's and is fixed.

THE THREE, and each one names the fix shape. Take the shape given rather than your own — these were
adjudicated against measurements, and a different shape reopens the question.

  MF2R-1  `crates/snapdown-core/src/util/id.rs` — the id helper has no entropy and collides. Every id
          minted in one millisecond is identical when the clock has millisecond resolution. Fix:
          entropy comes IN through a port, `id_from_parts(unix_millis: u64, rand_b: [u8; 10])`, and
          extend the `Clock` port with `now_unix_millis()` so the port and the helper compose. Rewrite
          the test to assert what matters.

  MF2R-2  `crates/snapdown-core/tests/test_no_io.rs` — the traversal drops every target-gated edge, so
          it never reaches `getrandom`, which its own forbidden list names. And `getrandom` really is
          compiled into `snapdown-core` under workspace feature unification. Fix: **drop `uuid` from
          `snapdown-core` entirely** — once MF2R-1 lands, the core builds an id from bytes it was
          handed and formatting sixteen bytes as a lowercase hyphenated string is a few lines. Then
          also fix the traversal with `cargo metadata --filter-platform <triple>` and drop the
          `target.is_none()` predicates.

  MF2R-3  `web/ui/src/components/Modal.tsx` — `isClosing` is never cleared on the close path, so the
          overlay renders forever and blocks the whole window. Three parts: make the close path
          complete, give `closing` a rule in `components.css`, and fix the two tests that claim
          coverage they do not have.

  Plus one small one folded in from the panel's F-5, now that the corpus question behind it is
  resolved: `web/ui/src/components/Toast.tsx` gives its action `tabIndex={-1}`, making a clickable
  control unreachable by keyboard. `design-system.md` has been corrected — a Toast MUST NOT take focus
  when it appears, and its action MUST still be reachable. Remove the `tabIndex={-1}`.

DO NOT fix anything else. Every other finding is recorded as follow-up, several are deliberately
routed to W1-S2, and two — the panel's F-5 aside — were decided against on purpose. The amendment says
which and why.

EXIT CONDITION
That story spec's frontmatter reads `status: done`. That frontmatter is what the coordinator judges
this step by — your chat report does not settle it.

THREE RULES THAT BIND YOU

1. DEBUGGING IS CONDITIONAL, NEVER A PHASE. When a test or build fails and you do not know why, run
   the skill `wdi-systematic-debugging` BEFORE proposing any fix. A third failed fix attempt is the
   signal to escalate, not a fourth. Never change a test, an assertion, or a guard to turn something
   green. Read that sentence twice: **two of these three findings are tests that could not fail.** The
   fix is to make them able to fail, not to make them pass.

2. THE CORPUS IS NOT YOURS TO CHANGE. `.what/`, `.how/`, `.control/`, and `.constitution/` are
   read-only. The coordinator has already made the two corpus edits this round needs. If you believe
   a fix is impossible without touching the corpus, REPORT that — do not edit it.

3. VERIFICATION IS RUN, NOT ASSUMED. Every one of these MUST actually be executed before you report
   done, and their real output MUST be in your report:

     # from the repository root
     cargo fmt --all -- --check
     cargo clippy --workspace --all-targets -- -D warnings
     cargo test --workspace
     uv run .constitution/method/scripts/validate.py --check

     # from apps/desktop
     npm run typecheck
     npm run lint
     npm run test
     npm run build

     # from web/ui
     npm run typecheck
     npm run lint
     npm run test
     # NO build here — web/ui ships source. Round 1's amendment was wrong to ask for one.

   `validate.py` is expected RED at exactly 8 findings, and `.github/validate-baseline.txt` holds
   exactly those eight. If your change makes a baseline line disappear, remove that line from the
   baseline in the same commit — the workflow fails in that direction too, on purpose. Report the
   validator's finding count and any difference from the baseline.

   Two extra proofs are required this round, because these are the findings a green suite hid:
   - For MF2R-2, run `cargo tree --workspace -e normal -i getrandom@0.4.3` and paste the real output.
     If `snapdown-core` still appears anywhere in that tree, the fix is not done.
   - For MF2R-3, your new test MUST fail against the current `Modal.tsx` before you change it. Run it,
     see it fail, then fix the component. Paste both outputs.

GIT
Commit locally, with a clear message naming this as fix round 2. DO NOT PUSH and do not open a PR —
the coordinator is the only hand that pushes.

BLOCKED CONDITIONS
- `blocked / review repair loop exceeded 5 iterations` — report it; do not attempt a sixth.
- `blocked / intent gap` — revert, save a patch file, and report the patch path.

WHEN DONE
Report with:
  orca orchestration send --type worker_done --subject "<status>" --body "<per finding: what you
  changed; the two extra proofs; then the real output of every verification command; then anything you
  could not fix and why>" --task-id <task_id> --dispatch-id <dispatch_id> --outcome succeeded
  --files-modified "<paths>" --json
Use --outcome failed on failure; never encode failure only in prose. If Orca rejects the lifecycle
message, say so and ALSO write your report to
_bmad-output/specs/w1-settings/build-W1-S1-fix2-report.md. Write that file either way.
