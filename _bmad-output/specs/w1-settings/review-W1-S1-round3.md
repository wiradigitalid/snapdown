---
title: 'W1-S1 code review — panel pass 3 (final)'
story: W1-S1
reviewer: fresh reviewer, panel pass 3
date: '2026-08-23'
commit_reviewed: f1704c1
baseline: 6a470fd
verdict: '0 must-fix, 9 follow-up'
---

# W1-S1 — code review, panel pass 3

## Verdict

**No must-fix. The code is clean against the SPEC, the story's acceptance criteria, AD-1..AD-9,
`design-system.md`, `inventory-db.md`, and `DEC-001`.** I recommend the pull request opens.

Nine follow-ups are recorded below. Every one is either a new low-severity observation with no
reachable defect, or a latent weakness with no path on any machine this project builds on. None
should hold the PR, and I am explicitly **not** re-reporting anything the two change logs already
routed elsewhere as if it were new.

## A note on the working tree

The task brief described round 2 as uncommitted. It is not — it is committed as `f1704c1`
("fix(w1-s1): return trip round 2 …"), and `git status --short --untracked-files=all` was **empty**
at the start of this review and empty again at the end. I reviewed `f1704c1` against `6a470fd`
(67 files, +17854/-39). No stray working-tree state existed to review separately.

## What I ran, and what it returned

Every command below was executed by me in this worktree, not read from a report.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0, no diff |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, zero warnings |
| `cargo test --workspace` | 5 tests, all pass (3 core unit, 1 `snapdown_core_has_no_io_dependency`, 1 store) |
| `apps/desktop` → `npm run typecheck` | exit 0 |
| `apps/desktop` → `npm run lint` | exit 0 |
| `apps/desktop` → `npm run test` | 1 test pass (`app_renders_shell`) |
| `apps/desktop` → `npm run build` | exit 0, `vite v7.3.6`, 39 modules, dist written |
| `web/ui` → `npm run typecheck` | exit 0 |
| `web/ui` → `npm run lint` | exit 0 |
| `web/ui` → `npm run test` | 16 tests pass |
| `uv run .constitution/method/scripts/validate.py --check` | **RED, exactly 8 findings** |

`web/ui` has no `build` script, which is correct per round 2's correction.

### The validator baseline matches, in both directions

I reproduced the workflow's own comparison locally rather than eyeballing it:

```
sed '/^Skipped:/,$d' validate.log | grep -E '^  V[0-9]+' | sort > cur.txt
sort .github/validate-baseline.txt > base.txt
cmp -s cur.txt base.txt   ->  BASELINE MATCHES EXACTLY
```

8 lines on each side, byte-identical: 4 x V18 (W1-S2..S5 have no story file yet), 2 x V24
(`bmad-project-context` template citations), 2 x V25 (`mcp-bridge`, `web-api`). **`desktop-app` is
correctly gone from V25**, which is what the SPEC required of this wave. `korpus.yml:38-45` fails on
a new finding *and* on a vanished baseline line, and prints `comm` in both directions — the shape
MF-12 specified.

## A. `util/id.rs` and `ports/mod.rs` — MF2R-1 and MF2R-2 genuinely fixed

I did not reason about the byte layout; I extracted `id_from_parts` verbatim into a standalone
`rustc` program outside the repository and measured it.

**Byte layout is RFC 9562 §5.7-conformant.** `id.rs:5-28`:

- bytes 0-5 — 48-bit big-endian `unix_ts_ms` ✓
- byte 6 high nibble — version `0x7`. Measured across 200,000 random samples: **0 bad version
  nibbles**.
- byte 6 low nibble + byte 7 — 12-bit `rand_a`, from `rand_b[0..2]` masked `0x0FFF` ✓
- byte 8 top 2 bits — variant `0b10`. Measured across the same 200,000 samples: **0 bad variant
  nibbles**.
- byte 8 low 6 bits + bytes 9-15 — 62-bit `rand_b`, from `rand_b[2..10]` ✓

**No random bits are overwritten.** I flipped each of the 80 input bits individually and checked
whether the output string changed: **74 of 80 input bits affect the output**. That is exactly the
RFC's random capacity (12-bit `rand_a` + 62-bit `rand_b` = 74). The six bits that do not reach the
output are `rand_b[0]` bits 4-7 and `rand_b[2]` bits 6-7 — precisely the bits the RFC's fixed
version and variant nibbles leave no room for. Mapping 80 supplied bits into 74 slots must discard
six; nothing is clobbered, and each of the 74 slots is fed by exactly one independent input bit, so
a uniform source yields a uniform 74-bit tail.

**Two calls with different `rand_b` do not collide in practice.** 500,000 distinct `rand_b` values
at one fixed millisecond produced **500,000 distinct ids, 0 collisions**. All three collision shapes
MF2R-1 measured are gone: the millisecond-resolution case (`nanos % 1_000_000 == 0`) no longer
exists because there is no `nanos` parameter, and there is no `u16` cast to wrap.

**Formatting is exactly right.** `id.rs:30-48` produces 36 characters, and I verified
character-by-character that positions 8/13/18/23 are hyphens and every other position is a lowercase
ASCII hex digit — 8-4-4-4-12. Sample: `018bcfe5-6800-7fff-bfff-ffffffffffff`.

**`uuid` is gone from the core.** `crates/snapdown-core/Cargo.toml:9-15` lists only `serde`,
`serde_json`, `thiserror` (and `cargo_metadata` as a dev-dependency). `cargo tree -p snapdown-core -e
normal` shows 13 crates and no `uuid`, no `getrandom`, no `libc`. The only textual `uuid` left in the
crate is a test name and the forbidden-list entry.

**`EntropySource` is a shape an adapter can satisfy.** `ports/mod.rs:9-11` —
`fn random_bytes_10(&self) -> [u8; 10]`, `&self`, infallible, returns an owned array. A store-side
adapter over `getrandom`/`rand` satisfies this in three lines; `&self` rather than `&mut self` is
right for a stateless OS source. `Clock` at `ports/mod.rs:4-7` now carries both `now_rfc3339()` and
`now_unix_millis()`, so the port and `id_from_parts(unix_millis, rand_b)` compose — F-2 closed.

## B. `tests/test_no_io.rs` — the guard is real, and I broke it on purpose to prove it

I ran three probes against the real crate, each reverted immediately afterwards.

**Probe A — the exact shape MF2R-2 named.** Added
`[target.'cfg(windows)'.dependencies] libc = "0.2"` to `snapdown-core/Cargo.toml`.
Result: **FAILED** at `test_no_io.rs:103` — "must not have transitive dependency on forbidden crate:
libc". The `target.is_none()` requirement is gone from both predicates and `--filter-platform` does
the pruning, so target-gated normal edges are now visited.

**Probe B — fail-closed, and multi-hop reach through a gate.** Added a plain normal dependency on
`home = "0.5"`, which is *not* on the forbidden list and whose own `windows-sys` dependency is
target-gated. Result: **FAILED** at `test_no_io.rs:127` — "unexpected transitive dependency `home`.
Only explicitly permitted crates are allowed." The printed traversal included `home`, `windows-sys`
**and** `windows-link` — so it reaches two hops *through* a target gate. **The allowlist fails
closed**, and the forbidden list is now belt-and-braces rather than the load-bearing check.

**Probe C — the filter is doing real work, not passing everything.** Added
`[target.'cfg(target_os = "linux")'.dependencies] libc = "0.2"` on this Windows host. Result:
**passed**, correctly — `--filter-platform x86_64-pc-windows-msvc` pruned the Linux-only edge. So
the filter prunes as well as admits.

**What it would still miss, stated honestly:** a crate reachable only through a *build*-dependency
chain is not visited (`dep_kinds` kind `Build` is not `Normal`). That is correct scoping — a
build-dependency runs at build time and is not linked into the crate — and MF-2 asked for the normal
graph. And, as round 2's F-3 already records, a graph check structurally cannot see a direct
`std::fs` / `SystemTime::now()` / `std::env` call, because `std` is not a graph node; that is routed
to W1-S2 and I am not re-raising it.

The one genuine weakness I found is the hardcoded triple, recorded as **FU-1** below. It is not
reachable on any machine this project builds on today.

## C. `Modal.tsx`, `components.css`, `components.test.tsx` — MF2R-3 fixed, and measured

`Modal.tsx:42` now clears `isClosing` **unconditionally** in the `else` branch, not only when
`isOpen` goes true. I drove every path the brief names with a throwaway test file
(`web/ui/src/test/zz_probe.test.tsx`, since deleted) against a parent that owns `isOpen`:

| Path | Result |
| --- | --- |
| Escape, stateful parent | dialog **unmounts**, `.modal-overlay` gone from the DOM ✓ |
| Scrim click, stateful parent | dialog **unmounts**, `.modal-overlay` gone ✓ |
| Parent sets `isOpen=false` with neither | returns `null` at `Modal.tsx:94` immediately ✓ |
| Unmount mid-close | no residual node, and `console.error` never called ✓ |
| Open -> close -> **reopen** -> close again | reopens at `data-state="open"`, closes again ✓ |
| Focus restore | `document.activeElement` returns to the outside trigger after close ✓ |

**The overlay cannot get stuck in any of them.** The one remaining wedge is a parent that *ignores*
`onClose` entirely: the overlay stays mounted at `data-state="closing"`. That is recorded as **FU-2**
— it is a consumer-contract violation (such a parent leaves a controlled modal open by definition),
there is no consumer at all in this wave, and the only delta versus the correct behaviour is that
the stuck overlay is transparent.

**`closing` now has a visual rule**, `components.css:134-138`, so `data-state="closing"` means
something. `design-system.md:70` requires `open · closing`; both states have rules.

**The stylesheet actually loads.** I checked the chain rather than assuming it:
`main.tsx:4` -> `apps/desktop/src/styles/tokens.css:1` `@import` -> `web/ui/src/styles/tokens.css:1`
`@import "./components.css"`. I then grepped the **built** bundle,
`apps/desktop/dist/assets/index-xvQL0pAd.css`: 6 `focus-visible` occurrences,
`.btn-primary:not(:disabled):hover`, `.btn-primary:not(:disabled):active`,
`.checkbox-input:focus-visible`, and `.modal-overlay[data-state=closing]` are all present in the
shipped CSS. MF-5's repair is live in the running app, not merely on disk.

**The committed tests are a real guard, and I proved it.** I reintroduced the MF2R-3 defect by
deleting the `setIsClosing(false)` at `Modal.tsx:42` and re-ran the committed suite:
`components.test.tsx:130` ("unmounts when closed via Escape or scrim click from stateful parent")
**FAILED**. So the guard catches the exact regression it exists for. `components.test.tsx:150` is now
named "renders title, content, and **traps tab focus**" and does assert it — Tab on the last element
wraps to the first (`:167-169`), Shift+Tab on the first wraps to the last (`:171-174`). The name no
longer overclaims.

The gap that remains is **FU-3**: the test at `:130` is named "via Escape **or scrim click**" but
only exercises Escape. Round 2 asked for both. I measured the scrim path myself and it is correct,
and both paths are guarded by the same single line — so the Escape assertion catches the scrim
regression transitively. Incomplete coverage, not a weakened guard, and not a must-fix.

## D. `Toast.tsx` — both halves of the corpus rule hold

`design-system.md:69` and `:94-95` require two things at once. Both are true:

- **MUST NOT take focus when it appears.** `Toast.tsx` calls `.focus()` nowhere, and the container
  carries `tabIndex={-1}` (`Toast.tsx:30`), keeping it out of the tab order. Nothing steals focus on
  mount.
- **Its action MUST be reachable by keyboard.** `tabIndex={-1}` is gone from the action button
  (`Toast.tsx:52-68`) — it is a plain `<button type="button">`, not disabled, so it is a tab stop.
  `pointerEvents: 'auto'` at `:64` is a mouse concern and does not affect focusability.
  `components.test.tsx:111-126` asserts the button exists, lacks `tabIndex="-1"`, and fires.

## Everything else I checked, and found clean

- **MF-1 / MF-4** — `apps/desktop/index.html` exists and mounts `#root` matching `main.tsx:6`;
  `tauri.conf.json:17` points the window at `index.html`, not a route nothing serves.
  `npm run build` succeeds and writes `dist/`.
- **MF-8 / `inventory-db.md`** — **no persisted state exists in this wave.** A sweep for
  `localStorage`, `sessionStorage`, `indexedDB`, `std::fs`, `File::create`, `write_all`,
  `app_data_dir`, `app_config_dir`, `create_dir`, and `ran_before` across all `.rs`/`.ts`/`.tsx`
  returned **nothing**. `.ran_before` and the whole `is_first_run` shape are gone from `main.rs`, and
  the required comment is at `main.rs:56-57`. So nothing is owed a row in `inventory-db.md`, and
  there is no corpus drift. (F-11, Settings opening on every launch, is the coordinator's own MF-8
  instruction and is routed to W1-S2 — not re-raised.)
- **MF-10** — `vite: "^7.0.0"` in both `apps/desktop/package.json:34` and
  `web/ui/package.json:28`; the build banner confirms **7.3.6** resolved. `DEC-001` honoured.
- **MF-11** — `App.tsx` has no "Save Configuration" button, no success toast, and no hardcoded
  "Active Route". No `invoke_handler` is registered, and nothing claims to persist. No control lies.
- **MF-13** — `apps/desktop/src-tauri/gen/schemas` is ignored (`.gitignore:9-10`, confirmed via
  `git check-ignore -v`) and `git ls-files apps/desktop/src-tauri/gen` returns nothing.
- **MF-7** — `Markdown.tsx` is gone from `web/ui/src/components/` and from `index.ts`; no code
  references it. `design-system.md` records it as "Not yet built. W3 creates it", so corpus and code
  agree.
- **MF-9** — I confirmed the new `web/ui` gates are not vacuous: `npx eslint . -f json` reports
  **14 files inspected**, including all nine components and the test file. `tsconfig.json:19`
  includes `src`. `desktop-ci.yml:48-64` runs `web/ui` typecheck, lint, and test plus the desktop
  build.
- **AD-6, nothing leaves the machine** — a sweep for `fetch(`, `XMLHttpRequest`, `WebSocket`,
  `reqwest`, `hyper`, `ureq`, and `http(s)://` across all `.rs`/`.ts`/`.tsx` returned only the three
  crate names on `test_no_io.rs`'s own forbidden list. No telemetry, no update check, no crash
  reporter. `capabilities/default.json` grants `core:default` only.
- **Excluded stack** — no Next.js, no Express, no Go, no `web/api/` (`web/` holds only `ui`), no
  `crates/snapdown-mcp` (`crates/` holds only the two library crates).
- **Public repository** — a credential sweep restricted to the 67 files this story changed matched
  only the words "token"/"tokens.css" in design-token prose. No key, secret, password, private key,
  or capture-derived fixture. Icons are 105-360 byte synthetic placeholders.
- **Language** — every identifier, file name, and config key is English.
- **Tray and single instance** — `main.rs:20-22` registers `tauri-plugin-single-instance` first and
  focuses the existing window; `main.rs:27-54` builds the tray with exactly "Settings" and "Quit",
  `show_menu_on_left_click(false)`, and a left-click handler. Window is `"visible": false`
  (`tauri.conf.json:22`), so the app starts to the tray. I did **not** launch the app — a separate
  worker owns that.

## Follow-ups

None of these should hold the PR.

**FU-1 — `test_no_io.rs:6-12` hardcodes an `x86_64` triple, so the guard is arch-blind.**
`cfg!(target_os = ...)` picks the OS but the architecture is fixed at `x86_64`. On an
`aarch64-pc-windows-msvc` or `aarch64-apple-darwin` host, `--filter-platform` would prune against the
wrong platform and an arch-gated OS dependency could slip past. Not reachable today — this host and
`windows-latest` are both x86_64 — but the fix is cheap: compose the triple from `cfg!(target_arch)`
as well, or fail loudly on an unrecognised host. **New; the strongest of these follow-ups.**

**FU-2 — `.modal-overlay[data-state="closing"]` has no `pointer-events: none`
(`components.css:134-138`).** A parent that ignores `onClose` leaves a transparent overlay that still
swallows every click at `--z-modal`. No consumer in this wave. **New, speculative.**

**FU-3 — `components.test.tsx:130` is named for Escape *or scrim click* and asserts only Escape.**
Round 2 asked for both. The scrim path is correct (I measured it) and shares the guarded line, so
this is missing redundancy, not a hole. Adding the six-line scrim assertion is worthwhile in W1-S2.
**New.**

**FU-4 — `TextField` and `TextArea` labels are not programmatically associated with their controls.**
`TextField.tsx:29-38` and `TextArea.tsx:36-40` render a bare `<label>` with no `htmlFor`, no `id`,
and no wrapping, so neither input has an accessible name; `aria-invalid` and `aria-describedby` are
also absent, so `errorMessage` is not announced. A grep for `htmlFor|aria-label*|aria-invalid` across
`web/ui/src/components/` returns **nothing**. `Checkbox` is fine — it wraps its input in the label.
No clause in `design-system.md` mandates the association (its rules cover `focus-visible`, which is
present), so this is not a contract violation — but W1-S3 builds the real Settings screen on these
two elements, so it is worth closing first. **New.**

**FU-5 — `Modal.tsx:137` sets `outline: 'none'` on the dialog container.** The container is
`tabIndex={-1}` and receives programmatic focus at `Modal.tsx:35` and `:70`, so in a modal with no
focusable children a keyboard user sees no focus indicator. Narrow; `ConfirmDialog` always has
buttons. **New, low.**

**FU-6 — `Button` does not default to `type="button"` (`Button.tsx:22-27`).** Inside a `<form>` it
would submit. There is no form in this wave. **New, low.**

**FU-7 — `snapdown-store` declares `uuid` and `chrono` and uses neither**
(`crates/snapdown-store/Cargo.toml:15-16`). W1-S2 will use them; until then they are dead
dependencies. Harmless — they are outside the core, so `test_no_io` is unaffected. **New, low.**

**FU-8 — `App.tsx:60-64` renders an inert "Vault Path" `TextField`.** It makes no claim (MF-11's
lying Save button and toast are both gone) but a typed value is silently discarded. The real screen
is W1-S3. **New, low.**

**FU-9 — `unix_millis >= 2^48` truncates silently (`id.rs:5-10`).** That is the RFC 9562 field
width, so it is conformant, and the boundary is the year 10889. Noted only for completeness.
**New, trivial.**

### Already-recorded follow-ups I confirmed still stand, and deliberately did NOT re-report

Round 1's F1 (`csp: null`, routed to W3), F2 (no `rust-toolchain.toml`), F6 (`defaultValue` char
count), F7 (`Modal` `aria-labelledby`), **F9 (literal `rgba(0,0,0,0.5)` scrim, `borderRadius: '50%'`,
`2px` ring — still at `Modal.tsx:105` and `MarkerBadge.tsx:23,26`)**, F10, F11
(`SettingKey::Custom`), F12, F13 (`bundle.active: true`), F14 (`kodesh87/*` in CI); and round 2's
F-3 (source-level deny, routed to W1-S2), F-7 (`npm install` not `npm ci`, routed to W1-S2), F-11
(Settings opens every launch, closed by W1-S2).

**On F9 specifically, because it is the one place the corpus now reads ambiguously.**
`design-system.md:47-50` says the `--color-scrim` token "is recorded here rather than in a component
so that whichever wave next touches `Modal` finds it" — and round 2 *did* touch `Modal`. Read
literally that could be an obligation on this round. But F9 was explicitly recorded as a follow-up in
round 1 and re-affirmed as standing in round 2, and my brief forbids re-reporting a deliberately
routed follow-up as a must-fix. So I have left it as a follow-up. **The coordinator may want to
sharpen that sentence in `design-system.md`** so the next reviewer does not have to make the same
judgement call. Also confirmed: F8 is resolved — `--color-marker-ring` now appears in the token
table at `design-system.md:39`.

## Housekeeping

Every probe I ran was reverted. `crates/snapdown-core/Cargo.toml`, `Cargo.lock`, and
`web/ui/src/components/Modal.tsx` were each restored from a backup, and
`web/ui/src/test/zz_probe.test.tsx` was deleted. The standalone id-generator program lives in the
session scratchpad, outside the repository. `git status --short --untracked-files=all` is **empty**,
and the full gate suite was re-run green afterwards at `f1704c1`.

I edited no application code, no corpus file, committed nothing, pushed nothing, and did not launch
the desktop app.
