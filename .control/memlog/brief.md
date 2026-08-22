---
topic: Snapdown — screenshot findings composed into Markdown for coding agents
artifact: .what/_product-brief/brief.md
updated: 2026-08-22T22:34
---

- (event) headless run, intent create — owner authorised writing and approval of G1..G5 in one pass without further questions
- (note) owner brain dump captured verbatim as the only source; no research subagents spawned, no _bmad-output run folder exists for this brief
- (decision) one problem: the note-to-image binding is lost when several visual findings are handed to a coding agent
- (decision) one primary user: the agent-assisted developer. The coding agent is a secondary consumer, not the primary
- (decision) one measure: median handoff time for a five-finding review under 120 seconds with zero mis-attached notes
- (decision) editor does NOT auto-open after a capture; default is a toast with an Open action. Owner delegated the call and the loop has to survive six runs in ninety seconds
- (decision) numbered markers are the only annotation. Arrows and callouts are invisible to the reader that matters
- (decision) three handoff paths ship because no single one covers all four criteria — see addendum, Options weighed
- (assumption) agent reading cost tracks pixel area, so downscaling is the dominant compression lever
- (assumption) a coding agent can open relative image paths from a Markdown file it is handed
- (assumption) Windows global hotkeys register from a user-level process without administrator rights
- (decision) delete is hard: a finding leaving takes its image file with it. No soft delete
- (decision) publishing is an act on a named bundle, never a sync. Captures may contain personal data
- (gap) no host and no domain yet for the web service — both filed as external prerequisites
- (note) commercial content deliberately excluded: repo is public, per repo-guide.md content boundary
- (event) brief and addendum written; G1 review lenses structure+prose applied at write time
