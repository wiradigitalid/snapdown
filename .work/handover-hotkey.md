# Handover — Snapdown, tab Hotkeys

Salin seluruh isi berkas ini sebagai pesan pertama di sesi chat baru.

---

Kamu melanjutkan pekerjaan di **Snapdown** khusus untuk **tab Hotkeys di layar Settings** — bukan seluruh
produk. Sesi lain sedang/pernah menangani bagian lain (Fine-tune slider, vault, window shadow, dst);
jangan sentuh itu kecuali diminta.

- Repo: `D:\Developer\wiradigital.id\snapdown-eval-mattpocock`, branch `eval/mattpocock-skills`
- Baca `AGENTS.md` lebih dulu — ia mengikat. Branch ini pakai skill mattpocock/skills, BUKAN WDI Method;
  `AGENTS.md`-nya sudah menjelaskan kenapa dan apa yang beda
- **Belum ada satu pun perubahan di-commit.** `git status` akan menunjukkan beberapa file `.slint`/`.rs`
  ter-modify plus batch tiket lama di `.scratch/codex-qa-followup/` — itu memang belum diminta commit.
  Jangan commit sampai diminta eksplisit

## Cara kita bekerja

Loopnya: **kamu coding → kamu build release → aku uji sendiri → aku lapor cacat → kamu perbaiki**.

- Aku menulis dalam Bahasa Indonesia, jawab aku dalam Bahasa Indonesia. Komentar kode, pesan commit, dan
  dokumen tetap Inggris
- Setelah selesai coding, **build dan jalankan aplikasinya untukku**, jangan hanya bilang sudah selesai:
  ```
  tasklist //FI "IMAGENAME eq Snapdown.exe"   # matikan dulu kalau ada, ia mengunci exe-nya sendiri
  taskkill //F //IM Snapdown.exe
  cargo build --release -p snapdown-desktop
  ```
  lalu jalankan `target\release\Snapdown.exe`
- Lalu **beritahu apa yang perlu kuuji**, spesifik dan berurut prioritas
- `to-tickets` dan `implement` (skill mattpocock) hanya bisa AKU jalankan sebagai slash command sendiri —
  kamu tidak bisa memicunya sendiri walau diminta

## Verifikasi wajib sebelum bilang selesai

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
```

`apps/desktop` **tidak punya `package.json`** (`DEC-007`, pindah ke Slint) — jangan jalankan perintah npm
di sana. `cargo build` biasa TIDAK membangun aplikasi Tauri lama; abaikan itu, produk sekarang murni Slint
jadi `cargo build --release -p snapdown-desktop` sudah cukup dan sudah benar untuk repo ini.

## Berkas yang relevan untuk Hotkeys

- `crates/snapdown-core/src/domain/setting.rs` — `HotkeyAction` (Capture/OpenEditor), `SettingKey`
  (termasuk `HotkeyCapture`/`HotkeyOpenEditor` untuk kombinasinya dan `HotkeyCaptureEnabled`/
  `HotkeyOpenEditorEnabled` untuk toggle on/off yang TERPISAH dari kombinasinya)
- `apps/desktop/src/hotkey.rs` — `DesktopHotkeyRegistrar`: `init_from_store`, `validate_and_rebind`,
  `clear`, `set_enabled`/`is_enabled` (baru). Semua test-nya ada di bagian bawah file yang sama
- `apps/desktop/ui/components/settings.slint` — tab Hotkeys (`root.tab == 1`): baris per-hotkey,
  panel sticky di bawah saat listening, checkbox Enabled
- `apps/desktop/ui/appwindow.slint` — meneruskan properti/callback antara `AppWindow` dan `SdSettings`
- `apps/desktop/src/main.rs` — cari `on_hotkey_key_pressed`, `on_hotkey_cleared`,
  `on_hotkey_enabled_toggled`, `display_shortcut`, `shortcut_from_key`, `load_settings_into_window`

## Apa yang baru saja dikerjakan (BELUM DIUJI sama sekali oleh siapa pun)

Owner minta redesign UX Hotkeys supaya mirip `wira-desk` (`D:\Developer\wiradigital.id\wira-desk`, model
hotkey-nya jadi rujukan sejak awal). Yang baru ditambahkan hari ini:

1. **Tidak ada lagi teks mentah "CommandOrControl"** di layar. String yang TERSIMPAN di database tetap
   `"CommandOrControl+Shift+S"` (format yang dibutuhkan crate `global_hotkey`) — hanya TAMPILANNYA yang
   diterjemahkan, lewat `display_shortcut()` di `main.rs` (`Ctrl` di Windows, `Cmd` kalau suatu saat ada
   build macOS — dicek lewat `#[cfg(target_os = "macos")]`, tapi produk ini de facto Windows-only sekarang).
2. **Panel sticky 2x2** (Ctrl+Win baris atas, Alt+Shift baris bawah) menggantikan satu baris 4 chip,
   label "Win" ikut `hotkey-meta-key-label` (baru, di-set dari Rust, "Win" di Windows / "Command" di mac).
3. Tombol shortcut sekarang bertuliskan **"Listening…"** (bukan "Press keys… (Esc)"), dan **klik tombol
   yang sama sekali lagi membatalkan listening** — sebelumnya cuma Escape yang bisa.
4. **Checkbox "Enabled" per baris** — mematikan hotkey TIDAK LAGI menghapus kombinasinya (beda dari
   tombol "Clear" yang tetap ada, dan tetap menghapus beneran). Nyalakan lagi → coba register ulang
   kombinasi yang SAMA; kalau ternyata sudah direbut aplikasi lain di antara waktu off itu, muncul pesan
   gagal dan tetap off (tidak setengah-nyala). Domain-nya: `DesktopHotkeyRegistrar::set_enabled` +
   `SettingKey::HotkeyCaptureEnabled`/`HotkeyOpenEditorEnabled` — lihat test-testnya di `hotkey.rs` untuk
   bentuk perilaku yang diharapkan (`disabling_a_hotkey_keeps_its_shortcut_but_stops_it_registering`, dst).

Semua ini sudah lolos `cargo test --workspace` (termasuk 5 test baru khusus enable/disable +
1 test `display_shortcut`) dan build release sudah dijalankan — tapi **belum ada satu pun yang dilihat
mata manusia**. Ini prioritas pengujian nomor satu di sesi ini.

## Yang perlu diuji lebih dulu

1. Tab Hotkeys: shortcut tampil "Ctrl+Shift+..." bukan "CommandOrControl+...".
2. Klik tombol shortcut (sekarang "Listening…") → panel 2x2 di bawah menyala sesuai tombol yang ditekan.
   Klik tombol yang sama lagi (tanpa Esc) → listening batal.
3. Matikan checkbox "Enabled" salah satu baris → status jadi "Off. Turn it on to use this shortcut
   again.", TAPI kolom shortcut tetap menunjukkan kombinasinya (bukan "Not set"). Nyalakan lagi →
   kombinasi lama aktif lagi tanpa perlu diketik ulang.
4. Coba tekan hotkey yang barusan di-disable → pastikan benar-benar tidak memicu apa pun (capture/buka
   editor), karena OS-nya sendiri sudah unregister kombinasinya.

## Yang masih terbuka, menunggu keputusan pemilik

Dari perbandingan layar Hotkeys yang jalan sekarang dengan mockup desain aslinya
(`.how/settings/01-ux/assets/06b-settings-hotkeys.html`), ada **dua celah nyata, belum terdokumentasi
sebagai keputusan sengaja**:

- **Hanya 2 dari 4 shortcut yang dirancang yang pernah dibangun.** Yang hilang: "Toggle Window",
  "Copy Current Active Finding", "Quick Fullscreen Screenshot". Tidak ada catatan kenapa di-drop.
- **Badge status disederhanakan.** Mockup punya 4 warna beda (hijau Active, biru Listening, merah
  Conflict dengan background baris merah, kuning warning yang didefinisikan tapi tak dipakai). Yang
  jalan sekarang cuma titik warna + teks polos, tanpa treatment "Conflict" yang mencolok.

Tanyakan ke pemilik: mau dibangun (jadi tiket baru lewat `/to-tickets`, yang HARUS dia jalankan sendiri
sebagai slash command) atau memang keluar cakupan untuk sekarang dan dicatat alasannya di kode/komentar.

## Gotcha Slint yang baru ditemukan sesi ini — jangan diulang risetnya

- **`changed <property>` cuma menerima nama properti TANPA kualifikasi**, milik elemen tempat ia
  dideklarasikan sendiri. `changed root.foo => {}` di dalam anak yang bersarang itu SYNTAX ERROR
  ("expected '=>'"). Kalau perlu mengawasi properti `root` dari scope yang lebih dalam, buat alias
  dua-arah lokal (`property <T> mirror <=> root.foo;`) di elemen yang paling dekat dengan id yang mau
  ditulis, lalu `changed mirror => { some-id.text = ...; }`.
- **Sebuah id TIDAK BISA diakses melewati batas kondisional `if`.** `if cond : Foo { some-id := Bar {} }`
  membuat `some-id` tak terlihat dari luar blok `if` itu — termasuk dari root component. Taruh
  properti mirror/`changed` di scope kondisional YANG SAMA dengan id-nya, bukan lebih luar.
- **`SdTextField.text` adalah alias dua-arah** (`text <=> input.text` di `text-field.slint`) ke widget
  `TextInput` bawaan Slint. Begitu ada input manual SEKALI, Slint MEMATIKAN binding reaktif `text: expr`
  itu untuk selamanya — ini akar bug slider/textbox desync yang baru diperbaiki. Solusinya BUKAN
  binding deklaratif lagi, tapi push imperatif lewat `changed` pada properti sumbernya (lihat pola di
  `settings.slint` bagian "Fine-tune size and quality" — komentarnya menjelaskan lengkap).
- **Tombol Win/meta tidak pernah bisa menyelesaikan sebuah shortcut** — `shortcut_from_key()` di
  `main.rs` sengaja tidak menerima `meta` sebagai modifier penutup (Windows reserve tombol itu). Chip
  "Win" di panel sticky menyala mengikuti tombol fisik yang ditekan (murni display), tapi kombinasi
  yang hanya pakai Win tidak akan pernah ter-bind. Ini BUKAN bug baru, jangan dilaporkan ulang sebagai
  temuan — kalau pemilik minta Win bisa dipakai, itu perubahan baru yang perlu didiskusikan dulu.

## Belum dites siapa pun sebelumnya juga (dari ronde tiket 08-13, masih menunggu konfirmasi)

- Tiket 08 (bypass aksi hotkey saat Settings terbuka) — sudah dikonfirmasi pemilik OK.
- Sisanya di batch itu juga sudah dikonfirmasi OK di ronde sebelumnya kecuali dua yang jadi fokus sesi
  ini (slider desync dan "Open the Editor after a hotkey capture" masih membuka editor) — keduanya
  sudah diperbaiki di atas tapi belum dikonfirmasi ulang oleh pemilik untuk perbaikan TERBARU-nya.

---

## HASIL PENGUJIANKU

*(diisi manual sebelum mengirim — kalau kosong, tanyakan dulu sebelum mulai coding)*

```
1. Teks shortcut (Ctrl bukan CommandOrControl)   →
2. Listening… + klik-untuk-batal                 →
3. Checkbox Enabled (off lalu on lagi)            →
4. Hotkey benar-benar tidak jalan saat disabled   →

Keputusan soal 2 hotkey yang belum dibangun + badge conflict:  →

Lainnya / catatan bebas:

```
