---
type: design-system
scope: _platform
status: draft
created: "2026-08-23"
updated: "2026-08-23"
---

# Design System — Snapdown

## Living Specification & Interactive Catalog
The complete visual catalog and interactive design system artifact is permanently housed in the `.how/` platform layer:
👉 [`.how/_platform/assets/design-system.html`](assets/design-system.html)
👉 Complete Flow Machine: [`.how/_platform/assets/ui-ux-complete-flow.html`](assets/ui-ux-complete-flow.html)

## Where the values actually live

| Source of truth | Path | Holds |
|---|---|---|
| Token stylesheet | `web/ui/src/styles/tokens.css` | Every colour, space, radius, type step, elevation, z-index. The **only** place a literal colour may appear |
| Desktop reset | `apps/desktop/src/styles/tokens.css` | Imports the above; adds the desktop reset and nothing else |
| Shared UI package | `web/ui/src/` | The base elements the desktop imports as `@snapdown/ui`. Where an element's states actually live |

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

### Colour — surface and text

| Token | For | Resolves in |
|---|---|---|
| `--color-bg` | The window's own ground. Never transparent | `tokens.css` |
| `--color-surface` | A panel, card, or rail sitting on the ground | `tokens.css` |
| `--color-surface-raised` | A panel above another panel — a modal, a popover, a selected row | `tokens.css` |
| `--color-surface-sunken` | **New.** An inset well: a preview area, a code block, a recorder chip at rest | to add |
| `--color-text` | Body text on `--color-bg` or `--color-surface` | `tokens.css` |
| `--color-text-muted` | Secondary text. MUST still meet AA — muted is not an exemption | `tokens.css` |
| `--color-border` | The line between two surfaces | `tokens.css` |
| `--color-border-strong` | A control's own edge — input, recorder chip, segmented control | `tokens.css` |

### Colour — meaning

Each of these carries a **paired text token**. That pairing is the mechanism that makes `NFR-16`
satisfiable: a background is never used without the foreground proven against it.

| Token pair | For | Resolves in |
|---|---|---|
| `--color-accent` / `--color-accent-text` | The one primary action on a surface | `tokens.css` |
| `--color-danger` / `--color-danger-text` | Delete, revoke, unpublish | `tokens.css` |
| `--color-success-bg` / `--color-success-text` | **New.** A hotkey that is registered and active | to add — replaces literal `#dcfce7`/`#166534` |
| `--color-warning-bg` / `--color-warning-text` | **New.** A hotkey that failed to register at startup | to add — replaces literal `#fef3c7`/`#854d0e` |
| `--color-info-bg` / `--color-info-text` | **New.** A recorder chip while it is listening | to add — replaces literal `#eff6ff` |
| `--color-neutral-bg` / `--color-neutral-text` | **New.** A disabled or inert badge | to add — replaces literal `#f1f5f9`/`#64748b` |
| `--color-marker` / `--color-marker-text` / `--color-marker-ring` | A numbered Marker badge burned onto an image | `tokens.css` |

### Colour — theme-invariant, and defined here anyway

`AD-10` is explicit that a deliberately theme-invariant token **MUST still be defined in the token
file**, and must say where it is defined why it does not follow the theme. Three groups qualify, and
all three live here rather than in a component's own `DESIGN.md`:

| Token | For | Why it does not follow the theme |
|---|---|---|
| `--color-marker` / `-text` / `-ring` | A numbered Marker badge | Burned into an exported image read on another machine under another theme |
| `--color-overlay-scrim` | The capture overlay's scrim | Drawn over the Reviewer's own screen content, not over a Snapdown surface |
| `--color-overlay-ring` | The selected region's edge | Must stay visible over any content the Reviewer happens to be capturing |
| `--canvas-checker` | Transparency behind an image that does not fill its pane | Sits behind image content, not behind app chrome |

They are the one place a literal value is correct, and `NFR-17`'s lint rule must be scoped to allow
them **in this file only**.

An earlier draft of `.how/finding/01-ux/DESIGN.md` declared `--overlay-*` and `--canvas-checker` as
component-local tokens that "stay here rather than being promoted". That contradicted `AD-10`, which
was written later and is a spine invariant; where a component document and the spine disagree, the
spine wins. Corrected 2026-08-23.

### Space, radius, type, elevation

Unchanged from `tokens.css` and not restated here: `--space-1`..`--space-6`, `--radius-sm|md`,
`--font-ui`, `--font-mono`, `--text-xs`..`--text-xl`, `--shadow-raised`, `--z-overlay|toast|modal`.

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
- `🤖` Local MCP Agent Bridge (`port:3849`)
- `⚙️` Settings & Preferences Modal

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
| `Button` | default · hover · active · focus-visible · disabled · busy | `web/ui` |
| `TextField` | default · focus-visible · invalid · disabled · read-only | `web/ui` |
| `TextArea` | default · focus-visible · invalid · disabled · read-only | `web/ui` |
| `Checkbox` | checked · unchecked · indeterminate · disabled | `web/ui` |
| `SegmentedControl` | each option unselected · selected · focus-visible · disabled | `web/ui` |
| `Toggle` | on · off · **indeterminate** · focus-visible · disabled | `web/ui` |
| `Badge` | success · warning · neutral · danger · info | `web/ui` |
| `MarkerBadge` | ordinal number · focused · dragged · theme-invariant | `web/ui` |
| `MarkerLayer` | idle · inserting · dragging · delete-key-bound | `web/ui` |
| `HotkeyChip` | bound · listening · unbound · conflicted | `web/ui` |
| `Disclosure` | collapsed · expanded · focus-visible | `web/ui` |
| `Modal` / `ConfirmDialog`| mounted · backdrop-blur · keydown-esc · confirm · cancel | `web/ui` |
| `EmptyState` | an illustration slot, a sentence, one action | `web/ui` |
| `ErrorState` | what failed, in the Reviewer's terms, and what they can do | `web/ui` |
| `Toast` | info · success · danger, auto-dismiss and manual | `web/ui` |
| `TokenEstimator` | calculated image + text sum · live telemetry | `web/ui` |
| `LoupeViewport` *(W10)* | 6x pixel grid · center reticle · hex/rgb inspector | `web/ui` |
| `AxisGuides` *(W9)* | crosshair horizontal/vertical axes · edge-flipped HUD | `web/ui` |
| `SnappingHighlight` *(W11)*| magnetic boundary box · 8px threshold · alt-disable | `web/ui` |

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
