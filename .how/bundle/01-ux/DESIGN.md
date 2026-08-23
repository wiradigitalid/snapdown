---
type: ux
component: bundle
document: design
created: "2026-08-23"
updated: "2026-08-23"
---

# DESIGN — Bundle

Tokens, elements, and both themes are in `.how/_platform/design-system.md`.

## Tokens

| Token | For |
|---|---|
| `--preview-line-height` | `1.55` — the Markdown preview, tuned for `--font-mono` at `--text-sm` |
| `--bundle-list-width` | `240px` |
| `--item-list-width` | `280px` |

Nothing else. This surface is deliberately the plainest in the product: its content is the Reviewer's
deliverable, and product styling on top of it competes with the thing being handed over.

## Screens

| Screen | LC | Purpose |
|---|---|---|
| Bundles | `LC-014` `bundles-editor` | List, preview, item list. Serves `FR-11`, `FR-12`, `FR-14` |
| Compose Bundle | `LC-031` `compose-bundle-dialog` | Naming and confirming. Serves `FR-10` |
| Publish a Bundle | `LC-022` `publish-dialog` | Existing. Frozen by `DEC-005` |

`LC-031` is new and registered as part of landing this document. Inventory row 9 named the screen; no
build unit carried it.

## Layout and states

### Bundles (`LC-014`)

```
┌ list 240px ──┐┌─ preview (flex) ───────────┐┌ items 280px ─┐
│ checkout-p-1 ││ # checkout-pass-1          ││ 1  the CTA…  │
│  5 · 12 Aug  ││                            ││ 2  spacing…  │
│ nav-review   ││ ![](./img/f-01.jpg)        ││ 3  contrast… │
│  3 · 04 Aug  ││                            ││ 4  the modal…│
│              ││ 1. the CTA drops below…    ││ 5  focus…    │
│              ││                            ││              │
├──────────────┤└────────────────────────────┘├──────────────┤
│              │                              │[Copy Markdown]│
│              │                              │[Publish][Del] │
└──────────────┘                              └──────────────┘
```

- The **preview is the centre and it grows**, because it is the artifact.
- Actions sit at the **item list's foot**, not floating over the preview. Copy Markdown is the one
  `primary` Button on the surface; Publish is `secondary`; Delete is `danger`.
- All three panels draw from `--color-surface` on `--color-bg` and take the height available. The
  shipped build paints `#f8fafc`, `#ffffff`, and `#e0f2fe` literals at a fixed height, which produces
  both the dark-theme contrast failure and roughly a third of the window left empty beneath.

| State | Rendering |
|---|---|
| Empty | One centred `EmptyState`: "No bundles yet", one sentence — "Select findings on the Findings tab and choose Compose." No button, because the action lives on another surface and a button that navigates away pretends otherwise |
| Loading | Skeleton rows in the list; preview and item list hold their shape |
| Nothing selected | List populated; preview shows muted "Select a bundle". Distinct from empty |
| Populated | Normal |
| Item file missing | The affected item row carries a `--color-warning-bg` badge. The preview and the actions still work |
| Error | Centred `ErrorState`, one Retry |

The preview renders in `--font-mono` at `--text-sm` on `--color-surface-sunken`. It is a read-only
region, **not** a disabled input: a disabled control is announced as unavailable, and the content is
available — it is the editing that does not exist (`FR-11`, Non-Goals).

### Compose Bundle (`LC-031`)

A modal over Findings at `--color-surface-raised` with `--shadow-raised`, `520px` wide.

```
┌──────────────────────────────────────────┐
│ Compose bundle                           │
│                                          │
│ Name  [checkout-pass-1               ]   │
│                                          │
│ 5 findings, in capture order:            │
│  1 ▣ the CTA drops below the fold…       │
│  2 ▣ spacing on the summary row…         │
│  …                                       │
│                                          │
│            [Cancel]  [Compose]           │
└──────────────────────────────────────────┘
```

| State | Rendering |
|---|---|
| Default | Name focused. Compose disabled until the name is valid |
| Name taken | `TextInput` invalid, message naming the existing Bundle, refused while typing |
| Composing | Compose busy; the modal stays open and is not dismissable |
| Failed | Message inside the modal. Nothing was written — no half-Bundle, no orphaned images |
| Empty | Unreachable — the modal only opens from a non-empty selection |

## Do's and don'ts for this surface

**Do** let the preview be the largest thing on the screen.
**Do** say in the delete confirmation that the source Findings stay.
**Do** confirm a clipboard write visibly.

**Don't** give the preview a cursor. A Bundle is recomposed, never patched.
**Don't** offer a button in the empty state that only navigates elsewhere.
**Don't** write a literal colour in a component.
**Don't** style the Markdown into something prettier than what `FR-12` actually copies. What is shown
must be the bytes that are handed over.
