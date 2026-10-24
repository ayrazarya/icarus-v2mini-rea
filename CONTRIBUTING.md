# Panduan Cara Koding & Kontribusi (Contributing Guide)

Terima kasih atas minat Anda untuk mempelajari atau berkontribusi pada proyek **ICARUS V2 MINI Linux Driver & Native GUI**! Dokumen ini menjelaskan standar penulisan kode (*coding standards*), aturan arsitektur, cara menambahkan fitur baru, serta prosedur pengujian (*Quality Assurance*) yang wajib dipatuhi.

---

## 1. Standar & Pedoman Koding Rust (Rust 2024 Edition)

Proyek ini menerapkan aturan kompilator dan *linter* yang sangat ketat di [`Cargo.toml`](Cargo.toml):

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
redundant_clone = "deny"
clone_on_copy = "deny"
needless_collect = "deny"
manual_while_let_some = "deny"
or_fun_call = "warn"
```

### Aturan Utama yang Wajib Dipatuhi:
1. **Dilarang Menggunakan `unsafe` (`unsafe_code = "forbid"`)**:
   - Gunakan *slice indexing* dengan pemeriksaan batas (`.get(...)`), *pattern matching*, dan fungsi konversi byte bawaan (`u16::from_le_bytes`, `u16::from_be_bytes`).
   - Saat memotong string UTF-8 (misalnya nama makro 30-byte di EEPROM), selalu gunakan `.char_indices()` agar tidak pernah memotong di tengah karakter multi-byte.
2. **Evaluasi Waktu Kompilasi (`const fn` & `#[must_use]`)**:
   - Fungsi murni (*pure functions*) yang menghitung nilai register, encoding/decoding byte, atau membangun paket `Command` wajib dideklarasikan sebagai `#[must_use] pub const fn`.
3. **Hindari Alokasi Memori yang Tidak Perlu (*Zero-Allocation Mindset*)**:
   - Gunakan `&str` atau `Cow<'static, str>` alih-alih mengalokasikan `String` baru pada jalur panas (*hot path*).
   - Gunakan `std::mem::take` ketika memindahkan `Vec` antar-struct konfigurasi daripada melakukan `.clone()`.
   - Jangan memanggil `.unwrap_or(expensive_fn())`; gunakan `.unwrap_or_else(|| expensive_fn())`.
4. **Penanganan Error yang Eksplisit (`thiserror` & `anyhow`)**:
   - **Jangan gunakan `.unwrap()` atau `.expect()`** pada kode produksi (di luar `tests/`).
   - Di dalam modul protokol (`src/protocol/`) dan perangkat (`src/device/`), kembalikan error terstruktur (`ProtocolError` atau `anyhow::Result` dengan `.context(...)`).
   - Pastikan operasi pada mouse wireless yang sedang *sleep/offline* melakukan *fail-fast* melalui `dev.read_device_online()?` agar tidak menyebabkan *freeze* akibat *timeout* beruntun.
5. **Menjaga Integritas Dokumentasi**:
   - Setiap tipe publik (`pub struct`, `pub enum`, `pub fn`) harus memiliki doc-comment (`///`) yang menjelaskan tujuan dan invariannya.

---

## 2. Cara Menambahkan Fitur Baru (Step-by-Step Workflow)

### A. Menambahkan Register atau Perintah Perangkat Keras Baru
1. **Tambahkan Konstanta Alamat/Opcode**:
   - Daftarkan alamat EEPROM di [`src/protocol/addr.rs`](src/protocol/addr.rs) atau opcode di [`src/protocol/mod.rs`](src/protocol/mod.rs).
2. **Buat Konstruktor Paket (`const fn`)**:
   - Tambahkan metode builder `#[must_use] pub const fn` pada `Command` di [`src/protocol/command.rs`](src/protocol/command.rs).
3. **Tambahkan Metode Driver di `src/device/`**:
   - Implementasikan fungsi baca/tulis pada submodul yang sesuai di [`src/device/`](src/device/mod.rs) (misalnya `sensor.rs` atau `lighting.rs`).
   - Jangan lupa memanggil `dev.commit_config(1)?` pada subcommand CLI mandiri jika register tersebut membutuhkan *commit* agar langsung aktif di perangkat keras.
4. **Tambahkan Unit Test Protokol**:
   - Tambahkan pengujian byte-level dan verifikasi checksum di [`tests/test_protocol.rs`](tests/test_protocol.rs).

### B. Menambahkan Subcommand CLI Baru
1. Tambahkan varian enum pada `Commands` di [`src/cli/mod.rs`](src/cli/mod.rs).
2. Implementasikan fungsi handler `cmd_*` di dalam [`src/cli/commands/`](src/cli/commands/info.rs).
3. Hubungkan routing-nya pada fungsi `run_cli()` di [`src/main.rs`](src/main.rs).

### C. Menambahkan Fitur atau Kontrol Baru di GUI
1. **Tambahkan String Terjemahan (EN & ID)**:
   - Tambahkan metode terjemahan pada `AppLanguage` di submodul [`src/i18n/`](src/i18n/mod.rs) yang sesuai, dan pastikan terjemahan Bahasa Inggris (`English`) maupun Bahasa Indonesia (`Indonesian`) lengkap tanpa pencampuran bahasa.
2. **Perbarui State atau Tab UI**:
   - Tambahkan komponen UI di [`src/gui/tabs/`](src/gui/tabs/sensor.rs) dan gunakan throttling (`queue_lighting_sync` / `queue_dongle_sync`) jika kontrol tersebut berupa slider *real-time*.
3. **Tambahkan Pengujian Lokalisasi**:
   - Verifikasi string baru di [`tests/test_i18n.rs`](tests/test_i18n.rs).

---

## 3. Checklist Verifikasi QA Sebelum Commit / Pull Request

Sebelum mengirimkan perubahan kode, jalankan rangkaian perintah verifikasi berikut dari direktori root proyek dan pastikan semuanya lulus tanpa *warning* atau *error*:

```bash
# 1. Format seluruh kode sesuai standar rustfmt
cargo fmt --all --check

# 2. Jalankan analisis statis Clippy pada semua target & fitur (0 warning diizinkan)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Jalankan seluruh suite pengujian unit & integrasi (51+ tests)
cargo test --workspace --all-features

# 4. Pastikan kompilasi profil release berjalan bersih
cargo build --release
```
