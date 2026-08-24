---
id: SPEC-06-SAVED-BUNDLES-DRAWER
title: Saved Bundles History Drawer & Vault Management
status: ready-for-dev
source_prototype: .how/_platform/assets/ui-ux-complete-flow.html (State 5)
dedicated_html_asset: .how/bundle/01-ux/assets/05-saved-bundles-drawer.html
companions:
  - web/ui/src/styles/tokens.css
  - web/ui/src/components/BundlesDrawer.tsx
  - .what/bundle/SRS-bundle.md
  - .how/bundle/SDD-bundle.md
---

# SPEC-06: Saved Bundles History Drawer & Vault Management

## 1. Scope & Objective
Implements the **Saved Bundles History Drawer (State 5)** accessed via the top titlebar button (`📚 Bundles History`) or shortcut `Ctrl + H`:
1. **Slide-Out Overlay Drawer**: Sliding in from the right edge with backdrop blur.
2. **Saved Bundle Cards**: Displays saved bundles from SQLite Vault with finding counts, token sizes, timestamps, and quick action icon buttons (`📋` Copy MD, `👁️` View/Re-share, `🗑️` Hard Delete).
3. **Delete Confirmation Dialog**: Native OS/modal confirmation before permanently removing bundle records and associated burned image files.

---

## 2. Drawer Architecture & Layout (State 5)

```
+-----------------------------------------------------------------------------------------------+
|  SNAPDOWN STUDIO WORKSPACE (Dimmed Backdrop)                | 📚 SAVED BUNDLES HISTORY [✕]   |
|                                                             +---------------------------------+
|                                                             | [🔍 Search bundles...]          |
|                                                             +---------------------------------+
|                                                             | BUNDLE CARD #1                  |
|                                                             | 📦 Checkout Flow Defect Report  |
|                                                             | 📅 Aug 24, 2026 · 3 findings    |
|                                                             | 🪙 ~1,450 tokens                |
|                                                             |                                 |
|                                                             | [📋 Copy MD] [👁️ View] [🗑️ Del] |
|                                                             +---------------------------------+
|                                                             | BUNDLE CARD #2                  |
|                                                             | 📦 Settings Dark Mode Audit     |
|                                                             | 📅 Aug 23, 2026 · 5 findings    |
|                                                             | 🪙 ~2,100 tokens                |
|                                                             |                                 |
|                                                             | [📋 Copy MD] [👁️ View] [🗑️ Del] |
|                                                             +---------------------------------+
+-----------------------------------------------------------------------------------------------+
```

---

## 3. Interaction & Functional Requirements

### FR-HIST-1: Drawer Toggle & Loading
- Clicking `📚 Bundles History` in Titlebar or pressing `Ctrl + H` opens the drawer (`transform: translateX(0)`).
- Queries `get_bundles()` from SQLite Vault store and lists bundles sorted newest first.
- Pressing `Esc` or clicking the backdrop closes the drawer.

### FR-HIST-2: Icon Actions on Saved Bundle Cards
- **`📋 Copy MD` (Copy Markdown)**: Copies the full CommonMark document of the bundle to the clipboard immediately and displays a subtle confirmation toast (`"Copied bundle markdown to clipboard"`).
- **`👁️ View` (View & Re-share)**: Opens the 3-Column Modal (`SPEC-05`) in read-only inspection mode to review items, re-copy links, or test local MCP handoff.
- **`🗑️ Delete` (Hard Delete with Confirmation)**:
  - Prompts a confirmation dialog: *"Are you sure you want to permanently delete bundle '{title}' and its {n} exported screenshots?"*.
  - Upon confirmation, invokes `delete_bundle(bundle_id)`:
    - Removes bundle and item rows in SQLite (`W6-S9`, `FR-14`).
    - Removes stored WebP/PNG burned image files from the Vault filesystem directory (`vault-fs`).
    - Animates card removal from the history list.

---

## 4. Test Obligations
- `vitest::drawer_slides_in_and_lists_saved_bundles_from_store`
- `vitest::copy_md_button_copies_saved_bundle_markdown`
- `vitest::view_button_opens_bundle_review_modal`
- `vitest::delete_button_prompts_confirmation_and_deletes_bundle_files`
- `vitest::pressing_esc_closes_saved_bundles_drawer`
