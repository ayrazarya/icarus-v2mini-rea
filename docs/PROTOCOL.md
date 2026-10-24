# ICARUS V2 MINI — Reverse-Engineered USB HID Protocol Specification

Dokumen ini mendokumentasikan protokol komunikasi perangkat keras **ICARUS V2 MINI** (Vendor ID `0x3554`) yang direkonstruksi melalui *reverse engineering* dari pustaka resmi Windows (`HIDUsb.dll` dan `ICARUS V2 MINI_v1.0.1.2_setup.exe`).

---

## 1. Identifikasi Perangkat Keras (USB VID & PID)

| Mode Koneksi | Vendor ID (VID) | Product ID (PID) | MCU / IC Penerima | Endpoint Komunikasi |
|---|---|---|---|---|
| **USB Wired (Kabel)** | `0x3554` | `0xF511` | Nordic nRF52840 | Interface 1, Report ID `0x08` |
| **2.4GHz Standard Dongle** | `0x3554` | `0xF512` | CX52650N | Interface 1, Report ID `0x08` |
| **4K / 4GHz High-Speed Dongle** | `0x3554` | `0xF510` | WCH CH32V305 | Interface 1, Report ID `0x08` |

> [!IMPORTANT]
> Seluruh paket perintah (*Output Report*) dan balasan (*Input Report*) dikirim melalui **Interface 1** dengan ukuran tetap **17 byte** dan selalu diawali oleh **Report ID `0x08`**.

---

## 2. Struktur Frame Paket 17-Byte

Setiap transaksi HID terdiri dari tepat 17 byte (`PACKET_SIZE = 17`):

```text
+--------+--------+--------+--------+--------+-------------------+--------+
| Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 |   Bytes 5 .. 15   | Byte 16|
+--------+--------+--------+--------+--------+-------------------+--------+
| Rpt ID | Opcode | Addr H | Addr L | Length | Payload (11 Byte) | Chksum |
|  0x08  |  u8    |  u8    |  u8    |  u8    |   [u8; 11]        |  u8    |
+--------+--------+--------+--------+--------+-------------------+--------+
```

- **Byte 0 (`0x08`)**: Vendor HID Report ID.
- **Byte 1 (`Opcode`)**: Jenis perintah atau kueri perangkat keras.
- **Byte 2..3 (`Address`)**: Alamat register EEPROM 16-bit dalam format **Big-Endian** (`[ (addr >> 8) as u8, (addr & 0xFF) as u8 ]`). Untuk perintah non-memori (seperti kueri baterai), bernilai `0x00, 0x00`.
- **Byte 4 (`Length`)**: Jumlah byte aktif di dalam payload (`0` hingga `10`).
- **Byte 5..15 (`Payload`)**: Data parameter perintah (diisi `0x00` jika tidak terpakai).
- **Byte 16 (`Checksum`)**: Byte verifikasi integritas paket.

### Algoritma Checksum Frame & Sub-Blok (`0x55 - sum`)
1. **Checksum Paket 17-Byte (`Byte 16`)**:
   $$\text{Checksum}_{\text{frame}} = \left( \text{0x55} - \sum_{i=0}^{15} \text{Byte}[i] \right) \bmod 256$$
2. **Checksum Sub-Blok Memori EEPROM**:
   Banyak struktur data di dalam EEPROM (seperti pasangan byte 2-byte, blok DPI 4-byte, dan warna RGB 4-byte) memiliki byte *checksum* internal di akhir blok dengan rumus yang sama:
   $$\text{Checksum}_{\text{block}} = \left( \text{0x55} - \sum \text{DataBytes} \right) \bmod 256$$

---

## 3. Tabel Opcode Perintah (`Byte 1`)

| Opcode | Nama Konstanta | Fungsi OEM (`HIDUsb.dll`) | Deskripsi |
|---|---|---|---|
| `0x02` | `CMD_PC_DRIVER_STATUS` | `CS_UsbFinder_WritePcDriverStatus` | Memberitahu MCU bahwa driver PC sedang aktif (`data[0] = 1` atau `0`) |
| `0x03` | `CMD_QUERY_ONLINE` | `CS_UsbFinder_ReadDeviceOnline` | Mengecek apakah mouse wireless sedang aktif/terhubung (`resp.data[0] == 1`) |
| `0x04` | `CMD_QUERY_BATTERY` | `CS_UsbFinder_ReadBatteryStatus` | Membaca persentase baterai (`data[0]`), status cas (`data[1]`), dan tegangan mV (`data[2..4]` BE) |
| `0x05` | `CMD_DONGLE_PAIR_CID_MID` | `CS_UsbServer_EnterDonglePairWithCidMid` | Memulai pairing 2.4G dengan filter Customer ID (`CID`) dan Model ID (`MID`) |
| `0x06` | `CMD_ENTER_DONGLE_PAIR` | `CS_UsbServer_EnterDonglePair` | Memasuki mode pairing 2.4GHz standar |
| `0x07` | `CMD_WRITE` | `WriteCidMidData` / `WriteMemory` | Menulis `N` byte (maks 10 byte per paket) ke alamat register EEPROM |
| `0x08` | `CMD_READ` | `ReadMemory` | Membaca `N` byte dari alamat register EEPROM |
| `0x09` | `CMD_READ_DONGLE_PAIR_STATUS` | `CS_UsbServer_ReadDonglePairStatus` | Membaca status proses pairing dongle 2.4GHz (`data[0] == 1` berarti sukses) |
| `0x0E` | `CMD_ENTER_UPDATE_MODE` | `CS_UsbFinder_EnterUpdateMode` | Memasukkan MCU ke mode bootloader ISP untuk pembaruan firmware |
| `0x0F` | `CMD_APPLY_CONFIG` | `ApplyConfig` | Menerapkan perubahan register sementara |
| `0x14` | `CMD_SET_4K_DONGLE_RGB` | `CS_UsbFinder_Set4KDongleRGBValue` | Mengatur warna & mode LED pada receiver dongle 4K |
| `0x15` | `CMD_GET_4K_DONGLE_RGB` | `CS_UsbFinder_Get4KDongleRGB` | Membaca konfigurasi LED dari receiver dongle 4K |
| `0x16` | `CMD_SET_LONGRANGE_MODE` | `CS_UsbFinder_SetLongRangeMode` | Mengaktifkan/menonaktifkan penguatan sinyal RF jarak jauh |
| `0x17` | `CMD_GET_LONGRANGE_MODE` | `CS_UsbFinder_GetLongRangeMode` | Membaca status mode RF jarak jauh |
| `0x18` | `CMD_SET_DONGLE_RGB_BAR` | `CS_UsbFinder_SetDongleRGBBarMode` | Mengatur mode indikator bar LED pada dongle |
| `0x19` | `CMD_GET_DONGLE_RGB_BAR` | `CS_UsbFinder_GetDongleRGBBarMode` | Membaca mode indikator bar LED pada dongle |

---

## 4. Peta Register Memori EEPROM (`src/protocol/addr.rs`)

| Alamat (`u16`) | Nama Konstanta | Ukuran | Format Data & Penjelasan |
|---|---|---|---|
| `0x0000` | `POLLING_RATE` / Commit | 2 Byte | Saat ditulis `[profile_id, 0x55 - profile_id]`: menyimpan/mengaktifkan profil (`1..=5`). Saat dibaca/ditulis untuk Polling Rate: `[code, 0x55 - code]` (`0x01`=1000Hz, `0x02`=500Hz, `0x04`=250Hz, `0x08`=125Hz) |
| `0x0002` | `MAX_DPI_STAGES` | 2 Byte | `[count, 0x55 - count]` — Jumlah stage DPI aktif (`1..=6`) |
| `0x0004` | `ACTIVE_DPI_STAGE` | 2 Byte | `[stage_idx, 0x55 - stage_idx]` — Indeks stage DPI aktif (`0..=5`) |
| `0x000A` | `LOD` | 2 Byte | `[lod, 0x55 - lod]` — Lift-Off Distance (`0x01` = 1mm, `0x02` = 2mm) |
| `0x000C` + `4*i` | `DPI_STAGE_BASE` | 4 Byte/stage | `[val_x_low, val_y_low, high_bits_xy, chk]` — Nilai DPI 10-bit untuk stage `i` (`0..8`) |
| `0x002C` + `4*i` | `DPI_COLOR_BASE` | 4 Byte/stage | `[R, G, B, 0x55 - (R + G + B)]` — Warna RGB untuk indikator stage DPI `i` |
| `0x004C` | `LED_EFFECT_MODE` | 2 Byte | `[mode, 0x55 - mode]` (`0x00`=Off, `0x01`=Solid, `0x02`=Breathing, `0x03`=Neon) |
| `0x004E` | `LED_BRIGHTNESS` | 2 Byte | `[level, 0x55 - level]` (`0x00`..`0x0A`) |
| `0x0050` | `LED_SPEED` | 2 Byte | `[speed, 0x55 - speed]` (`0x01`..`0x0A`) |
| `0x0060` + `4*b` | `BUTTON_MAP_BASE` | 4 Byte/btn | `[action_type, param1, param2, chk]` untuk tombol `b` (`0..6`) |
| `0x0090` | `FW_VERSION` | 4 Byte | `[major, minor, patch, build]` versi firmware perangkat keras |
| `0x00A0` | `DPI_LED_MODE` | 6–7 Byte | Paket komposit LED DPI (`[mode, brightness, speed, R, G, B, chk]`) |
| `0x00A9` | `DEBOUNCE` | 2 Byte | `[ms, 0x55 - ms]` — Waktu debounce tombol dalam kelipatan genap (`0, 2, 4, .., 20` ms) |
| `0x00AB` | `MOTION_SYNC` | 2 Byte | `[en, 0x55 - en]` (`1` = Aktif, `0` = Nonaktif) |
| `0x00AD` | `SLEEP_TIMEOUT` | 2 Byte | `[units_10s, 0x55 - units_10s]` — Waktu tidur idle dalam satuan 10 detik |
| `0x00AF` | `ANGLE_SNAP` | 2 Byte | `[en, 0x55 - en]` (`1` = Aktif, `0` = Nonaktif) |
| `0x00B1` | `RIPPLE_CONTROL` | 2 Byte | `[en, 0x55 - en]` (`1` = Aktif, `0` = Nonaktif) |
| `0x00B5` | `PERFORMANCE_MODE` | 2 Byte | `[en, 0x55 - en]` — Mode performa tinggi sensor |
| `0x00B7` | `SENSOR_SLEEP_SECONDS` | 2 Byte | `[hi, lo]` — Waktu tidur sensor dalam detik (Big-Endian `u16`) |
| `0x0100` + `32*b` | `KEY_COMBINATION_BASE` | 32 Byte/btn | Konfigurasi detail tombol untuk Keyboard Shortcut, Media Key, atau FireKey |
| `0x0300` + `384*s`| `MACRO_SLOT_BASE_ADDR` | 384 Byte/slot| Buffer bytecode makro on-board untuk slot `s` (`0..16`) |

---

## 5. Encoding 10-Bit DPI PixArt PAW3395 (`src/dpi.rs`)

Sensor **PixArt PAW3395** menggunakan indeks register 10-bit (`0..=519` untuk `50..=26,000` DPI dalam langkah 50 DPI):
- Untuk $\text{DPI} \le 10.100$: menggunakan tabel lookup non-linear `SENSOR_3335` (`0x00`..`0xCE`).
- Untuk $\text{DPI} > 10.100$: menggunakan formula $\text{val}_{10} = \lfloor (\text{DPI} - 50) / 50 \rfloor$.

Keempat byte pada alamat `0x000C + (stage * 4)` dikodekan secara bijektif untuk mendukung **Separate X/Y DPI**:
- `Byte 0`: `val_x & 0xFF` (8-bit bawah sumbu X)
- `Byte 1`: `val_y & 0xFF` (8-bit bawah sumbu Y)
- `Byte 2`:
  - Jika $\text{DPI}_X == \text{DPI}_Y$: `(val_x >> 8) & 0x03` (kompatibel 100% dengan driver Windows OEM).
  - Jika $\text{DPI}_X \neq \text{DPI}_Y$: `((val_x >> 8) & 0x03) | (((val_y >> 8) & 0x03) << 2) | 0x80` (bit `0x80` menandakan bit tinggi X dan Y dikemas secara independen).
- `Byte 3`: `(0x55 - Byte0 - Byte1 - Byte2) & 0xFF`.

---

## 6. Format Bytecode Makro On-Board (`0x0300`, 384 Byte per Slot)

Direkonstruksi dari fungsi `MacroKeyToBuffer` dan `BufferToMacroKey` di `HIDUsb.dll`:

```text
+----------+--------------------+----------+-----------------------------------+----------+
| Offset 0 |   Offsets 1 .. 30  | Offset 31|       Offsets 32 .. 32 + 5*N      | Akhir    |
+----------+--------------------+----------+-----------------------------------+----------+
| name_len | Nama Makro (UTF-8) | evt_count| Array Event 5-Byte (Maks 64 Event)| Checksum |
|   (u8)   |   (Maks 30 Byte)   |   (u8)   |     [flags, 0x00, usage, dL, dH]  |   (u8)   |
+----------+--------------------+----------+-----------------------------------+----------+
```

Setiap event makro berukuran **5 byte**:
- `Byte 0` (`flags`): `0x80` = `KeyDown`, `0x40` = `KeyUp`, `0x00` = `Delay`.
- `Byte 1`: `0x00` (dicadangkan).
- `Byte 2` (`usage`): Kode USB HID Usage tombol keyboard (`0x04`..`0xE7`) atau tombol mouse (`0xF0`..`0xF4`). Bernilai `0x00` untuk event `Delay`.
- `Byte 3..4` (`delay_ms`): Durasi tunda dalam milidetik (`u16` Little-Endian, `1..=65535`).
