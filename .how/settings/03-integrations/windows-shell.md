---
type: integration
component: settings
third_party: Microsoft Windows 11
created: "2026-08-23"
updated: "2026-08-23"
---

# Integration — the Windows shell

The only third party this component has. It is written down because the owner is outside the team and
can change it without telling anyone, which is the whole reason this slot exists.

| | |
|---|---|
| **Owner** | Microsoft. Not contactable, not negotiable, not versioned on our schedule |
| **Reached through** | Win32 APIs directly — the `windows` crate, `global-hotkey`, and `tray-icon` — since `DEC-007`. Reached through Tauri v2 plugins before that |
| **What we depend on** | Global hotkey registration · run-at-sign-in registration · the native folder picker · opening a folder in Explorer · the tray icon |

## Three surfaces, and what breaks when Microsoft moves

A fourth — theme — is kept below for its history, but it is no longer one of them; see that
section.

### Global hotkey registration

`RegisterHotKey` is synchronous and either succeeds or does not. What Microsoft controls is **who else
got there first**, and that changes with every program the Reviewer installs.

The product's posture: refuse at binding when the combination is taken, bind anyway when it is not,
and report at each startup if registration then fails (`BR-26`). Never silently swallow.

**What we cannot detect:** a combination that registers successfully and never fires, because
something intercepts it lower in the stack. Windows reports success and the Reviewer sees a dead key.
`[MISSING]` — no health check exists. A periodic one was rejected: it would be the only background
task in the product, and `NFR-6`'s 150 MB idle budget is written for a product that has none.

**`NFR-7` is load-bearing here:** every registration must succeed without administrator rights. If a
Windows update ever changed that, the capture loop would need elevation, which the brief's premise
does not survive. `OQ-5` records this as an assumption and it has held so far.

### Run-at-sign-in registration

Reached through `WindowsRegistryAutoStartBackend` (`apps/desktop/src/startup.rs`), which writes a
registry entry under the current user directly via the `windows` crate. Reached through the Tauri
autostart plugin before `DEC-007`; the registry mechanism itself did not change.

The truth lives in Windows and is read back rather than remembered (`BR-114`). That is why anything —
a cleanup tool, a group policy, a profile reset — may remove it, and the product treats that as the
Reviewer's answer rather than as an error to correct (`SCN-02`, run 4).

**What Microsoft can break:** the registry location, the manifest-based mechanism replacing it, or a
policy that forbids the write. In each case registration fails loudly and the toggle shows `Off`. The
product degrades to *the Reviewer launches Snapdown themselves*, which is worse and is not broken.

### The folder picker and Explorer

Both are shell invocations with no return contract beyond "a path" and "it opened". The path is never
typed (`UC-14` step 2), so a Reviewer cannot express a path the shell would not have produced.

**What we do not trust:** the picker's claim that a folder is writable. `BR-115` validates by writing,
because a network drive, a synced folder, or a policy-locked path can all report themselves writable
and refuse the write.

### Theme

**This surface changed shape at `DEC-007`, not just implementation.** The paragraph below (`W6-S1`,
`420ecce`) describes the pre-`DEC-007` webview, where the theme followed Windows automatically:

> The Windows theme reached the webview as a CSS media query. Microsoft owned when it changed and
> whether a change was signalled to a running process. `NFR-17` required a theme change to be
> honoured without a restart, which meant the product had to not read the theme once at boot. Every
> colour was a CSS custom property inside a `prefers-color-scheme` block, and no JavaScript read a
> token value, so the repaint was the browser's.

**The Slint rebuild does not read Windows' theme setting at all.** `is-dark` (`apps/desktop/ui/theme.slint`)
is driven from a persisted `Setting` (`theme_setting_key()`), flipped by the Reviewer's own
sun/moon toggle (`on_theme_toggle_clicked` in `apps/desktop/src/main.rs`), not by Microsoft. Nothing
here still depends on Windows for theme — this whole component's Windows dependency, so this
integration file no longer has anything to say about it. Whether `NFR-17`'s wording ("honoured
without a restart") still describes the right promise once the Reviewer chooses instead of the OS is
a question for `wdi-question` or `wdi-decision`, not something this cleanup pass decides.

## What is deliberately not here

**The tray icon**, beyond noting it exists. It is `desktop-app`'s shell affordance, holds no state,
and `inventory-screen.md` says explicitly that it is not a screen.

**The Windows credential store.** It is a Windows integration and it belongs to `agent-access` and
`sharing`, which own the secrets in it. `BR-119` keeps it out of this component entirely.
