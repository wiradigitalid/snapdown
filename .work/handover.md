# Handover — Snapdown, iterasi coding → test

Salin seluruh isi berkas ini sebagai pesan pertama di sesi chat baru.

---

Kamu melanjutkan pekerjaan di **Snapdown**, aplikasi desktop screenshot-ke-Markdown milikku.

- Repo: `D:\Developer\wiradigital.id\snapdown` (publik, `wiradigitalid/snapdown`), branch `main`
- Baca `AGENTS.md` lebih dulu — ia mengikat, dan bagian `## Code` serta `### Pitfalls` berisi kesalahan mahal yang sudah pernah dibuat repo ini
- Repo ini memakai **WDI Method**. `.control/registry/defects.yaml` adalah ingatan proyek

## Cara kita bekerja

Loopnya: **kamu coding → kamu build release → aku uji sendiri → aku lapor cacat → kamu perbaiki dan catat.**

- Aku menulis dalam Bahasa Indonesia, jawab aku dalam Bahasa Indonesia. Komentar kode, pesan commit, dan dokumen tetap Inggris (aturan `AGENTS.md`)
- Setelah selesai coding, **jalankan aplikasinya untukku** — jangan hanya bilang sudah selesai:
  ```
  Get-Process -Name Snapdown        # matikan dulu, ia mengunci exe-nya sendiri
  cargo build --release -p snapdown-desktop
  start target\release\Snapdown.exe
  ```
- Lalu **beritahu apa yang perlu kuuji**, spesifik dan berurut prioritas. Aku tidak akan membuka berkas
- **Jangan commit sampai aku minta.** Kalau aku minta: stage path eksplisit satu per satu — worktree ini dipakai bersama sesi lain, dan `.work/` tidak pernah ikut

## Verifikasi wajib sebelum bilang selesai

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python -c "import runpy,sys; sys.argv=['validate.py']; runpy.run_path('.constitution/method/scripts/validate.py', run_name='__main__')"
```

`validate.py` baseline-nya **RED — 17 findings across 2 validators** (V3 + V13). Naik dari 17 berarti kamu yang menyebabkannya.

## Pelajaran termahal sesi lalu — jangan diulang

**Guard berbasis grep membuktikan sambungan ADA. Ia tidak bisa membuktikan URUTAN BEKERJA.**

Tiga ronde berturut-turut aku melaporkan fitur yang tidak jalan sementara seluruh test suite hijau:

- `BUG-73` — area sentuh diikat ke geometri *live* yang ia sendiri hasilkan; umpan balik, setiap gestur tersendat
- `BUG-74` — Callout tak bisa diketik, karena kolom teks menutupi klik yang mestinya memindahkannya
- `BUG-75` — dobel-klik tak pernah sampai, karena **klik biasa menulis ke SQLite lalu me-reload model**, memusnahkan elemennya di tengah gestur

Setiap kali, guard-nya lulus. `OQ-23` (kelas test komposisi) masih terbuka dan itulah yang hilang.

Konsekuensinya untukmu: kalau perubahanmu menyentuh **gestur, urutan, atau sesuatu yang dirender**, katakan terus terang bahwa kamu tidak bisa mengujinya, dan minta aku yang menguji.

## Aturan lain yang sudah menggigit

- **Jangan pernah commit screenshot.** Repo ini publik; `korpus.yml` menolaknya
- **Decode output gambar di test.** Header PNG palsu pernah lolos lima wave dan tiga audit
- **`let _ =` pada `Result` yang menopang invariant itu cacat**, bukan gaya
- **Panic di proses desktop membunuh tray, hotkey, dan overlay sekaligus** — rilis Windows tak punya konsol, jadi Reviewer melihat *tidak terjadi apa-apa*
- **Warna hanya boleh di `apps/desktop/ui/theme.slint`.** Dijaga test
- **Perlakuan visual jadi komponen SEBELUM dipakai kedua kali** — `.constitution/project/design-system-guide.md`, dan `test_design_system.rs` menjaganya secara mekanis
- Sebelum merencanakan pekerjaan atas sebuah baris cacat, **grep simbol yang disebut `fix:`-nya** — register bisa basi tanpa suara

## Di mana otoritas desain berada

- `.constitution/project/design-system-guide.md` — aturannya
- `.how/finding/01-ux/assets/`, `.how/settings/01-ux/assets/`, `.how/bundle/01-ux/assets/` — desain G3 sebagai HTML. **Ini yang mengikat**, dan sesi lalu terbukti sudah menjawab pertanyaan yang kukira belum terjawab
- `archive/desktop-tauri/` — build React lama. Berisi jawaban untuk hal yang pernah dibuat sekali (context menu, Settings). Baca sebelum merancang ulang permukaan yang sudah pernah ada
- `D:\Developer\wiradigital.id\wira-desk` — produk desktopku yang lain. Model hotkey-nya jauh lebih baik dan sudah sebagian diadopsi; masih ada yang bisa diambil

## Yang baru saja mendarat

Dua commit besar, keduanya sudah di-push ke `main`:

- `a06a8f3` — **`CAP-11` utuh**: skema anotasi, port, burn, kanvas, handle `FR-33`, undo/redo, urutan z, context menu. Plus `FR-34`–`FR-38`
- `d816768` — **layar Settings** empat tab; **`DEC-010`** yang membuat `encoder_quality` nyata (PNG indexed, 26% dari lossless, error kanal 1); rasio resize; model hotkey dari wira-desk; bayangan window; pemindahan Vault

## BELUM DIUJI SIAPA PUN — ini yang harus kuuji dulu

Urut dari yang paling berisiko:

1. **Ukuran file capture** — ambil capture baru, bandingkan ukurannya dengan capture lama di Vault. Harusnya jauh lebih kecil dan tidak ada beda yang terlihat mata. Kalau tidak mengecil, jalur palette tidak kena
2. **Bayangan window** — murni Win32 (`DwmExtendFrameIntoClientArea`), tak pernah dilihat mata
3. **Pemindahan Vault** — Settings → Choose folder. Ia mengonfirmasi dan menghitung file dulu. **Pakai Vault yang tidak kusayangi.** Setelah pindah, Snapdown harus di-restart dan seluruh Finding harus tetap terbuka
4. **Rebind hotkey** — klik shortcut, tekan kombinasi. Coba juga: tekan huruf polos tanpa modifier (harus ditolak dengan alasan), dan `Ctrl+Alt+Del` (harus menyebut pemiliknya)
5. **Cek kedatangan hotkey** — setelah di-bind, tekan chord-nya di mana saja; barisnya harus bilang "Pressed just now"
6. **Slider resize** — Settings → Fine-tune → "Resize every capture to" 80%, lalu capture; dimensi tersimpan harus 80%
7. **Layar Settings secara desain** — checkbox (bukan switch), preset segmented, tab horizontal, Done di footer
8. **Tile Assemble** dan **perataan ikon** di strip resolusi/size

## Yang sudah kuuji dan berhasil

Marker & anotasi (gambar, geser, resize, handle, dobel-klik ketik, undo/redo, persistensi), burn ke Bundle, copy burned image, context menu, hapus banyak Finding sekaligus, capture dari tray, Open file location, slider tidak lagi tumbuh dari tengah.

## Cacat terbuka — 12

```
BUG-59  critical  Local API tidak ada, MCP Bridge tak bisa menyentuh produk sama sekali
BUG-7   high      riwayat repo publik masih memuat screenshot (separuh sisanya urusan pemilik)
BUG-23  high      publish menelan gagal-baca gambar dan menerbitkan tanpa gambarnya
BUG-54  high      theme.slint gagal WCAG AA di enam pasangan, termasuk label tombol utama dark mode
BUG-60  high      library.db korup → Library kosong senyap, kerja satu sesi hilang
BUG-61  high      sembilan permukaan Editor tanpa implementasi Slint (Library, Share, laci Bundle, orphan)
BUG-2   medium    tiga layar `sharing` tercatat terkirim, tidak ada di repo
BUG-8   medium    container web-ui tercatat built:true, tidak ada
BUG-37  medium    overlay capture tidak mengikuti design system Editor
BUG-57  medium    dua tombol toolbar masih stub (library, laci Bundle)
BUG-28  low       overlay capture butuh sesaat untuk muncul
BUG-77  low       Delete Finding & Open file location hanya lewat klik kanan — FR-37 melarangnya
```

**Saranku berikutnya: `BUG-59`.** Ia satu-satunya `critical` dan ia memblokir separuh alasan produk ini ada — Snapdown sekarang menghasilkan bundle beranotasi yang bagus, dan jalur agent-nya masih manual.

---

## HASIL PENGUJIANKU

*(diisi manual olehku sebelum mengirim — kalau bagian ini kosong, tanyakan dulu sebelum mulai coding)*

```
1. Ukuran file capture       →
2. Bayangan window           →
3. Pemindahan Vault          →
4. Rebind hotkey             →
5. Cek kedatangan hotkey     →
6. Slider resize             →
7. Desain layar Settings     →
8. Tile Assemble & ikon      →

Lainnya / catatan bebas:

```
