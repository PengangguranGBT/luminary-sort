# luminary-sort

Fast, minimal black-and-white photo & video sorter for photographers and videographers.

Built with **Rust**, **egui 0.27**, and **eframe**. Monochrome-only UI, no sticker symbols, lightweight and efficient.

![luminary-sort GUI](https://img.shields.io/badge/status-working-brightgreen)
![Rust](https://img.shields.io/badge/rust-1.81+-orange)
![License: MIT](https://img.shields.io/badge/license-MIT-blue)

---

## Fitur

- **Import** folder foto/video dan buat index (`.lumi`)
- **Search & filter** berdasarkan nama file dan tags (case-insensitive)
- **Multi-selection**: klik tunggal, Ctrl+klik (toggle), Shift+klik (range), Ctrl+A (semua)
- **Navigasi keyboard**: ↑/↓ pindah seleksi, Enter buka lightbox, Esc hapus seleksi/tutup lightbox
- **Lightbox preview** dengan tampilan gambar besar + metadata (file, jenis, ukuran, tanggal, tags, rating, session)
- **Edit metadata di lightbox**: rating (0–5), flagged (⚑/☆), tags (pisah koma)
- **Edit di baris daftar**: rating (drag), flagged (toggle)
- **Project management**: buat, daftar, pilih, hapus, export project
- **Export**: file terpilih atau seluruh project ke folder tujuan
- **Auto-save**: index otomatis tersimpan setelah perubahan project/rating/flagged/tags
- **Status bar**: notifikasi sukses/error dengan tombol hapus
- **UI minimal hitam-putih**, tanpa simbol stiker

---

## Cara Install & Build

```bash
# Clone repo
git clone https://github.com/<username>/luminary-sort.git
cd luminary-sort

# Build development
cargo build

# Build release (disarankan untuk penggunaan sehari-hari)
cargo build --release

# Binary ada di target/release/luminary-sort
```

---

## Cara Penggunaan

### CLI (Tanpa GUI)

```bash
# 1. Scan folder dan buat index baru
cargo run -- import /path/to/folder/foto --index index.lumi

# 2. Lihat isi index
cargo run -- list --index index.lumi

# 3. Buat selection berdasarkan tag/rating
cargo run -- select --index index.lumi --tags vacation --rating 4 --out selection.lumi

# 4. Export selection ke folder
cargo run -- export --selection selection.lumi --dest /path/to/export

# 5. Buat thumbnail
cargo run -- thumbnail --root /path/to/foto --dest /path/to/thumbs --max-dim 512

# 6. Project management
cargo run -- project --index index.lumi --action add --name "Wedding 2026" --tags wedding
cargo run -- project --index index.lumi --action list
cargo run -- project --index index.lumi --action export --name "Wedding 2026" --dest /path/to/export
```

### GUI

```bash
# Jalankan GUI dan load index langsung
cargo run -- gui index.lumi
```

#### Langkah demi langkah di GUI:

| Aksi | Caranya |
|------|---------|
| Muat index | File → Import → masukkan path `.lumi` → Load index |
| Cari file | Ketik di kolom "Search:" di bawah |
| Pilih satu file | Klik baris file |
| Pilih banyak (Ctrl) | Ctrl + klik setiap file |
| Pilih range (Shift) | Klik file pertama, lalu Shift + klik file terakhir |
| Pilih semua | Ctrl + A |
| Hapus seleksi | Klik file yang dipilih lagi, atau tekan Esc |
| Buka lightbox | Double-klik file, atau Enter |
| Tutup lightbox | Klik ✕ Close, atau Esc |
| Edit rating | Geser angka ★ (0–5) di baris atau lightbox |
| Edit flagged | Klik ☆ / ⚑ di baris atau lightbox |
| Edit tags | Di lightbox, edit teks tags pisah koma |
| Buat project | Ketik nama di "New project:", klik Create |
| Pilih project | Klik ✓ di baris project |
| Hapus project | Klik ✕ di baris project |
| Export file terpilih | File → Export → masukkan folder → Export selected |
| Export project | Pilih project → masukkan Destination → Export |

#### Navigasi Keyboard

| Tombol | Fungsi |
|--------|--------|
| ↑ / ↓ | Pindah seleksi ke file atas/bawah |
| Shift + ↑ / ↓ | Perpanjang seleksi range |
| Enter | Buka lightbox untuk file terpilih |
| Esc | Tutup lightbox (jika terbuka) atau hapus seleksi |
| Ctrl + A | Pilih semua file yang terfilter |

---

## Contoh Workflow Lengkap

```bash
# 1. Scan folder foto dan buat index
cargo run -- import ~/Pictures/Vacation --index vacation.lumi

# 2. Buka GUI
cargo run -- gui vacation.lumi

# 3. Di GUI: cari "beach", pilih foto favorit (Ctrl+klik)
# 4. Beri rating 5 pada foto terbaik (drag kolom ★)
# 5. Tandai foto diedit dengan ⚑
# 6. Buat project "Best Shots", tambahkan file favorit
# 7. Export project ke ~/Exports/BestShots
```

---

## Struktur Project

```
luminary-sort/
├── Cargo.toml          # Manifes crate dan dependensi
├── Cargo.lock
├── .gitignore
├── README.md           # Dokumentasi ini
├── src/
│   ├── main.rs         # Entry point CLI + GUI (clap + eframe)
│   ├── model.rs        # Data model (Index, MediaFile, Project, MediaKind)
│   ├── gui.rs          # Antarmuka egui (grid, search, lightbox, project, export)
│   ├── export.rs       # Logika export file ke folder
│   ├── collection.rs   # Penanganan koleksi/project
│   ├── thumbnail.rs    # Generasi thumbnail gambar/video
│   ├── scan.rs         # Scan folder media
│   └── metadata.rs     # Ekstraksi metadata file
└── target/             # Build output (diabaikan git)
```

---

## Teknologi

| Komponen | Library | Versi |
|----------|---------|-------|
| Bahasa | Rust | 2021 edition |
| CLI parsing | clap | 4 |
| GUI | egui / eframe | 0.27 |
| Serialisasi | ron, serde | 0.8 / 1.0 |
| UUID | uuid | 1 (v4) |
| Image processing | image | 0.25 |
| Date/time | chrono | 0.4 |
| Parallelism | rayon | 1.10 |
| Error handling | thiserror, anyhow | 2.0 / 1.0 |
| Walk directory | walkdir | 2.5 |

---

## Lisensi

MIT License. Lihat file `LICENSE` untuk detail.

---

## Kontribusi

Pull request sangat diterima. Untuk perubahan besar, silakan buka issue terlebih dahulu untuk membahas apa yang ingin diubah.

1. Fork repo ini
2. Buat branch fitur (`git checkout -b fitur/nama-fitur`)
3. Commit perubahan (`git commit -am 'Tambah fitur X'`)
4. Push ke branch (`git push origin fitur/nama-fitur`)
5. Buka Pull Request

---

## Kontak

Buat laporan bug atau saran fitur, buka issue di GitHub repository ini.
