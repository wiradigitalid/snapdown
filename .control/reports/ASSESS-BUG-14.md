---
type: assessment
subject: BUG-14
status: Reference
created: "2026-08-24"
---

# What fixing BUG-14 involves

Written so the owner can decide with facts rather than with a paragraph. **This opens nothing and
changes no code.** Whether a wave follows is theirs.

## The architecture already planned for this, and the crate was never built

`ARCHITECTURE-SPINE.md:245` lays out the workspace:

```
snapdown-capture/    # screen capture, overlay geometry, image reduction
```

and assigns capabilities to it:

| Capability | Owner per the spine |
|---|---|
| `CAP-1` Capture | **`snapdown-capture`**, `apps/desktop/src-tauri` |
| `CAP-2` Image reduction | **`snapdown-capture`** |

**`crates/` contains `snapdown-bridge`, `snapdown-core` and `snapdown-store`. There is no
`snapdown-capture`.** Two capabilities are assigned to a crate that does not exist.

`components.yaml` registers `LC` entries for `region-capturer` and `capture-note-field`. Neither has
an implementation.

So this is not a design gap. The design is right and was never built, and nothing noticed because
nothing decoded an image.

## Three simulations, not one

`BUG-14` was registered on the first two. The third was found while writing this.

| Where | What it actually does |
|---|---|
| `commands/capture.rs:197` `generate_placeholder_image` | Writes 17 bytes: PNG signature, width, height, quality. No IHDR, IDAT or IEND |
| `store/src/image/pipeline.rs:26` `reduce_image` | Same fake header, then copies `input_bytes[16..]` through. Its own comment says *"downscaled payload simulation"* |
| **`store/src/image/burner.rs:20` `burn_markers`** | **Same fake header. Markers are never drawn onto anything.** Its test asserts only that the first 8 bytes are a PNG signature and the dimensions are preserved |

The third matters because `AD-4` — *an image is reduced exactly once, at capture, and no original is
kept* — governs the burn, and `FR-8`'s promise that a handed-over Bundle carries its Markers **on the
image** depends entirely on it.

## What a fix has to touch

**New dependencies.** A screen-capture crate and an image codec. Neither exists in the workspace today;
the complete dependency list is `serde serde_json thiserror uuid chrono rusqlite rand tempfile
tiny_http ureq base64 tauri tauri-build tauri-plugin-{single-instance,global-shortcut,autostart}
winreg`.

**A crate that should exist.** `snapdown-capture`, per the spine. Building it where the architecture
already put it costs nothing extra and closes the gap between the document and the tree.

**Three implementations**, in this order — each is useless without the one before it:

1. a real grab of the requested region, replacing `generate_placeholder_image`
2. a real decode-scale-encode, replacing `reduce_image`
3. a real draw, replacing `burn_markers`

**The dimension arithmetic already written is correct and must be kept.**
`compute_reduced_dimensions_for_pair`, the `Auto` resolution `W6-S4` just built, and the fixed presets
are all right. They are the only part of this that works, and they are what the new code plugs into.

## What breaks when real images arrive, and this is the expensive half

Every test in this repository asserts **dimensions, never pixels**. A fake PNG passes all of them.
When real bytes arrive, a large number of assertions become either wrong or meaningless — and the
dangerous outcome is not that they fail. It is that they keep passing while proving nothing.

Files that assert on image bytes or dimensions today:

```
core/src/domain/{finding,image,markdown,setting}.rs
core/tests/test_markdown_serializer.rs
store/src/image/{burner,pipeline}.rs
store/src/sqlite/{finding_store,migrations,settings_store}.rs
store/src/vault/sweeper.rs
store/tests/{test_bundle_deletion,test_golden_markdown,test_image_reduction,test_orphan_sweeper}.rs
```

**`test_golden_markdown.rs` is the sharpest case.** `AD-9` promises a Bundle copies the *same bytes*.
A golden test over fabricated bytes proves byte-identity of a fabrication.

## Sizing, honestly

This is **wave-sized, not story-sized**, and the reason is the test surface rather than the three
implementations. Writing a real capture path is ordinary work. Converting a suite that measures
dimensions into one that measures images — without landing on assertions that cannot fail, which this
project has now done three times — is the part that needs care.

It also has a dependency the earlier waves did not: **it is the first work here that cannot be
verified without looking at a picture.** `OQ-24` records four dispatched UI verifications and four
failures, and the fifth cannot be attempted usefully until this lands, because there is currently
nothing to photograph.

## What this does not decide

Whether to do it at all, when, and against which release. `DEC-005` freezes `sharing`; nothing here
touches it. The owner's call.
