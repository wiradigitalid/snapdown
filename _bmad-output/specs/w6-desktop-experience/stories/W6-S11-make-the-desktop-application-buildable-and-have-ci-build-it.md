---
id: W6-S11
title: 'W6-S11: Make the desktop application buildable, and have CI build it'
type: 'chore'
wave: W6
status: done
created: '2026-08-23'
review_loop_iteration: 0
followup_review_recommended: false
dependencies:
  - W6-S7
files:
  - apps/desktop/package.json
  - apps/desktop/package-lock.json
  - .constitution/project/codebase-stack-guide.md
  - .github/workflows/desktop-ci.yml
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S11-step1-plan.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** 
The repository currently provides no reproducible method to build the bundled desktop application (`BUG-11`). Running `cargo build --release -p snapdown` only compiles the Rust executable, which at runtime requests `devUrl` (`http://localhost:5173`) from `tauri.conf.json`, finds no running dev server, and fails with `ERR_CONNECTION_REFUSED`. The Tauri CLI required to build the frontend and bundle it into `frontendDist: "../dist"` is absent across the repository (no npm dependency, no script, no cargo install), `desktop-ci.yml` never builds or tests the bundled application, and `codebase-stack-guide.md` documents no desktop build command. Consequently, `BR-121` (*a build produces exactly one desktop executable*) is only asserted statically against manifest files, and no UI claim in Wave W6 can be verified in a real running binary.

**Approach:**
1. **Make Tauri CLI installable via npm devDependency:** Add `@tauri-apps/cli` pinned to `^2.0.0` (matching Tauri v2 runtime libraries in `Cargo.toml` and `@tauri-apps/api` in `apps/desktop/package.json`) under `devDependencies` in `apps/desktop/package.json`, and add `"tauri": "tauri"` and `"build:app": "tauri build"` (or `npm run tauri build`) to `scripts`. Choosing `@tauri-apps/cli` over `cargo install tauri-cli` ensures zero additional Rust crate compilation overhead during npm setup, leverages the existing Node/npm CI environment, and locks deterministically via `package-lock.json`.
2. **Document Desktop Build Commands in `codebase-stack-guide.md`:** Update `.constitution/project/codebase-stack-guide.md` under Section 3 (Verification Commands) to explicitly document the desktop build command (`npm --prefix apps/desktop run tauri build` or `npx --prefix apps/desktop tauri build`) and dev execution (`npm --prefix apps/desktop run tauri dev`), placed prominently alongside existing workspace verification commands.
3. **Add Desktop Bundle Build & Verification Step to CI:** In `.github/workflows/desktop-ci.yml`, add a dedicated step in `web-check` (or a dedicated job having both Node and Rust toolchains present) that executes `npm --prefix apps/desktop run tauri build -- --no-bundle` (or full build) to build the release desktop binary. Assert that the build produces the executable `apps/desktop/src-tauri/target/release/Snapdown.exe` (or `target/release/Snapdown.exe`).
4. **Decide CI Artifact Handling:** Keep CI fast and deterministic by building and verifying the existence of the desktop binary without uploading GitHub Actions release artifacts for standard PR/push checks (avoiding storage bloat and unnecessary upload latency, while providing full verification of build reproducibility and single-binary invariant `BR-121`).
5. **Plan the Manual Product Verification Check:** Explicitly specify the manual verification procedure (`manual::a_freshly_built_binary_loads_its_bundled_frontend_not_devurl`) to be executed by launching the built release binary with no Vite dev server running and confirming it mounts the bundled frontend without network errors.

## Boundaries & Constraints

**Always:**
- Pin `@tauri-apps/cli` to major version 2 (`^2.0.0`) matching the Tauri v2 crates in `Cargo.toml` (`tauri = "2.0"`) and `@tauri-apps/api: "^2.0.0"`.
- Update `apps/desktop/package-lock.json` via clean npm install so `npm ci` remains strictly reproducible.
- Document the build command in `.constitution/project/codebase-stack-guide.md` (the product's authorized stack guide in `.constitution/project/`).
- Ensure CI (`.github/workflows/desktop-ci.yml`) runs the real Tauri build and verifies that the output binary `Snapdown.exe` exists and is the sole desktop executable (`AD-11`, `BR-121`).
- The manual verification `manual::a_freshly_built_binary_loads_its_bundled_frontend_not_devurl` MUST NOT be faked as an automated headless test in CI.

**Block If:**
- Tauri CLI version resolution requires upgrading or breaking compatibility with existing Tauri v2 Rust plugins (`single-instance`, `global-shortcut`, `autostart`).
- CI environment lacks required Windows SDK or webview dependencies preventing Tauri release compilation.

**Never:**
- Do not modify corpus documents in `.what/`, `.how/`, or `.constitution/method/` (`codebase-stack-guide.md` in `.constitution/project/` is explicitly permitted).
- Do not leave `@tauri-apps/cli` unpinned or floating to a different major version.
- Do not commit built binary artifacts, `.exe` files, or captured test screenshots to the git repository (`BUG-7`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| NPM Dependency Install | `npm --prefix apps/desktop ci` | Installs `@tauri-apps/cli` v2 into `apps/desktop/node_modules/.bin/tauri` | Clean exit code 0 |
| Tauri CLI Version Check | `npx --prefix apps/desktop tauri --version` or `npm --prefix apps/desktop run tauri -- --version` | Outputs Tauri CLI v2.x version string matching Tauri v2 ecosystem | Non-zero exit code if CLI binary missing |
| Desktop App Build (Local) | `npm --prefix apps/desktop run tauri build` | Runs `beforeBuildCommand` (`npm run build`), compiles Rust binary with embedded assets into `target/release/Snapdown.exe` | Fails fast with clear compiler/bundler error message |
| CI Build Step | GitHub Actions runner executes Tauri build in `desktop-ci.yml` | `apps/desktop` compiles release binary; step asserts `target/release/Snapdown.exe` exists | Fails workflow if binary not generated |
| Standalone Launch (Manual Check) | Launch `Snapdown.exe` directly with no Vite server running on `http://localhost:5173` | Window opens with title `Snapdown Editor`, renders navigation rail and Editor shell; no `ERR_CONNECTION_REFUSED` | Manual inspection protocol |

</intent-contract>

## Code Map

- `apps/desktop/package.json` -- Desktop container manifest; add `@tauri-apps/cli` to `devDependencies`, add `"tauri": "tauri"` script.
- `apps/desktop/package-lock.json` -- Lockfile tracking `@tauri-apps/cli` dependency graph for deterministic `npm ci`.
- `.constitution/project/codebase-stack-guide.md` -- Project stack guide; document `npm --prefix apps/desktop run tauri build` and `tauri dev` in Section 3 Verification Commands.
- `.github/workflows/desktop-ci.yml` -- CI workflow; add bundle build step in desktop CI workflow asserting `Snapdown.exe` generation.

## Tasks & Acceptance

**Execution:**
- `apps/desktop/package.json` -- Add `@tauri-apps/cli: "^2.0.0"` to `devDependencies` and `"tauri": "tauri"` to `scripts` -- Makes Tauri CLI reproducible and invocable from the repository.
- `apps/desktop/package-lock.json` -- Regenerate lockfile via `npm install` in `apps/desktop` -- Guarantees clean deterministic installs via `npm ci`.
- `.constitution/project/codebase-stack-guide.md` -- Add desktop application build and dev commands under Section 3 Verification Commands -- Makes build instructions discoverable in the product stack guide.
- `.github/workflows/desktop-ci.yml` -- Add Tauri build step to CI workflow that runs the desktop build and asserts single release binary `Snapdown.exe` creation -- Validates `ci::desktop_ci_builds_a_bundled_desktop_artifact` and `ci::the_build_produces_exactly_one_desktop_executable_from_a_real_build` against real builds.

**Acceptance Criteria:**
- Given `apps/desktop/package.json`, `@tauri-apps/cli` is listed in `devDependencies` pinned to `^2.0.0`, and `"tauri"` script is present.
- Given `.constitution/project/codebase-stack-guide.md`, Section 3 contains the exact commands to build and run the desktop application using Tauri CLI.
- Given `.github/workflows/desktop-ci.yml`, the workflow contains a step that builds the desktop application and verifies the existence of `Snapdown.exe`.
- Given manual verification `manual::a_freshly_built_binary_loads_its_bundled_frontend_not_devurl`, the protocol defines launching `Snapdown.exe` without a dev server to confirm it loads embedded assets.

## Spec Change Log

<!-- Append-only. Populated during review loops. -->

## Design Notes

**Why `@tauri-apps/cli` in `package.json` over `cargo install tauri-cli`:**
1. Avoids compiling the CLI from source on developer machines and CI runners (which takes several minutes for `tauri-cli` crate compilation).
2. Uses existing Node.js toolchain and `package-lock.json` pinning mechanism.
3. Aligns with standard Tauri project layout where frontend and Tauri CLI share script entry points.

## Verification

**Commands:**
- `npx --prefix apps/desktop tauri --version` -- expected: Tauri CLI 2.x version printed
- `npm --prefix apps/desktop run tauri build` -- expected: Successful release build producing `target/release/Snapdown.exe` (or `apps/desktop/src-tauri/target/release/Snapdown.exe`)
- `npm --prefix web/ui run typecheck` -- expected: Clean typecheck
- `npm --prefix apps/desktop run typecheck` -- expected: Clean typecheck
- `cargo test --workspace` -- expected: All unit/integration tests pass
- `uv run .constitution/method/scripts/validate.py --check` -- expected: Corpus validation passes

**Manual checks (if no CLI):**
- Execute `manual::a_freshly_built_binary_loads_its_bundled_frontend_not_devurl`: Launch the built `Snapdown.exe` directly on Windows with port 5173 closed. Inspect that the window displays the "Snapdown Editor" title and loads the React UI rather than `ERR_CONNECTION_REFUSED`.
