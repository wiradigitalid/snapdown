---
type: addendum
parent: prd
initiative: capture-to-markdown
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Addendum — PRD: Capture to Markdown

Depth that earned a place beside the PRD but would derail it. Nothing here is a promise, and nothing
here may be cited as a design.

## Rejected alternatives

| Option | Why it lost |
| --- | --- |
| Write the Note in the Editor rather than at capture time | The Note is cheapest to write in the second the Reviewer noticed the thing. Deferring it means either the Editor opens on every Capture — which breaks the loop — or a queue of un-noted images the Reviewer has to go back and interpret, which is exactly the failure the product exists to remove. |
| Open the Editor after every Capture, as a general capture tool does | The loop has to survive six runs in ninety seconds. A window taking focus after each one turns six Captures into six dismissals. Kept as a setting, off by default. |
| One overlay window spanning the whole virtual desktop | Simpler until the monitors have different DPI scaling, at which point the selection rectangle and the pixels it maps to stop agreeing. Per-monitor overlays cost more code and are correct. |
| Keep the original full-resolution capture beside the reduced one | Sounds free and is not: the Vault doubles, deletion has two files to keep consistent, and nothing ever reads the original. If the reduction was too aggressive the answer is to change the Quality Budget and re-capture, which takes seconds. |
| Reduce images at compose time rather than capture time | Puts the expensive work in the path the Reviewer is waiting on, and leaves the Vault full of unreduced files in the meantime. Reduce once, on the way in. |
| Marker numbers the Reviewer assigns by hand | Two sequences that have to be kept in step is the drift this feature exists to prevent. One sequence, owned by the Markers, renumbered on delete. |
| Freeform annotation layers (arrows, boxes, highlights) stored as vectors | Every one of them is invisible to the reader that matters. A numbered badge is the only annotation that survives being turned into text. |
| Soft delete with a recycle bin | Makes the Vault and the Library disagree by design, and the whole point of BG-5 is that a review leaves completely. Confirmation once is the safety, not a bin. |
| Store Notes inside the image file's metadata | Survives file moves and needs no database — but multi-select, Bundles, Marker ordering, and Publication state are all queries, and answering them by re-parsing a folder is a database with worse ergonomics. |
| One rolling Markdown file per day instead of named Bundles | Cheap to write, useless to hand over. A Handoff has a subject, and the grouping is what makes it readable. |
| Make a Bundle a live view over its Findings | Then a Bundle already handed to an agent changes underneath the conversation about it. A Bundle is a snapshot; that is what makes it citable. |
| Let a Bundle's Markdown be edited in place | The Bundle would drift from the Library that produced it, with no way to tell which was right. Recompose instead. |

## Options weighed

### Where the Note is written

Criteria fixed before scoring: keystrokes per Finding; whether focus is stolen; whether the Reviewer
can still see what they are describing; whether an un-noted backlog can accumulate.

| Placement | Keystrokes | Focus kept | Subject visible | No backlog |
|---|---|---|---|---|
| Inline field at the selected region | fewest | yes | yes | yes |
| A dialog window after release | more | no | partly | yes |
| In the Editor, later | most | no | no | no |
| Not at all — image only | fewest | yes | yes | no note exists |

The inline field wins on every criterion that was set, which is unusual enough to be worth recording:
it means the decision is not a trade-off and should not be revisited as if it were one.

### Marker rendering

Criteria: the Marker must be visible to a machine reading the image; it must not obscure the thing it
points at; it must survive image reduction; the Reviewer must be able to reposition it.

Two shapes were considered — burning the badge into the stored image at capture time, or storing
Marker coordinates and burning them in only when a Bundle is composed. The second is what the PRD
promises, and the reason is FR-8's requirement that a Marker be repositionable and renumberable: a
badge already burned into the file cannot be moved. It does mean the Bundle's image and the Finding's
image are not the same bytes, which is why FR-14 has to delete both.

### Ordering inside a Bundle

Selection order is the ordering, and no second mechanism exists in r1. The alternative — a drag-to-
reorder step during composition — was left out because it introduces a second source of truth for
"what order are these in" while the first one is free. If reordering is wanted later it belongs as a
property of the selection, not of the composed Bundle.

## Mechanism and transport

Not a design. The SDD owns that, and a builder MUST NOT follow this section.

- Per-monitor transparent overlay windows, created on hotkey and destroyed on save or cancel. Their
  lifetime being that short is what keeps them from interfering with anything.
- The reduction step wants to be off the path that dismisses the overlay, so that NFR-2's 500 ms does
  not include encoding time. That implies the save is recorded first and the file is finished
  immediately after, and it implies a Finding can briefly exist with its image still being written —
  which the Editor has to tolerate.
- Marker coordinates want to be stored normalised to the image, not in pixels, so that they survive
  any later change to the Quality Budget.
- The Note's numbered lines and the Marker list are one structure. Anything that stores them as two
  lists joined by a number will drift on the first delete.
- Deletion of a file and deletion of its row want to be one unit of work that either completes or
  does not, because NFR-5 is stated as an invariant rather than as a best effort.

## Sizing

Nothing sized. Wave sizing happens at G4 and G5 against the story list.

One figure recorded because it drives the FR-4 and NFR-3 defaults: a full-screen capture on a
3840 × 2160 monitor is about 8.3 megapixels; the same view at a 1600 px long edge is about
1.4 megapixels, roughly a sixth. Source: the primary user's own monitor resolution, not a benchmark,
and it is the reason OQ-3 is open rather than answered.

### Two path conventions for one Bundle — `FR-12` and `NFR-8`

`NFR-8` governs the **stored** `bundle.md` and keeps its image links relative to that file's own
folder. `FR-12` governs the **clipboard**, and as of 2026-08-31 it permits those same links to be
rendered as absolute paths. The two are not in conflict: they serve two different readers, and the
stored file is not what the clipboard hands over.

The mechanism belongs here rather than in the PRD. One composer takes a base path and serves both
renderings; a second serializer alongside the first is what must **not** be built, because two
serializers drift and the golden-file test only pins one of them.

Still unsettled, and tracked on the Bundle Library map's ticket 03: which encoding the absolute form
uses, whether the Reviewer is told that images do not travel on a text clipboard, and whether
`Open file location` survives the answer. A Windows absolute path is not valid in Markdown as it
stands — backslashes escape and spaces terminate a link — so whatever is chosen has to be proven
against a Vault path containing a space.

### Rendering a Bundle as a PDF — `FR-39`

`FR-39` states what the Reviewer gets and deliberately names no engine, because `prd.md` carries no
solution shape. The mechanism is nonetheless settled work rather than an open question, and this is
the pointer to where it is recorded so nobody redoes it:

**`.scratch/bundle-library/issues/07-research-the-pdf-render-engine.md`** holds the whole
investigation — the engine chosen and why, its licence, the disk and idle-memory cost measured rather
than estimated, how a screenshot too tall for a page is fitted, and why the escape set for inserted
text is closed by definition. It also records the rejected candidate and the specific way it failed:
it was a third of the size and silently dropped every image across three input forms, which is the
kind of failure only decoding the output reveals.

Two things there are deliberately **not** settled, and both belong to the Export PDF effort rather
than to this PRD:

- **Packaging** — whether the exporter runs in-process or as a separate crate. The research reversed
  itself twice on it, which marks it as an architectural judgement rather than a measurable fact, so
  it is owed a `DEC-` at the point the work is planned.
- **The two tall-image thresholds** — the clamp and the slice points are solved and measured from a
  PDF's own placement matrices, but they still need calibrating against real screenshots rather than
  synthetic ones.

## Personas and research detail

Two facts about the primary user shaped this initiative more than any persona detail, and both come
from their own account rather than from research:

- They review in bursts. Four or five Findings arrive minutes apart, so every requirement about focus,
  toasts, and not opening windows traces back to this one fact.
- They have abandoned this workflow before, in a general capture tool plus a folder plus manual
  pasting. The reason was not any single missing feature; it was that the tool became the thing being
  managed. CAP-6 exists because of that, not because settings are a feature.

No external research was run for this PRD and `_bmad-output/` holds no run folder for it.
