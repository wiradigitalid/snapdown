# Project Non-Technical Log

**Loaded when:** a non-technical fact that constrains this product's build changes, or when finding
out what already holds outside the code.

Written through the `wdi-log` skill's `fact` intent, never by hand directly.

One-sentence test: **if the number were deleted and the fact would still be useful to whoever is
building, it belongs here.**

Categories: `domain` · `account` · `legal` · `schedule` · `organisation`.

## In force

| id | Date | Category | Fact | Effect | Source |
|---|---|---|---|---|---|
| NTL-1 | 2026-08-28 | legal | Slint 1.17.1 is tri-licensed and this product uses the **Royalty-free 2.0** branch. Nothing in it forbids selling the application; what it forbids is distributing Slint on its own, embedded use, and exposing Slint's own API. Its §2 **requires attribution**: either the `AboutSlint` widget on an About screen reachable from the main menu, or a "Made with Slint" badge on the public page the binary is downloaded from | An obligation **already in force**, and unmet: a search for `AboutSlint`, `Made with Slint`, `slint-ui` and `sixtyfps` across the whole repository returns nothing, and the product has no About screen at all (`BUG-61`). The cheap half is a badge on the release page; the `AboutSlint` widget lands when the Settings/About surface is built | licence text of `slint 1.17.1`, checked 2026-08-28 |
| NTL-2 | 2026-08-28 | legal | Snapdown is MIT, copyright Wira Digital Indonesia, and `git log` shows a **single human author**. There are no external contributors | Full copyright ownership, so dual-licensing, relicensing or closing a future release are all still available. That position ends at the first external pull request accepted without a CLA | `LICENSE`, `git log --format=%ae \| sort -u`, checked 2026-08-28 |

## No longer in force

| id | Fact | Stopped holding | Superseded by |
|---|---|---|---|
| — | — | — | — |
