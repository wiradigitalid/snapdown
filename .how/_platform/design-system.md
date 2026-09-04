---
type: design-system
scope: _platform
status: draft
created: "2026-08-23"
updated: "2026-09-04"
---

# Design System — Snapdown

## Living Specification & Interactive Catalog
The complete visual catalog and interactive design system artifact is permanently housed in the `.how/` platform layer:
👉 [`.how/_platform/assets/design-system.html`](assets/design-system.html)
👉 Complete Flow Machine: [`.how/_platform/assets/ui-ux-complete-flow.html`](assets/ui-ux-complete-flow.html)

## Where the values actually live

| Source of truth | Path | Holds |
|---|---|---|
| Token file | `apps/desktop/ui/theme.slint` | Every colour, in both themes. The **only** place a literal colour may appear |
| Surfaces | `apps/desktop/ui/appwindow.slint` | Every screen and every component of the desktop app, composing from the tokens above |

**All three rows this table used to hold were retired on 2026-09-01, and the table is smaller because
the product is.** They named a token stylesheet and a shared package inside `web/ui`, deleted under
`OQ-27`, and a desktop reset under `apps/desktop/src/styles/` that `DEC-007` retired when the app left
React for Slint. Note the shape of the failure rather than the individual paths: this table's whole job
is to say where the values live, and every row in it was wrong for weeks. Nothing in the tooling
noticed — `V24` never checks an `apps/` path, because its pattern only matches a handful of top-level
prefixes and `apps` is not one of them.

**One rule governs this whole document.** A literal colour outside the token stylesheet is a defect,
not a shortcut.

**This document was written against a build that broke it 23 times.** `apps/desktop/src/**/*.tsx` held
`#ffffff`, `#f8fafc`, `#f1f5f9`, `#e0f2fe`, `#dcfce7`, `#fef3c7` and the rest — every one chosen for a
light background, while this file also defined a dark theme under `prefers-color-scheme`. Where the
two met they disagreed: `FindingsView` and `BundleView` painted white panels, the shell painted
`--color-text` white over them, and the result was white on white. Not a styling slip — the
predictable outcome of two colour authorities.

**Resolved by `W6-S1` at `420ecce`.** A grep for a hex literal outside this file now returns nothing, and
the lint rule `NFR-17` asked for refuses a new one. The paragraph above is kept in the past tense
rather than deleted, because the rule only reads as a rule once you know what it cost.

## Themes

Two, and neither is a preference inside Snapdown. The Windows theme decides, via `prefers-color-scheme`,
and a change while running is honoured without a restart (`NFR-17`).

Every token below is defined in **both** themes. A token defined in only one is the defect this section
exists to prevent — and a component MUST NOT reach past a token to a literal because "it only shows in
one theme."

## Tokens

**The four tables below still name the pre-Slint CSS scheme (`--color-*` custom properties in
`tokens.css`), and `tokens.css` itself was deleted with `web/ui` under `OQ-27` on 2026-09-01.**
`apps/desktop/ui/theme.slint` is the real, shipped source of truth today (confirmed by
`test_theme_contrast.rs`'s own first line), and its token set is not a rename of this one — it is
organised differently and is smaller in places:

- A clean correspondence exists for the surface/text/border basics: `--color-bg` → `bg-app`,
  `--color-surface` → `bg-card`, `--color-text` → `text-primary`, `--color-text-muted` →
  `text-muted`, `--color-border` → `border-subtle`, `--color-border-strong` → `border-strong`,
  `--color-accent`/`-text` → `accent-primary`/`text-on-accent`.
- **`--color-warning-bg`/`-text`, `--color-info-bg`/`-text`, and `--color-neutral-bg`/`-text` have
  no counterpart in `theme.slint` at all** — verified by reading the file in full on 2026-09-04, not
  inferred from a naming mismatch. Either the states they served (a failed hotkey, a listening
  recorder, a disabled badge) no longer exist in the shipped UI, or they are still drawn from a
  literal colour, which `AD-10` forbids.
- `--color-danger`/`-text` has no paired token either; the closest is the single, unpaired
  `semantic-error`.
- The Marker badge's actual colour is `annotation-stroke` (its fill) and `text-on-plate` (its
  label) — not a `--color-marker*` triple — per `theme.slint`'s own comment that it "borrowed
  `semantic-error`... and now takes `annotation-stroke`" as of 2026-09-02.
- `--color-surface-raised` and `--color-surface-sunken` have no `theme.slint` counterpart; `bg-hover`
  and `bg-subtle` are the nearest candidates but serve different roles (a hover state, a readout
  strip) and neither is verified as the intended replacement.

**This is a design-system audit, not a rename, and it is left to `wdi-ux` / `wdi-component` rather
than guessed here:** whether a missing category was dropped on purpose, still wants a literal
somewhere it should not, or needs a new `theme.slint` property is a design decision, and inventing
an answer would put a second wrong mapping in this document in place of the first.

The two theme-invariant tokens this document already named — the capture overlay's scrim and its
selection ring — **do** exist verbatim: `overlay-scrim` and `overlay-ring` in `theme.slint`, exactly
as `AD-10` requires for a token that does not follow `is-dark`. `--canvas-checker` (transparency
behind a non-filling image) has no match; `canvas-ground` is a different, solid-colour token for the
canvas's ground plate, not a checker pattern.

An earlier draft of `.how/finding/01-ux/DESIGN.md` declared `--overlay-*` and `--canvas-checker` as
component-local tokens that "stay here rather than being promoted". That contradicted `AD-10`, which
was written later and is a spine invariant; where a component document and the spine disagree, the
spine wins. Corrected 2026-08-23.

### Space, radius, type, elevation

`theme.slint` carries only `radius-sharp` and `radius-pill` today. The rest of this list —
`--space-1`..`--space-6`, `--font-ui`, `--font-mono`, `--text-xs`..`--text-xl`, `--shadow-raised`,
`--z-overlay|toast|modal` — was "unchanged from `tokens.css`" until that file was deleted under
`OQ-27`; whether each survives as a Slint property, a literal, or nothing is unverified.

### Typography & Font Stacks
- **Primary UI**: `Plus Jakarta Sans`, `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`
- **Monospace Telemetry**: `JetBrains Mono`, `ui-monospace, SFMono-Regular, Menlo, Consolas, monospace`

### Standardized Iconography System
Zero-dependency, high-contrast visual glyphs:
- `🔴` Capture Region Scrim (`Ctrl+Shift+S`)
- `🔢` Insert Step Marker Stamp
- `✂️` Crop Image Mode
- `🗑️` Delete Action (Marker & Bundle)
- `📦` Assemble Bundle Action
- `📋` Copy Markdown to Clipboard
- `⚙️` Settings & Preferences Modal

A `🤖` **Local MCP Agent Bridge** (`port:3849`) glyph stood here until `DEC-016` withdrew the Local
API and the MCP Bridge outright on 2026-09-04; there is no running channel left for an icon to name.

One addition the density work needs:

| Token | For | Resolves in |
|---|---|---|
| `--space-0` | `2px` — the gap inside a badge or between a label and its own control | to add |
| `--radius-full` | A pill: a badge, a segmented-control thumb | to add |

## Base elements & Component Inventory

Registered as `LC` of type `ui-element` in `components.yaml`. Each row's **States it MUST support** is
what `NFR-16` is checked against — every state is a state something is readable in, or it is a defect.

| Element | States it MUST support | Implementation |
|---|---|---|
| `Button` | default · hover · active · focus-visible · disabled · busy | `apps/desktop/ui` |
| `TextField` | default · focus-visible · invalid · disabled · read-only | `apps/desktop/ui` |
| `TextArea` | default · focus-visible · invalid · disabled · read-only | `apps/desktop/ui` |
| `Checkbox` | checked · unchecked · indeterminate · disabled | `apps/desktop/ui` |
| `SegmentedControl` | each option unselected · selected · focus-visible · disabled | `apps/desktop/ui` |
| `Toggle` | on · off · **indeterminate** · focus-visible · disabled | `apps/desktop/ui` |
| `Badge` | success · warning · neutral · danger · info | `apps/desktop/ui` |
| `MarkerBadge` | ordinal number · focused · dragged · theme-invariant | `apps/desktop/ui` |
| `MarkerLayer` | idle · inserting · dragging · delete-key-bound | `apps/desktop/ui` |
| `HotkeyChip` | bound · listening · unbound · conflicted | `apps/desktop/ui` |
| `Disclosure` | collapsed · expanded · focus-visible | `apps/desktop/ui` |
| `Modal` / `ConfirmDialog`| mounted · backdrop-blur · keydown-esc · confirm · cancel | `apps/desktop/ui` |
| `EmptyState` | an illustration slot, a sentence, one action | `apps/desktop/ui` |
| `ErrorState` | what failed, in the Reviewer's terms, and what they can do | `apps/desktop/ui` |
| `Toast` | info · success · danger, auto-dismiss and manual | `apps/desktop/ui` |
| `TokenEstimator` | calculated image + text sum · live telemetry | `apps/desktop/ui` |
| `LoupeViewport` *(W10)* | 6x pixel grid · center reticle · hex/rgb inspector | `apps/desktop/ui` |
| `AxisGuides` *(W9)* | crosshair horizontal/vertical axes · edge-flipped HUD | `apps/desktop/ui` |
| `SnappingHighlight` *(W11)*| magnetic boundary box · 8px threshold · alt-disable | `apps/desktop/ui` |

**`Implementation` corrected 2026-09-04 to `apps/desktop/ui` from `web/ui`, deleted under `OQ-27`.**
This is a location fix only. Which `.slint` file under that tree each row actually lands in — and
whether every row still names a real, currently-built element — is unverified and needs the same
per-component sweep `AGENTS.md`'s reachability pitfall already demands elsewhere: confirm each is
**instantiated** in a `.slint` file, not merely plausible from its name.

`Toggle`'s **indeterminate** state is not decoration. `FR-18` requires the startup control to reflect
the real OS registration and never an intended one, and reading that state is asynchronous. Without a
third state the control has to guess during the read, and the shipped build guesses `true` and renders
`false` a moment later. Indeterminate is what makes the requirement satisfiable.

`EmptyState` and `ErrorState` are elements rather than per-screen prose because every screen owes both
(`ux-guide` check 4), and eight hand-written empties drift into eight different voices.

## Do's and Don'ts

**Do** put every colour in `tokens.css` and reference it by token name.
**Do** pair every meaning background with its own text token, and check the pair once.
**Do** give `EmptyState` exactly one action — the action that ends the emptiness.
**Do** let a panel take the height its content needs.

**Don't** write a hex literal in a component. The lint rule of `NFR-17` refuses it.
**Don't** style for one theme and assume the other inverts. It does not; `--color-marker` is proof.
**Don't** stretch one group to match a neighbour's height. The shipped Settings screen does this and
loses roughly a third of the window to it.
**Don't** add an annotation tool. Numbered Markers are the whole annotation vocabulary — a PRD
Non-Goal, and independently the conclusion Cobalt Capture reached for the same audience.
