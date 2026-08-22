# Handover — Snapdown W1, orchestrator role passing to a new session

Scratch, per `repo-guide.md` § `.work/`. Delete when W1 closes.

Written 2026-08-23 by the outgoing orchestrator (Claude), **second revision** — the first was wrong
about the UI verification and is superseded by this. Everything durable is in the corpus or under
`_bmad-output/specs/w1-settings/`. If this file and the corpus disagree, the corpus wins.

## Read these first, in this order

1. `AGENTS.md` — the **orchestrator role is no longer bound to any CLI.** Whichever agent this
   session runs in is the orchestrator. Only the **coder** role is bound, to OpenCode or Cursor.
2. `_bmad-output/specs/w1-settings/SPEC.md` — the canonical contract for this wave. The files in its
   `companions:` frontmatter are part of it.
3. `.claude/skills/wdi-build/SKILL.md` — the five-step pipeline you are mid-way through. Read all of
   it; the Red Flags list at the bottom is the part that matters.
4. `.control/registry/waves.yaml` — W1 `open`, W2..W5 `planned`.

## Ready-made dispatch briefs — use these, do not re-derive them

`_bmad-output/specs/w1-settings/dispatch-briefs/` holds four files, committed so they survive the
session that wrote them:

| File | What it is |
|---|---|
| `W1-S1-ui-verification-finish.md` | **Fire this first.** Finishes the unfinished UI verification. Ready as written |
| `W1-S2-step1-plan.md` | W1-S2 Step 1. Names the four things W1-S1 left and the two follow-ups routed here |
| `EXAMPLE-panel-review-brief.md` | The panel brief that produced the pass finding the three worst defects. Copy its shape |
| `EXAMPLE-build-fix-brief.md` | The fix-round brief that worked. Note how it demands proofs |

Two briefs, not one, are examples on purpose: the shape of the brief is what decided whether a worker
did the job or looked like it did.

## Where the work stands, exactly

Wave **W1** open. Release r1, PRD `capture-to-markdown`, component `settings`
(`mode: catalog`, `risk_accepted: medium`), size L. Five stories.

Branch `kodesh87/w1-settings`, head **`3da6372`**, worktree
`D:/Developer/orca-workspaces/snapdown/w1-settings`. PR **#1**:
https://github.com/wiradigitalid/snapdown/pull/1

**One PR per wave, not per story.** W1-S2..S5 push to this branch and update PR #1.

| Story | State |
|---|---|
| **W1-S1** | Code done. Panel clean on the **third** pass: 0 must-fix. PR open. **Two things outstanding — see below** |
| W1-S2 | Not started. Brief ready |
| W1-S3 | Not started |
| W1-S4 | Not started |
| W1-S5 | Not started |

### W1-S1, outstanding item 1 — the UI verification is unfinished

This is the correction the first revision of this file got wrong. A UI worker ran, produced **eight
artifacts**, cleaned up properly — Vite stopped, no `desktop` process left — and then **ended its
session without writing `report.md` or `commands.md`**. The evidence exists; the record does not.

The orchestrator **cannot read those artifacts itself.** A UI snapshot or accessibility tree returns a
whole tree into an orchestrator's context and exhausts it; `AGENTS.md` forbids it. That is not a
formality — it is why the report is the only route the evidence has.

Two claims are settled by artifacts that happen to be plain text, and those the orchestrator did read:

- **Claim 5, single instance** — `claim5_single_instance_log.txt`: the second launch exited 0, the
  process count stayed at 1, the original PID was unchanged.
- **Claim 4, the window renders** — `claim4_6_uia_tree.txt`: the tree carries the "Snapdown Settings"
  heading and a `text-field-input` with `IsKeyboardFocusable='True'`.

Still owed: the report itself, plus claim 3 (left-clicking the tray shows the window) and claim 7
(hover and active on the primary button), which have no artifact at all.

**Fire `dispatch-briefs/W1-S1-ui-verification-finish.md`.** It carries all of this, including the one
judgement call: this build opens Settings on every launch on purpose, so "a window appeared at
startup" is **not** a failure of claim 1.

That same accessibility tree hardened a follow-up into something concrete: the Edit's accessible name
is its placeholder (`e.g. D:/SnapdownVault`), not `Vault Path`, so a screen reader announces the wrong
thing. Routed to W1-S3, which rewrites that screen.

### W1-S1, outstanding item 2 — Step 4 and Step 5

**Step 4, the story-closing checklist.** Three items, not yet answered on the record: a decision worth
remembering; a trap for the next agent; test names matching what `waves.yaml` records. Answer all
three before treating the story as closed.

**Step 5, CI.** PR #1 triggered the first CI this repository has ever had, and it needed one fix
already: `Validate Corpus` failed in its `Install uv` step because `enable-cache: true` errors when
its dependency glob matches nothing, and there is no `uv.lock` here. Fixed in `3da6372`.

State at handover:

| Check | Result |
|---|---|
| `Validate Corpus` | **pass** — and this is the meaningful one: the baseline mechanism works in CI, not just locally |
| `Desktop Frontend Check` | **pass** |
| `Rust Workspace Build & Test` | **pending** — never yet observed to conclude on `windows-latest` |

**You owe the Rust job's verdict.** Confirm every check concludes, and confirm the checks belong to
the **pushed head SHA** — a green report from a stale run is a false report. Classify any failure
before acting: a defect from this change goes to Step 2 with `wdi-systematic-debugging` if the cause
is unknown; infrastructure or flake gets **one** re-run and MUST NOT be patched around; a guard
failure means fixing the content, never the guard.

**New follow-up, unfixed on purpose:** every check runs **twice**, because `push` on `kodesh87/*` and
`pull_request` both fire. Wasteful and it compounds with the already-recorded follow-up about
hardcoded branch patterns in a public repo's CI. Left alone rather than churning another CI run
mid-handover.

## W1-S2, and what it must close

`dispatch-briefs/W1-S2-step1-plan.md` is written and ready. Its routing decisions were adjudicated
against measurements, so do not re-derive them. In short:

- W1-S1 left `crates/snapdown-store` a deliberate skeleton. This story fills it.
- **First run is wrong on purpose.** `apps/desktop/src-tauri/src/main.rs` opens Settings
  unconditionally with a comment naming the finding. The decision already taken: first run is "the
  `setting` table holds no rows" — no new Setting key, no flag file, no corpus change.
- `crates/snapdown-store` declares `uuid` and `chrono` and uses neither. Use them or drop them.
- Only two tables: `setting` and `schema_version`, exactly as `inventory-db.md` rows 8 and 9 say.
- **Follow-up F-3, routed here:** the no-I/O guard is a dependency-**graph** check, so `snapdown-core`
  could call `std::fs`, `std::env`, or `SystemTime::now()` directly and stay green — `std` is not a
  graph node. Add a source-level deny. The graph test stays; this is the half it cannot cover.
- **Follow-up F-7, routed here:** `desktop-ci.yml` uses `npm install`, not `npm ci`, so the committed
  lockfiles do not gate the build.

Two rules decide the story, both in `SPEC.md`: a corrupt `library.db` MUST refuse to open and MUST NOT
be replaced by a fresh empty one; and the Vault adapter MUST **resolve** a path and refuse anything
escaping the Vault root — resolve, never string-match, because W4's image route and W5's publish path
both rely on that single check.

## Two blockers

**1. W2's review panel cannot be staffed today.** `finding` is `risk_accepted: low`, so a two-reviewer
panel is **required**, and both reviewers must be CLI families other than the builder's. Builder is
OpenCode, so the panel is Claude + Cursor. **Cursor refuses to run:** the account's plan rejects a
named model — "Free plans can only use Auto." The model on each CLI is the user's setting and an agent
MUST NOT change it. So this needs the user to set Cursor to Auto or raise the plan, **before W2
reaches Step 3**. W1 was unaffected: `settings` is `medium` and its diff touched no money, no personal
data, and no third party, so the second reviewer was optional there.

**2. Orca marks every OpenCode dispatch `agent_prompt_stalled`.** The readiness probe does not match
OpenCode 1.18.21, so the dispatch row reads `failed` and `worker_done` is rejected as "capability is
revoked" — while the worker runs normally. The sequence that works every time:

```
orca orchestration worker-start --task <T> --worktree <exact-worktree-id> --agent opencode
# read the terminal; if the prompt did not land:
orca terminal wait --terminal <handle> --for tui-idle --timeout-ms 60000
orca orchestration worker-start --task <T> --retry-of <dispatch> --worktree <exact-id> --terminal <handle>
```

`--worktree` MUST be the exact `<repo-id>::<path>` selector — it defaults to the current worktree and
then rejects the terminal as a mismatch. Claude and Cursor land first time (`stage: input_accepted`).

Because the lifecycle message is unreliable, **judge every step from the spec's frontmatter `status`
on disk**, never from a worker's chat report. `wdi-build` requires that regardless.

OpenCode also blocks on a permission prompt when a UI worker copies screenshots out of
`%LOCALAPPDATA%\Temp\orca-computer-use\`. Approve with Tab then Enter twice ("Allow always", then
"Confirm"). **A blocked worker looks exactly like a busy one**, so any monitor you arm MUST match the
failure signature and not only the success one.

## Traps that cost the outgoing orchestrator real time

- **A worker can finish the work and leave no record.** That is what happened to the UI verification,
  and it is the single most expensive thing in this handover. Brief the deliverable as the exit
  condition, in the worker's own terms, and say plainly that an artifact-free claim is worthless.
- **Anything in a session's scratchpad dies with the session.** Two dispatch briefs nearly went that
  way; they are now committed under `dispatch-briefs/`. Write briefs into the repo.
- **Do not edit the corpus inside a live worker's worktree.** Coordinator edits to `.how/`,
  `.control/`, and `.github/` were swept into the worker's own commit `f1704c1`. Nothing was lost and
  the content is right, but that commit now claims work it did not do. Edit on `main` and merge, or
  edit while no worker is live.
- **The heredoc trap.** Long markdown with quotes and backticks through a bash heredoc fails with
  `unexpected EOF`. Write the file with a file-writing tool, or write to scratch and copy. Backticks
  inside a double-quoted string get command-substituted — one memlog entry lost a word that way, and
  a memlog is append-only, so the fix is a follow-up entry, never an edit.
- **Findings versus skips.** `korpus.yml` gates against `.github/validate-baseline.txt`, which holds
  **findings only** — everything after the validator's `Skipped:` heading is a skip, and a skip
  appearing or disappearing is not a regression. The workflow fails in **both** directions: a new
  finding, and a baseline line that no longer appears. A wave that fixes a finding MUST remove its
  line in the same commit. The baseline is 8 lines today.
- **Two return trips is the cap, per story, and W1-S1 spent both.** Pass 1 found 13 must-fix, pass 2
  found 3 more that pass 1's own repair had introduced, pass 3 found none. The panel MUST be re-run
  after every fix round — that is how the second set was caught. On hitting the cap you escalate and
  MUST NOT open a PR carrying an unresolved must-fix.

## What made a worker brief actually work

The briefs that worked all did the same four things. Both examples in `dispatch-briefs/` show them:

- Name the exit condition as a **fact on disk** — "that spec's frontmatter reads `status: done`" — and
  say the chat report does not settle it.
- List every verification command **with the directory it runs from**, and demand the real output.
- When a finding is a test that cannot fail, say so in those words and say the fix is to make it able
  to fail, not to make it pass. Two of the three round-2 findings were exactly that.
- Demand a proof for anything a green suite would hide. Round 2 was told to run
  `cargo tree --workspace -e normal -i getrandom@0.4.3` and to watch the new Modal test **fail before**
  changing the component. Both came back honest, and neither would have without being asked.

## State of the corpus

G1–G4 closed, `gates_passed: [G1, G2, G3, G4]`. Do not reopen one without the change-control matrix
in `delivery-flow-guide.md`.

`.constitution/project/codebase-stack-guide.md` is still the empty seeded file, deliberately — its own
header forbids filling it before code ratifies it. **W1's Phase 4 distillation fills it**, from this
wave's real commands. Until then the commands live in `SPEC.md`.

`.constitution/project/inventory-readers.py` is still the shipped skeleton with `SKELETON = True`.
`inventory.py` refuses to run until that line is deleted, and `wdi-init` intent `readers` fills it.
Needed at Phase 4, not before.

Phase 4, once the five stories are done — in this order, stopping at the first failure: registry
catch-up (every `LC` registered, every `touches` resolving, V12); inventories re-derived from code with
`inventory.py`, and the plan-versus-reality gap **reported, not patched**; structure maps refreshed;
distillation; RTM green; then `status: closed`.

## The prompt to open the new session with

```
You are the orchestrator for Snapdown wave W1. Read .work/HANDOVER-w1.md first, then
AGENTS.md — the orchestrator role is not bound to any CLI now; only the coder role is,
to OpenCode or Cursor.

Start by firing the ready-made brief at
_bmad-output/specs/w1-settings/dispatch-briefs/W1-S1-ui-verification-finish.md — a UI
worker did that verification and ended without writing its report, and you cannot read
its artifacts yourself.

Then finish W1-S1 Step 4 (the three-item story-closing checklist) and Step 5 (the Rust
CI job on PR #1 has never been observed to conclude). Then run W1-S2 through W1-S5 with
the wdi-build five-step pipeline, using the briefs in that same folder. Then close the
wave through Phase 4.

Two things will stop you if you do not plan for them: Orca marks every OpenCode dispatch
agent_prompt_stalled even though the worker runs fine — the handover has the workaround —
and W2's panel cannot be staffed until the user fixes Cursor's plan.

Judge every step from the spec's frontmatter status on disk, never from a worker's chat
report.
```
