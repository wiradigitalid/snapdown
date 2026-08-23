# UX Audit Report — Verification of BUG-4 in Running Product

> **CORRECTION BY THE COORDINATOR, 2026-08-23.** The headline below reads "No — BUG-4 is not fixed".
> **That conclusion is wrong, and the fault is mine, not the auditor's.**
>
> The binary it tested was built with `cargo build --release -p snapdown`, which is **not** how a
> Tauri application is built. Without the Tauri CLI the release binary keeps requesting the `devUrl`
> from `tauri.conf.json` (`http://localhost:5173`) instead of the bundled `frontendDist` — which is
> exactly what the screenshot shows: `localhost refused to connect · ERR_CONNECTION_REFUSED`.
>
> **The frontend never loaded, so nothing about the frontend was tested.** `BUG-4`'s status is
> unchanged and still unverified in the product: `mount.test.tsx` proves the mount decision in jsdom
> and nothing has yet proved it in a running application.
>
> The auditor did its job. It launched the binary it was given, photographed what it saw, and said
> plainly that everything downstream could not be tested. Every "could not test" below is honest.
>
> **Two real findings came out of it:**
>
> 1. **`BUG-11`** — there is no reproducible way to build this application. The Tauri CLI is not a
>    dependency, not a script, not documented, and CI never produces a desktop artifact. The shipped
>    `Snapdown.exe` was built by means nobody wrote down.
> 2. **The window title reads `Snapdown Editor`** — visible in the screenshot. `W6-S2`'s persona
>    naming (`DEC-003`, `FR-27`) is correct in a real running window. That is the **first** thing
>    about this product ever confirmed outside jsdom.

## 1. Headline Question: Does the capture hotkey now show an overlay?

**As reported by the auditor: No.** Pressing `Ctrl+Shift+S` in the running product
(`target/release/Snapdown.exe`) does not show a capture overlay. When launched, the application's
window (titled `Snapdown Editor`) displays a webview network error page (`localhost` refused to
connect). No capture overlay appears on screen.

**As corrected above: this tested a binary whose frontend never loaded.** The question remains open.

---

## 2. Detailed Audit Questions & Findings

1. **Does the capture hotkey now show an overlay?**
   - **No.** Pressing `Ctrl+Shift+S` does not display a capture overlay. The window remained showing the localhost network error page.

2. **Does the overlay dim every monitor, or only one?**
   - **Could not test:** No overlay rendered on any monitor.

3. **Is there a live `W × H` readout while dragging? Where does it sit?**
   - **Could not test:** No drag selection or overlay was reachable in the running application.

4. **Where does the Note field appear — anchored to the region, or somewhere fixed?**
   - **Could not test:** Capture flow could not be initiated.

5. **Does `Enter` save? Does a Finding appear in the Findings list afterwards?**
   - **Could not test:** No capture could be completed to produce a finding.

6. **Does `Esc` cancel cleanly, leaving no Finding?**
   - **Could not test:** No overlay was displayed to cancel via `Esc`.

7. **Is the rail's focus ring visible when tabbing?**
   - **Could not test:** The navigation rail did not render because the webview loaded a network error page instead of the frontend bundle.

8. **Anything unreadable: text whose colour is close to its background:**
   - The application rendered the default browser localhost network error page.

---

## 3. Screenshot Status

- **`shot-overlay.png`**: Captured and saved to `.work/ux-audit/shot-overlay.png`. Shows the `Snapdown Editor` window displaying the localhost network error page upon launching and triggering `Ctrl+Shift+S`.
- **`shot-capture-note.png`**: **Impossible to capture.** Capture overlay did not render.
- **`shot-findings-after-capture.png`**: **Impossible to capture.** Findings UI did not load.
- **`shot-rail-focus.png`**: **Impossible to capture.** Navigation rail was not rendered.
