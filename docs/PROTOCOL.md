# ICARUS V2 MINI — Reverse-Engineered USB HID Protocol Specification

This document specifies the low-level USB HID wire protocol and EEPROM memory layout for the **ICARUS V2 MINI** gaming mouse (Vendor ID `0x3554`), reconstructed by reverse-engineering the official Windows driver binary (`ICARUS V2 MINI_v1.0.1.2_setup.exe` and `HIDUsb.dll`).

---

## 1. Hardware USB Identification (VID & PIDs)

| Connection Mode | Vendor ID (VID) | Product ID (PID) | MCU / Receiver IC | HID Endpoint |
|---|---|---|---|---|
| **USB Wired Mode** | `0x3554` | `0xF511` | Nordic nRF52840 | Interface 1, Report ID `0x08` |
| **2.4GHz Standard Dongle** | `0x3554` | `0xF512` | CX52650N | Interface 1, Report ID `0x08` |
| **4K / 4GHz High-Speed Dongle** | `0x3554` | `0xF510` | WCH CH32V305 | Interface 1, Report ID `0x08` |

> [!IMPORTANT]
> All vendor command frames (*Output Reports*) and device replies (*Input Reports*) are transmitted over **Interface 1** using a fixed **17-byte packet size** prefixed with **Report ID `0x08`**.

---

## 2. 17-Byte Frame Structure

Every HID transaction consists of exactly 17 bytes (`PACKET_SIZE = 17`):

```text
+--------+--------+--------+--------+--------+-------------------+--------+
| Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 |   Bytes 5 .. 15   | Byte 16|
+--------+--------+--------+--------+--------+-------------------+--------+
| Rpt ID | Opcode | Addr H | Addr L | Length | Payload (11 Byte) | Chksum |
|  0x08  |  u8    |  u8    |  u8    |  u8    |   [u8; 11]        |  u8    |
+--------+--------+--------+--------+--------+-------------------+--------+
```

- **Byte 0 (`0x08`)**: Vendor HID Report ID.
- **Byte 1 (`Opcode`)**: Command or hardware query opcode.
- **Byte 2..3 (`Address`)**: 16-bit EEPROM register address in **Big-Endian** order (`[ (addr >> 8) as u8, (addr & 0xFF) as u8 ]`). Set to `0x00, 0x00` for non-memory opcodes (such as battery queries).
- **Byte 4 (`Length`)**: Number of active payload bytes (`0` to `10`).
- **Byte 5..15 (`Payload`)**: Command payload data (zero-padded when unused).
- **Byte 16 (`Checksum`)**: Frame integrity verification byte.

### Frame & Sub-Block Checksum Invariant (`0x55 - sum`)
1. **17-Byte Packet Checksum (`Byte 16`)**:
   $$\text{Checksum}_{\text{frame}} = \left( \text{0x55} - \sum_{i=0}^{15} \text{Byte}[i] \right) \bmod 256$$
2. **EEPROM Sub-Block Checksum**:
   Most EEPROM data structures (2-byte register pairs, 4-byte DPI entries, and 4-byte RGB color blocks) include a trailing sub-block checksum computed using the same formula:
   $$\text{Checksum}_{\text{block}} = \left( \text{0x55} - \sum \text{DataBytes} \right) \bmod 256$$

---

## 3. Command Opcode Table (`Byte 1`)

| Opcode | Constant Name | OEM Symbol (`HIDUsb.dll`) | Description |
|---|---|---|---|
| `0x02` | `CMD_PC_DRIVER_STATUS` | `CS_UsbFinder_WritePcDriverStatus` | Notifies the MCU whether the host driver is active (`data[0] = 1` or `0`) |
| `0x03` | `CMD_QUERY_ONLINE` | `CS_UsbFinder_ReadDeviceOnline` | Queries whether the wireless mouse is awake and online (`resp.data[0] == 1`) |
| `0x04` | `CMD_QUERY_BATTERY` | `CS_UsbFinder_ReadBatteryStatus` | Reads battery percentage (`data[0]`), charging state (`data[1]`), and voltage in mV (`data[2..4]` BE) |
| `0x05` | `CMD_DONGLE_PAIR_CID_MID` | `CS_UsbServer_EnterDonglePairWithCidMid` | Initiates 2.4G pairing filtered by Customer ID (`CID`) and Model ID (`MID`) |
| `0x06` | `CMD_ENTER_DONGLE_PAIR` | `CS_UsbServer_EnterDonglePair` | Puts the 2.4GHz receiver into standard pairing mode |
| `0x07` | `CMD_WRITE` | `WriteCidMidData` / `WriteMemory` | Writes `N` bytes (up to 10 bytes per packet) to an EEPROM register address |
| `0x08` | `CMD_READ` | `ReadMemory` | Reads `N` bytes from an EEPROM register address |
| `0x09` | `CMD_READ_DONGLE_PAIR_STATUS` | `CS_UsbServer_ReadDonglePairStatus` | Queries 2.4GHz pairing completion status (`data[0] == 1` indicates success) |
| `0x0E` | `CMD_ENTER_UPDATE_MODE` | `CS_UsbFinder_EnterUpdateMode` | Switches the MCU into ISP bootloader mode for firmware flashing |
| `0x0F` | `CMD_APPLY_CONFIG` | `ApplyConfig` | Triggers hardware configuration reload |
| `0x14` | `CMD_SET_4K_DONGLE_RGB` | `CS_UsbFinder_Set4KDongleRGBValue` | Configures LED mode, brightness, speed, and RGB color on the 4K receiver dongle |
| `0x15` | `CMD_GET_4K_DONGLE_RGB` | `CS_UsbFinder_Get4KDongleRGB` | Reads active LED configuration from the 4K receiver dongle |
| `0x16` | `CMD_SET_LONGRANGE_MODE` | `CS_UsbFinder_SetLongRangeMode` | Enables or disables long-range RF transmission power boost |
| `0x17` | `CMD_GET_LONGRANGE_MODE` | `CS_UsbFinder_GetLongRangeMode` | Queries long-range RF mode status |
| `0x18` | `CMD_SET_DONGLE_RGB_BAR` | `CS_UsbFinder_SetDongleRGBBarMode` | Sets the 4K dongle status bar indicator mode |
| `0x19` | `CMD_GET_DONGLE_RGB_BAR` | `CS_UsbFinder_GetDongleRGBBarMode` | Reads the 4K dongle status bar indicator mode |

---

## 4. EEPROM Register Memory Map (`src/protocol/addr.rs`)

| Address (`u16`) | Constant Name | Size | Wire Format & Description |
|---|---|---|---|
| `0x0000` | `POLLING_RATE` / Commit | 2 Bytes | Writing `[profile_id, 0x55 - profile_id]` commits/activates profile (`1..=5`). For Polling Rate: `[code, 0x55 - code]` (`0x01`=1000Hz, `0x02`=500Hz, `0x04`=250Hz, `0x08`=125Hz) |
| `0x0002` | `MAX_DPI_STAGES` | 2 Bytes | `[count, 0x55 - count]` — Number of enabled DPI stages (`1..=6`) |
| `0x0004` | `ACTIVE_DPI_STAGE` | 2 Bytes | `[stage_idx, 0x55 - stage_idx]` — Active DPI stage index (`0..=5`) |
| `0x000A` | `LOD` | 2 Bytes | `[lod, 0x55 - lod]` — Lift-Off Distance (`0x01` = 1mm, `0x02` = 2mm) |
| `0x000C` + `4*i` | `DPI_STAGE_BASE` | 4 Bytes/stage | `[val_x_low, val_y_low, high_bits_xy, chk]` — 10-bit PAW3395 DPI entry for stage `i` (`0..8`) |
| `0x002C` + `4*i` | `DPI_COLOR_BASE` | 4 Bytes/stage | `[R, G, B, 0x55 - (R + G + B)]` — RGB indicator color for DPI stage `i` |
| `0x004C` | `LED_EFFECT_MODE` | 2 Bytes | `[mode, 0x55 - mode]` (`0x00`=Off, `0x01`=Solid, `0x02`=Breathing, `0x03`=Neon) |
| `0x004E` | `LED_BRIGHTNESS` | 2 Bytes | `[level, 0x55 - level]` (`0x00`..`0x0A`) |
| `0x0050` | `LED_SPEED` | 2 Bytes | `[speed, 0x55 - speed]` (`0x01`..`0x0A`) |
| `0x0060` + `4*b` | `BUTTON_MAP_BASE` | 4 Bytes/btn | `[action_type, param1, param2, chk]` for physical button `b` (`0..6`) |
| `0x0090` | `FW_VERSION` | 4 Bytes | `[major, minor, patch, build]` hardware firmware version |
| `0x00A0` | `DPI_LED_MODE` | 6–7 Bytes | Composite DPI LED packet (`[mode, brightness, speed, R, G, B, chk]`) |
| `0x00A9` | `DEBOUNCE` | 2 Bytes | `[ms, 0x55 - ms]` — Switch debounce time in even 2ms steps (`0, 2, 4, .., 20` ms) |
| `0x00AB` | `MOTION_SYNC` | 2 Bytes | `[en, 0x55 - en]` (`1` = Enabled, `0` = Disabled) |
| `0x00AD` | `SLEEP_TIMEOUT` | 2 Bytes | `[units_10s, 0x55 - units_10s]` — Idle sleep timeout in 10-second units |
| `0x00AF` | `ANGLE_SNAP` | 2 Bytes | `[en, 0x55 - en]` (`1` = Enabled, `0` = Disabled) |
| `0x00B1` | `RIPPLE_CONTROL` | 2 Bytes | `[en, 0x55 - en]` (`1` = Enabled, `0` = Disabled) |
| `0x00B5` | `PERFORMANCE_MODE` | 2 Bytes | `[en, 0x55 - en]` — High-performance sensor mode |
| `0x00B7` | `SENSOR_SLEEP_SECONDS` | 2 Bytes | `[hi, lo]` — Sensor sleep timeout in seconds (Big-Endian `u16`) |
| `0x0100` + `32*b` | `KEY_COMBINATION_BASE` | 32 Bytes/btn | Extended button action payload for Keyboard Shortcuts, Media Keys, or FireKey |
| `0x0300` + `384*s`| `MACRO_SLOT_BASE_ADDR` | 384 Bytes/slot| On-board macro bytecode buffer for slot `s` (`0..16`) |

---

## 5. Bijective 10-Bit PAW3395 DPI Encoding (`src/dpi.rs`)

The **PixArt PAW3395** optical sensor uses 10-bit register indices (`0..=519` covering `50..=26,000` DPI in 50 DPI increments):
- For $\text{DPI} \le 10,100$: uses the non-linear OEM lookup table `SENSOR_3335` (`0x00`..`0xCE`).
- For $\text{DPI} > 10,100$: uses $\text{val}_{10} = \lfloor (\text{DPI} - 50) / 50 \rfloor$.

The 4-byte entry at `0x000C + (stage * 4)` is encoded bijectively to support **Separate X/Y DPI** across the entire 50–26,000 DPI range:
- `Byte 0`: `val_x & 0xFF` (lower 8 bits of X-axis index)
- `Byte 1`: `val_y & 0xFF` (lower 8 bits of Y-axis index)
- `Byte 2`:
  - When $\text{DPI}_X == \text{DPI}_Y$: `(val_x >> 8) & 0x03` (100% byte-identical with the Windows OEM driver).
  - When $\text{DPI}_X \neq \text{DPI}_Y$: `((val_x >> 8) & 0x03) | (((val_y >> 8) & 0x03) << 2) | 0x80` (bit `0x80` flags independent X/Y upper bits).
- `Byte 3`: `(0x55 - Byte0 - Byte1 - Byte2) & 0xFF`.

---

## 6. On-Board Macro Bytecode Format (`0x0300`, 384 Bytes per Slot)

Reconstructed from `MacroKeyToBuffer` and `BufferToMacroKey` in `HIDUsb.dll`:

```text
+----------+--------------------+----------+-----------------------------------+----------+
| Offset 0 |   Offsets 1 .. 30  | Offset 31|       Offsets 32 .. 32 + 5*N      | Trailing |
+----------+--------------------+----------+-----------------------------------+----------+
| name_len | Macro Name (UTF-8) | evt_count| 5-Byte Event Array (Max 64 Events)| Checksum |
|   (u8)   |   (Max 30 Bytes)   |   (u8)   |     [flags, 0x00, usage, dL, dH]  |   (u8)   |
+----------+--------------------+----------+-----------------------------------+----------+
```

Each macro event occupies **5 bytes**:
- `Byte 0` (`flags`): `0x80` = `KeyDown`, `0x40` = `KeyUp`, `0x00` = `Delay`.
- `Byte 1`: `0x00` (reserved).
- `Byte 2` (`usage`): Standard USB HID Usage code for keyboard keys (`0x04`..`0xE7`) or mouse buttons (`0xF0`..`0xF4`). Always `0x00` for `Delay` events.
- `Byte 3..4` (`delay_ms`): Delay duration in milliseconds (`u16` Little-Endian, clamped to `1..=65535`).
