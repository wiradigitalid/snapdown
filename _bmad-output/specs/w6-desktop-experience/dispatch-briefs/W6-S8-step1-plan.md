# W6-S8 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S8`

## What is left here, and what already landed

`.how/bundle/01-ux/DESIGN.md` § `Bundles (LC-014)` says it directly, including what is already done:

> All three panels draw from `--color-surface` on `--color-bg` and take the height available. The
> shipped build painted `#f8fafc`, `#ffffff`, and `#e0f2fe` literals at a fixed height, which produced
> both the dark-theme contrast failure and roughly a third of the window left empty beneath. **The
> literals are gone as of `W6-S1`; the fixed height is not** — that is this surface's own work, in
> `W6-S8`.

So the colour half is done. This story owns the **layout** and the **states**.

```
┌ list 240px ──┐┌─ preview (flex) ───────────┐┌ items 280px ─┐
```

- **The preview is the centre and it grows, because it is the artifact.**
- Actions sit at the **item list's foot**, not floating over the preview. `Copy Markdown` is the one
  `primary` Button on the surface; `Publish` is `secondary`; `Delete` is `danger`.
- All three panels take the height available. The fixed height is what leaves a third of the window
  dark beneath them.

## The two things here that are principles, not decoration

### A read-only region, not a disabled input

```
vitest::the_preview_is_a_read_only_region_and_not_a_disabled_input
```

`DESIGN.md` gives the reason and it is worth carrying verbatim:

> a disabled control is announced as unavailable, and the content is **available** — it is the
> **editing** that does not exist (`FR-11`, Non-Goals).

A screen reader told "unavailable" about a Bundle's Markdown is being told something false. The
preview renders in `--font-mono` at `--text-sm` on `--color-surface-sunken`.

### An empty state that does not pretend

```
vitest::the_empty_state_offers_no_button_that_only_navigates_away
```

> One centred `EmptyState`: "No bundles yet", one sentence — "Select findings on the Findings tab and
> choose Compose." **No button, because the action lives on another surface and a button that
> navigates away pretends otherwise.**

`W6-S1` landed `EmptyState` in `@snapdown/ui`. Use it.

## Every state, and one of them is a defect class this wave keeps meeting

| State | Rendering |
|---|---|
| Empty | The centred `EmptyState` above, no button |
| Loading | Skeleton rows in the list; preview and item list hold their shape |
| Nothing selected | List populated; preview shows muted "Select a bundle". **Distinct from empty** |
| Populated | Normal |
| **Item file missing** | The affected item row carries a `--color-warning-bg` badge. **The preview and the actions still work** |
| Error | Centred `ErrorState`, one Retry |

```
vitest::an_item_whose_image_copy_is_missing_is_flagged_and_the_bundle_still_opens
```

That last one is the same shape as `W6-S9` and `W6-S10`: report what is wrong and keep working, rather
than failing the whole thing or pretending nothing happened. A missing image copy must not take the
Bundle down with it.

## The remaining two tests

```
vitest::bundles_renders_correctly_in_both_windows_themes
vitest::copy_markdown_announces_its_result
```

**Colour lives in exactly one file** — `web/ui/src/styles/tokens.css`, defined for both themes
(`AD-10`), and `W6-S1` landed a lint rule that refuses a literal anywhere else. The theme test must
assert behaviour, not a copy of the token values: `web/ui/src/test/contrast.test.ts` parses the token
file and was verified by mutation. Follow it.

`copy_markdown_announces_its_result` is an accessibility requirement, not a toast preference — a
silent success is indistinguishable from a silent failure.

## Boundaries

- `LC-031` `compose-bundle-dialog` is a **separate screen** in the same DESIGN document, and inventory
  row 9 named it while no build unit carried it. **It is not in this story's test list.** If the plan
  concludes it cannot be avoided, say so explicitly rather than absorbing it — a story that quietly
  grows a second screen is how a wave loses its shape.
- **`DEC-005` freezes `sharing`.** `Publish` is on this surface as a `secondary` Button and
  `LC-022 publish-dialog` is explicitly marked frozen. Render the button; do not touch the publish
  path.
- `W6-S3` landed the two-column Settings frame and `W6-S7` landed the Findings surface with its three
  regions. Both are the pattern for panels that take the height available.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **A green unit test does not mean the component is reachable.** Grep for `<ComponentName` across
  `apps/desktop/src` and `web/ui/src` before closing, excluding its own file and its tests. Four
  components in this repository once shipped mounted nowhere.
- **Verification is run, not assumed.** All of `AGENTS.md` § Code. Four traps are recorded there.
- **Write UTF-8, no BOM.** No scratch files in the commit, never a captured screenshot, and **do not
  push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S8-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
