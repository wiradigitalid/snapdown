# W6-S6 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S6`

## What is already right, and what is not

`AUDIT-4` photographed this group on 2026-08-24, in the first properly built binary. Two findings, and
the first one is good news:

- **The keystroke recorder exists and says what to do** — `Click to record`. The owner asked to set
  hotkeys by pressing keys rather than typing a string, and that interaction is there.
- **`Capture Region` reads `Disabled` and is bound to nothing.** On a fresh profile the capture hotkey
  does nothing at all, so the capture path — the reason the product exists — is unreachable by
  keyboard until the Reviewer finds the Settings panel and records a combination. `FR-1` and `UC-15`
  assume a hotkey is there to press.

So the interaction is right and the **default is missing**. Say in the plan what the shipped default
is and where it comes from.

## The four chip states, from `.how/settings/01-ux/DESIGN.md` § Hotkeys group

One row per action: label, `HotkeyChip`, `Badge`, and a clear affordance.

| Chip state | Rendering |
|---|---|
| bound | `--color-surface-sunken`, `--font-mono`, `--radius-full`, the combination in text |
| listening | `--color-info-bg` / `--color-info-text`, a `2px` `--color-accent` ring, reading "Press keys… Esc to cancel" |
| unbound | Dashed `--color-border-strong`, reading "Click to set" |
| conflicted | `--color-warning-bg` / `--color-warning-text`, with the conflict named on the line beneath |

`W6-S1` landed `HotkeyChip` and `Badge` in `@snapdown/ui`. Use them; do not invent a second chip.

## The five tests, and what each is really about

```
vitest::a_listening_chip_stops_listening_when_focus_leaves_it
```

A chip left listening after focus moves away swallows the next keystroke the Reviewer meant for
something else. This is a leak, not a cosmetic state.

```
vitest::a_snapdown_internal_conflict_is_worded_differently_from_an_os_conflict
```

Two different problems with two different remedies. *Another Snapdown action already uses this* is
something the Reviewer can fix here, in this panel. *Windows already uses this* is not — they must
choose a different combination. Wording them identically tells the Reviewer to look in the wrong
place.

```
vitest::a_cleared_hotkey_reads_disabled_rather_than_empty
```

An empty chip is ambiguous between *not set* and *broken*. `Disabled` is a state the Reviewer chose.

```
vitest::a_startup_registration_failure_carries_a_badge_before_the_reviewer_acts
```

`DESIGN.md`'s own rule for this surface: **put a failure under the control that failed.** A hotkey
that could not be registered at startup must say so before the Reviewer clicks it and discovers it.

```
vitest::the_active_and_disabled_badges_carry_a_word_not_only_a_colour
```

**This is the accessibility floor, not a preference.** A badge distinguished only by colour is
unreadable to anyone who cannot separate those two colours, and `EXPERIENCE.md` sets the same floor
for Markers. `DESIGN.md` says the badge reads **Active** or **Disabled** in words.

Note what that line also records: the badge previously used literal `#dcfce7` on `#166534` — a
light-theme pair rendered inside a dark shell. **The literals are gone as of `W6-S1`**; the words are
this story's work.

## Boundaries

- The Settings **frame** landed in `W6-S3` — two columns packed by content height. This group sits in
  column B beneath Quality Budget. Do not re-lay-out the panel.
- The **startup toggle's** behaviour is `W6-S5`, and the Quality Budget's contents are `W6-S4`.
- **Colour lives in exactly one file**: `web/ui/src/styles/tokens.css`, both themes (`AD-10`), and a
  lint rule refuses a literal anywhere else.
- `BR-26` is named in `rules-settings.md` as a rule that looks local and is not: `finding` owns the
  capture action the hotkey triggers, and re-registration crosses both components. A hotkey change
  takes effect **without a restart**. Read that rule before planning the re-registration path.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **A green unit test does not mean the component is reachable.** Grep for `<ComponentName` across
  `apps/desktop/src` and `web/ui/src` before closing, excluding its own file and its tests.
- **A test that cannot fail is a review finding.** Assert the behaviour, not a copy of the input —
  `web/ui/src/test/contrast.test.ts` is the pattern and was verified by mutation.
- **Verification is run, not assumed.** All of `AGENTS.md` § Code.
- **Write UTF-8, no BOM.** No scratch files in the commit, never a captured screenshot, and **do not
  push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S6-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
