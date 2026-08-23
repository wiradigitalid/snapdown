# W6-S8 · Step 2 — BUILD

The plan is done and approved. Implement it. **This is the last story in wave W6.**

Read `AGENTS.md` first. Run `bmad-build-auto` with the spec path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S8-bundles-preview-as-the-centre-and-a-read-only-region-not-a-disabled-input.md`

The spec is complete and its `<intent-contract>` is the owner's. **Do not edit anything inside it.**
The three-panel layout, every state, and the five tests are already written there. This step ends when
its frontmatter reads `status: done`.

## What already landed, and what is left

`W6-S1` removed the colour literals — `#f8fafc`, `#ffffff`, `#e0f2fe` — that produced the dark-theme
contrast failure. **The fixed height is still there**, and it is what leaves roughly a third of the
window dark beneath the panels. That is this story's work.

```
┌ list 240px ──┐┌─ preview (flex) ───────────┐┌ items 280px ─┐
```

**The preview is the centre and it grows, because it is the artifact.** Actions sit at the item list's
foot, not floating over the preview: `Copy Markdown` is the one `primary` Button on the surface,
`Publish` is `secondary`, `Delete` is `danger`. All three panels take the height available.

## The two rules here that are principles, not decoration

**A read-only region, not a disabled input.** `DESIGN.md`'s reason, verbatim:

> a disabled control is announced as unavailable, and the content is **available** — it is the
> **editing** that does not exist (`FR-11`, Non-Goals).

A screen reader told "unavailable" about a Bundle's Markdown is being told something false.

**An empty state that does not pretend.** One centred `EmptyState`: "No bundles yet", one sentence —
"Select findings on the Findings tab and choose Compose." **No button**, because the action lives on
another surface and a button that navigates away pretends otherwise. `W6-S1` landed `EmptyState`; use it.

## The state this wave keeps meeting

An item whose image copy is missing gets a `--color-warning-bg` badge on its row, **and the preview and
the actions still work**. Same shape as `W6-S9` and `W6-S10`: report what is wrong and keep working,
rather than failing the whole thing or pretending nothing happened. A missing image must not take the
Bundle down with it.

`Nothing selected` is **distinct from** `Empty` — list populated, preview shows a muted "Select a
bundle".

## Boundaries

- `LC-031` `compose-bundle-dialog` is a **separate screen** and is not in this story's tests. If it
  cannot be avoided, report that rather than absorbing it.
- **`DEC-005` freezes `sharing`.** Render the `Publish` button; do not touch the publish path.
- **Colour lives in exactly one file** — `web/ui/src/styles/tokens.css`, both themes — and a lint rule
  refuses a literal anywhere else. The theme test must assert behaviour, not a copy of the token
  values; `web/ui/src/test/contrast.test.ts` is the pattern and was verified by mutation.
- `copy_markdown_announces_its_result` is accessibility, not a toast preference: a silent success is
  indistinguishable from a silent failure.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **A green unit test does not mean the component is reachable.** Grep for `<ComponentName` across
  `apps/desktop/src` and `web/ui/src` before closing, excluding its own file and its tests.
- **A test that cannot fail is a review finding.**
- **Verification is run, not assumed.** Both halves of `AGENTS.md` § Code.
- **Write UTF-8 with NO BOM.** A BOM makes the frontmatter parser report the story as having no status
  at all; it happened three times this wave.
- No scratch files in the commit, never a captured screenshot, and **do not push.**
- **Set the frontmatter to `status: done` when you are finished.**

## Done means

`cargo test --workspace`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and the `web/ui` and `apps/desktop` scripts
all exit **0**, the five named tests execute, and the spec's frontmatter reads `status: done`.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
