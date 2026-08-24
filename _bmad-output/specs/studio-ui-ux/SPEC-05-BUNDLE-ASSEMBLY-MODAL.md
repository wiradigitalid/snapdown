---
id: SPEC-05-BUNDLE-ASSEMBLY-MODAL
title: 3-Column Bundle Review, Handoff & Assembly Modal
status: ready-for-dev
source_prototype: .how/_platform/assets/ui-ux-complete-flow.html (State 4)
dedicated_html_asset: .how/bundle/01-ux/assets/04-bundle-assembly-modal.html
companions:
  - web/ui/src/styles/tokens.css
  - web/ui/src/components/BundleComposer.tsx
  - .what/bundle/SRS-bundle.md
  - .how/bundle/SDD-bundle.md
---

# SPEC-05: 3-Column Bundle Review, Handoff & Assembly Modal

## 1. Scope & Objective
Implements the unified **3-Column Modal Window (State 4)** for reviewing multiple findings, inspecting the generated CommonMark markdown document, selecting agent handoff channels, and executing atomic bundle assembly with automatic filmstrip queue cleanup.

---

## 2. 3-Column Modal Architecture (State 4)

```
+-------------------------------------------------------------------------------------------------------------------------+
| 📦 Review & Assemble Bundle                                                        [Est. Total: ~1,450 tk]  [✕ (Esc)]   |
+-------------------------------------------------------------------------------------------------------------------------+
| COLUMN 1: BUNDLE CONTENTS          | COLUMN 2: GENERATED MARKDOWN            | COLUMN 3: HANDOFF & ASSEMBLE             |
|                                    |                                         |                                          |
| Bundle Title:                      | # Review Findings: Checkout Defects     | 🚀 Handoff Channels                      |
| [Input: "Checkout Flow Defects"]   |                                         |                                          |
|                                    | ## 1. Payment Modal Bug                 | [📋 Copy CommonMark Markdown]            |
| Included Findings (3 items):       | ![Screenshot](bundles/item_1_burned.png)| Ready to paste into Claude/ChatGPT chat. |
| +--------------------------------+ | > **Summary**: Total price overlaps...  |                                          |
| | [Thumb 1] Payment Modal Bug    | |                                         | [🔑 Local MCP Server Bridge]             |
| | 3 markers (~520 tk)        [✕] | | **Annotations**:                      | `snapdown-bridge` on port 3849.          |
| +--------------------------------+ | - **[1]** Total price element overlaps..|                                          |
| | [Thumb 2] Missing Currency     | | - **[2]** CTA button clipped...         | [🌐 Publish Unlisted Web URL]            |
| | 1 marker  (~410 tk)        [✕] | |                                         | Generates edge-cached shareable URL.     |
| +--------------------------------+ | ## 2. Missing Currency Symbol           |------------------------------------------|
| | [Thumb 3] Error Banner Text    | | ![Screenshot](bundles/item_2_burned.png)| 📦 Execution Action                      |
| | 2 markers (~480 tk)        [✕] | | ...                                   |                                          |
| +--------------------------------+ |                                         | [ 📦 Assemble & Save Bundle ]            |
|                                    |                                         | (Freezes bundle & cleans up filmstrip)   |
+-------------------------------------------------------------------------------------------------------------------------+
```

---

## 3. Detailed Column Specs & Interaction Contracts

### Column 1: Bundle Contents & Thumbnail Inspection
- **Bundle Title Input**: Editable string defaulted to timestamp or first finding's summary.
- **Finding Rows**:
  - Click thumbnail: Triggers **Quick Lightbox Preview** and auto-scrolls/highlights the finding section in Column 2.
  - **`✕` (Exclude Button)**: Removes the finding from current bundle assembly and returns it to the active filmstrip queue without deleting the file.

### Column 2: Live Generated Markdown Document
- Live, read-only CommonMark preview rendered with syntax highlighting (`var(--font-mono)`).
- Formatted with pure relative image paths (`bundles/{bundle_id}/item_{pos}_burned.webp`) matching `AD-9` and `FR-11`.

### Column 3: Handoff Channels & Atomic Assembly
- **Channel 1 (`📋 Copy CommonMark Markdown`)**: Direct copy to OS clipboard.
- **Channel 2 (`🔑 Local MCP Server Bridge`)**: Displays active bridge command and port for local coding agents.
- **Channel 3 (`🌐 Publish Unlisted Web URL`)**: Publishes bundle to unlisted edge web reader.
- **Primary Assembly Action (`📦 Assemble & Save Bundle`)**:
  1. Creates permanent `Bundle` and `BundleItem` records in SQLite Vault.
  2. Burns markers onto screenshot copies in Vault (`snapdown-store::burner`).
  3. Serializes final CommonMark file to disk (`bundle.md`).
  4. Automatically deletes the assembled raw screenshots from the active filmstrip queue (Clean Workspace).
  5. Closes modal and navigates back to clean Studio workspace with a success toast notification.

---

## 4. Test Obligations
- `vitest::modal_renders_all_selected_findings_in_column_1`
- `vitest::clicking_thumbnail_triggers_quick_preview_and_scrolls_markdown`
- `vitest::excluding_finding_updates_markdown_and_total_token_sum`
- `vitest::assemble_and_save_cleans_up_assembled_findings_from_filmstrip`
- `vitest::copy_markdown_channel_copies_formatted_commonmark`
