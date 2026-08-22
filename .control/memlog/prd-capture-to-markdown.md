---
topic: Capture to Markdown — the desktop review loop
artifact: .what/_prd/capture-to-markdown/prd.md
updated: 2026-08-22T22:42
---

- (event) headless run, intent create. Two initiatives split by the reader test: someone looking for MCP or web publishing would not open this document
- (decision) CAP-1..CAP-6, FR-1..FR-18, NFR-1..NFR-8, UJ-1..UJ-4 allocated from requirements.yaml; the sequence continues into the agent-handoff PRD
- (decision) Note is written at capture time, inline at the region. Wins on every criterion set — recorded in the addendum because a non-trade-off should not be revisited as one
- (decision) editor does not auto-open after a capture; setting exists, default off
- (decision) image reduction happens once on the way in; the unreduced capture is not retained
- (decision) Marker coordinates are stored, not burned into the Finding image, because FR-8 requires repositioning and renumbering
- (decision) selection order is the only ordering inside a Bundle; no reorder step in r1
- (decision) hard deletion only. NFR-5 states the no-orphan property as an invariant, and FR-15 reports violations
- (assumption) not auto-opening the editor is what the Reviewer wants — OQ-9
- (assumption) reading cost tracks pixel area, so the long-edge cap is the dominant lever — OQ-2
- (assumption) recomposing is acceptable in place of editing a Bundle's Markdown — OQ-12
- (gap) default long edge of 1600px is a working answer, unmeasured against a real agent's reading cost — OQ-3
- (change) stack answered mid-run by the owner: Tauri v2 + Rust, desktop UI React + Vite + TypeScript (Svelte was the earlier plan). Lands as AD-N at G3, not in this PRD
- (event) PRD and addendum written; review lenses structure+prose applied at write time
