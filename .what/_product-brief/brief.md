---
title: "Product Brief: Snapdown"
status: draft
created: "2026-08-22"
updated: "2026-08-23"
---

# Product Brief: Snapdown

## Executive Summary

Snapdown is a Windows desktop tool that turns visual review into something a coding agent can act
on. Press a hotkey, drag a region, type what is wrong with it. Do that five more times. Then select
the findings that belong together and Snapdown writes one Markdown file where every note sits under
the image it describes — with numbered markers on the image matching numbered lines in the note.

The gap it closes is small and it is paid for on every review. Today the reviewer screenshots,
pastes into a chat, types a note, screenshots again, pastes again — and within three findings
neither the reviewer nor the agent can still say which note belonged to which picture. The
attachment between an observation and its evidence is the thing being lost, and it is the only
thing that made the observation useful.

Snapdown keeps that attachment as a first-class fact, from the moment of capture through to the
handoff. The handoff has three shapes and they carry the same content: a Markdown file to paste, an
MCP server the agent reads on the same machine after the user hands it a key, and an unlisted web
URL for an agent that runs somewhere else. Because agents are billed by what they read, every image
is downscaled and re-encoded on the way in, under a budget the user sets once.

## The Problem

A developer working with a coding agent spends much of the day looking at running software and
finding things wrong with it. The findings are visual — this button is misaligned, this table shows
the wrong total, this empty state says nothing. Describing them in words costs more than showing
them, so the reviewer screenshots.

What happens next is the problem. The screenshot goes to the clipboard, into the agent chat, and
then a note is typed after it. On the second finding this still works. On the fourth, the reviewer
is scrolling back up to check which image they were describing, and the agent — which sees a flat
sequence of images and text — has no reliable way to bind note four to image four. So the reviewer
starts writing "in the third screenshot, the one with the sidebar…", which is the cost showing up
in plain sight: re-describing the image in order to point at it.

Three further costs ride along with it:

- **No batching.** Because attachment degrades with volume, reviews get handed over one finding at
  a time, and every one of them restarts the agent's context.
- **Token waste.** A full-resolution screenshot of a 4K monitor is an expensive thing to hand a
  model, and nothing in the current path reduces it.
- **Nothing is left behind.** The review existed only inside a chat transcript. There is no artifact
  to re-send, revise, or hand to a second agent, and no way to reach an agent that is not on the
  same machine as the screenshot.

The coping strategy today is a capture tool plus a folder plus manual pasting: three tools, none of
which knows that a note and an image are one thing.

## The Solution

One uninterrupted loop, then one handoff.

**Capture.** A global hotkey dims the screen. Precision crosshair guides, a magnifying loupe with live pixel grid/color readout, and intelligent auto-detection of windows and sub-panels (with dynamic cutout highlighting and a top-center Fullscreen shortcut) assist the Reviewer. The Reviewer can 1-click select a detected window/panel or drag a custom region, and re-select prior to saving. On release, a small note field appears at the region — type the finding, press save. The overlay closes and the screen is yours again. Do it again immediately; nothing has stolen focus and nothing had to be opened.

**Compose.** The Snapdown Editor holds every finding taken so far, each as an image with its note.
Notes are editable. Numbered markers can be placed on an image, and placing marker `3` creates line
`3.` in that finding's note, waiting for its sub-comment. The canvas also provides rich visual annotations
and privacy redactions — outlined Shapes, draggable Arrows, Callout bubbles with adjustable tail points,
floating Text with font customisation, and Blur redaction boxes — which are burnt into the visual image
without polluting or altering the Markdown note structure. Findings can be deleted, and when one goes
its image file goes with it.

**Hand off.** Select the findings that belong to one concern and Snapdown writes them into one
Markdown bundle — a named group with its own images, its own file, and its own lifetime. The bundle
is then read in whichever way suits the agent: copied as text, read over MCP from a key the user
pastes into the chat, or fetched from an unlisted URL by an agent running elsewhere.

Every image is downscaled and re-encoded on capture against a budget the user sets, because the
whole point is that an agent reads this.

## What Makes This Different

A general capture tool is better at capture and will stay better at it. Those tools are built for a
human audience — arrows, callouts, effects, a canvas. Snapdown is built for a machine audience, and
that changes what matters: the note-to-image binding has to survive serialisation into text, the
image has to be cheap to read, and the output has to be a file rather than a picture on a clipboard.

The honest differentiator is not technical. It is that the output format is the point rather than an
export option, and that the three handoff paths — clipboard, MCP, URL — are the same bundle rather
than three features. There is no moat here beyond building the loop properly and keeping it fast.

One tool is aimed at the same problem rather than a neighbouring one: Cobalt Capture
(`https://cobaltcapture.com/`), a browser-based visual feedback tool that publishes a review as a
public URL plus Markdown for a named list of coding agents. Two things about it are worth carrying.
It **deliberately avoids annotation-heavy markup** — a crop taken at capture time and an editable
paragraph beside each screenshot, and nothing else. That corroborates rather than threatens the
numbered-markers-only choice: a second team reached the same conclusion from the same audience.
And its own differentiator is **voice dictation of the note**, on the argument that speaking
captures the aside a reviewer would not have bothered to type. Snapdown has no answer to that and
does not currently need one, but it is the first idea to reach for if the note field proves to be
where the loop slows down.

Where Snapdown differs from it is the axis Cobalt gave up: Snapdown is local-first and installable,
the capture is a global hotkey rather than a browser tab, and nothing leaves the machine unless the
Reviewer publishes a named bundle. Cobalt's "no install, no signup" is a real advantage for a
first try and the opposite of what a constraint on personal data allows.

## Who This Serves

| Role | Need | Tier |
|---|---|---|
| Agent-assisted developer | Report several visual findings to a coding agent at once, without any of them losing the image it refers to | **primary** |
| Coding agent | Read a review as text plus images, with each note unambiguously bound to one image, at a token cost that does not crowd out the task | secondary |
| Agent running off the capture machine | Reach the same review over the network, without the reviewer copying files to a server by hand | secondary |
| Repo owner | A public repository that never carries a captured screenshot, a token, or a client's name | secondary |

Shared across all four: a finding and its evidence are one object, and every surface has to treat
them that way.

## Goals

- **BG-1** — core value: a reviewer hands an agent several findings at once, and every note is
  unambiguously attached to the image it describes.
- **BG-2** — capture to handoff happens in one pass, with no file management and no re-describing an
  image in order to point at it.
- **BG-3** — a handoff costs as few agent tokens as it can while its images stay legible.
- **BG-4** — the same review reaches an agent that is not running on the capture machine.
- **BG-5** — a review is disposable on purpose: a finding, or a whole bundle, leaves together with
  its image files and leaves nothing orphaned.
- **BG-6** — the tool never becomes the thing being managed: hotkeys, target folder, compression, and
  startup behaviour are set once and stay out of the way.
- **BG-7** — Snapdown's own surface costs the Reviewer no attention. Every screen is legible at a
  glance, every control is answerable without a manual, and no screen asks the Reviewer for a number
  they have no way to judge.
- **BG-8** — a review is readable by someone who does not have Snapdown: the document exported from a
  Bundle carries every Finding, in the composer's order, with its image and its note intact.

**BG-7's bar is split, and the split is the point.** It used to end *"the bar is the experience of
Snagit and of Cobalt Capture, on a product that does less than either"*, applied to the whole product.
That was put to the owner as `OQ-20` on 2026-09-01 and they split it. The **capture** half — the part
a person operates with their hands — keeps Snagit and Cobalt Capture as its benchmark. The **handoff**
half is measured against `BG-2` and `BG-3` instead, time and tokens, because its reader is a machine
and a human-tool benchmark spends effort there on affordances an agent cannot use. What the split buys
is checkability: *median handoff under 120 seconds* can be verified and *feels as good as Snagit*
cannot, and `DEC-005` lifts only on a bar that is **met and verified** by its own wording.

BG-6 and BG-7 look adjacent and are not the same goal. BG-6 is about **frequency** — a setting the
Reviewer touches once. BG-7 is about **cost per encounter** — what that one touch, and every capture
after it, actually demands of them. A product can fully satisfy BG-6 with a screen nobody can read.

## Success Criteria

The measure: **median time to hand off a five-finding review, from first hotkey press to the agent
having all five findings in context, is under 120 seconds — with zero notes attached to the wrong
image.** Measured on the primary user's own reviews, monthly, three months after first release.

Supporting signals, none of which replaces the measure above:

- A bundle's images cost materially less to read than the raw captures they came from, at a quality
  the reviewer still accepts without going back to the original.
- Reviews are handed over in batches rather than one finding at a time.
- Deleting a finding or a bundle leaves no image file behind in the target folder.

For BG-7, and stated separately because it is measured differently — not by timing the loop but by
watching a first encounter with it:

- **A Reviewer who has never seen Snapdown reaches their first handed-over bundle without being
  told how.** No screen is explained to them, and no control is defended.
- **Every setting can be answered from its own screen.** A control that requires the Reviewer to
  guess a number, or to leave and measure something, has failed regardless of its default.
- **Every text element meets WCAG AA contrast in both the Windows light and dark themes.** This is
  the one criterion here with a number, and it is checkable by a test rather than by opinion.
- **Nothing on a primary screen requires scrolling to be discovered.** Scrolling to read more is
  fine; scrolling to find out that something exists is not.

These four are how the condition in DEC-005 — "until the experience bar set at G2 is met" — is
actually checked. G2 sharpens them into `FR` and `NFR`; until it does, DEC-005 names a bar that
nobody can measure, and its own Cost section admits as much.

## Scope

### Scope In

- Windows 11 desktop application, installable and runnable without an account.
- Region-select screen capture behind a user-editable global hotkey, featuring full-screen precision crosshair guides, pixel loupe magnifier with live dimensions, smart window/panel auto-detection with dynamic cutout preview, and top-center Fullscreen shortcut.
- A note per capture, written at capture time and editable afterwards.
- Numbered markers on an image, bound to numbered lines in that finding's note.
- Automatic downscale and re-encode on capture, under settings the user controls.
- A user-chosen target folder for image files, with deletion of a finding deleting its file.
- Multi-select of findings, and composition into a named Markdown bundle.
- Bundles as the grouping unit: listed, re-openable, deletable together with their images.
- An MCP server the user grants access to by pasting a key, plus a loopback API behind it.
- Publishing a chosen bundle to an unlisted web URL, and a web service that serves it.
- Run-at-Windows-startup as a setting.

### Scope Out

- macOS and Linux builds.
- Video, GIF, and scrolling capture.
- OCR, or any reading of what the screenshot contains.
- Freehand drawing, arrows, blur, redaction, and callout shapes. Numbered markers are the only
  annotation.
- Editing the captured pixels after the fact — crop, rotate, resize.
- Team accounts, shared libraries, permissions, or any second user.
- Cloud sync of the whole library. Publishing is per bundle and per act.
- An agent initiating a capture. Snapdown is read by agents, never driven by them.
- A public index, gallery, or search over published bundles.
- Editing a bundle's Markdown after it is written. A bundle is recomposed, not patched.

## Constraints

- **Windows 11 is the only capture platform in the first release.** Forbids designing the capture
  path against a cross-platform abstraction before the Windows one is proven.
- **This repository is public.** Forbids committing a captured screenshot, a token, a client's name,
  or any test fixture derived from real capture output.
- **A captured screenshot may contain personal data.** Forbids automatic upload, background sync, or
  any publish the user did not perform on a named bundle.
- **An agent MUST NOT reach the library without the user handing it a key.** Forbids an always-on
  discovery endpoint, an unauthenticated loopback API, and MCP access that works before the key is
  pasted.
- **No account, sign-in, or network call is a precondition for capturing.** Forbids putting any part
  of the capture loop behind connectivity.
- **Deleting a finding deletes its image file.** Forbids a soft-delete that leaves the file on disk,
  and forbids a bundle that outlives the images it points at.
- **The desktop experience is finished before publishing and agent access are advanced.** Forbids a
  new capability wave on either while BG-7 is unmet, and forbids widening a surface that has already
  failed a first encounter. It does not forbid fixing a defect in what already shipped. Recorded as
  `DEC-005`; it lifts by its own terms when the BG-7 criteria above are verified, and needs no
  superseding decision to do so.
- **One installed executable carries the whole desktop product.** Forbids a second binary for the
  editor, and forbids the product and its window disagreeing about their own name. Recorded as
  `DEC-003`. The `mcp-bridge` is not an exception to this: an MCP client launches it, the Reviewer
  never does, and it owns no window.

## Assumptions

- A coding agent handed a Markdown file with relative image paths can open those images. If it
  cannot, the primary handoff path is worthless and MCP becomes the only one.
- Agent reading cost tracks image pixel area closely enough that downscaling is the dominant lever,
  ahead of encoder choice.
- A UI screenshot downscaled to roughly 1600 px wide and re-encoded lossily stays legible enough
  that the reviewer does not reach for the original.
- Numbered markers are sufficient annotation for a machine audience, and the arrows and callouts of
  a human-audience tool add nothing an agent can use.
- Windows global hotkeys can be registered from a user-level process without administrator rights.
- The reviewer is willing to paste a key into an agent chat to grant access, and prefers that to the
  agent having standing access.
- An agent on a remote host can fetch an HTTPS URL and the images it references.
- Snagit and Cobalt Capture are the right experience benchmark for BG-7. Both are built for a human
  reader; Snapdown's reader is a machine. If the benchmark is wrong, effort goes into affordances no
  agent can use, and BG-7 is met against the wrong standard. Filed as `OQ-20`.
- Four named Quality Budget presets are distinguishable enough that a Reviewer picks between them
  rather than leaving Auto forever. If not, Advanced and Custom are cost with no buyer. Filed as
  `OQ-18` and recorded as the reversal trigger in `DEC-004`.
- The Reviewer's want from MCP is unformed rather than unstated, so holding agent access still costs
  delay and not rework. Filed as `OQ-17`.

## Prerequisites

- A Rust toolchain and Node on the build machine. Satisfied.
- A host to run the web service on, reachable over HTTPS. Not satisfied.
- A domain or subdomain for published bundle URLs. Not satisfied.
- A Windows 11 machine for capture testing, because capture cannot be verified headlessly.
  Satisfied.

## Vision

Visual review stops being something that happens inside a chat window and becomes an artifact with a
shape. A reviewer accumulates findings the way they accumulate commits: cheaply, in passing, without
deciding up front what they are for. Bundles become the unit that gets handed around — to an agent,
to a second agent checking the first, to a colleague, to a ticket. The desktop app stays small and
the formats stay boring, so that whatever reads reviews in two years can still read these.
