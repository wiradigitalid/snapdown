# W6-S11 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S11`

## Why this story is fourth, ahead of six stories with more visible value

**Until it lands, nothing in this wave can be verified in the product.**

Every remaining story writes UI. The only instrument for checking UI is a running application, and
this repository currently has no reproducible way to produce one. `W6-S2`'s fix for `BUG-4` — the
capture path — is still unproven outside jsdom for exactly this reason.

## `BUG-11`, and what is already known about it

`cargo build --release -p snapdown` compiles and links, and **the binary it produces is not the
application.** Launched, its webview requests the `devUrl` from `tauri.conf.json`
(`http://localhost:5173`), nothing serves it, and the window shows `ERR_CONNECTION_REFUSED`.
Photographed on 2026-08-23; the screenshot is not committed, the finding is in
`.work/ux-audit/AUDIT-3.md`.

The bundled `frontendDist` is only wired in by the **Tauri CLI**, and the CLI is absent everywhere:

- `cargo tauri --version` → *no such command*
- `npx --no-install tauri --version` → *could not determine executable*
- `apps/desktop/package.json` — no tauri package, no `tauri` script
- `Cargo.toml` — the tauri **libraries** and three plugins, no `tauri-cli`
- `codebase-stack-guide.md`, `README.md`, every workflow — no build command
- `desktop-ci.yml` — no bundle step, no artifact, no release

**The command itself is not a mystery, and this is the part to carry.** A `W1` reviewer diagnosed
this exact failure and raised it as a must-fix:

> *"`vite build` fails, therefore `cargo tauri build` fails, therefore `frontendDist: "../dist"` is
> never produced. `cargo tauri dev` serves `http://localhost:5173` with no document."*
> — `_bmad-output/specs/w1-settings/review-W1-S1-reviewer-a.md:46`

So the command is `cargo tauri build`. What never happened is making the CLI installable from the
repository, and carrying that command anywhere a reader would look.

**And one more thing W1 recorded that nobody carried out.** Its UI verification brief said:

> *"HOW TO RUN IT: `cargo tauri dev` from `apps/desktop`, or build and run `target/debug/desktop.exe`
> **with the Vite dev server up**."*
> — `_bmad-output/specs/w1-settings/dispatch-briefs/W1-S1-ui-verification-finish.md:65`

**Every UI claim this project has ever made was made against a dev server.** A bundled binary has
never been verified to work by anyone.

## What the plan must cover

1. **Make the CLI installable from the repository.** Either `@tauri-apps/cli` as a devDependency of
   `apps/desktop` with a `tauri` script, or a pinned `cargo install tauri-cli`. Pick one, and say why
   in the plan — the choice affects whether CI needs a Rust install step or a node one.
   Whichever you pick, **pin the version against the `tauri = "2.0"` libraries in `Cargo.toml`.** A
   CLI on a different major than the runtime is its own class of confusion.
2. **Carry the build command into `codebase-stack-guide.md`**, beside the verification commands,
   where a reader will find it. Not into a dispatch brief. Not into a review file. That is precisely
   how it got lost.
3. **Have `desktop-ci.yml` build the bundle.** This is what makes `BR-121` — *a build produces
   exactly one desktop executable* — checkable against a real build rather than a declaration. Today
   it is asserted by a unit test that reads `Cargo.toml` and `tauri.conf.json`, which is the right
   proxy for the declaration and proves nothing about a build.
4. **Decide what CI does with the artifact.** Building and discarding still proves the build works,
   and is the cheap answer. Uploading it makes a tester's life easier and costs storage. Say which
   and why; do not leave it implicit.

## The check that cannot be automated, and must not pretend to be

`waves.yaml` records this story's third test as
`manual::a_freshly_built_binary_loads_its_bundled_frontend_not_devurl`, and the `manual::` prefix is
deliberate. Proving a built binary loads its own frontend means launching it and looking. This
project has no reliable way to do that from CI — `OQ-24` records three dispatched UI verifications
producing three different failures in one day.

**Do not plan an automated test with that name.** It would claim an instrument that does not exist.
Plan the manual check, say who runs it and what they look for.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing build whose cause is unknown → run
  `wdi-systematic-debugging` before proposing any fix.
- **The corpus is not yours to change**, with one exception this story is explicitly granted:
  `codebase-stack-guide.md` lives in `.constitution/project/`, which is this product's own room, and
  item 2 above is the point of the story. Nothing in `.what/` or `.how/` may be touched.
- **Verification is run, not assumed.** And note two traps caught on 2026-08-23:
  `cmd | tail` reports `tail`'s exit code, not `cmd`'s — a `cargo build` that failed with *package ID
  specification did not match any packages* was read as success because of it. And a long-lived
  worktree goes stale the moment a story adds a dependency; run `npm ci` before believing a local red.
- **Never commit a captured screenshot**, and no scratch files in the commit.
- **Do not push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S11-*.md` exists, carries an
`<intent-contract>`, and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
