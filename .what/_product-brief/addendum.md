---
type: addendum
parent: brief
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Addendum — product brief

Depth that earned a place but would derail the brief's narrative. Nothing here is a promise; the
brief and the PRDs carry those.

## Rejected alternatives

| Option | Why it lost |
| --- | --- |
| Browser extension instead of a desktop app | The findings are not all in a browser. Terminal output, native windows, and the editor itself are exactly where the interesting defects are, and an extension cannot see them. |
| A clipboard watcher — keep the existing capture tool, catch what it copies | Nothing in the clipboard carries a note, so the binding this product exists for would have to be reconstructed after the fact from timestamps. It also makes the product a parasite on a tool the user might stop paying for. |
| Freehand annotation without structured binding | Plain freehand drawings that try to act as finding descriptions fail serialisation into text. However, vector canvas overlays (transparent outlined shapes, directional arrows, callouts, floating text, and blur redactions) that serve strictly as visual image context and privacy protection — while keeping numbered markers as the sole structured finding bindings — are supported and rendered directly to image layers. |
| Attach findings to the chat one at a time, but auto-caption them | Auto-captioning replaces the reviewer's observation with a description of the pixels. The reviewer's judgement is the payload; describing the image is the cost being removed, not the value. |
| Publish every capture to the web automatically, and drop the local library | Captures can contain personal data, and an automatic upload makes that the default. It also makes capture depend on connectivity, which breaks the one part of the loop that has to be instant. |
| Store notes inside the image file's metadata rather than a database | Survives file moves and needs no store — but multi-select, bundles, marker ordering, and publish state all need queries, and re-parsing every file in a folder to answer them is a database with worse ergonomics. |
| OCR the capture so an agent can read it as text | Doubles as a reason to skip images entirely, which is wrong: the layout, the alignment, and the visual defect are the finding. OCR of a UI screenshot also fails exactly where UI defects live. |
| One long Markdown file per day rather than named bundles | Cheap to write and useless to hand over. A handoff has a subject; the grouping is what makes the bundle worth reading, and the user asked for focused groups explicitly. |
| Let an agent trigger a capture over MCP | Turns Snapdown into a remote-control surface for whatever holds the key, on a machine full of personal data. The value is the reviewer's judgement at capture time, which an agent cannot supply. |

## Options weighed

### How the handoff reaches the agent

Criteria, fixed before scoring: works with an agent that has no MCP support; works when the agent
runs on another host; requires no standing access to the library; costs nothing to operate.

| Path | No MCP needed | Works off-machine | No standing access | Zero ops |
|---|---|---|---|---|
| Copy the Markdown and paste it | yes | yes | yes | yes |
| Write the bundle to a folder the agent already reads | yes | no | no | yes |
| MCP server the agent reads after a key is pasted | no | no | yes | yes |
| Unlisted web URL the agent fetches | yes | yes | yes | no — needs a host |

No single row wins, and that is why all three shipped paths exist. The second row was rejected as a
feature: writing into a repo the agent watches means the library is standing-readable, which the
brief's fourth constraint forbids.

### Image cost reduction

Criteria: predictable effect on agent reading cost; text in a UI screenshot stays legible; no
dependency the desktop app has to ship a codec licence for.

Downscaling to a maximum long edge is the dominant lever and is applied first. Re-encoding is second
and is worth having, but a format choice that gains a few per cent while risking legibility is not.
The setting the user actually needs is one slider over pixel budget plus one over quality, with
defaults chosen so the reviewer never touches either. Concrete defaults belong in the PRD as an
`NFR`, not here.

## Mechanism and transport

Not a design. The SDD owns that, and a builder MUST NOT follow this section.

Thinking that surfaced while deciding the promises:

- The overlay has to cover every monitor, and a per-monitor transparent window is the shape that
  does not fight the compositor. A single window spanning a multi-monitor virtual desktop behaves
  badly when the monitors have different scale factors.
- The note field appearing at the region rather than in a window is what keeps the loop
  uninterrupted. A dialog that takes focus and has to be dismissed is the behaviour the product is
  reacting against.
- The MCP surface needs to work for clients that only speak stdio, while the library lives in a
  running desktop process. That implies two pieces rather than one, and the key the user pastes is
  what joins them.
- Marker numbering and note-line numbering are one sequence, not two that are kept in sync. Anything
  that treats them as two will drift.
- Publishing is an act on a bundle, with a record of when it happened and what URL it produced.
  Nothing about it is a sync.

## Sizing

Nothing sized yet. Wave sizing happens at G4/G5 against the story list, and a number written here
before the use cases exist would reappear later as a commitment nobody made.

The one figure worth recording, because it drives the compression default: a full-screen capture on a
3840 × 2160 monitor is roughly 8.3 megapixels, and the same view at a 1600 px long edge is roughly
1.4 megapixels — about a sixth. Source: the primary user's own monitor resolution, not a benchmark.

## Personas and research detail

The primary user is the repo owner, reviewing their own and their team's software while working
alongside coding agents for most of the day. Two facts about them shape the product more than any
persona detail:

- They review in bursts. Findings arrive four or five at a time, minutes apart, and the loop has to
  survive being run six times in ninety seconds.
- Their agents are not all in one place. Some run on the capture machine, some on a remote host, and
  the same review has to reach both without being prepared twice.

No external research was commissioned, and `_bmad-output/` holds no run folder for this brief. The
brief's claims about the current coping strategy come from the primary user's own account of it and
are recorded as such rather than as findings.
