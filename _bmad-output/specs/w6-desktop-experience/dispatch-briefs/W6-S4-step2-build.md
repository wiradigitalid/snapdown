# W6-S4 · Step 2 — BUILD

The plan is done and approved. Implement it.

Read `AGENTS.md` first. Run `bmad-build-auto` with the spec path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S4-the-quality-budget-as-a-named-intent-with-auto-deriving-per-capture.md`

The spec is complete and its `<intent-contract>` is the owner's. **Do not edit anything inside it.**
The migration, the `NamedBudget` enum, the fixed preset pairs, the `Auto` resolution table, the
command signatures and the eight tests are all already written there. This step ends when its
frontmatter reads `status: done`.

## The one assertion this whole story exists for

```
assert resolved(A) != resolved(B)
```

`SCN-03`. A tooltip at `312 × 118` and a 4K dashboard at `3840 × 2160` must resolve **different**
pairs. Without it, `Auto` can ship as the old constant wearing a new label and every other consequence
of `FR-5` still passes. A test that captures both and finds identical parameters is a **failing test**.

## Do not turn the four existing tests into tautologies

```
crates/snapdown-core/src/domain/image.rs:92-93
crates/snapdown-core/src/domain/setting.rs:171
crates/snapdown-store/src/image/pipeline.rs:92
crates/snapdown-store/tests/test_image_reduction.rs:16
```

They assert the old `1600` constant. They are **not broken today** — each fails if the constant
changes, which is what a test should do. The cheapest move available is to re-point them at whatever
`Auto` produces, and that turns each one into assert-what-the-code-does: passes forever, proves
nothing.

The plan already says what to do instead: replace them with `SCN-03`'s variance shape, and **keep real
assertions on the fixed presets** — `Sharp`, `Balanced`, `Small` — where a stated number is still a
promise worth pinning. `Balanced` is `(1600, 75)`, so the old number survives where it belongs.

This wave has already caught this exact failure once: `W6-S1`'s contrast test asserted a hardcoded copy
of the token values and passed whatever `tokens.css` said. It now parses the token file and was
verified by mutation.

## The test that decides whether `DEC-004` actually landed

```
vitest::a_reviewer_who_never_opens_advanced_never_sees_a_raw_number
```

`DEC-004`'s Why is that the promise was correct and **the presentation defeated it** — two bare numeric
boxes, `1600` and `75`, with no unit the Reviewer could reason about. `AUDIT-4` photographed exactly
that on 2026-08-24. If a raw number is still on screen for someone who never opened Advanced, this
story has not landed however green the rest is.

```
vitest::editing_an_advanced_value_moves_the_control_to_custom_visibly
```

The honesty rule. A control sitting on `Balanced` while holding hand-edited numbers is claiming a
state it is not in — the same principle as `BR-20` in `W6-S9` and the Vault report in `W6-S10`.

## Migration numbering — settled

v6 landed with `W6-S9`. **This story takes v7.** Existing migrations v1–v6 are immutable.

## Boundaries

- The Settings **frame** landed in `W6-S3` — two columns packed by content height, with
  `--settings-group-gap`, `--settings-column-min` and `--settings-row-height` in `tokens.css`. This
  group sits in column B. **Do not re-lay-out the panel.**
- The startup toggle is `W6-S5`, the hotkey rows are `W6-S6`.
- **Colour lives in exactly one file** and a lint rule refuses a literal anywhere else.
- `AppState` is a plain struct and `W6-S9` extracted `_impl` functions taking `&AppState` from the
  Tauri commands. That is the pattern for anything needing a test — reaching for `tauri::test`
  produces a binary that cannot start at all, which cost that story two attempts.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-` —
  and `DEC-004` is applied.
- **Verification is run, not assumed.** All of `AGENTS.md` § Code, both halves — this story touches
  Rust and the webview. Four traps are recorded there; the newest is that `cmd; echo "EXIT=$?"` makes
  the harness report 0 whatever `cmd` did.
- **A green unit test does not mean the component is reachable.** Grep before closing.
- **Write UTF-8, no BOM, keep trailing newlines.** No scratch files in the commit — `W6-S9` left a
  `test_ro.rs` at the repo root. **Do not push.**
- **Set the frontmatter to `status: done` when you are finished.**

## Done means

`cargo test --workspace`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and the `web/ui` and `apps/desktop` scripts
all exit **0**, the eight named tests execute, and the spec's frontmatter reads `status: done`.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
