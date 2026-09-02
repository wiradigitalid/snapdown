# 12: Hand a Bundle over from its row — Copy Markdown and Open file location

**What to build:** Hovering a Library row reveals its two everyday actions. **Copy Markdown** puts the
Bundle's **whole stored document** on the clipboard — same words, same order — with every image link
rewritten to an absolute path a local agent can open, in the form ticket 03 settled (forward slashes,
wrapped in `<>`); the rewriting is the composer rebasing its own document (ticket 10), never the UI
editing text. The toast follows the house pattern of saying what did and did not travel: it states
that the copied text carries the images' locations on this disk, because those paths include the
operator's user name and travel wherever the text is pasted. **Open file location** opens the Bundle's
own folder — its Markdown and image copies together — in the file manager, using the existing
open-a-folder path rather than the select-a-file one, because the object is the folder (`FR-43`). The
stored file itself is untouched and keeps its folder-relative links (`NFR-8`). This is the first
implementation `FR-12` has ever had.

**Blocked by:** 10 (the composer's rebase), 11 (the row)

**Status:** ready-for-agent

- [ ] Both actions appear on row hover and in the row's menu, as the artboards place them
- [ ] After Copy Markdown, the clipboard holds a document that differs from the stored one only in
      image link destinations, which are absolute, forward-slashed and `<>`-wrapped — the string
      handed to the clipboard is produced by the composer's rebase and asserted at that seam
- [ ] The Copy Markdown toast states that the text carries the images' disk locations
- [ ] Open file location opens the Bundle's folder (not its parent with the folder selected); a folder
      that no longer exists produces a toast saying so rather than silence
- [ ] Both work for a sealed Bundle (one whose original Findings are gone) exactly as for an unsealed
      one — nothing here reads a Finding
- [ ] The stored Markdown file is byte-identical before and after both actions
- [ ] The callback-reachability test covers both callbacks; the reachability test from ticket 11 is
      extended to their bindings
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** Copy Markdown, then paste into a real Markdown reader (VS Code preview, Obsidian)
      and confirm every image renders; paste into a plain text editor and confirm the paths and the
      `<>` are as expected. Open file location on a Bundle and confirm Explorer opens its folder with
      the Markdown and images inside. This is also the first observation `OQ-1` has ever had — note
      what happened
