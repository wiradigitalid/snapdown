---
id: W6-S1
title: 'W6-S1: Every colour through tokens, both themes, enforced by a lint rule'
type: 'feature'
wave: W6
status: ready-for-dev
created: '2026-08-23'
dependencies: []
files:
  - web/ui/src/styles/tokens.css
  - web/ui/src/styles/components.css
  - web/ui/src/components/SegmentedControl.tsx
  - web/ui/src/components/HotkeyChip.tsx
  - web/ui/src/components/Toggle.tsx
  - web/ui/src/components/EmptyState.tsx
  - web/ui/src/components/ErrorState.tsx
  - web/ui/src/components/Badge.tsx
  - web/ui/src/components/Checkbox.tsx
  - web/ui/src/index.ts
  - web/ui/src/test/contrast.test.ts
  - web/ui/src/test/components.test.tsx
  - web/ui/src/test/tokens.test.ts
  - web/ui/eslint.config.js
  - apps/desktop/eslint.config.js
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/BundleView.tsx
  - apps/desktop/src/components/CaptureOverlay.tsx
  - apps/desktop/src/components/HotkeySection.tsx
  - apps/desktop/src/components/OrphanReportView.tsx
  - web/ui/src/screens/AgentAccessView.tsx
  - web/ui/src/components/BundleComposer.tsx
  - web/ui/src/components/FindingsEditor.tsx
  - web/ui/src/components/MarkerLayer.tsx
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/design-system.md
  - .how/_platform/cross-cutting.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** 
The application currently violates the single colour authority principle (AD-10, NFR-16, NFR-17). Over 23 hex colour literals exist in React components across `apps/desktop/src` and `web/ui/src` (e.g. `#ffffff`, `#f8fafc`, `#e2e8f0`, `#dcfce7`, `#166534`, `#fef3c7`, `#854d0e`, `#eff6ff`). When running under Windows Dark Theme (`prefers-color-scheme: dark`), views like `FindingsView` and `BundleView` paint hardcoded light panels while shell text adopts white token colours, producing unreadable white-on-white text. Furthermore, critical design system tokens (paired meaning tokens, sunken surface, `--space-0`, `--radius-full`) and base UI elements (`SegmentedControl`, `HotkeyChip`, `EmptyState`, `ErrorState`, and `Toggle`'s indeterminate state) are missing from `@snapdown/ui`. There is no ESLint enforcement preventing colour literals, nor automated contrast ratio assertions across themes.

**Approach:**
1. Define all missing design tokens in `web/ui/src/styles/tokens.css` for **both** light and dark themes:
   - Four meaning pairs with proven foreground/background contrast: `--color-success-bg`/`--color-success-text`, `--color-warning-bg`/`--color-warning-text`, `--color-info-bg`/`--color-info-text`, `--color-neutral-bg`/`--color-neutral-text`.
   - `--color-surface-sunken` for inset wells/preview areas/recorder chips at rest.
   - Spacing token `--space-0` (`2px` / `0.125rem`) and radius token `--radius-full` (`9999px`).
   - Theme-invariant tokens explicitly declared and commented: `--color-marker*` (amber `#f59e0b`, text `#000000`, ring `#ffffff` in both themes), capture overlay scrim (`rgba(0,0,0,0.4)`), region ring (`#3b82f6`), and `--canvas-checker`.
2. Replace all hardcoded hex/rgba colour literals in `apps/desktop/src` and `web/ui/src` with design tokens.
3. Implement missing base UI elements in `web/ui/src/components/`:
   - `SegmentedControl`: keyboard-accessible option switcher with unselected, selected, focus-visible, and disabled states.
   - `HotkeyChip`: recording/display chip supporting `bound`, `listening`, `unbound`, and `conflicted` states.
   - `Toggle`: binary and asynchronous OS-state toggle supporting `on`, `off`, `indeterminate` (load-bearing for W6-S5 async OS reads), `focus-visible`, and `disabled`.
   - `EmptyState`: illustration slot, heading, description, and exactly one actionable button.
   - `ErrorState`: Reviewer-centric failure explanation and actionable recovery/retry button.
   - `Badge`: variant mapping using paired meaning tokens (`success`, `warning`, `info`, `neutral`, `danger`).
4. Implement an ESLint rule in both `web/ui/eslint.config.js` and `apps/desktop/eslint.config.js` that rejects any colour literals (hex `#...`, `rgb()`, `rgba()`, `hsl()`, `hsla()`, named literal colours) outside `web/ui/src/styles/tokens.css`.
5. Implement comprehensive automated test suites in Vitest:
   - Token completeness asserting every token is defined in both `:root` and `@media (prefers-color-scheme: dark)`.
   - Automated WCAG AA contrast ratio assertions (>= 4.5:1 for normal text, >= 3.0:1 for large text/UI components) checking every text element and meaning pair against **its own background** under both light and dark themes.
   - AST/source scan test verifying zero colour literals exist in component source trees.
   - Component rendering and state verification tests for all new/updated primitives.

## Boundaries & Constraints

**Always:**
- Every colour token MUST be defined in `web/ui/src/styles/tokens.css` for both light theme (`:root`) and dark theme (`@media (prefers-color-scheme: dark)`).
- Every meaning background token MUST be paired with its proven foreground text token and consumed together (`--color-success-bg` with `--color-success-text`, `--color-warning-bg` with `--color-warning-text`, `--color-info-bg` with `--color-info-text`, `--color-neutral-bg` with `--color-neutral-text`).
- Theme-invariant tokens (`--color-marker`, `--color-marker-text`, `--color-marker-ring`, capture overlay scrim, region ring, canvas checkerboard) MUST retain identical literal values across both themes and carry explanatory comments in `tokens.css`.
- `Toggle`'s `indeterminate` state MUST be a distinct rendered visual state (e.g. horizontal bar/neutral glyph), inert or neutral, preventing premature assumed states (`FR-18`, `BR-108`).
- Contrast assertions MUST evaluate text against its direct bounding container background (its own background), not the global page ground.
- ESLint and automated tests MUST fail if any colour literal (`#[0-9a-fA-F]{3,8}`, `rgba?\(`, `hsla?\(`) is added outside `tokens.css`.
- Base components MUST meet accessibility standards: full keyboard navigation (`Tab`, `Space`, `Enter`, Arrow keys for segmented control), `focus-visible` styling with 2px outline, ARIA attributes (`aria-checked`, `aria-selected`, `aria-disabled`, `role="switch"`, `role="radiogroup"`).

**Block If:**
- Upstream requirements or documents demand hardcoded colours or per-screen color overrides bypassing tokens.
- Design tokens cannot satisfy WCAG 2.1 AA contrast ratio (4.5:1 for text, 3:1 for UI elements).

**Never:**
- Do not introduce inline hex, rgb, rgba, hsl literals into any TSX, TS, or non-token CSS file.
- Do not make dark mode an in-app toggle; theme MUST follow system `prefers-color-scheme` dynamically without requiring an app restart.
- Do not add annotation tools, extra colors, or secondary shapes to `Marker` or `MarkerBadge` (numbered markers are the sole annotation vocabulary).
- Do not alter backend Rust crates (`snapdown-core`, `snapdown-store`, `snapdown-capture`) or modify corpus files in `.what/`, `.how/`, `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Light theme rendering | OS `prefers-color-scheme: light` | All panels use light tokens (`--color-bg: #f8fafc`, `--color-surface: #ffffff`, `--color-text: #0f172a`, etc.) with WCAG AA >= 4.5:1 | Verified via automated token contrast test |
| Dark theme rendering | OS `prefers-color-scheme: dark` | All panels resolve dark tokens (`--color-bg: #090d16`, `--color-surface: #131b2e`, `--color-text: #f8fafc`, etc.) without white-on-white text | Verified via automated token contrast test |
| Theme change at runtime | OS switches between light and dark while app is open | All surfaces instantly repaint with corresponding theme tokens without reload or state loss | Enforced by CSS variable inheritance |
| Theme-invariant markers | Render Marker (1-99) in light or dark theme | Marker badge background is amber `#f59e0b`, text is black `#000000`, border ring is white `#ffffff` in both themes | Verified by `tokens.test.ts` |
| Theme-invariant capture overlay | Overlay active over desktop content | Scrim is `rgba(0, 0, 0, 0.4)`, selection box is `#3b82f6` border with translucent fill regardless of OS theme | Verified by overlay styling |
| `SegmentedControl` selection | Click or keyboard arrow navigation between segments | Updates selected value, updates `aria-selected="true"`, focuses active segment, applies focus-visible ring | Ignores clicks/keys on `disabled` options |
| `HotkeyChip` at rest (bound) | Valid shortcut assigned (e.g. `Ctrl+Alt+S`) | Displays formatted key badges on sunken surface with neutral border and text | Renders clear badge if unbound |
| `HotkeyChip` recording (listening) | User clicks chip to record new shortcut | Chip enters listening state with `--color-info-bg`, `--color-info-text`, accent border ring; blurs or ESC cancels | Reverts to prior binding on ESC/blur |
| `HotkeyChip` conflict state | Duplicate shortcut or OS conflict detected | Chip displays with `--color-warning-bg`/`--color-warning-text` or danger tokens with conflict indicator | Displays conflict message in helper text |
| `Toggle` indeterminate | Startup setting read is pending (`indeterminate = true`) | Renders neutral indeterminate indicator; `aria-checked="mixed"`; does not trigger `onChange` | Distinct visual state, not true/false |
| `EmptyState` render | Empty view (e.g. no findings, no bundles) | Displays centered icon/slot, heading, description, and exactly one action button calling `onAction` | Action button hidden if `onAction` not provided |
| `ErrorState` render | Error boundary or operation failure | Displays error icon, human-readable error title, actionable remediation text, and Retry/Dismiss button | Safely handles empty/undefined error messages |
| Hex literal in component | Developer adds `#123456` in any `.tsx` or `.ts` file | ESLint fails with error `Colour literal forbidden. Use design tokens from tokens.css` | Blocks CI build and lint gate |

</intent-contract>

## Code Map

- `web/ui/src/styles/tokens.css` -- Design token stylesheet defining surface, meaning, typography, spacing, radius, and theme-invariant tokens for light and dark schemes.
- `web/ui/src/styles/components.css` -- Component base styles including interactive pseudo-classes (`:hover`, `:active`, `:focus-visible`), toggle animations, and segmented control styling.
- `web/ui/src/components/SegmentedControl.tsx` -- New primitive for single-choice option group with full keyboard navigation and accessible roles.
- `web/ui/src/components/HotkeyChip.tsx` -- New primitive for recording and displaying hotkeys with `bound`, `listening`, `unbound`, and `conflicted` states.
- `web/ui/src/components/Toggle.tsx` -- New primitive for binary and indeterminate toggle switches with `on`, `off`, `indeterminate`, `focus-visible`, and `disabled` states.
- `web/ui/src/components/EmptyState.tsx` -- Standardized empty state element with illustration slot, heading, description, and single primary action.
- `web/ui/src/components/ErrorState.tsx` -- Standardized error presentation element with failure description and remediation/retry actions.
- `web/ui/src/components/Badge.tsx` -- Semantic badge component consuming paired meaning tokens (`success`, `warning`, `info`, `neutral`, `danger`).
- `web/ui/src/components/Checkbox.tsx` -- Updated base checkbox supporting indeterminate and keyboard focus-visible states with semantic tokens.
- `web/ui/src/index.ts` -- Shared UI export barrel exporting all base components, tokens, and types.
- `web/ui/src/test/contrast.test.ts` -- Automated Vitest test suite calculating WCAG 2.1 AA contrast ratios for all text tokens and meaning pairs in light and dark themes.
- `web/ui/src/test/tokens.test.ts` -- Automated Vitest test verifying token completeness across themes, theme-invariant declarations, and AST scanning for forbidden colour literals.
- `web/ui/src/test/components.test.tsx` -- Vitest suite verifying states, keyboard interactions, and ARIA attributes for `SegmentedControl`, `HotkeyChip`, `Toggle`, `EmptyState`, `ErrorState`, and `Badge`.
- `web/ui/eslint.config.js` -- ESLint configuration adding rule to reject hex and color literals outside `tokens.css`.
- `apps/desktop/eslint.config.js` -- ESLint configuration adding rule to reject hex and color literals outside `tokens.css`.
- `apps/desktop/src/App.tsx` -- Desktop shell updated to use design tokens for header and navigation styling.
- `apps/desktop/src/components/BundleView.tsx` -- Refactored to replace 13 hex literals with semantic tokens.
- `apps/desktop/src/components/CaptureOverlay.tsx` -- Refactored to use theme-invariant scrim and selection ring tokens.
- `apps/desktop/src/components/HotkeySection.tsx` -- Refactored to use `HotkeyChip`, `Badge`, and warning/info/success token pairs.
- `apps/desktop/src/components/OrphanReportView.tsx` -- Refactored to use semantic status tokens for missing/orphan listings.
- `web/ui/src/screens/AgentAccessView.tsx` -- Refactored to eliminate hex literals and use `Badge` and token pairs.
- `web/ui/src/components/BundleComposer.tsx` -- Refactored to eliminate hex literals and use semantic surface/border tokens.
- `web/ui/src/components/FindingsEditor.tsx` -- Refactored to eliminate hex literals and use semantic surface/border tokens.
- `web/ui/src/components/MarkerLayer.tsx` -- Refactored to eliminate hex literals and use theme-invariant marker tokens.

## Tasks & Acceptance

**Execution:**
- [ ] `web/ui/src/styles/tokens.css` -- Add missing tokens to light (`:root`) and dark (`@media (prefers-color-scheme: dark)`) themes:
  - Meaning pairs: `--color-success-bg`/`--color-success-text`, `--color-warning-bg`/`--color-warning-text`, `--color-info-bg`/`--color-info-text`, `--color-neutral-bg`/`--color-neutral-text`
  - Inset surface: `--color-surface-sunken`
  - Spacing and radius: `--space-0` (`2px`), `--radius-full` (`9999px`)
  - Theme-invariant tokens with explanatory comments: `--color-marker*`, `--color-overlay-scrim`, `--color-overlay-ring`, `--canvas-checker`
  - Correct dark theme `--color-marker-ring` to `#ffffff` and `--color-marker` to `#f59e0b`
- [ ] `web/ui/src/styles/components.css` -- Add styling classes and focus-visible outlines for new base elements (`SegmentedControl`, `HotkeyChip`, `Toggle`, `Badge`, `EmptyState`, `ErrorState`).
- [ ] `web/ui/src/components/SegmentedControl.tsx` -- Implement `SegmentedControl` component supporting option list, keyboard arrow navigation, focus-visible outline, and disabled states.
- [ ] `web/ui/src/components/HotkeyChip.tsx` -- Implement `HotkeyChip` component supporting `bound`, `listening`, `unbound`, and `conflicted` states with key capture and cancel on blur/ESC.
- [ ] `web/ui/src/components/Toggle.tsx` -- Implement `Toggle` switch component supporting `on`, `off`, `indeterminate`, `focus-visible`, and `disabled` states.
- [ ] `web/ui/src/components/Badge.tsx` -- Implement `Badge` component with variants (`success`, `warning`, `info`, `neutral`, `danger`) using paired meaning tokens.
- [ ] `web/ui/src/components/EmptyState.tsx` -- Refactor `EmptyState` to support illustration slot, heading, description, and single action button.
- [ ] `web/ui/src/components/ErrorState.tsx` -- Implement `ErrorState` component with error details and retry/action handler.
- [ ] `web/ui/src/index.ts` -- Export new and updated components and types (`SegmentedControl`, `HotkeyChip`, `Toggle`, `Badge`, `EmptyState`, `ErrorState`).
- [ ] `web/ui/eslint.config.js` & `apps/desktop/eslint.config.js` -- Configure custom ESLint rule / restriction to forbid hex literals (`#[0-9a-fA-F]{3,8}`) and color functions (`rgb`, `rgba`, `hsl`, `hsla`) in source files outside `tokens.css`.
- [ ] `apps/desktop/src/` & `web/ui/src/` -- Refactor all components to remove all hex and color literals:
  - `apps/desktop/src/App.tsx`
  - `apps/desktop/src/components/BundleView.tsx`
  - `apps/desktop/src/components/CaptureOverlay.tsx`
  - `apps/desktop/src/components/HotkeySection.tsx`
  - `apps/desktop/src/components/OrphanReportView.tsx`
  - `web/ui/src/screens/AgentAccessView.tsx`
  - `web/ui/src/components/BundleComposer.tsx`
  - `web/ui/src/components/FindingsEditor.tsx`
  - `web/ui/src/components/MarkerLayer.tsx`
- [ ] `web/ui/src/test/contrast.test.ts` -- Implement automated WCAG 2.1 AA contrast ratio tests verifying all text/background pairs pass >= 4.5:1 in both light and dark themes.
- [ ] `web/ui/src/test/tokens.test.ts` -- Implement token parity test (both themes define identical token sets), theme-invariance checks, and codebase regex scanner ensuring zero hex literals survive.
- [ ] `web/ui/src/test/components.test.tsx` -- Implement component test suite covering `SegmentedControl`, `HotkeyChip`, `Toggle` (including indeterminate), `Badge`, `EmptyState`, and `ErrorState`.

**Acceptance Criteria:**
- Given `web/ui/src/styles/tokens.css`, all tokens (surface, sunken, meaning pairs, space-0, radius-full, theme-invariant markers/overlay/checker) are defined in both `:root` and `@media (prefers-color-scheme: dark)`.
- Given any TS, TSX, or non-token CSS file in `apps/desktop/src` and `web/ui/src`, a search for `#[0-9a-fA-F]{3,8}` returns zero matches.
- Given ESLint, running `npm --prefix web/ui run lint` and `npm --prefix apps/desktop run lint` passes with zero errors, and fails if a color literal is introduced outside `tokens.css`.
- Given the contrast verification test `npm --prefix web/ui run test`, every text element and meaning pair meets or exceeds WCAG AA (>= 4.5:1 for standard text, >= 3.0:1 for large/UI) against its own background in both light and dark themes.
- Given `SegmentedControl`, the user can switch options via mouse click and Arrow Left / Arrow Right keys, and the active element receives focus-visible ring.
- Given `HotkeyChip`, the component renders bound shortcuts with mono styling, enters listening state on click, captures keyboard shortcuts, cancels on ESC or blur, and displays warning/danger tokens when conflicted.
- Given `Toggle`, the component renders `indeterminate` state visually distinct from `on` or `off` with `aria-checked="mixed"`.
- Given all test suites, `npm --prefix web/ui run typecheck`, `npm --prefix web/ui run lint`, `npm --prefix web/ui run test`, `npm --prefix apps/desktop run typecheck`, `npm --prefix apps/desktop run lint`, and `npm --prefix apps/desktop run test` all pass cleanly.

## Verification

**Commands:**
- `npm --prefix web/ui run typecheck` -- expected: TypeScript compiles with zero errors across all components and tests
- `npm --prefix web/ui run lint` -- expected: ESLint passes with zero warnings/errors, enforcing no color literals outside `tokens.css`
- `npm --prefix web/ui run test` -- expected: All Vitest suites pass, including contrast assertions, token parity, and component interaction tests
- `npm --prefix apps/desktop run typecheck` -- expected: Zero TypeScript diagnostic errors in desktop app
- `npm --prefix apps/desktop run lint` -- expected: ESLint passes with zero warnings/errors in desktop app
- `npm --prefix apps/desktop run test` -- expected: Vitest tests pass in desktop app
- `npm --prefix apps/desktop run build` -- expected: Production Vite build succeeds
- `cargo test --workspace` -- expected: All Rust crate unit and integration tests remain green

## Spec Change Log

### 2026-08-23 — round 1 must-fix, from panel adjudication

**Finding: `web/ui/src/test/contrast.test.ts` asserts a hardcoded copy of the token values instead of
the values in `tokens.css`.**

The file declares `const lightTokens = { bg: '#f8fafc', text: '#0f172a', textMuted: '#475569', ... }`
and computes contrast over that object. Nothing in it reads `src/styles/tokens.css`.

**Why this is a must-fix and not a follow-up.** It is a test that cannot fail in the way it claims to.
Change `--color-text-muted` in `tokens.css` to something with a 2:1 ratio and this suite still passes,
because it is checking its own copy. `NFR-16` says *every text element meets WCAG AA against its own
background*, enforced by *an automated contrast assertion* — an assertion over a literal is not that.
It is also a second source of truth for colour, which is precisely what `AD-10` exists to forbid, so
the test contradicts the invariant the story was written to establish.

`tokens.test.ts` in the same commit does it correctly: it reads `src/styles/tokens.css` from disk and
asserts presence and light/dark parity. That is the pattern to follow.

**Required change.** Parse the token values out of `tokens.css` — both the `:root` block and the
`prefers-color-scheme: dark` block — and compute every contrast assertion over the parsed values. No
colour literal may remain in the test file. Add one assertion that proves the parse is live: a pairing
whose ratio is computed from the file, not stated.

**Not in scope for this round.** The token *names* in the code (`--color-overlay-scrim`,
`--color-overlay-ring`) differ from the names in `.how/_platform/design-system.md` (`--overlay-dim`,
`--overlay-region-ring`). The code's naming is the more consistent of the two and **the document is
being corrected to match it**, by the coordinator, not by this story. Do not rename anything.

Everything else in round 1 was verified green by the coordinator and stands: no colour literal outside
the token file anywhere under `apps/desktop/src` or `web/ui/src`; `cargo fmt`, `cargo test --workspace`,
both typechecks, both lints, and both vitest suites (70 tests) pass; the lint rule, `Badge`,
`ErrorState`, `HotkeyChip`, `SegmentedControl` and `Toggle` are in place.
