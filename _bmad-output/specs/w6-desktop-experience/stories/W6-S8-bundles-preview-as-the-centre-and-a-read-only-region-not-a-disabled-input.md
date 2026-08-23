---
id: W6-S8
title: 'W6-S8: Bundles — preview as the centre, and a read-only region not a disabled input'
type: 'feature'
wave: W6
status: done
created: '2026-08-24'
dependencies:
  - W6-S1
  - W6-S2
  - W6-S9
files:
  - web/ui/src/styles/tokens.css
  - web/ui/src/components/BundlesEditor.tsx
  - web/ui/src/index.ts
  - apps/desktop/src/components/BundleView.tsx
  - apps/desktop/src/services/bundle.ts
  - apps/desktop/src/App.tsx
  - web/ui/src/test/bundles_editor.test.tsx
  - apps/desktop/src/test/bundle_view.test.tsx
  - web/ui/src/test/tokens.test.ts
  - web/ui/src/test/contrast.test.ts
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S8-step1-plan.md
  - .how/bundle/01-ux/DESIGN.md
  - .what/bundle/04-usecases/EXPERIENCE.md
  - .what/bundle/SRS-bundle.md
  - .how/bundle/SDD-bundle.md
  - .control/registry/defects.yaml
  - .how/_platform/design-system.md
  - .how/_platform/cross-cutting.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
The desktop application's Bundles surface (`LC-014` `bundles-editor`) currently suffers from layout deficiencies, accessibility violations, improper empty-state design, and missing state representations:
1. **Fixed height & broken three-region layout (`.how/bundle/01-ux/DESIGN.md`):**
   - In `apps/desktop/src/components/BundleView.tsx`, the panels use a fixed/minimum height (`minHeight: '360px'`) and container flex-column wrapping instead of a full-height 3-region layout spanning 100% of the available viewport height (`height: 100%`). This leaves roughly a third of the window empty and dark beneath them at 1024×720 and higher resolutions.
   - The surface does not structure the 3 regions specified in `DESIGN.md`: Bundle list (240px, `--bundle-list-width`), Markdown preview (flex growing centre), and Item list (280px, `--item-list-width`).
   - Missing layout design tokens in `tokens.css`: `--bundle-list-width: 240px`, `--item-list-width: 280px`, and `--preview-line-height: 1.55`.
2. **The preview is not an accessible read-only region (`EXPERIENCE.md`, `NFR-16`):**
   - The Markdown preview must be a dedicated read-only region (`role="region"` with accessible name `aria-label="Markdown Preview"`), NOT a disabled `<textarea>` or `<input>`. A disabled control is announced to assistive technologies as unavailable, which is false: the content is available, it is the editing that does not exist (`FR-11`, Non-Goals).
   - The preview must have no cursor; a Bundle is recomposed, never patched, and an editable preview would silently break that invariant.
3. **Empty state offers an action button that only navigates away (`EXPERIENCE.md`):**
   - A Bundle is created exclusively from Findings on the Findings surface (`FR-9`, `FR-10`). An empty state button that pretends to create a bundle locally or merely navigates away violates `DESIGN.md`.
   - The surface must render a centered `EmptyState`: "No bundles yet", "Select findings on the Findings tab and choose Compose.", with NO action button.
4. **Missing state matrix implementations:**
   - `loading`: Skeleton placeholder rows in the list while preview and item list retain structural frame without layout shift.
   - `nothing-selected`: Bundle list populated, preview displays muted prompt ("Select a bundle to preview content."), item list empty/placeholder.
   - `item-missing`: When an item's image copy in the vault is missing, the item row in the item list carries a `--color-warning-bg` badge ("Missing"), while the preview and all actions continue to work (`an_item_whose_image_copy_is_missing_is_flagged_and_the_bundle_still_opens`).
   - `error`: Centered `ErrorState` ("The Library could not be read") with a single Retry button.
5. **Action placement & accessibility announcement (`FR-12`, `FR-14`, `DEC-005`):**
   - Actions must sit at the item list's foot, not floating over the preview. `Copy Markdown` is the primary action (`variant="primary"`); `Publish` is secondary (`variant="secondary"`, frozen per `DEC-005`); `Delete` is danger (`variant="danger"`).
   - `Copy Markdown` must visibly and accessibly announce its result via a toast or live region (`copy_markdown_announces_its_result`), because a silent clipboard write is indistinguishable from a silent failure.
   - `Delete Bundle` must trigger a confirmation dialog (`ConfirmDialog`) explicitly clarifying that the bundle's markdown and image copies will be removed from the vault while the original Findings remain intact in the library (`FR-14`).

**Approach:**
1. **Define Bundle Design Tokens in `web/ui/src/styles/tokens.css`:**
   Add layout tokens in `:root` and dark theme blocks:
   - `--preview-line-height: 1.55;` (Markdown preview monospace line height)
   - `--bundle-list-width: 240px;` (width of the left bundle list pane)
   - `--item-list-width: 280px;` (width of the right item list pane)
2. **Build Composite `BundlesEditor` in `web/ui/src/components/BundlesEditor.tsx` (`LC-014`):**
   - Create a clean 3-panel flex layout taking `height: 100%` and `width: 100%`:
     - **Bundle List (240px):**
       - Displays bundle items with title, item count, and formatted composed timestamp (e.g. `5 items · 12 Aug`).
       - Selection highlight using `--color-info-bg` / `--color-surface-raised` with accent border.
       - Keyboard focusable and navigable list items (`role="listbox"` / `role="option"` or semantic list).
     - **Markdown Preview (Flex Growing Centre):**
       - Landmark region (`role="region" aria-label="Markdown Preview"`).
       - Read-only `<pre>` / scrollable viewport without text cursor (`user-select: text; cursor: default;`).
       - Styled with `--font-mono`, `--text-sm`, `line-height: var(--preview-line-height)`, `background-color: var(--color-surface-sunken)`, and `color: var(--color-text)`.
       - Renders the exact CommonMark string that `copy_bundle_to_clipboard` outputs (`AD-9`, `FR-12`).
     - **Item List (280px):**
       - Displays bundle items in bundle order (`position: 1, 2, ...`).
       - Each row displays the ordinal badge and finding note first line / image filename.
       - If `item.is_missing` or image file is missing, displays a warning badge (`Badge variant="warning"`) while preserving bundle opening and preview functionality.
       - **Pinned Action Footer at Foot of Item List:**
         - `Copy Markdown` button (`variant="primary"` full-width or primary prominence).
         - Button row: `Publish` (`variant="secondary"`, disabled/frozen per `DEC-005`) and `Delete` (`variant="danger"`).
   - **State Matrix Handling:**
     - `empty`: 3 columns collapse to centered `<EmptyState heading="No bundles yet" description="Select findings on the Findings tab and choose Compose." />` with no button.
     - `loading`: Skeleton placeholder items in bundle list; preview and item list retain frame structure.
     - `nothing-selected`: Bundle list populated, preview displays muted selection prompt, item list inert.
     - `populated`: Full 3-column layout active with loaded bundle details and items.
     - `item-missing`: Warning badge on item row, preview and actions remain fully operational.
     - `error`: Centered `<ErrorState title="The Library could not be read" message="..." actionLabel="Retry" onAction={onRetry} />`.
3. **Refactor `apps/desktop/src/components/BundleView.tsx` Container:**
   - Mount `<BundlesEditor />` as the root component taking 100% height.
   - Connect to `listBundles`, `deleteBundle`, and `copyBundleToClipboard`.
   - Implement `handleCopyMarkdown`: writes to clipboard and triggers feedback toast / announcement.
   - Implement `handleDeleteBundle`: opens `ConfirmDialog` explaining that bundle files are deleted while source Findings remain in the Library.
4. **Implement Comprehensive Automated Vitest Suites:**
   - `the_preview_is_a_read_only_region_and_not_a_disabled_input`: Asserts preview has `role="region"` / accessible label, is not a `<textarea>` or `<input>`, and has no `disabled` attribute.
   - `the_empty_state_offers_no_button_that_only_navigates_away`: Asserts that when bundles is empty, `EmptyState` is rendered with heading and description, but contains no `<button>`.
   - `an_item_whose_image_copy_is_missing_is_flagged_and_the_bundle_still_opens`: Asserts that when an item is flagged missing, the warning badge is rendered in the item list, the markdown preview renders intact, and Copy/Delete actions remain enabled.
   - `bundles_renders_correctly_in_both_windows_themes`: Asserts contrast and design token compliance across light and dark themes using `tokens.css` parser.
   - `copy_markdown_announces_its_result`: Asserts that clicking Copy Markdown writes to clipboard and renders feedback notification / aria-live confirmation.

## Boundaries & Constraints

**Always:**
- All three panels MUST draw from `--color-surface` on `--color-bg` and take 100% available viewport height (`height: 100%`, `min-height: 0`) without fixed-height cutoffs (`DESIGN.md`).
- The preview MUST be a read-only region (`role="region"` with accessible name `aria-label="Markdown Preview"`), NOT a disabled `<textarea>` or `<input>` (`EXPERIENCE.md`, `NFR-16`).
- The empty state MUST offer NO action button that navigates away or pretends to create a bundle (`DESIGN.md`, `EXPERIENCE.md`).
- Actions MUST sit at the foot of the item list pane (`Copy Markdown` as `primary`, `Publish` as `secondary`, `Delete` as `danger`).
- `Publish` button MUST remain visible and frozen (`DEC-005`, `BR-120`, `LC-022`); do not modify the publishing API or backend publish paths.
- `Copy Markdown` MUST visibly and accessibly announce its result upon clipboard write (`NFR-16`).
- Deletion confirmation MUST explicitly state that original Findings remain intact in the library (`FR-14`, `EXPERIENCE.md`).
- When an item's image copy is missing, the row MUST carry a `--color-warning-bg` badge without preventing previewing, copying, or deleting the bundle (`DESIGN.md`).
- All styles MUST consume design tokens from `tokens.css` (`AD-10`); zero inline hex/rgb color literals permitted outside `tokens.css`.

**Block If:**
- Upstream requirements in `.what/`, `.how/`, or `.control/` contradict the 3-panel layout or design system tokens.
- Layout causes vertical scrolling of the entire surface at 1024×720 resolution.

**Never:**
- Do not make the preview an editable or disabled `<textarea>`/`<input>` control.
- Do not place action buttons inside the empty state.
- Do not float actions over the markdown preview; actions belong at the foot of the item list.
- Do not introduce color literals in TSX or CSS files (`AD-10`).
- Do not modify or unfreeze the `sharing` / `publish-dialog` path (`DEC-005`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Initial Load (Empty Library) | `bundles.length === 0`, `isLoading === false` | 3 panels collapse into centered `EmptyState` ("No bundles yet", "Select findings on the Findings tab and choose Compose.") with no button | Displays populated layout when bundle is created |
| Initial Load (Populated Library) | `bundles.length > 0`, default selection | First bundle selected; 240px bundle list, flexible markdown preview region, 280px item list with footer actions active | Fallback to next available bundle on delete |
| Loading State | `isLoading === true` | Bundle list renders skeleton row placeholders; preview and item list maintain structural layout | Transitions cleanly when load completes |
| Nothing Selected | `bundles.length > 0`, `selectedBundleId === null` | Bundle list active; preview displays muted "Select a bundle to preview content."; item list empty | Selecting any bundle restores details |
| Read-Only Markdown Preview | Valid bundle selected | Monospace CommonMark preview rendered inside `role="region" aria-label="Markdown Preview"`, no text cursor, scrollable | Unformatted raw bytes preserved |
| Missing Item Image File | Bundle contains item whose image copy is missing | Item row in 280px pane displays `--color-warning-bg` badge ("Missing"); preview and Copy/Delete actions remain functional | Item flagged without crashing surface |
| Copy Markdown Action | User clicks `Copy Markdown` button | Markdown copied to clipboard via `navigator.clipboard`; accessible toast / message announces success | Logs error and shows error toast if clipboard write fails |
| Delete Bundle Action | User clicks `Delete` button | Opens `ConfirmDialog` naming bundle and stating source findings remain in Library; on confirm, deletes bundle via IPC | Displays error toast if deletion fails |
| Publish Action | User clicks `Publish` button | Publish dialog / frozen status handled per `DEC-005` (`variant="secondary"`) | No unprompted sharing logic modifications |
| Library Store Failure | IPC `list_bundles` rejects | Surface renders centered `ErrorState` ("The Library could not be read") with Retry button | Retries fetch on click |

</intent-contract>

## Code Map

- `web/ui/src/styles/tokens.css` -- Adds `--bundle-list-width: 240px`, `--item-list-width: 280px`, and `--preview-line-height: 1.55` to `:root` and dark theme blocks.
- `web/ui/src/components/BundlesEditor.tsx` -- Pure UI component for Bundles surface (`LC-014`); renders 3-column layout (240px bundle list, flex markdown preview read-only region, 280px item list with pinned action footer), handles all 6 states (`empty`, `loading`, `nothing-selected`, `populated`, `item-missing`, `error`).
- `web/ui/src/index.ts` -- Exports `BundlesEditor`, types, and related props.
- `apps/desktop/src/components/BundleView.tsx` -- Desktop container for Bundles; connects to Tauri IPC services (`listBundles`, `deleteBundle`, `copyBundleToClipboard`), manages selection, deletion confirmation modal, and copy notification toast.
- `apps/desktop/src/services/bundle.ts` -- Tauri IPC service wrapper for bundle operations.
- `apps/desktop/src/App.tsx` -- Mounts `BundleView` within `EditorShell` under the `bundles` navigation tab.
- `web/ui/src/test/bundles_editor.test.tsx` -- Vitest tests for `BundlesEditor`: 3-panel layout, read-only region accessibility, empty state without button, missing item warning badge, skeleton loading, and pinned action footer.
- `apps/desktop/src/test/bundle_view.test.tsx` -- Desktop integration tests for `BundleView`: IPC data fetching, selection switching, copy announcement toast, and delete confirmation dialog.
- `web/ui/src/test/tokens.test.ts` -- Verifies bundle layout and preview line-height design tokens exist in `tokens.css`.
- `web/ui/src/test/contrast.test.ts` -- Verifies WCAG 2.2 AA contrast compliance across light and dark themes for bundle surface tokens.

## Tasks & Acceptance

**Execution:**
- [ ] `web/ui/src/styles/tokens.css` -- Add `--bundle-list-width: 240px`, `--item-list-width: 280px`, and `--preview-line-height: 1.55` under `:root` and `@media (prefers-color-scheme: dark)`.
- [ ] `web/ui/src/components/BundlesEditor.tsx` -- Implement `BundlesEditor` as a 3-column flex layout taking 100% viewport height (`height: 100%`):
  - **Bundle List (240px):** Title, item count, formatted composed date (`5 items · 12 Aug`), newest first, active selection indicator.
  - **Markdown Preview (Flex Growing Centre):** `role="region" aria-label="Markdown Preview"`, monospace text (`--font-mono`, `--text-sm`, `line-height: var(--preview-line-height)`), sunken surface (`--color-surface-sunken`), read-only without cursor.
  - **Item List (280px):** Items in bundle order (`1, 2, ...`) with ordinal badge and finding summary/note line; warning badge (`Badge variant="warning"`) for items with missing image copies.
  - **Pinned Action Footer:** `Copy Markdown` button (`variant="primary"`), `Publish` button (`variant="secondary"`, frozen), `Delete` button (`variant="danger"`).
  - **All 6 States:** `empty` (centered `EmptyState` without button), `loading` (skeleton placeholders), `nothing-selected` (muted prompt), `populated`, `item-missing` (badge on affected row while preview/actions work), `error` (centered `ErrorState` with retry).
- [ ] `web/ui/src/index.ts` -- Export `BundlesEditor` and associated TypeScript interfaces.
- [ ] `apps/desktop/src/components/BundleView.tsx` -- Refactor `BundleView` container:
  - Mount `<BundlesEditor />` taking `height: 100%`.
  - Connect IPC calls (`listBundles`, `deleteBundle`, `copyBundleToClipboard`).
  - Wire `handleCopyMarkdown` to write to clipboard and display accessible toast/feedback.
  - Wire `handleDeleteBundle` with `ConfirmDialog` stating that bundle files are deleted while source Findings remain in the Library.
- [ ] `web/ui/src/test/bundles_editor.test.tsx` -- Implement tests for `BundlesEditor`:
  - `the_preview_is_a_read_only_region_and_not_a_disabled_input`
  - `the_empty_state_offers_no_button_that_only_navigates_away`
  - `an_item_whose_image_copy_is_missing_is_flagged_and_the_bundle_still_opens`
  - `bundles_renders_correctly_in_both_windows_themes`
  - `copy_markdown_announces_its_result`
- [ ] `apps/desktop/src/test/bundle_view.test.tsx` -- Update `BundleView` integration tests covering IPC list/copy/delete workflows and confirmation dialog.
- [ ] `web/ui/src/test/tokens.test.ts` -- Add assertions verifying `--bundle-list-width`, `--item-list-width`, and `--preview-line-height`.

**Acceptance Criteria:**
- Given the Bundles surface, when rendered, the layout spans 3 columns (240px bundle list, flexible markdown preview region, 280px item list) taking 100% available height without fixed-height cutoffs or dark gaps beneath (`DESIGN.md`).
- Given a selected Bundle, the Markdown preview is rendered inside a read-only region (`role="region"` with `aria-label="Markdown Preview"`), is NOT a disabled `<textarea>` or `<input>`, and provides no text cursor (`EXPERIENCE.md`, `NFR-16`).
- Given an empty library, the Bundles surface renders a centered `EmptyState` ("No bundles yet", "Select findings on the Findings tab and choose Compose.") with NO action button (`DESIGN.md`).
- Given a Bundle containing an item whose image copy is missing, the item row in the 280px list carries a `--color-warning-bg` badge ("Missing"), while the preview and all actions continue to function normally.
- Given the item list, the actions sit pinned at the foot with `Copy Markdown` as `variant="primary"`, `Publish` as `variant="secondary"`, and `Delete` as `variant="danger"`.
- Given the `Copy Markdown` button, clicking it copies the Markdown bytes to clipboard and triggers a visible/accessible confirmation announcement (`copy_markdown_announces_its_result`).
- Given the `Delete` button, clicking it triggers a confirmation dialog confirming bundle file deletion while explicitly noting that source Findings remain intact in the library.
- Given both light and dark Windows themes, all tokens in `tokens.css` satisfy WCAG 2.2 AA contrast ratios (>= 4.5:1 for text), and zero color literals exist outside `tokens.css`.
- Given the full automated verification suite, all Vitest test suites, TypeScript typechecks, and ESLint checks pass with zero errors.

## Verification

**Commands:**
- `npm --prefix web/ui run typecheck` -- expected: TypeScript compiles cleanly with zero errors
- `npm --prefix web/ui run lint` -- expected: ESLint passes with zero warnings/errors (no colour literals outside tokens.css)
- `npm --prefix web/ui run test` -- expected: All Vitest suites pass in `web/ui` (including `bundles_editor.test.tsx`, `tokens.test.ts`, `contrast.test.ts`)
- `npm --prefix apps/desktop run typecheck` -- expected: Zero TypeScript errors in desktop app
- `npm --prefix apps/desktop run lint` -- expected: ESLint passes with zero warnings/errors in desktop app
- `npm --prefix apps/desktop run test` -- expected: All Vitest suites pass in `apps/desktop` (including `bundle_view.test.tsx`, `mount.test.tsx`, `shell.test.tsx`)
- `npm --prefix apps/desktop run build` -- expected: Production Vite build succeeds
- `cargo test --workspace` -- expected: All Rust crate tests pass

## Spec Change Log

- 2026-08-24: Created W6-S8 Step 1 PLAN specification for Bundles surface layout, read-only preview region, honest empty state, state matrix, and accessibility announcement.