# UX audit — Snapdown Desktop, shipped build

**Date:** 2026-08-23 · **Binary:** `target/release/Snapdown.exe`, built at commit `4c5c901`
**Windows theme during the run:** dark
**Method:** an Orca-supervised UI worker drove the real application; the orchestrator verified every
finding below against the captured screenshots and the source.

## Coverage — what was tested, and what was not

The worker **exhausted its 1M-token context** partway through and was stopped. Screenshots are
expensive in context, and the brief asked for too many in one worker. This is recorded rather than
smoothed over, because the second half of the brief was never executed.

| Asked for | Result |
|---|---|
| Four primary tabs, screenshot + accessibility tree | **Done** — `shot-{settings,findings,bundles,agent-access}.png`, `tree-*.txt` |
| Vault folder Browse | **Done** — `shot-folder-browse.png` |
| Startup toggle, both directions | **Partly** — first toggle done; the worker died on the second |
| Hotkey recording, and a conflicting binding | **NOT TESTED** |
| Capture overlay, region select, note, confirm | **NOT TESTED** |
| Finding detail and marker canvas | **NOT TESTED** |
| Both Windows themes | **NOT TESTED** — only dark |
| Settings scroll extent, measured | **NOT MEASURED** — observed from the screenshot only |

Nothing below is inferred from the untested rows. Where a defect could only be confirmed in dark
theme, it says so.

**The screenshots themselves are NOT committed.** The product brief forbids it — this repository is
public and a Capture may contain personal data — and nothing in CI enforced that until this audit
staged five of them and was caught by hand. `.gitignore` now covers `.work/**/*.png` and friends; a
real guard is still missing and is recorded below.

## Screens

| Screen | Renders | Verdict |
|---|---|---|
| Editor shell | Yes — window titled `Snapdown`, four tabs, all reachable | Navigation works |
| Settings | Yes | Three defects, below |
| Findings | Yes, empty state | **Contrast failure** |
| Bundles | Yes, empty state | **Contrast failure** |
| Agent access | Yes | Not examined in detail |

## The three owner-reported defects — verdicts

### 1. "The application is called Desktop, not Snapdown" — **REFUTED as a design fault, CONFIRMED as a build fault**

The running build is titled `Snapdown` in the title bar, the header, and `tauri.conf.json`
(`productName: "Snapdown"`). But `target/release/` held **two** executables:

```
target/release/desktop.exe    19:13:21   ← stale, predates the rename commit
target/release/Snapdown.exe   19:31:21   ← current, matches commit 4c5c901 at 19:31:30
```

The owner ran `desktop.exe`. The rename shipped; the old artifact was never removed. Both stale
binaries (`release` and `debug`) were deleted during this audit.

**This one fault explains three of the owner's reports.** The stale build predates the commit that
added tab navigation, the Vault Browse button, and the hotkey recorder — which is why the owner also
reported being unable to find the Editor and unable to set a hotkey manually. All three exist and
work in the current build.

Covered by `FR-27`, and its build consequence is now a stated requirement: a second desktop executable
in the output directory is a build failure, not clutter.

### 2. "I can only ever see Settings" — **REFUTED**

All four tabs render in the header and all four are reachable; the worker navigated to each. The
app opens on Settings by default (`initialTab = 'settings'` in `App.tsx`), which is a defensible
first-run choice and a poor returning-user one — but it is not the absence of navigation. Same root
cause as defect 1.

### 3. "Capture Region and Editor labels are white on a white background" — **CONFIRMED, on a different screen**

The owner placed this in the Hotkeys section. In the current build the Hotkeys labels render white on
dark and are readable (`shot-settings.png`).

The defect is real and it is on **Findings and Bundles**. Both paint their panels with light-theme
literals while the shell paints text from tokens that follow `prefers-color-scheme`:

```
BundleView.tsx:93   backgroundColor: '#f8fafc'
BundleView.tsx:114  backgroundColor: isSelected ? '#e0f2fe' : '#ffffff'
BundleView.tsx:137  backgroundColor: '#ffffff'
BundleView.tsx:173  backgroundColor: '#f1f5f9'
```

`shot-findings.png` and `shot-bundles.png` show large white panels inside a dark window. The muted
placeholder text on them is light grey on white — poor. Any element using `--color-text` on those
panels is **white on white** under the dark theme.

**23 distinct hex literals** live outside the token file across `apps/desktop/src/**/*.tsx`. Every one
was chosen against a light background. This is a mechanism failure, not four bad panels — patching the
panels would leave it in place.

Now covered by `NFR-16`, `NFR-17`, `AD-10`, and `BR-107`.

## Further defects found

### 4. "Run at Windows startup" is off, and the control lies before it is right

`shot-settings.png` shows the checkbox **unchecked**. `get_startup_status` reads the real Windows
registration and nothing registers it at first run, so the effective default is **off** — a capture
tool the Reviewer must remember to launch.

Separately, `App.tsx` initialises `useState<boolean>(true)` and only then reads the OS. The control
renders enabled, then repaints to disabled. `FR-18` requires it to reflect the real registration and
never a remembered intention; there is no state for *not yet known*.

Now covered by the amended `FR-18`, `BR-108`, and a cross-cutting agreement.

### 5. Roughly a third of the Settings window is empty

The General group (one checkbox) and the Quality Budget group (four controls) sit in equal-height
grid columns. General inherits Quality Budget's height, leaving ~300 px of dead space, and the
Hotkeys group is still pushed below the fold — a scrollbar is visible in `shot-settings.png`.

The owner's words were "settingannya bisa padat-padat" — it can be dense. Now covered by `FR-29`.

### 6. The Quality Budget asks for two numbers nobody can judge

`Max Long Edge (px) = 1600`, `Encoder Quality (10-100) = 75`. The PRD's own § 8 records that 1600 px
"has not been measured." A value the team cannot defend is one the Reviewer certainly cannot.
Rewritten as `FR-5` under `DEC-004`.

### 7. Both list surfaces render fixed-height panels

Findings and Bundles both leave large dead areas below their panels rather than filling the window.
Visible in both screenshots.

## What this audit could not settle

- **Light theme.** Untested. `NFR-17`'s both-themes render test is the right instrument, not another
  manual pass.
- **The capture overlay, the note field, and the marker canvas.** Never reached. These are the most
  frequently used surfaces in the product and they have no visual record. This is the largest gap.
- **The hotkey conflict message.** `FR-17` requires a refused combination to name the conflict. Not
  exercised.
- **Whether Settings fits at the 1024×720 minimum after the redesign.** Nothing to measure yet.

## A finding about the repository, not the product

The brief's constraint — *this repository is public; forbids committing a captured screenshot, a
token, a client's name, or any test fixture derived from real capture output* — has been `active`
since G1 and **nothing enforces it**. `korpus.yml` validates the corpus; `desktop-ci.yml` builds.
No workflow refuses a forbidden path, and `.gitignore` had no rule for image output.

It was caught here only because someone read `git status` before committing. That is not a control.
Filed as follow-up for the wave that touches CI.
