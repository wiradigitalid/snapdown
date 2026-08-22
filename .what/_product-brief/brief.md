---
title: "Product Brief: Snapdown"
status: draft
created: "2026-08-22"
updated: "2026-08-22"
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

**Capture.** A global hotkey dims the screen. Drag a region. On release, a small note field appears
at the region — type the finding, press save. The overlay closes and the screen is yours again. Do
it again immediately; nothing has stolen focus and nothing had to be opened.

**Compose.** The Snapdown Editor holds every finding taken so far, each as an image with its note.
Notes are editable. Numbered markers can be placed on an image, and placing marker `3` creates line
`3.` in that finding's note, waiting for its sub-comment. Findings can be deleted, and when one goes
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

## Success Criteria

The measure: **median time to hand off a five-finding review, from first hotkey press to the agent
having all five findings in context, is under 120 seconds — with zero notes attached to the wrong
image.** Measured on the primary user's own reviews, monthly, three months after first release.

Supporting signals, none of which replaces the measure above:

- A bundle's images cost materially less to read than the raw captures they came from, at a quality
  the reviewer still accepts without going back to the original.
- Reviews are handed over in batches rather than one finding at a time.
- Deleting a finding or a bundle leaves no image file behind in the target folder.

## Scope

### Scope In

- Windows 11 desktop application, installable and runnable without an account.
- Region-select screen capture behind a user-editable global hotkey.
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
