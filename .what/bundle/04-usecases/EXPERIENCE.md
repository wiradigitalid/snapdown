---
type: ux
component: bundle
document: experience
created: "2026-08-23"
updated: "2026-08-23"
---

# EXPERIENCE — Bundle

## Information architecture

Bundles is one of the Editor's four primary surfaces (`FR-28`). It holds three regions: the **bundle
list**, the **Markdown preview**, and the **item list** for the selected Bundle.

A Bundle is only ever created from Findings, and only from the Findings surface (`FR-9`, `FR-10`).
There is no "new empty Bundle" here, and that absence is deliberate: a Bundle with nothing in it is a
name with no observation behind it, and the product has no use for one.

Composition happens in a modal over Findings, not on this surface. Nothing that happens here creates a
Bundle; this surface is where a Bundle is read, handed over, and eventually thrown away.

## Voice and tone

The Bundle *is* the deliverable, so the product says very little on this surface. The preview shows the
Reviewer's own Markdown, and the only product words are the three actions: Copy Markdown, Publish,
Delete.

"Copy Markdown" rather than "Export" — because `FR-12` puts it on the clipboard and the Reviewer pastes
it into an agent chat. Export implies a file dialog that does not exist.

## Component patterns

| Pattern | Behaviour |
|---|---|
| **Bundle list** | Name, item count, composed date. Newest first |
| **Markdown preview** | The exact bytes `FR-12` copies, in `--font-mono`, scrollable, read-only |
| **Item list** | The Findings inside, in Bundle order, each with its note's first line |
| **Compose modal** | Over Findings. Names the Bundle and confirms the selection |

**The preview is read-only, and that is a promise rather than a limitation.** `FR-11` and the Non-Goals
say a Bundle is recomposed, never patched — an edited Bundle would drift from the Library that produced
it, and the images it points at would no longer be explained by any Finding. An editable preview would
quietly break that. The surface therefore shows the Markdown and offers no cursor in it.

## State patterns

| Surface state | What the Reviewer sees |
|---|---|
| **Empty** | "No bundles yet", and one sentence saying where a Bundle comes from — select Findings and compose. The action is on the *other* surface, so the empty state says so rather than offering a button that leads nowhere |
| **Loading** | The three regions hold their shape; the list shows skeleton rows |
| **Nothing selected** | List populated, preview shows a muted invitation. Distinct from empty |
| **Populated** | Normal |
| **Images missing** | The Bundle's own image copies are gone from the Vault. The preview renders, the item list flags which items lost their file, and Delete is still offered. It does not pretend the Bundle is intact |
| **Error** | The Library could not be read. One message, one Retry |

The **compose modal** has its own set: composing · naming refused because the name is empty or taken ·
composing failed, with nothing half-written.

## Interaction primitives

- **Copy Markdown is one action with visible confirmation.** A toast, because a silent clipboard write
  is indistinguishable from a failure.
- **Delete is confirmed once, and it is real.** `FR-14` removes the Bundle's image copies with it.
- **Deleting a Bundle does not delete the Findings it came from.** The confirmation says so — the
  Reviewer will otherwise assume it cascades, because deleting a Finding *does* delete its file.
- **Publish is present and frozen.** `DEC-005` holds `sharing` still, so the action shows the current
  publication state and gains no new behaviour this release. Removing it would break `FR-28`.
- **No drag to reorder.** Order is fixed at composition, because the Markdown is written once.

## Accessibility floor

**WCAG 2.2 AA**, per `NFR-16`.

- The preview is a read-only region with an accessible name, not a disabled text field. A disabled
  field is announced as unavailable, which is the wrong meaning: the content is available, it is the
  editing that does not exist.
- Copy Markdown announces its result. A clipboard write with no announcement is invisible to a screen
  reader, and it is the primary handoff path in the whole product.
- The item list is a list, and each row names its Finding and its position in the Bundle.
- Contrast holds in both Windows themes. `BundleView` used to paint `#f8fafc`, `#ffffff`, `#e0f2fe`
  and `#f1f5f9` panels regardless of theme, so under the Windows dark theme the shell's white text
  landed on them. Resolved by `W6-S1`; the promise is what remains, and it is now checked by an
  assertion rather than by inspection.

## Key flows

**Wira hands over one Bundle — `UJ-3`.** He has five Findings from a pass over the staging site. On
Findings he ticks all five and clicks Compose. He names it `checkout-pass-1`. Snapdown writes one
Markdown file where each note sits under the image it describes, with the Markers burned in. He lands
on Bundles with it selected, reads the preview to confirm the order is the order he meant, and clicks
Copy Markdown. He pastes into his agent. **The climax beat is the paste working first time** — every
image path resolves, because `NFR-8` requires them to resolve relative to the Markdown file's own
folder.

**Wira throws away last month's review.** Bundles, select `checkout-pass-1`, Delete. The confirmation
names the Bundle, says its image copies go with it, and says the five original Findings stay. He
confirms. The Vault holds no file the Library does not point at (`NFR-5`).

## Edge cases

| Moment | What the Reviewer does next |
|---|---|
| A Bundle name is already taken | Refused while typing, in the modal, naming the existing Bundle |
| Composition fails partway | Nothing is written — no half-Bundle, no orphan images. `FR-10` is all-or-nothing |
| A Finding in the Bundle was deleted afterwards | The Bundle is intact; it holds its own image copies. The item list still names the Finding. This is `FR-13` working, not a fault. **It does not hold in the shipped product: `BUG-1`.** `bundle_item.finding_id` cascades, so the item list silently loses its row while the Markdown and the image copy survive. Fixed by `W6-S9` |
| A Bundle's own image copy is missing from the Vault | The item is flagged, the Bundle still opens, Delete still works, and the orphan report is offered |
| Copy Markdown while the clipboard is locked by another program | The failure is reported. A silent failure here loses the handoff and the Reviewer would not know |
| A Bundle of one Finding | Allowed and normal. `OQ-16` records that the extra step may want a shortcut; it is not a different path |
| The Bundle is published and then deleted | The publication is withdrawn with it (`BR-23`). The confirmation says so before, not after |
