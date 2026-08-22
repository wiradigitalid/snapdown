---
topic: Snapdown architecture spine and the G3 blueprint
artifact: .how/_platform/ARCHITECTURE-SPINE.md
updated: 2026-08-22T23:00
---

- (event) G3 blueprint written in one pass: spine, C4 L1/L2/L3, three inventories, cross-cutting, business rules, five SRS, five domain models, five SDD skeletons
- (decision) paradigm: hexagonal. Rust domain core with ports; every UI and every agent-facing surface is an adapter. One promise implemented once, three handoff paths as translations
- (decision) stack set by the owner mid-run: Tauri v2 + Rust; React + Vite + TypeScript for both front ends; Go with net/http and chi for web-api. Next.js and Express excluded by the owner
- (decision) nine AD. Trimmed from eleven: the UUIDv7 id convention and the UTC timestamp rule moved to cross-cutting.md, because cross-cutting is their described home and an AD would be a second version
- (decision) AD-1 exists because a Marker and its Note line are one thing. Everything else in this product is replaceable; that binding is the product
- (decision) AD-8 born specifically for : a Publication slug unrelated to every Library id, so one leaked URL is not a way to find the next
- (decision) AD-9 born because the pressure to render a Bundle slightly differently per surface is constant. A golden-file test across the three paths is the only thing that catches it
- (decision) four containers, all built: true. Embedded SQLite and the Vault folder are NOT containers — no runtime anyone deploys. web-ui IS one: a SPA runs in the reader's browser, which is its own process
- (decision) five Product Components: finding, bundle, settings, agent-access, sharing. Sliced by object, not by moment — so that a Note is born and edited inside one component and AD-1 never spans a seam
- (decision) ownership: finding owns Finding/Note/Marker, bundle owns Bundle/BundleItem, settings owns Setting, agent-access owns AccessKey, sharing owns Publication. platform_owns is empty on purpose
- (decision) mode: global outline. finding, agent-access, sharing raised to guarded; settings lowered to catalog, so its G4 is skipped by design
- (decision) risk_accepted low on four components, medium on settings. Nothing is high, so V23 needs no risk-acceptance DEC-
- (decision) 23 UC, four critical: UC-7 delete Findings, UC-12 delete a Bundle, UC-17 issue an Access Key, UC-20 publish. Every one is an irreversible action or a disclosure that cannot be recalled
- (decision) FR-4 image reduction carries no_uc with its reason: no actor, no initiating step. It is a property of UC-1 and UC-2, asserted by NFR-3
- (decision) 30 BR, all cross-component by construction. Local rules wait for G4
- (note) V25 reports four findings — no code map heading for the four built:true containers. Correct at G3: the registry holds the plan, the map holds the tree, and no folder may be listed before a file creates it. Closes at the first wave
- (note) two dangling citations in installed files are a METHOD defect, not ours: .control/structure-*.md templates cite .constitution/structure-guide.md, which lives at .constitution/method/structure-guide.md. Fixed in our re-derived maps; the templates in the package still carry it
- (note) the spine template's WDI override names .how/_platform/architecture/ as the home; architecture-guide.md repeals that folder and names .how/_platform/ARCHITECTURE-SPINE.md. The Accepted guide wins; the template disagrees and that is a method defect to report
- (event) spine excluded from doc_standards by design — reviewed manually through wdi-review before G3 closes
- (note) correction to the AD-8 entry above, which lost a word to shell quoting: AD-8 was born specifically for the sharing component. The log is append-only, so the entry stands and this is its fix
