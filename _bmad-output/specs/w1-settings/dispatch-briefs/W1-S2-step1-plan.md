STEP 1 of 5 — PLAN ONLY. Snapdown wave W1, story W1-S2.

You are in the worktree D:/Developer/orca-workspaces/snapdown/w1-settings, branch
kodesh87/w1-settings. W1-S1 is complete and committed: the Cargo workspace, the Tauri v2 shell, the
React webview, the tray, the shared UI library, and CI all exist. This story fills
`crates/snapdown-store`, which W1-S1 left as a skeleton on purpose.

WHAT TO DO
Invoke the skill `bmad-build-auto` under folder+id dispatch:
  spec_folder = _bmad-output/specs/w1-settings/
  story_id    = W1-S2
Resolve everything else from `_bmad-output/specs/w1-settings/stories.yaml` and
`_bmad-output/specs/w1-settings/SPEC.md`. Read SPEC.md in full first — it is the canonical contract,
and the files in its `companions:` frontmatter are part of it. SPEC.md § W1-S2 is this story's scope.

Halt after planning.

Produce the story spec at `_bmad-output/specs/w1-settings/stories/W1-S2-<slug>.md` with frontmatter
carrying `status: ready-for-dev`. Write NO application code in this step.

WHAT W1-S1 LEFT YOU, and the plan MUST account for all four

1. `crates/snapdown-core/src/ports/mod.rs` declares `Clock` (with `now_rfc3339` and
   `now_unix_millis`), `EntropySource`, `SettingsStore`, `BlobStore`, `HotkeyRegistrar`, and
   `StartupRegistrar`. This story implements `SettingsStore` and `BlobStore`, and it needs real
   `Clock` and `EntropySource` adapters — the core has neither a clock nor entropy of its own, by
   design, and `crates/snapdown-core/tests/test_no_io.rs` enforces that.
2. `crates/snapdown-core/src/util/id.rs` exposes `id_from_parts(unix_millis: u64, rand_b: [u8; 10])`.
   Every id this story mints goes through it, with the timestamp and the entropy supplied by the
   adapters. Nothing else generates an id.
3. **First run is currently wrong on purpose and this story closes it.**
   `apps/desktop/src-tauri/src/main.rs` opens Settings unconditionally with a comment naming the
   finding. The decision already taken: first run is "the `setting` table holds no rows". No new
   Setting key, no flag file, no corpus change. Implement that once the store exists.
4. `crates/snapdown-store` declares `uuid` and `chrono` in its manifest and uses neither yet. Use
   them or drop them — a declared-and-unused dependency was recorded as a follow-up against W1-S1 and
   this story is where it resolves.

TWO FOLLOW-UPS ROUTED TO THIS STORY, and the plan MUST cover both

- **F-3 from the panel.** The no-IO guard is a dependency-graph check, so `snapdown-core` could call
  `std::fs`, `std::env`, or `SystemTime::now()` directly and the test would stay green — `std` is not
  a graph node. This story starts writing real I/O next door, so the boundary begins to matter. Add a
  source-level deny: clippy `disallowed-methods` in a `clippy.toml` scoped to the core, or a CI check.
  The graph test stays; this is the half it structurally cannot cover.
- **F-7 from the panel.** `.github/workflows/desktop-ci.yml` uses `npm install`, not `npm ci`, so the
  committed lockfiles do not gate the build. Switch both installs to `npm ci`.

WHAT THIS STORY MUST NOT DO
- Only two tables: `setting` and `schema_version`, exactly as `.how/_platform/inventory-db.md` rows 8
  and 9 specify. The `finding`, `note`, `marker`, `bundle`, `bundle_item`, `publication`, and
  `access_key` tables belong to later waves and MUST NOT be created here.
- No Settings screen work — that is W1-S3. No hotkeys — W1-S4. No startup registration — W1-S5.
- No capture, no Editor, no MCP, no Go.

THE TWO RULES THAT DECIDE THIS STORY, from SPEC.md § W1-S2
- A corrupt or unreadable `library.db` MUST refuse to open and MUST NOT be replaced by a fresh empty
  one. Silently recreating it is data loss dressed as recovery.
- The Vault adapter MUST **resolve** a path and refuse anything that escapes the Vault root — resolve,
  never string-match. This is the single place that check lives for the whole product; W4's image
  route and W5's publish path both rely on it, and it is the one hostile input this product has.

EXIT CONDITION
The story spec exists and its frontmatter reads `status: ready-for-dev`. That frontmatter is what the
coordinator judges this step by — your chat report does not settle it.

RULES THAT BIND YOU
- The corpus is NOT yours to change. `.what/`, `.how/`, `.control/`, `.constitution/` are read-only.
  If the SPEC asks for something they do not support, report it as an upstream gap; do not invent it.
- Do NOT commit and do NOT push. The coordinator is the only hand that pushes.
- `blocked / spec failed ready-for-development standard` — name the failing criteria; do not
  hand-patch it into a pass.
- `blocked / intent gap` — state your unanswered questions verbatim. Do not guess.

WHEN DONE
Report with:
  orca orchestration send --type worker_done --subject "<status>" --body "<the spec path, its status
  frontmatter, and anything you could not resolve>" --task-id <task_id> --dispatch-id <dispatch_id>
  --outcome succeeded --files-modified "<paths>" --json
Use --outcome failed on failure. If Orca rejects the lifecycle message, say so — the coordinator reads
the artifact.
