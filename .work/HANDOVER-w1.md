# Handover — Snapdown W1, orchestrator role passing to a new session

Scratch, per `repo-guide.md` § `.work/`. Delete this file when W1 closes.

Written 2026-08-23 by the outgoing orchestrator (Claude). Everything durable is already in the
corpus or in `_bmad-output/specs/w1-settings/`; nothing here is authority. If this file and the
corpus disagree, the corpus wins.

## Read these first, in this order

1. `AGENTS.md` — and note it now says the **orchestrator role is not bound to any CLI**. Whichever
   agent this session runs in is the orchestrator. Only the **coder** role is bound, to OpenCode or
   Cursor.
2. `_bmad-output/specs/w1-settings/SPEC.md` — the canonical contract for this wave, and the files in
   its `companions:` frontmatter are part of it.
3. `.claude/skills/wdi-build/SKILL.md` — the five-step pipeline you are in the middle of. Read the
   whole thing; the Red Flags list at the bottom is the part that matters.
4. `.control/registry/waves.yaml` — W1 is `open`, W2..W5 are `planned`.

## Where the work actually stands

Wave **W1** is open. Release r1, PRD `capture-to-markdown`, component `settings`
(`mode: catalog`, `risk_accepted: medium`), size L. Five stories.

| Story | State |
|---|---|
| **W1-S1** | Code done. Panel passed on the **third** pass: 0 must-fix. PR **#1 open**. Steps 4 and 5 not finished — see below |
| W1-S2 | Not started. **The plan brief is written and ready to fire** — see below |
| W1-S3 | Not started |
| W1-S4 | Not started |
| W1-S5 | Not started |

Branch `kodesh87/w1-settings`, worktree `D:/Developer/orca-workspaces/snapdown/w1-settings`, four
commits ahead of `main`. PR: https://github.com/wiradigitalid/snapdown/pull/1

**One PR per wave, not per story.** W1-S2 through W1-S5 push to this same branch and update PR #1.
The wave is the tracker Task; that is the mapping `delivery-flow-guide.md` sets.

## What you owe immediately

1. **Finish W1-S1 Step 4.** The story-closing checklist is three items and it is not yet answered on
   the record: a decision worth remembering, a trap for the next agent, and test names matching what
   `waves.yaml` records. Answer all three before you treat the story as closed.
2. **W1-S1 Step 5 — watch CI.** PR #1 triggered the first CI run this repository has ever had.
   `korpus.yml` and `desktop-ci.yml`. Confirm every check concludes, and confirm the checks belong to
   the **pushed head SHA** — a green report from a stale run is a false report. Classify any failure
   before acting: a defect from this change goes back to Step 2 with `wdi-systematic-debugging` when
   the cause is unknown; infrastructure or flake gets **one** re-run and MUST NOT be patched around; a
   guard failure means fixing the content, never the guard.
3. **Collect the UI verification report.** A UI worker was still running when this handover was
   written — OpenCode, in the worktree, terminal `term_0b618b4b-f0cc-40f5-bcd5-76f264abdc35`. Its
   brief and its artifacts are in `_bmad-output/specs/w1-settings/ui-verify-W1-S1/`. Seven artifacts
   had landed; `report.md` had not. Two claims are already verified by hard evidence in those files
   (see below). Read the folder, judge from the artifacts and not from any prose, and if the worker
   died mid-run, re-dispatch the remaining claims rather than assuming them.

## The UI verification, and why it exists

The orchestrator is **forbidden** from driving a real UI: one snapshot returns a whole accessibility
tree and exhausts the context. It is dispatched to a coder, in the **same** worktree, running
**alone** — one desktop has one keyboard focus and one accessibility tree.

Seven claims were briefed. Two are settled by evidence already on disk:

- **Single instance** — `claim5_single_instance_log.txt`: the second launch exited 0, the process
  count stayed at 1, the original PID was unchanged.
- **The window renders** — `claim4_6_uia_tree.txt`: the tree carries the "Snapdown Settings" heading
  and a `text-field-input` with `IsKeyboardFocusable='True'`.

Still owed: tray-only start, the tray menu's two items, left-click behaviour, keyboard focus rings
across the window, and hover/active on the primary button.

That same tree also **hardened a follow-up into something concrete**: the Edit's accessible name is
its placeholder (`e.g. D:/SnapdownVault`), not `Vault Path`, so a screen reader announces the wrong
thing. Routed to W1-S3, which rewrites that screen.

## W1-S2 is ready to dispatch

The plan brief exists, and it names four things W1-S1 deliberately left unfinished plus two
follow-ups routed to this story. **Do not re-derive it** — the routing decisions were adjudicated
against measurements. Re-create it from `SPEC.md` § W1-S2 plus the story spec's two
`## Spec Change Log` entries, which is where all of it is recorded:

- W1-S1 left `crates/snapdown-store` a deliberate skeleton. This story fills it.
- **First run is wrong on purpose.** `apps/desktop/src-tauri/src/main.rs` opens Settings
  unconditionally, with a comment naming the finding. The decision already taken: first run is
  "the `setting` table holds no rows" — no new Setting key, no flag file, no corpus change. Close it
  once the store exists.
- `crates/snapdown-store` declares `uuid` and `chrono` and uses neither. Use them or drop them.
- Only two tables: `setting` and `schema_version`, exactly as `inventory-db.md` rows 8 and 9 say.
  The other seven tables belong to later waves and MUST NOT be created here.
- **Follow-up F-3, routed here:** the no-I/O guard is a dependency-**graph** check, so
  `snapdown-core` could call `std::fs`, `std::env`, or `SystemTime::now()` directly and stay green —
  `std` is not a graph node. Add a source-level deny (clippy `disallowed-methods`, or a CI check).
  The graph test stays; this is the half it structurally cannot cover.
- **Follow-up F-7, routed here:** `desktop-ci.yml` uses `npm install`, not `npm ci`, so the committed
  lockfiles do not gate the build.

Two rules decide W1-S2, and both are in `SPEC.md`: a corrupt `library.db` MUST refuse to open and
MUST NOT be replaced by a fresh empty one; and the Vault adapter MUST **resolve** a path and refuse
anything escaping the Vault root — resolve, never string-match, because W4's image route and W5's
publish path both rely on that single check.

## Two blockers you will hit

**1. The review panel for W2 cannot be staffed today.** `finding` is `risk_accepted: low`, so a
two-reviewer panel is **required**, and both reviewers must be CLI families other than the builder's.
Builder is OpenCode, so the panel is Claude + Cursor. **Cursor refuses to run**: the account's plan
rejects a named model — "Free plans can only use Auto." The model on each CLI is the user's setting
and MUST NOT be changed by an agent. So this needs the user to set Cursor to Auto or raise the plan,
**before W2 reaches Step 3**. W1 is unaffected: `settings` is `medium` and its diff touches no money,
no personal data, and no third party, so a second reviewer was optional there.

**2. Orca marks every OpenCode dispatch `agent_prompt_stalled`.** The readiness probe does not match
OpenCode 1.18.21, so the dispatch row reads `failed` and `worker_done` is rejected as
"capability is revoked" — while the worker is running normally. The workaround that works, every time:

```
orca orchestration worker-start --task <T> --worktree <exact-worktree-id> --agent opencode
# read the terminal; if the prompt did not land:
orca terminal wait --terminal <handle> --for tui-idle --timeout-ms 60000
orca orchestration worker-start --task <T> --retry-of <dispatch> --worktree <exact-id> --terminal <handle>
```

`--worktree` MUST be the exact `<repo-id>::<path>` selector; it defaults to the current worktree and
then rejects the terminal as a mismatch. Claude and Cursor dispatches land first time
(`stage: input_accepted`).

Because the lifecycle message is unreliable, **judge every step from the spec's frontmatter `status`
on disk**, never from a worker's chat report. `wdi-build` requires that anyway.

OpenCode will also block on a permission prompt when a UI worker copies screenshots out of
`%LOCALAPPDATA%\Temp\orca-computer-use\`. Watch for it — a blocked worker looks exactly like a busy
one, so any monitor you arm MUST match the failure signature and not only the success one.

## Traps that cost the outgoing orchestrator real time

- **Do not edit the corpus inside the worker's worktree while a worker is running.** Coordinator edits
  to `.how/`, `.control/`, and `.github/` got swept into the worker's own commit `f1704c1`. Nothing
  was lost and the edits are correct, but the commit now claims work it did not do. Edit on `main`
  and merge, or edit while no worker is live.
- **The heredoc trap.** Long markdown with quotes and backticks piped through a bash heredoc fails
  with `unexpected EOF`. Write the file with a file-writing tool, or write to a scratch file and copy.
  Backticks inside a double-quoted string get command-substituted — one memlog entry lost a word that
  way, and because a memlog is append-only the fix is a follow-up entry, never an edit.
- **`validate.py` findings versus skips.** `korpus.yml` gates against
  `.github/validate-baseline.txt`, which holds **findings only** — everything after the `Skipped:`
  heading is a skip, and a skip appearing or disappearing is not a regression. The workflow fails in
  **both** directions: a new finding, and a baseline line that no longer appears. So a wave that fixes
  a finding MUST remove its line in the same commit. The baseline is 8 lines today.
- **Two return trips is the cap, and it is per story.** W1-S1 spent both. Pass 1 found 13 must-fix,
  pass 2 found 3 more that pass 1's own repair had introduced, pass 3 found none. The panel MUST be
  re-run after every fix round — that is how the second set was caught. On hitting the cap you
  escalate to the user and MUST NOT open a PR carrying an unresolved must-fix.

## What good looks like in a worker brief

The three briefs that worked all did the same things, and the ones that needed a second try did not:

- Name the exit condition as a fact on disk — "that spec's frontmatter reads `status: done`" — and
  say plainly that the chat report does not settle it.
- List every verification command with the directory it runs from, and demand the **real output**.
- When a finding is a test that cannot fail, say so in those words and say the fix is to make it able
  to fail, not to make it pass. Two of the three round-2 findings were exactly that.
- Demand a proof for anything a green suite would hide. Round 2 was told to run
  `cargo tree --workspace -e normal -i getrandom@0.4.3` and to see the new Modal test **fail before**
  changing the component. Both proofs came back honest, and neither would have without being asked.
- Say what is deliberately out of scope, and say why, or the worker fixes the follow-ups too.

## State of the corpus

G1 through G4 are closed and `gates_passed: [G1, G2, G3, G4]`. Do not reopen one without the
change-control matrix in `delivery-flow-guide.md`.

`.constitution/project/codebase-stack-guide.md` is still the empty seeded file, deliberately: its own
header forbids filling it before code ratifies it. **W1's Phase 4 distillation is what fills it**,
from this wave's real commands. The verification commands live in `SPEC.md` until then.

Phase 4, when the five stories are done, in this order and stopping at the first failure: registry
catch-up (every `LC` registered, every `touches` resolving — V12), inventories re-derived from code
with `inventory.py` and the plan-versus-reality gap **reported not patched**, structure maps refreshed,
distillation, RTM green, then `status: closed`.

`.constitution/project/inventory-readers.py` is still the shipped skeleton with `SKELETON = True`.
`inventory.py` refuses to run until that line is deleted, and `wdi-init` intent `readers` is what
fills it — needed at Phase 4, not before.
