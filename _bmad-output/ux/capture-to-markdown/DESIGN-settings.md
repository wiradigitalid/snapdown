---
type: ux
component: settings
document: design
created: "2026-08-23"
updated: "2026-08-23"
---

# DESIGN — Settings

Tokens, elements, and both themes are in `.how/_platform/design-system.md`. Nothing here restates a
value; this document is layout and the component-specific delta.

## Tokens

Component-specific only. Everything else is inherited.

| Token | For |
|---|---|
| `--settings-group-gap` | `var(--space-4)` — between groups in a column |
| `--settings-column-min` | `380px` — below this the two columns become one |
| `--settings-row-height` | `32px` — one control row, so a group's height is countable in advance |

Anything reusable was promoted rather than defined here: `--color-surface-sunken`, the four meaning
pairs, `--space-0`, and `--radius-full` are all in the design system, because the hotkey chip and the
badges that need them appear on more than this surface.

## Screens

| Screen | LC | Purpose |
|---|---|---|
| Editor shell | `LC-028` `editor-shell` | The window persona, the primary navigation, and the surface frame. Serves `FR-27`, `FR-28` |
| Settings | `LC-015` `settings-screen` | The five groups. Serves `FR-5`, `FR-16`, `FR-17`, `FR-18`, `FR-29` |

`LC-028` is new and is registered as part of landing this document. It exists because `FR-27` and
`FR-28` are promises about the *frame*, and a promise with no build unit behind it is a promise nobody
checks. In the shipped build the frame is inline JSX at the top of `App.tsx`, owned by nothing.

## Layout and states

### Editor shell (`LC-028`)

A **left navigation rail**, full window height, `200px` wide, `--color-surface` on `--color-bg`.

The shipped build uses a top tab row. The rail replaces it for one reason that is not taste: vertical
space is the scarce resource on this window, and `FR-29` requires Settings to fit at 1024×720 without
scrolling. A top tab row costs ~64px of height on every surface. A rail costs width, which Findings and
Bundles were already spending on a list column anyway.

```
┌──────────────────────────────────────────────────────────┐
│ ⬛ Snapdown            [Snapdown Editor — window title]   │
├──────────┬───────────────────────────────────────────────┤
│ Findings │                                               │
│ Bundles  │              surface content                  │
│ Settings │                                               │
│ Agent    │                                               │
│  access  │                                               │
│          │                                               │
│──────────│                                               │
│ ⏺ Capture│  ← the capture action, always reachable        │
└──────────┴───────────────────────────────────────────────┘
```

- The product mark and wordmark sit at the rail's top. The **window title bar** reads `Snapdown Editor`
  (`FR-27`) and does not change as the Reviewer moves between surfaces.
- The active item carries a filled `--color-accent` background **and** a left edge bar. Two signals,
  because `NFR-16`'s accessibility floor forbids state carried by colour alone.
- The **Capture** action pins to the rail's foot, separated by a rule. It is the only action in the
  chrome, and it belongs there because it is the one thing the Reviewer does from anywhere.

| State | Rendering |
|---|---|
| Loading | Rail fully rendered; content area shows the surface's own loading state. The rail never flickers |
| Populated | Normal |
| Error | The rail still works. A surface that failed shows its error inside the content area, never by blanking the frame |
| Empty | Not applicable — the rail always has four items |

### Settings (`LC-015`)

**Two columns, packed by content height, never stretched.**

```
┌─ column A (flex) ─────────────┐ ┌─ column B (flex) ─────────────┐
│ Startup                       │ │ Quality Budget                │
│  [◉] Run at Windows startup   │ │  [Auto|Sharp|Balanced|Small]  │
│      Starts in the tray when  │ │  Auto sizes each capture to   │
│      you sign in.             │ │  what it is. Most captures    │
├───────────────────────────────┤ │  land near 120 KB.            │
│ Vault folder                  │ │                               │
│  C:\Users\kodes\SnapdownVault │ │  Latest: 184 KB · 1408 px     │
│  [Browse…] [Apply] [Explorer] │ │  ▸ Advanced                   │
└───────────────────────────────┘ ├───────────────────────────────┤
┌───────────────────────────────┐ │ Hotkeys                       │
│ Agent access                  │ │  Capture Region   [Ctrl+⇧+S] ●│
│  Key issued · 2026-08-14      │ │  Open Editor      [Ctrl+⇧+E] ●│
│  [Copy] [Revoke]              │ └───────────────────────────────┘
└───────────────────────────────┘
```

The rule the shipped build broke: **a group takes the height its content needs.** The shipped screen
pairs a one-checkbox group with a four-control group in equal-height grid columns and leaves roughly a
third of the window empty. Here the columns are independent stacks; a short group is short.

Group order is by how often it is touched — startup and the Vault folder are first-run concerns and sit
where the eye lands, hotkeys and Quality Budget are the ones changed later. Agent access is last and
inert (`DEC-005`).

Below `--settings-column-min` the two columns become one and the surface may scroll. That is allowed:
`FR-29` binds at the **minimum supported** window size, and 1024×720 holds both columns.

#### Quality Budget group

The `FR-5` / `DEC-004` control. A four-option `SegmentedControl`, then one line of prose for the
selected option, then the readout, then the disclosure.

| Option | The line under it |
|---|---|
| **Auto** (default) | "Sizes each capture to what it is. Most captures land near 120 KB." |
| **Sharp** | "Keeps small text crisp. Files are larger." |
| **Balanced** | "A middle setting that does not change with the capture." |
| **Small** | "The smallest file that is still readable." |

The readout is `Latest: 184 KB · 1408 px · Auto` — size, dimension, and **which budget produced it**.
`FR-5` requires attribution, not just size, because under Auto the number moves for a reason the
Reviewer did not cause.

`▸ Advanced` collapses two `TextInput`s, max long edge and encoder quality. Editing either moves the
segmented control to a fifth segment, **Custom**, which appears only once it is occupied and stays
visible after. The Reviewer never leaves Auto without seeing that they left it.

| State | Rendering |
|---|---|
| Loading | Segments visible and inert; readout reads "—". No segment is pre-selected |
| Populated | Normal |
| Empty | No capture taken yet: readout reads "No captures yet". The control still works |
| Error | Budget could not be read: the group shows its own message and a Retry. The other four groups are unaffected |

#### Hotkeys group

One row per action: label, `HotkeyChip`, `Badge`, and a clear affordance.

| Chip state | Rendering |
|---|---|
| bound | `--color-surface-sunken`, `--font-mono`, `--radius-full`, the combination in text |
| listening | `--color-info-bg` / `--color-info-text`, a `2px` `--color-accent` ring, reading "Press keys… Esc to cancel" |
| unbound | Dashed `--color-border-strong`, reading "Click to set" |
| conflicted | `--color-warning-bg` / `--color-warning-text`, with the conflict named on the line beneath |

The badge reads **Active** or **Disabled** in words. It previously used literal `#dcfce7` on `#166534`,
which is a light-theme pair rendered inside a dark shell; it now uses the `--color-success-*` and
`--color-neutral-*` pairs, each proven against its own background once.

#### Startup group

A `Toggle` with three states. The indeterminate state renders as a dimmed track with no thumb position
and is not interactive; it is what the control shows before the real Windows registration has been
read (`FR-18`). It MUST NOT render on or off first.

## Do's and don'ts for this surface

**Do** let a group end where its content ends.
**Do** put a failure under the control that failed.
**Do** show which budget produced the latest capture, not just its size.

**Don't** stretch a group to match a neighbour.
**Don't** show a raw pixel or quality number outside Advanced.
**Don't** render the startup toggle in a definite state before Windows has answered.
**Don't** hide the Agent access group while it is frozen — `FR-28` requires it to stay reachable.
