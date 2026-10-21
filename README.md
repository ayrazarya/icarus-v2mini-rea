# ICARUS V2 MINI — Linux Driver & Native GUI

Native, high-performance Linux driver, configuration tool, and desktop GUI for the **ICARUS V2 MINI** ultralight gaming mouse, written from the ground up in Rust.

Reverse-engineered directly from the official Windows vendor driver (`ICARUS V2 MINI_v1.0.1.2_setup.exe` & `HIDUsb.dll`). It communicates directly with the mouse and wireless receiver via raw USB/HID on Linux **without Wine, Windows, or heavy runtime dependencies**.

---

##  Features & Highlights

- **Modern Native Desktop GUI**: Built using `egui` and `eframe`. Fluid 60fps interface with zero web-engine bloat (consumes <30MB RAM).
- **100% DPI Accuracy (PixArt PAW3395)**: Uses the exact sensor register lookup table extracted from official driver headers (`driver_sensor.h`), eliminating DPI mismatch issues (e.g. 6500 DPI correctly maps to register `0x4D`, not a miscalculated linear value).
- **Hardware-Accurate DPI Indicator Lamp**: The mouse LED is tied directly to the active DPI stage register (`0x2C + stage * 4`), composite packets (`0xA0`), and direct hardware registers (`0x4C..0x52`). Supports per-stage color customization, Esports presets, 14 reverse-engineered color chips, breathing, neon, solid, and off modes.
- **Accurate 6-Button Physical Layout**:
  - `1`: Left Click
  - `2`: Right Click
  - `3`: Middle Click / Scroll Wheel
  - `4`: Forward Button (Front Side)
  - `5`: Back Button (Rear Side)
  - `6`: DPI Button (Underneath the mouse base)
- **Advanced Sensor Tuning**:
  - Polling Rate: 125 Hz, 250 Hz, 500 Hz, 1000 Hz
  - Lift-Off Distance (LOD): 1mm (Low) / 2mm (High)
  - Angle Snapping (Angle Straightening)
  - Motion Sync
  - Ripple Control (high-DPI jitter smoothing)
  - Sensor Power Mode (Office vs. Performance)
  - Debounce Filtering (0–20 ms)
  - LED Sleep Idle Timer (10s – 5min, or Never)
- **Real-Time Battery Monitoring**: Live battery percentage read directly from the 2.4G/4G wireless dongle.
- **Dual Interface**:
  - **Desktop GUI** for interactive calibration.
  - **Fast CLI** for scripts, terminal setups, or automated profile switching.
- **Profile Management**: Export and import complete mouse states to human-readable TOML files.

---

## 🖥️ Hardware Specifications

| Component | Specification |
|---|---|
| **Vendor ID (VID)** | `0x3554` |
| **Product ID (USB Wired)** | `0xF511` |
| **Product ID (2.4G Wireless)** | `0xF512` |
| **Product ID (4G Wireless)** | `0xF510` |
| **Optical Sensor** | PixArt PAW3395 (50 – 26,000 DPI) |
| **Mouse MCU** | Nordic Semiconductor nRF52840 |
| **2.4G Wireless Dongle** | CX52650N |
| **4G Wireless Dongle** | WCH CH32V305 |
| **HID Protocol** | Interface 1 (Vendor Specific HID, Report ID `0x07`, 17-byte payloads) |

---

## 🚀 Installation & Setup

### 1. Build and Install via Cargo

Make sure you have Rust and development libraries installed (`libudev-dev` / `systemd-devel`):

```bash
# Ubuntu / Debian
sudo apt install -y build-essential libudev-dev pkg-config

# Arch Linux / Manjaro
sudo pacman -S --needed base-devel systemd

# Fedora
sudo dnf install -y gcc systemd-devel
```

Clone and install the release binary:

```bash
git clone https://github.com/ayrazarya/icarus-v2mini-linux.git
cd icarus-v2mini-linux

# Build and install directly to ~/.cargo/bin
cargo install --path . --force
```

Ensure `~/.cargo/bin` is in your `$PATH` (typically added by default with `rustup`).

---

### 2. Configure udev Permissions (Non-Root Access)

Linux requires permission to communicate with vendor HID endpoints without `sudo`. Install the provided udev rules:

```bash
sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

*(You can also generate the rule dynamically anytime via `icarus-v2mini udev-rule`)*

---

### 3. Desktop Application Shortcut

To launch the app directly from your application launcher (GNOME, KDE, Rofi, etc.):

```bash
cp icarus-v2mini.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
```

---

## 🎮 Launching the GUI

Simply run without arguments or select **ICARUS V2 MINI** in your application launcher:

```bash
icarus-v2mini
# Or explicitly:
icarus-v2mini gui
```

### GUI Tabs:
1. **Key Settings**: Remap any of the 6 physical buttons with interactive visual callout pins.
2. **Sensor Tuning**: Adjust DPI stages (1–6), calibrate exact DPI values, assign custom per-stage colors, toggle LOD, Angle Snap, Motion Sync, and Polling Rate.
3. **Lighting**: Set mode (Solid, Breathing, Neon, Off), brightness (0–10), breath speed (1–10), idle sleep timers, and select from Esports presets, ColorBoard swatches (A–N), or custom RGB sliders.
4. **Misc**: Configure debounce time, ripple smoothing, and export/import profiles.

---

## 💻 CLI Usage

All mouse settings can also be queried and changed instantly via command line:

### Status & Hardware Information
```bash
# List all connected ICARUS receivers/devices
icarus-v2mini list

# Show full device status (polling rate, active DPI, battery, etc.)
icarus-v2mini info

# Read current battery level (wireless only)
icarus-v2mini battery
```

### Polling Rate
```bash
# Get current polling rate
icarus-v2mini rate

# Set polling rate (125, 250, 500, or 1000 Hz)
icarus-v2mini rate 1000
```

### DPI Configuration
```bash
# Query active stage & DPI range
icarus-v2mini dpi

# Switch to DPI stage 2 (1-indexed)
icarus-v2mini dpi --stage 2

# Set DPI value for stage 2 to 1600 DPI
icarus-v2mini dpi --stage 2 --value 1600
```

### Sensor Tuning
```bash
# Set Lift-Off Distance (low = 1mm, high = 2mm)
icarus-v2mini lod low
icarus-v2mini lod high

# Angle Snapping (Angle Straightening)
icarus-v2mini angle-snap on
icarus-v2mini angle-snap off

# Motion Sync
icarus-v2mini motion-sync on
icarus-v2mini motion-sync off

# Button Debounce filter (0–20 ms)
icarus-v2mini debounce 4
```

### LED Controls
```bash
# Set active stage LED to solid cyan at full brightness
icarus-v2mini led --mode solid --color "#00F0FF" --brightness 10

# Set breathing red with speed 5
icarus-v2mini led --mode breathing --color "#FF2A2A" --speed 5

# Turn off LED
icarus-v2mini led --mode off
```

### Profiles
```bash
# Save current mouse settings to a profile
icarus-v2mini save ~/my-profile.toml

# Apply settings from a profile
icarus-v2mini apply ~/my-profile.toml
```

---

## 🔬 Protocol & Reverse Engineering Architecture

The mouse communicates using vendor HID report ID `0x07` with 17-byte command packets:

```
[0x07] [Opcode] [Addr_Hi] [Addr_Lo] [Length] [Data 0..11] [Checksum]
```

- **Read Opcode**: `0x08`
- **Write Opcode**: `0x07`
- **Response Opcode**: `0x09`
- **Checksum Calculation**:
  - Subsystem command checksum: `XOR(bytes[0..15])`
  - Register write checksum (`0x4C..0x52`, `0xA0`, `0x2C`): `0x55 - value`

### Crucial Register Map:
- `0x0C + stage * 4`: DPI Stage X/Y sensor register (using PAW3395 table, labeled `SENSOR_3335` in vendor SDK)
- `0x2C + stage * 4`: DPI Stage RGB color (`[R, G, B, 0x55 - (R + G + B)]`)
- `0x4C`: DPI LED Mode register
- `0x4E`: DPI LED Brightness register
- `0x50`: DPI LED Breathing Speed register
- `0xA0`: Composite LED packet (`[mode, bright, speed, R, G, B, chk]`)
- `0x01, 0x01`: Report Rate (`0x01` = 1000Hz, `0x02` = 500Hz, `0x04` = 250Hz, `0x08` = 125Hz)
- `0x01, 0x02`: Active DPI Stage Index
- `0x01, 0x05`: Wireless Battery Percentage

---

## 🧪 Testing

Run the automated test suite:

```bash
cargo test
```

All 21 test suites test register serialization, SENSOR_3335 roundtrip conversions, response packet parsing, and profile file integrity.

---

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

Copyright (c) 2026 ayrazarya
