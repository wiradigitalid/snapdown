# Kertas Kerja Spesifikasi Teknis: Migrasi Native GUI ke Slint UI (`snapdown-desktop`)

## 1. Latar Belakang & Tujuan
Mengeliminasi footprint memori tinggi (~120MB baseline) dan runtime overhead WebView2 dengan membangun native desktop frontend menggunakan **Slint UI** (`slint-rs`).

## 2. Struktur Arsitektur Baru
```
snapdown/
├── crates/
│   ├── snapdown-core/        (Domain murni: Finding, Bundle, Settings, Markdown)
│   ├── snapdown-store/       (SQLite database, Vault storage, Image reduction, Marker & Visual burner)
│   ├── snapdown-capture/     (Windows UIA & Win32 DWM screen capturer)
│   └── snapdown-bridge/      (CLI & Local HTTP loopback bridge MCP port 3849)
├── apps/
│   ├── desktop-slint/        (Native Slint GUI application: binary utama `Snapdown.exe`)
│   │   ├── build.rs          (Slint markup compiler)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs       (Event loop, Slint controller, tray, hotkey)
│   │   │   ├── controller/   (State synchronization dengan snapdown-store)
│   │   │   └── ui_bridge.rs  (Adapter data model Slint <-> Core/Store)
│   │   └── ui/               (Slint Markup Files)
│   │       ├── theme.slint   (Design tokens: colors, spacing, typography)
│   │       ├── main.slint    (Root Window Layout: Ribbon, Canvas, Filmstrip, Sidebar)
│   │       ├── components/
│   │       │   ├── ribbon.slint
│   │       │   ├── canvas.slint
│   │       │   ├── marker_layer.slint
│   │       │   ├── crop_overlay.slint
│   │       │   ├── filmstrip.slint
│   │       │   ├── properties_panel.slint
│   │       │   └── settings_view.slint
│   │       └── overlay.slint (Fullscreen Capture Overlay Window)
│   └── web-service/          (Go service untuk web sharing)
```

## 3. Matriks Kesetaraan Fitur (100% Parity Checklist)

### 3.1 Studio Ribbon & Actions
- [ ] Tombol capture region 🔴 (Ctrl+Shift+S), open file 📂, paste clipboard 📥 (38×38px).
- [ ] Tool palette icon-only (36×36px): Marker (1), Shape (2), Callout (3), Blur (4), Arrow (5), Text (6), Crop.
- [ ] Undo / Redo buttons dengan status disabled/enabled reaktif.
- [ ] Tombol Assemble Bundle 📦 dengan counter selected findings, Copy Image 📋, dan Share.

### 3.2 Canvas Viewport & Artboard
- [ ] Render gambar latar belakang pixel-perfect dengan aspect ratio terjaga.
- [ ] Status image readout di bawah kanvas (dimensi W × H px dan ukuran file KB).
- [ ] Empty state ramah pengguna jika belum ada screenshot.

### 3.3 Numbered Step Markers
- [ ] Single click langsung menempatkan marker berurutan (1, 2, 3...).
- [ ] Drag marker untuk memindahkan posisi koordinat (0.0..1.0).
- [ ] Sinkronisasi otomatis dua arah ke Markdown Note (menambah & menghapus baris `N. comment`).

### 3.4 Visual Annotation Layer (Drag Gesture Required)
- [ ] **Shape:** Kotak ber-border dengan warna kustom + 8-point resize handles.
- [ ] **Blur:** Area blur redaction untuk menyensor teks/data sensitif.
- [ ] **Arrow:** Garis panah berujung segitiga dengan handle titik awal & akhir.
- [ ] **Callout:** Balon teks berlatar kontras dengan ekor segitiga yang dapat diarahkan.
- [ ] **Text:** Teks mengambang bebas dengan opsi font family, font size, bold, dan italic.
- [ ] **Seleksi & Keyboard Actions:** Klik elemen untuk seleksi, tombol `Delete`/`Backspace` untuk menghapus.

### 3.5 Bottom Filmstrip Tray
- [ ] Thumbnail cards horizontal scroll.
- [ ] Multi-selection ala Windows Explorer (Click single, Ctrl+Click toggle, Shift+Click range).
- [ ] Context Menu (Assemble, Open location, Copy image, Copy burned image, Delete).

### 3.6 Right Properties & Notes Panel
- [ ] Tab header di atas: **Notes (📋)** dan **Properties (🎨)**.
- [ ] Tab Properties otomatis aktif jika elemen visual dipilih, dan disabled jika tidak ada elemen aktif.
- [ ] Editor textarea multiline untuk observation note.
- [ ] Token & LLM cost estimator.
- [ ] Property controls: Font Family, Size (+/-), Bold (dengan `✓`), Italic, Color picker, Delete button.

### 3.7 Capture Overlay Window
- [ ] Fullscreen transparent overlay.
- [ ] Auto-detection DWM window & UI Automation sub-panel (1-click suggestion).
- [ ] Live magnifier / coordinates readout.
- [ ] Un-dimmed cutout preview.

### 3.8 Global Integration
- [ ] Windows System Tray (Show, Capture, Settings, Exit).
- [ ] Global Hotkeys (Ctrl+Shift+S).
- [ ] Local Agent Bridge Loopback Server (Port 3849).
- [ ] Windows Registry Auto-start on boot.
