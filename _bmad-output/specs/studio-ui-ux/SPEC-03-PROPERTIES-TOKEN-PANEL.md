---
id: SPEC-03-PROPERTIES-TOKEN-PANEL
title: Right Properties Panel & Multimodal Token Breakdown
status: ready-for-dev
source_prototype: .how/_platform/assets/ui-ux-complete-flow.html (State 1)
dedicated_html_asset: .how/finding/01-ux/assets/01-studio-workspace.html
companions:
  - web/ui/src/styles/tokens.css
  - web/ui/src/components/FindingsEditor.tsx
  - .what/finding/SRS-finding.md
  - .how/finding/SDD-finding.md
---

# SPEC-03: Right Properties Panel & Multimodal Token Breakdown

## 1. Scope & Objective
Implements the dedicated full-height right panel (`width: 440px`, spanning from top ribbon to bottom window edge) focused on text observation entry and multimodal AI token budgeting.

---

## 2. Panel Component Sections

```
+-------------------------------------------------------------+
| 📝 NOTES & MARKERS                      [Finding #FND-001]  |
+-------------------------------------------------------------+
| OBSERVATION SUMMARY                                         |
| +---------------------------------------------------------+ |
| | [Textarea: High vertical room (~110px)]                 | |
| | "User reported payment modal failure on mobile..."      | |
| +---------------------------------------------------------+ |
+-------------------------------------------------------------+
| STEP MARKER NOTES (1:1 with Canvas Badges)                  |
| +---------------------------------------------------------+ |
| | (1) [Textarea: "Total Price element overlaps..."]   [🗑️]| |
| | (2) [Textarea: "CTA button clipped off-screen..."]  [🗑️]| |
| | (3) [Textarea: "Missing currency symbol..."]        [🗑️]| |
| +---------------------------------------------------------+ |
| [ + Add Step Marker Note ]                                  |
+-------------------------------------------------------------+
| 🪙 ESTIMATED LLM TOKENS                                     |
| +---------------------------------------------------------+ |
| | 🖼️ Image Resolution (1600x900 WebP)           ~420 tk  | |
| | 📝 Text Notes (Summary + 3 Markers)           ~115 tk  | |
| |---------------------------------------------------------| |
| | Total Finding Cost                            ~535 tk  | |
| +---------------------------------------------------------+ |
+-------------------------------------------------------------+
```

---

## 3. Functional & Calculation Rules

### FR-PROP-1: Observation Summary
- High textarea (`min-height: 100px`, `resize: vertical`).
- Auto-saves changes to the active finding record in SQLite store with 300ms debounce.

### FR-PROP-2: Step Marker Notes Synchronisation (`AD-1`)
- Renders an ordered list of marker note items:
  - Leading badge: Amber badge `(1)`, `(2)`, `(3)` matching canvas markers.
  - Multi-line textarea for observation notes.
  - Trailing `🗑️` delete button to delete both note line and corresponding canvas marker.
- Focusing a note textarea automatically highlights the corresponding marker on canvas.

### FR-PROP-3: Multimodal Token Budgeting Formula
- Live calculation displayed in the token summary box:
  $$\text{Image Tokens} = \left\lceil \frac{\text{Width} \times \text{Height}}{750} \right\rceil \quad (\text{standard Claude/GPT-4o vision grid estimate})$$
  $$\text{Text Tokens} = \left\lceil \frac{\text{Total Characters in Summary and Marker Notes}}{3.8} \right\rceil$$
  $$\text{Total Tokens} = \text{Image Tokens} + \text{Text Tokens}$$

---

## 4. Test Obligations
- `vitest::properties_panel_syncs_notes_with_canvas_marker_count`
- `vitest::typing_in_marker_note_updates_finding_state`
- `vitest::deleting_marker_note_removes_canvas_marker_and_reindexes`
- `vitest::token_estimator_calculates_accurate_image_and_text_budget`
