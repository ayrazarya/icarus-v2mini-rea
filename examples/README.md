# Examples & Hardware Diagnostic Utilities

Direktori `examples/` berisi contoh file konfigurasi profil serta utilitas diagnostik perangkat keras untuk mouse **ICARUS V2 MINI**.

---

## Daftar File

### 1. [`profile-default.toml`](profile-default.toml)
Contoh file konfigurasi profil lengkap dalam format TOML yang mencakup:
- Pengaturan sensor PixArt PAW3395 (`polling_rate`, `lod`, `angle_snapping`, `motion_sync`, `ripple_control`, `debounce_ms`).
- Konfigurasi 5 stage DPI beserta warna indikator RGB masing-masing stage.
- Konfigurasi efek pencahayaan LED (`mode`, `brightness`, `speed`, `color`).
- Pemetaan 6 tombol fisik mouse (`LeftClick`, `RightClick`, `MiddleClick`, `Forward`, `Back`, `DpiSwitch`).

**Cara menerapkan profil contoh ini ke mouse Anda:**
```bash
# Melalui CLI
icarus-v2mini apply examples/profile-default.toml

# Menyimpan konfigurasi mouse saat ini ke file TOML baru
icarus-v2mini save ~/my-esports-profile.toml
```

---

### 2. [`dpi_probe.rs`](dpi_probe.rs)
Utilitas diagnostik tingkat rendah (*low-level hardware probe*) untuk menginspeksi cara firmware mouse dan dongle membaca/menulis register DPI (`0x000C`..`0x002B`) serta menguji respons berbagai format paket HID secara langsung.

**Cara menjalankan utilitas `dpi_probe`:**
```bash
cargo run --example dpi_probe
```

> [!NOTE]
> Pastikan mouse atau dongle 2.4G/4K Anda terhubung dan aturan udev (`99-icarus-mouse.rules`) sudah terpasang sebelum menjalankan `dpi_probe`.
