# W6-S4 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S4`

## What is being built, and why the old shape failed

`DEC-004` is **applied** and is the contract. Quote it; do not re-derive it.

> The Reviewer chooses a Quality Budget by naming what they want — **Auto**, **Sharp**, **Balanced**,
> or **Small** — and Auto is the shipped default. Auto derives the maximum long edge and the encoder
> quality **from the captured region itself** rather than from a stored constant. The raw numbers
> remain settable behind an **Advanced** disclosure, and setting either one moves the budget to a
> fifth named state, **Custom**.

Its Why matters as much as its Decision:

> `FR-5` already promised that "both values have defaults the Reviewer never has to change to get a
> usable result." What shipped does not keep that promise: two numbered fields, `1600` and `75`, with
> no unit the Reviewer can reason about and no way to tell whether either is right. The owner named
> the shape they expected — TinyPNG, which asks nothing and returns a file that looks the same and
> weighs less. **The promise was correct; the presentation defeated it.**

`AUDIT-4` photographed the shipped state on 2026-08-24: two bare numeric boxes, `Max Long Edge (px)`
`1600` and `Encoder Quality (10-100)` `75`. That is what this story removes from the Reviewer's way.

## `SCN-03` is the only assertion a constant cannot pass

`.what/finding/05-scenarios/SCN-03-the-quality-budget-that-resolves-differently.md`. Read it in full.
It exists because without it **"Auto" can ship as the old constant wearing a new label** and every
other consequence of `FR-5` would still pass.

| Capture | Size | What Auto must do |
|---|---|---|
| **A** — a tooltip | `312 × 118` | **No downscale at all.** Every plausible cap is above 312, so the cap does nothing and the encoder decides everything. Auto resolves a **high** encoder quality, because 11 px text is exactly where lossy artefacts show and there are no pixels to spare |
| **B** — a 4K dashboard | `3840 × 2160` | **Downscaled hard.** The cap decides almost everything and the encoder barely matters, so Auto resolves a **lower** encoder quality — the downscale already removed the detail quality would have preserved |

```
assert resolved(A) != resolved(B)
```

A test that captures both and finds identical parameters is a **failing test**.

## The four existing tests that encode the old constant

Found by sweep on 2026-08-23:

```
crates/snapdown-core/src/domain/image.rs:92-93          assert 4K reduces to 1600x900
crates/snapdown-core/src/domain/setting.rs:171          assert the default is 1600
crates/snapdown-store/src/image/pipeline.rs:92          assert width == 1600
crates/snapdown-store/tests/test_image_reduction.rs:16  assert width == 1600
```

**They are not broken tests today.** Each fails if the constant changes, which is exactly what a test
should do. The trap is what happens when `Auto` removes the constant.

**Do not re-point them at whatever number `Auto` happens to produce.** That turns each one into a
tautology — assert-what-the-code-does, which passes forever and proves nothing. Replace them with
`SCN-03`'s shape, and **keep a real assertion on the fixed budgets** (`Sharp`, `Balanced`, `Small`),
where a stated number is still a promise worth pinning.

This wave has already caught this exact failure once: `W6-S1`'s contrast test asserted a hardcoded
copy of the token values and passed whatever `tokens.css` said. It now parses the token file and was
verified by mutation.

## Migrations — settled, do not re-derive

Highest existing version is **v6** (`bundle_item` without the `finding_id` foreign key, landed by
`W6-S9`). **This story takes v7.** It carries `quality_budget.named` and the three columns `NFR-18`
needs on `finding`, so that every stored Finding carries the pair that was applied to it.

## The two tests that are about the Reviewer, not the pipeline

```
vitest::a_reviewer_who_never_opens_advanced_never_sees_a_raw_number
vitest::editing_an_advanced_value_moves_the_control_to_custom_visibly
```

The first is the whole point of `DEC-004`. If a raw number is still on screen for someone who never
opened Advanced, the presentation still defeats the promise and the story has not landed.

The second is the honesty rule: a control that silently stays on `Balanced` while holding hand-edited
numbers is claiming a state it is not in. Same principle as `BR-20` in `W6-S9` and the Vault report in
`W6-S10` — never claim a state you have not achieved.

## Boundaries

- The Settings **frame** — two columns packed by content height — is `W6-S3`. This story owns what is
  **inside** the Quality Budget group.
- `.how/settings/01-ux/DESIGN.md` § `Quality Budget group` has the layout and the readout wording.
  Use it; do not invent a different presentation.
- `DEC-005` freezes `sharing` and `agent-access`. Nothing here should reach them.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-` —
  and `DEC-004` is applied. A deviation is **reported** and becomes a new `DEC-`.
- **Verification is run, not assumed.** All of `AGENTS.md` § Code. Four traps are recorded there.
- **A test that cannot fail is a review finding**, and this story is the most likely place in the
  whole wave to write one.
- **Write UTF-8, no BOM.** No scratch files in the commit, never a captured screenshot, and **do not
  push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S4-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
