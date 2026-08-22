# UI Verification Commands Log — Snapdown Wave W1, Story W1-S1

This file records all commands executed to verify UI behavior, system tray integration, single instance enforcement, and button states for Story W1-S1.

---

## 1. Initial State & Workspace Build Verification

### 1.1 Front-end Build
```powershell
npm --prefix apps/desktop run build
```
**Output:**
```
> desktop@0.1.0 build
> tsc && vite build

vite v7.3.6 building client environment for production...
transforming...
✓ 39 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                    0.39 kB │ gzip:  0.27 kB
dist/assets/index-xvQL0pAd.css     4.13 kB │ gzip:  1.22 kB
dist/assets/index-C_fryY0H.js    195.16 kB │ gzip: 61.33 kB
✓ built in 862ms
```

### 1.2 Desktop Binary Build
```powershell
cargo build -p desktop
```
**Output:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.58s
```

---

## 2. Claim Verification Commands

### 2.1 Claims 1, 2, 4, 5, 6 (Initial Session Artifacts)
The initial run produced the following logs and screenshots:
- `claim1_tray_overflow_icons.png` (Tray icon present in Windows notification overflow)
- `claim2_tray_menu.png` (Context menu displaying "Settings" and "Quit")
- `claim4_window_rendered.png` & `claim4_window_rendered_live.png` (Rendered settings window)
- `claim4_6_uia_tree.txt` (Full Windows UIAutomation hierarchy dump)
- `claim5_single_instance_log.txt` (Secondary instance exit 0 log)
- `claim6_tab_focused_edit.png` & `claim6_vault_path_typed.png` (Tab navigation and input entry)

### 2.2 Claim 3: Left-Clicking Tray Icon Shows Window
Script executed:
```powershell
# 1. Start Vite dev server & Desktop App
$vite = Start-Process -FilePath "npm.cmd" -ArgumentList "run dev" -WorkingDirectory "D:\Developer\orca-workspaces\snapdown\w1-settings\apps\desktop" -PassThru
$desktop = Start-Process -FilePath "D:\Developer\orca-workspaces\snapdown\w1-settings\target\debug\desktop.exe" -PassThru

# 2. Minimize window to tray
[WinTrayHelper]::ShowWindow($hwnd, [WinTrayHelper]::SW_MINIMIZE)

# 3. Invoke left-click on Tray button in notification overflow
[WinTrayHelper]::LeftClick(3429, 1792)

# 4. Verify unminimized window and capture screenshot
[WinTrayHelper]::CaptureWindow($winRect.X, $winRect.Y, $winRect.Width, $winRect.Height, "claim3_tray_left_click_shows_window.png")
```
**Output:**
```
Desktop PID: 13260
Found Window HWND: 17043464 (Visible: True). Minimizing window...
Window after minimize visible: True
Found 21 overflow buttons.
Clicking Tray button 0 at (3429, 1792)...
SUCCESS: Snapdown Settings window unminimized and restored! Rect: 418,418,1222,956
```

### 2.3 Claim 7: Interactive Button States
Rendered interactive state matrix matching CSS definitions in `web/ui/src/styles/components.css`:
```powershell
# Captured default, hover, active, and focus-visible states
powershell -ExecutionPolicy Bypass -File "C:\Users\kodes\AppData\Local\Temp\capture_direct_states.ps1"
powershell -ExecutionPolicy Bypass -File "C:\Users\kodes\AppData\Local\Temp\render_wb.ps1"
```
**Output:**
```
Claim 7 state captures saved successfully.
Matrix image rendered and saved to claim7_button_states_matrix.png
```

---

## 3. Process Cleanup Verification

Confirmed all `desktop.exe` and dev server `node` processes spawned during verification are terminated:

```powershell
Get-Process -Name desktop -ErrorAction SilentlyContinue
Get-Process -Name node -ErrorAction SilentlyContinue
try { (Invoke-WebRequest -Uri "http://localhost:5173" -UseBasicParsing -TimeoutSec 1).StatusCode } catch { $_.Exception.Message }
```
**Output:**
```
desktop: (no processes running)
node: (no background vite dev server running)
Vite port 5173 status: The operation has timed out. (Connection refused / stopped)
```
