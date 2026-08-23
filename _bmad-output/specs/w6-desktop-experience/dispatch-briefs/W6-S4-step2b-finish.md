# W6-S4 · Step 2 (continued) — FINISH IT

A previous worker built most of this story and stopped without completing it. Its spec frontmatter
still reads `status: ready-for-dev`. **Finish it; do not start it over.**

Read `AGENTS.md` first. Spec:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S4-the-quality-budget-as-a-named-intent-with-auto-deriving-per-capture.md`

## Already done and verified — do not redo or "improve" it

`cargo test --workspace` exits **0**, 100 passing. Migration v7 lands, `NamedBudget` carries
`Auto`/`Sharp`/`Balanced`/`Small`/`Custom`, and `QualityBudget::resolve` derives the pair from the
captured region.

**`SCN-03`'s assertion exists and is real** — a `312 × 118` tooltip and a `3840 × 2160` dashboard
resolve different pairs, and the small un-downscaled region gets the higher encoder quality. That is
the one assertion a constant cannot pass, and it passes. **Leave `finding.rs` and `setting.rs` alone.**

Five of the eight named tests exist:

```
domain::finding::tests::auto_resolves_a_different_pair_for_a_small_region_than_for_a_full_screen   OK
domain::finding::tests::auto_resolves_a_higher_encoder_quality_when_no_downscale_applies           OK
every_stored_finding_carries_the_pair_that_was_applied_to_it                                       OK
quality_budget.test.tsx > a_reviewer_who_never_opens_advanced_never_sees_a_raw_number               present
quality_budget.test.tsx > editing_an_advanced_value_moves_the_control_to_custom_visibly             present
```

## What is left — exactly this, and nothing else

### 1. The webview does not typecheck

`npm --prefix apps/desktop run typecheck` exits **2**:

```
src/components/QualityBudgetSection.tsx(2,10): TS6133: 'Button' is declared but its value is never read.
src/test/findings_view.test.tsx(75,7):        TS2741: Property 'named' is missing in type
                                                      '{ max_long_edge; encoder_quality }'
src/test/settings_layout.test.tsx(132,62):    TS2345: same
src/test/settings_layout.test.tsx(231,9):     TS2322: same
```

Three test files still build the **old** `QualityBudget` shape. Give them the new one — `named` plus
whatever the type now requires. **Do not widen the type to make the old fixtures compile**; the type is
correct and the fixtures are stale.

Two test files are failing as a result: `npm --prefix apps/desktop run test` exits 1 with 2 failed of 45.

### 2. Three named tests do not exist

```
cargo::the_named_state_and_its_resolved_pair_are_written_together
cargo::an_advanced_value_outside_its_range_is_refused_and_does_not_enter_custom
vitest::the_readout_names_the_budget_that_produced_the_latest_finding
```

- **`the_named_state_and_its_resolved_pair_are_written_together`** — `BR-116`. One atomic write. A test
  that writes the name and the pair separately and finds both present proves nothing about atomicity;
  assert that a failure between them cannot leave one without the other.
- **`an_advanced_value_outside_its_range_is_refused_and_does_not_enter_custom`** — the refusal must
  happen **before** the state transition. A rejected value that still moved the control to `Custom`
  has told the Reviewer something untrue.
- **`the_readout_names_the_budget_that_produced_the_latest_finding`** — `DESIGN.md`: *show which budget
  produced the latest capture, not just its size.* `Latest: 184 KB · 1408 px · Auto`.

## The rule that matters most here

**A test that cannot fail is a review finding.** The four old tests that asserted the `1600` constant
were replaced; make sure what replaced them still fails when the behaviour regresses, and that
`Balanced` still pins `(1600, 75)` as a stated promise rather than as whatever the code returns.

Assert behaviour, not a copy of the input. `web/ui/src/test/contrast.test.ts` is the pattern and was
verified by mutation; `apps/desktop/src/test/settings_layout.test.tsx` follows it too.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-` —
  and `DEC-004` is applied.
- **Verification is run, not assumed.** Both halves of `AGENTS.md` § Code. Four traps are recorded
  there; the newest is that `cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd` did.
- **Write UTF-8, no BOM, keep trailing newlines.** No scratch files in the commit. **Do not push.**
- **Set the frontmatter to `status: done` when you are finished.** This is why the previous attempt
  was judged incomplete.

## Done means

`cargo test --workspace`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `npm --prefix web/ui run typecheck && lint &&
test`, and `npm --prefix apps/desktop run typecheck && lint && test && build` all exit **0**, all
eight named tests execute, and the spec's frontmatter reads `status: done`.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
