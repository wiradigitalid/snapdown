# Assumptions

**Loaded when:** swept once per gate; MAY be skipped.

The **default** class for a question. The agent takes the answer itself and records it here, one
row: the assumption, plus the cost if it turns out wrong. This file **holds nothing**.

A row here MUST move up to `blocking.md` the moment it passes one of the three tests that file
states.

## Open

| id | Assumption | Cost if wrong | Taken | By |
|---|---|---|---|---|
| OQ-1 | A coding agent handed a Markdown file with relative image paths can open those images | The clipboard handoff — the primary path — is worthless, and MCP becomes the only one. Reopens G2 | 2026-08-22 | agent, G1 |
| OQ-2 | Agent reading cost tracks image pixel area closely enough that downscaling is the dominant compression lever, ahead of encoder choice | The compression settings optimise the wrong knob. BG-3 stays unmet while the feature looks done | 2026-08-22 | agent, G1 |
| OQ-3 | A UI screenshot at a 1600 px long edge, lossily re-encoded, stays legible enough that the reviewer does not reach for the original | The default is wrong, not the design. One settings default changes; nothing is rebuilt | 2026-08-22 | agent, G1 |
| OQ-4 | Numbered markers are sufficient annotation for a machine audience | Reviewers add shape annotations by hand elsewhere and the loop is broken again. Reopens the Scope Out list | 2026-08-22 | agent, G1 |
| OQ-5 | Windows global hotkeys can be registered from a user-level process without administrator rights | Capture needs elevation, which makes run-at-startup and the whole loop hostile. Reopens G3 for the capture component | 2026-08-22 | agent, G1 |
| OQ-6 | The reviewer prefers pasting a key per session over the agent holding standing access to the library | The key ceremony is friction with no buyer, and MCP goes unused | 2026-08-22 | agent, G1 |
| OQ-7 | An agent on a remote host can fetch an unlisted HTTPS URL and the images that URL references | BG-4 is unreachable by this design and the web surface needs a different shape | 2026-08-22 | agent, G1 |
| OQ-8 | An unguessable slug plus an optional read token is access control the owner accepts for published bundles | Publishing needs real authentication, which means accounts — and accounts are Scope Out | 2026-08-22 | agent, G1 |
| OQ-9 | Not auto-opening the editor after a capture is the behaviour the reviewer wants | One settings default flips. The setting exists precisely because this assumption may be wrong | 2026-08-22 | agent, G1 |
| OQ-10 | A single local SQLite store plus a folder of images is enough; no index, search, or full-text is needed at this size | The library becomes slow at some volume nobody has reached yet. Additive, not a rewrite | 2026-08-22 | agent, G1 |
| OQ-11 | The reviewer never needs two vault folders at once — one target folder at a time is sufficient | Moving between projects means changing a setting rather than switching a workspace. Friction, not breakage | 2026-08-22 | agent, G2 |
| OQ-12 | Recomposing a bundle is acceptable in place of editing its written Markdown | Bundles get edited outside Snapdown and drift from the library that produced them | 2026-08-22 | agent, G2 |
| OQ-16 | Composing a one-Finding Bundle is acceptable in place of publishing a single Finding directly | A single-screenshot handoff carries one extra step every time. The fix is a shortcut that composes behind the scenes, not a second publish path — additive, not a rewrite | 2026-08-22 | owner, G2 |

## Answered

None yet. A row leaves this table by moving to `answered.md`, never by being deleted.
