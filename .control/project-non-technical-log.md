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
| NTL-3 | 2026-09-03 | legal | The desktop icon set is **Lucide**, licensed ISC, with one icon (`hard-drive.svg`) being the Feather original Lucide inherited, licensed MIT. Only `blur.svg` and `marker.svg` are drawn in-house. Traced 2026-09-03: six commits of 2026-08-26 name Lucide in their messages, 14 of the 25 Lucide-named files are byte-identical to `lucide-static` ≥ 0.300, the rest are the same geometry serialised differently or a Lucide icon under another name. Neither licence restricts closed or commercial distribution; both require the copyright and permission notice to appear in every copy | An obligation in force since 2026-08-26 and unmet until today: no `lucide`, `Feather` or `ISC` string existed anywhere in the repository, and the About tab named only Slint and IBM Plex. Discharged the same day by the About tab of the Settings screen, `apps/desktop/assets/icons/NOTICE.md` (the verbatim notice, beside the files it covers), and a README section; asserted by `the_about_tab_and_the_readme_carry_the_icon_attribution`. Any future icon added from another set MUST land in `NOTICE.md` too | `git log` of `apps/desktop-slint/assets/icons` (2026-08-26); path comparison against `lucide-static@0.400.0` and `feather-icons@4.29.0`; `LICENSE` of `lucide-static@0.400.0` |
| NTL-2 | 2026-08-28 | legal | Snapdown is MIT, copyright Wira Digital Indonesia, and `git log` shows a **single human author**. There are no external contributors | Full copyright ownership, so dual-licensing, relicensing or closing a future release are all still available. That position ends at the first external pull request accepted without a CLA | `LICENSE`, `git log --format=%ae \| sort -u`, checked 2026-08-28 |

## No longer in force

| id | Fact | Stopped holding | Superseded by |
|---|---|---|---|
| — | — | — | — |

**`NTL-1` discharged, 2026-08-28.** The Slint acknowledgement is in the product, on the About tab of
the Settings screen (`apps/desktop/ui/components/settings.slint`), naming the Royalty-free Desktop,
Mobile, and Web Applications licence and linking slint.dev. IBM Plex's SIL OFL 1.1 is acknowledged
beside it. Asserted by `the_about_tab_carries_the_slint_attribution` so it cannot be edited away
without a red test.
