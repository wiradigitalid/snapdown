# 03: Decide what Copy Markdown puts on the clipboard

**Type:** grilling
**Status:** open
**Blocked by:** None (can start immediately)

## Question

`Copy Markdown` is the Library's primary hand-off action — the way a Reviewer gets a Bundle into a
coding agent or a chat. Decide exactly what lands on the clipboard.

- **The text itself.** The stored `Bundle.markdown` verbatim, or something rewritten on the way out?
- **The image problem.** Images cannot ride a text clipboard. The pasted Markdown will carry image
  links that only resolve for a reader with filesystem access to this machine's Vault. Decide
  whether the links are left as stored, rewritten to absolute paths so a local agent can open them,
  or something else — and what the Reviewer is told, if anything, about images not coming along.
- **Whether the Reviewer is warned at all**, or whether a plain "Copied" toast is enough.
- **Relationship to `Open file location`.** These two are siblings: one hands over text, the other
  hands over the file. Decide whether the answer here makes one of them redundant, or whether they
  serve genuinely different moments.

## Owner's answer, 2026-08-31 — narrows this ticket

**Copy Markdown puts the whole document on the clipboard with image links rewritten to absolute
paths.** The images stay where they are in the Vault; the absolute links let a local agent open them
from wherever it is working.

Tested and it holds, with one correction and one caveat:

- **It does not conflict with `NFR-8`.** That requirement governs the *stored* Markdown file, which
  keeps its folder-relative links. The clipboard is a different artifact with a different reader, so
  a different path convention is legitimate. Implement it as one composer with a base-path
  parameter, not a second copy of the serializer.
- **Windows absolute paths are not valid in Markdown as-is.** Backslashes are escape characters and
  spaces terminate a link, and the Vault may well sit in a path containing spaces:

  ```
  ![Finding 1](C:\Users\<user>\Snapdown Vault\bundles\b1\finding_1_burned.png)   broken twice over
  ![Finding 1](<C:/Users/<user>/Snapdown Vault/bundles/b1/finding_1_burned.png>) works
  ```

  Forward slashes plus `<>` wrapping, or percent-encoding, or a `file:///` URI. Whichever is chosen
  must be proven against a Vault path containing a space.
- **Caveat worth a decision:** an absolute path exposes the operator's OS username. Pasted into a
  shared or public chat, that goes with it.

**An environment variable was considered and rejected for this.** The proposal was to have Snapdown
export `SNAPDOWN_VAULT_PATH` globally and have the copied Markdown reference it instead of a real
path, keeping the operator's username out of pasted text. It fails on its first requirement: **no
CommonMark renderer expands variables**, so `$SNAPDOWN_VAULT_PATH/bundles/...` renders as literal
text and the link is simply broken. It also inherits three problems — already-running processes never
see a newly set variable, the Vault can move (`vault_migration.rs`) leaving the variable silently
stale, and writing to the user's environment is a system-level side effect that outlives the app.

Decisive point: for the clipboard, images **never** render for a human reader anyway — no chat client
loads local files. The only consumer of these links is a local agent, and an agent can open an
absolute path directly, with no expansion step. The variable adds a layer only an agent could
resolve, for a problem only an agent has, which the absolute path already solves. The idea is worth
taking up separately as a way for an agent to *find* the Vault — that is `agent-access` territory,
recorded in the map's Out of scope.

**What is still open:** which of the three encodings above to emit; whether a plain "Copied" toast
suffices or the Reviewer is told images are not on the clipboard; and whether this makes
`Open file location` redundant or leaves it serving a genuinely different moment.

**The prerequisite has landed.** The stored Markdown's image links were resolving one folder too
deep and loading nothing; fixed 2026-08-31 as `BUG-86`, so the stored document is now genuinely
portable and links read `./finding_N_burned.png`. Decide this ticket against that behaviour.

**Implementation note, not a decision:** the app already links `clipboard-win`, but every existing
use is `raw::set_bitmap_with` for images (`main.rs:2917-2922`). Copying text would be the first text
clipboard call in the codebase — small, and Windows-only like the rest, with a `#[cfg(not(windows))]`
stub alongside.
