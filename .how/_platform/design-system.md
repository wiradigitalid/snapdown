---
type: design-system
scope: _platform
status: draft
created: "2026-08-22"
updated: "2026-08-23"
---

# Design System — Snapdown

Two React front ends share this: the desktop webview inside `desktop-app`, and `web-ui` in a
reader's browser. They look alike on purpose — a published Bundle should read as the same document
the Reviewer composed — but they are not the same application, and this file is the only thing that
crosses between them.

## Where the values actually live

Nothing here is a value. Every token below names where it resolves, and this file MUST reference
rather than repeat — two homes for one hex code is two hex codes within a month.

| Source of truth | Path | Holds |
| --- | --- | --- |
| Shared token stylesheet | `web/ui/src/styles/tokens.css` | Every token's value, in both colour schemes |
| Desktop token import | `apps/desktop/src/styles/tokens.css` | Imports the shared file; overrides nothing |
| Base element components | `web/ui/src/components/` | The elements in the table below |

The shared file is authored once and consumed by both front ends. A token defined in only one of them
is a defect, not a local choice.

## Tokens

| Token | For | Resolves in |
| --- | --- | --- |
| `--color-bg`, `--color-surface`, `--color-surface-raised` | Page, card, and elevated card backgrounds | `tokens.css` |
| `--color-text`, `--color-text-muted` | Body copy and secondary copy | `tokens.css` |
| `--color-border`, `--color-border-strong` | Dividers and input outlines | `tokens.css` |
| `--color-accent`, `--color-accent-text` | The one accent, used for the primary action and nothing decorative | `tokens.css` |
| `--color-danger`, `--color-danger-text` | Destructive confirmations only — deletion, unpublish | `tokens.css` |
| `--color-marker`, `--color-marker-text`, `--color-marker-ring` | The numbered Marker badge, over arbitrary screenshot pixels. The ring is a separate token because the badge needs contrast against both light and dark screenshot content, and a fill alone cannot give it | `tokens.css` |
| `--space-1` … `--space-6` | Every margin and padding. No arbitrary pixel spacing | `tokens.css` |
| `--radius-sm`, `--radius-md` | Corners | `tokens.css` |
| `--font-ui`, `--font-mono` | Interface text, and Markdown or code | `tokens.css` |
| `--text-xs` … `--text-xl` | The whole type scale | `tokens.css` |
| `--shadow-raised` | The one elevation | `tokens.css` |
| `--z-overlay`, `--z-toast`, `--z-modal` | Stacking, so three surfaces do not each invent a number | `tokens.css` |

One token this table names is **missing from the stylesheet**: a scrim. `Modal` currently writes
`rgba(0, 0, 0, 0.5)` as a literal, which the rules below forbid. A `--color-scrim` token is the fix,
and it is recorded here rather than in a component so that whichever wave next touches `Modal` finds
it. The type scale in the stylesheet is `--text-xs` `--text-sm` `--text-base` `--text-lg` `--text-xl`;
this table's `--text-xs … --text-xl` is that range.

Both colour schemes are defined on `:root` and swapped under `prefers-color-scheme`. There is no
theme switcher: the Reviewer's system setting decides, and so does the reader's.

`--color-marker` is the one token with a hard requirement behind it. A Marker badge sits on top of an
arbitrary screenshot, so it needs a contrasting fill **and** a contrasting ring against both light and
dark content. It is not the accent, and reusing the accent for it is the mistake this row exists to
prevent.

## Base elements

| Element | States it MUST support | Implementation |
| --- | --- | --- |
| `Button` | default · hover · active · focus-visible · disabled · loading · `danger` variant | `web/ui/src/components/Button.tsx` |
| `TextField` | default · focus-visible · invalid · disabled · with a character count | `web/ui/src/components/TextField.tsx` |
| `TextArea` | the same, plus auto-grow. Used for a Note body and a Marker comment | `web/ui/src/components/TextArea.tsx` |
| `Checkbox` | unchecked · checked · indeterminate · focus-visible · disabled. Indeterminate is required by select-all over a partial selection | `web/ui/src/components/Checkbox.tsx` |
| `Toast` | one line, an optional action, auto-dismiss. MUST NOT take focus when it appears, and its action MUST still be reachable by keyboard | `web/ui/src/components/Toast.tsx` |
| `Modal` | open · closing. Focus trapped, Escape closes, focus returns to the trigger | `web/ui/src/components/Modal.tsx` |
| `ConfirmDialog` | a `Modal` whose confirm is `danger` and whose message names what will go and how many | `web/ui/src/components/ConfirmDialog.tsx` |
| `MarkerBadge` | 1–99, plus a dragging state. Fixed size regardless of image scale | `web/ui/src/components/MarkerBadge.tsx` |
| `EmptyState` | one heading, one sentence, at most one action | `web/ui/src/components/EmptyState.tsx` |
| `Markdown` | rendered CommonMark, read-only, with relative image resolution | **Not yet built.** W3 creates it, with a real CommonMark parser, when `bundle` first has bytes to render |

`web-ui` uses `Markdown`, `EmptyState`, and `MarkerBadge`. The desktop uses all of them. An element
only one side needs still lives here, because the alternative is it being written twice.

`Markdown` is the one element deliberately absent from the tree today. W1 built a four-branch string
splitter under that name and the panel rejected it: a component claiming a capability it does not have
is worse than an absent one, and no screen in W1 or W2 renders Markdown at all. W3 builds it against a
real CommonMark parser. AD-9 is why it has to be real — it renders the exact bytes a published Bundle
serves, so a lossy renderer there is a correctness defect, not a cosmetic one.

## Rules that bind every screen

| Rule | Prevents |
| --- | --- |
| Every destructive action goes through `ConfirmDialog`, and its message names what will go and how many | A deletion that is one misclick away, on an action BR-7 makes irreversible |
| The accent colour marks exactly one primary action per surface | Three buttons competing, so the Reviewer reads none of them |
| `--color-danger` appears only on a destructive confirmation | Danger colour becoming decoration, so a real warning reads as styling |
| Every interactive element has a visible `focus-visible` state | A capture loop that cannot be driven from the keyboard, which FR-2 requires |
| Every list has an `EmptyState`, and it says what to do next | A blank panel that reads as a bug |
| A `Toast` never takes focus on appearance and never needs dismissing | The capture loop being interrupted by its own confirmation, which FR-3 forbids |
| A `Toast`'s action is reachable by keyboard, even though the Toast does not take focus | A clickable control no keyboard user can reach — the earlier wording said "MUST NOT be focusable" and was read as forbidding that too |
| A `MarkerBadge` renders at a fixed size and its position comes from normalised coordinates | Badges drifting off target when an image is displayed at a different scale (AD-3) |
| No spacing, colour, radius, or font size is written as a literal. Tokens only | A second design system growing inside one component |
| Text in the interface is English, and it is the only language shipped | A half-translated interface, which is worse than an untranslated one |

## What this system deliberately does not cover

- **The Capture Overlay.** It is a transparent window drawn over arbitrary screen content, not a page.
  Its crosshair, dimming, and selection rectangle are `finding`'s, and forcing them through these
  tokens would make them harder to see, which is the one thing they must be.
- **Animation and motion.** There is none beyond a `Toast` appearing and a `Modal` opening. Nothing
  in this product benefits from motion, and a motion scale would invite some.
- **Icons.** There are barely any, and the ones there are come from the elements above. No icon set is
  adopted, and adopting one is a change to this file.
- **Layout and navigation.** Per screen, in `inventory-screen.md` and each component's `01-ux/`.
- **Print, email, or any non-screen medium.** A published Bundle is read on a screen or as raw
  Markdown, and raw Markdown carries no styling by design.
- **Density, theming, or user-configurable appearance.** Not a setting, because leaving it fixed
  breaks nothing.
