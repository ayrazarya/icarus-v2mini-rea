# ICARUS V2 MINI — Linux Driver & Native GUI

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux_Native_(X11%20%2F%20Wayland)-green.svg)]()
[![Hardware](https://img.shields.io/badge/Sensor-PixArt_PAW3395-red.svg)]()
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe-blueviolet.svg)]()

Native, ultra-lightweight, high-performance Linux driver, configuration tool, and desktop GUI for the **ICARUS V2 MINI** ultralight wireless gaming mouse, written from scratch in pure Rust.

Reverse-engineered directly from the official Windows vendor software (`ICARUS V2 MINI_v1.0.1.2_setup.exe` & `HIDUsb.dll`). It communicates directly with the mouse and 2.4G/4G wireless receivers via raw Linux HID endpoints (`hidapi`) **without Wine, Windows VMs, Electron, or heavy runtime dependencies**.

---

## 🌟 Highlights & Key Features

### 🖥️ Native Desktop GUI (`egui` / `eframe`)
- **Ultra-Lightweight**: Consumes under **30 MB of RAM** with zero web-engine / Electron bloat.
- **Hardware-Accelerated 60+ FPS Rendering**: Smooth, responsive vector UI supporting both **Wayland** and **X11**.
- **Interactive Hardware Visualizer**: 3D mouse visualizer with calibrated, clickable callout pins matching physical buttons 1:1.
- **Compact Profile Switcher**: Quick-access profile dropdown in the header to cycle through the 5 onboard hardware profiles instantly.

### 🎯 100% DPI & Sensor Accuracy (PixArt PAW3395)
- **Official Sensor Register Mapping**: Utilizes the exact PAW3395 nonlinear register lookup table (`0x00`–`0xCE` covering 50 to 26,000 DPI) reverse-engineered from vendor headers (`driver_sensor.h`).
- **No DPI Calculation Drift**: Eliminates the severe sensitivity drift found in naive linear formulas (e.g., 6,500 DPI accurately maps to hardware register `0x4D`).
- **Independent 6-Stage DPI Configuration**: Freely enable/disable stages (1–6), configure DPI values, and assign custom stage indicator colors.

### ⚡ Smart Multi-State Battery & Power Management
- **Live Hardware Monitoring**: Real-time polling of battery status directly from the nRF52840 MCU and wireless receiver.
- **Wired / USB Charging Detection**: Automatically detects USB wired mode (`PID 0xF511` drawing USB VBUS power ~4.83V) or wireless charging docks, displaying a glowing **⚡ Charging** indicator and live millivolt (`mV`) telemetry.
- **Tiered Battery Indicators**:
  - 🟢 **Optimal Battery (>50%)**: Solid emerald indicator.
  - 🟡 **Moderate Battery (20%–50%)**: Amber caution indicator.
  - ⚠️ **Low Battery Warning (≤20%)**: High-contrast warning alert.
  - 🪫 **Critical / Sekarat (≤10%)**: Urgent pulsing indicator to prevent mid-game dropouts.

### 🖱️ Calibrated 6-Button Remapping
Accurately maps to the physical mouse layout matching the official driver:
1. **Button 1**: Left Click
2. **Button 2**: Right Click
3. **Button 3**: Middle Click / Scroll Wheel
4. **Button 4**: Forward Button (Front Side)
5. **Button 5**: Back Button (Rear Side)
6. **Button 6**: DPI Switch (Underside of the mouse base)

Supported actions: Standard mouse clicks, DPI Cycle, DPI Up/Down, Polling Rate Cycle, Media controls, and Custom keyboard scancodes.

### ⚙️ Esports Sensor & Performance Tuning
- **Report / Polling Rate**: 125 Hz, 250 Hz, 500 Hz, and 1000 Hz.
- **Lift-Off Distance (LOD)**: 1mm (Low) / 2mm (High).
- **Angle Snapping**: Toggle angle straightening for precision line tracking.
- **Motion Sync**: Synchronizes sensor reports with USB polling intervals for maximum tracking smoothness.
- **Ripple Control**: High-DPI jitter smoothing filter.
- **Debounce Time**: Hardware switch debounce filter adjustable from 0 ms to 20 ms.
- **Sensor Power Modes**: Toggle between Office and High-Performance tracking states.

### 🌈 Complete RGB & DPI Lamp Control
- **Lighting Modes**: Solid, Breathing, Neon / Rainbow Cycle, and Off.
- **Fine Controls**: 10-level brightness adjustment and 10-level breathing speed.
- **Palette Presets**: Integrated Esports color presets, 14 reverse-engineered ColorBoard swatches (A–N), or full custom RGB color pickers.
- **Sleep Idle Timer**: Configurable power-saving sleep timer (10s, 30s, 1m, 2m, 3m, 5m, or Never).

### ⌨️ Comprehensive CLI
Headless command-line interface for terminal power users, scripts, Hyprland/Sway keybinds, and automated gaming profile switchers.

---

## 🛠️ Supported Hardware & Specifications

| Component | Specification | Details |
|---|---|---|
| **Vendor ID (VID)** | `0x3554` | ShenZhen F&G Electron Co., Ltd |
| **USB Wired PID** | `0xF511` | Direct USB-C wired connection (1000Hz, VBUS Power) |
| **2.4G Wireless PID** | `0xF512` | Standard 2.4G wireless nano dongle |
| **4G Wireless PID** | `0xF510` | High-speed 4K/4G wireless dongle |
| **Optical Sensor** | PixArt PAW3395 | Up to 26,000 DPI, 650 IPS, 50G acceleration |
| **Mouse MCU** | Nordic nRF52840 | 32-bit ARM Cortex-M4F, Bluetooth 5 & 2.4GHz proprietary |
| **2.4G Receiver IC** | CX52650N | Low-latency 2.4GHz USB bridge |
| **4G Receiver IC** | WCH CH32V305 | High-speed USB RISC-V microcontroller |
| **HID Interface** | Interface 1 | Vendor-defined HID collection, Report ID `0x07` (17-byte payloads) |

---

## 📥 Installation & Setup

### 1. Install System Dependencies

Make sure Rust (Cargo) and `libudev` development packages are installed:

```bash
# Ubuntu / Debian / Pop!_OS
sudo apt install -y build-essential libudev-dev pkg-config

# Arch Linux / Manjaro / EndeavourOS
sudo pacman -S --needed base-devel systemd

# Fedora / RHEL
sudo dnf install -y gcc systemd-devel

# openSUSE
sudo zypper install -y gcc systemd-devel
```

### 2. Build and Install Binary

Clone this repository and compile with Cargo:

```bash
git clone https://github.com/ayrazarya/icarus-v2mini-linux.git
cd icarus-v2mini-linux

# Compile release build and install to ~/.cargo/bin
cargo install --path . --force
```

Ensure `~/.cargo/bin` is in your environment `$PATH` (usually handled automatically by `rustup`).

### 3. Setup udev Rules (Non-Root USB Access)

By default, Linux limits raw access to vendor-specific HID interfaces. Install the udev rule to allow standard user access:

```bash
sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

> **Tip**: You can also regenerate the udev rule on any machine with:
> ```bash
> icarus-v2mini udev-rule | sudo tee /etc/udev/rules.d/99-icarus-mouse.rules
> sudo udevadm control --reload-rules && sudo udevadm trigger
> ```

### 4. Desktop Application Launcher Integration

Install the desktop shortcut and application icon:

```bash
# Install application icon
mkdir -p ~/.local/share/icons/hicolor/256x256/apps/
cp assets/app-icon.png ~/.local/share/icons/hicolor/256x256/apps/icarus-v2mini.png

# Install desktop entry
mkdir -p ~/.local/share/applications/
cp icarus-v2mini.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
```

The application will now appear in your desktop launcher (GNOME, KDE Plasma, Rofi, Wofi, App-Launcher) as **ICARUS V2 MINI**.

---

## 🎮 Graphical Interface (GUI)

Launch the GUI by running:

```bash
icarus-v2mini
# Or explicitly:
icarus-v2mini gui
```

### GUI Layout & Navigation

```
┌────────────────────────────────────────────────────────────────────────┐
│ ICARUS V2 MINI  PAW3395 LINUX    Profile: [Profile 1 ▾]   ● Connected   │
│ [ ⚡ USB Mode: 4830mV ]                                                 │
├────────────────────────────────────────────────────────────────────────┤
│  [🔘 Buttons]   [📡 DPI & Sensor]   [💡 Lighting]   [⚙️ Misc & Storage] │
├───────────────────────────────────┬────────────────────────────────────┤
│                                   │                                    │
│        3D Hardware Visualizer     │       Active Tab Configuration     │
│                                   │                                    │
│           (1) Left Click          │  - Remap buttons 1 to 6            │
│           (2) Right Click         │  - DPI calibration & per-stage LED │
│           (3) Middle Click        │  - LOD, Polling Rate, Angle Snap   │
│           (4) Side Forward        │  - Lighting modes, sleep timers    │
│           (5) Side Back           │  - Profile save / load / backup    │
│           (6) Underside DPI       │                                    │
│                                   │                                    │
└───────────────────────────────────┴────────────────────────────────────┘
```

1. **Buttons Tab**: Click any pin or list item to remap buttons. Supports left/right/middle click, forward/backward, DPI shifts, media controls, and keybinds.
2. **DPI & Sensor Tab**: Adjust stages 1–6 with granular sliders, assign custom RGB stage indicator colors, set polling rate (125–1000Hz), LOD (1mm/2mm), angle snapping, and motion sync.
3. **Lighting Tab**: Select modes (Solid, Breathing, Neon, Off), adjust brightness and breathing speed, and configure sleep idle timeout.
4. **Misc & Storage Tab**: Configure switch debounce filtering (0–20ms), toggle ripple control, and export/import profiles to `.toml` files.

---

## 💻 CLI Command Reference

Every hardware parameter can be inspected or modified from the command line without opening the GUI.

### Device Telemetry & Battery
```bash
# List all connected ICARUS devices and dongles
icarus-v2mini list

# Display comprehensive device status report
icarus-v2mini info

# Query battery status, charging state, and voltage telemetry
icarus-v2mini battery
```

*Example `battery` output:*
```
ICARUS V2 MINI Battery Status:
  Level: 100% [⚡ Charging / USB Mode] (4830 mV)
```

### Polling Rate
```bash
# Check current polling rate
icarus-v2mini rate

# Set polling rate to 1000Hz (options: 125, 250, 500, 1000)
icarus-v2mini rate 1000
```

### DPI Calibration & Switching
```bash
# Query active DPI stage and stage table
icarus-v2mini dpi

# Switch to DPI stage 2
icarus-v2mini dpi --stage 2

# Set DPI value of stage 2 to 1600 DPI
icarus-v2mini dpi --stage 2 --value 1600
```

### Sensor & Performance Tuning
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

# Button Debounce time (0 to 20 ms)
icarus-v2mini debounce 4
```

### RGB & DPI LED Control
```bash
# Set solid cyan LED at maximum brightness
icarus-v2mini led --mode solid --color "#00F0FF" --brightness 10

# Set breathing red with speed level 5
icarus-v2mini led --mode breathing --color "#FF2A2A" --speed 5

# Set neon / rainbow cycle mode
icarus-v2mini led --mode neon --speed 7

# Turn off LED completely
icarus-v2mini led --mode off
```

### Profile Management
```bash
# Export active hardware configuration to a TOML profile
icarus-v2mini save ~/.config/icarus/gaming.toml

# Apply and write a TOML profile to the mouse hardware
icarus-v2mini apply ~/.config/icarus/gaming.toml
```

---

## 🔬 Protocol & Reverse Engineering Deep Dive

Communication is established via Linux HID raw output reports on **Interface 1**, using 17-byte command frames framed with Report ID `0x07`.

### Frame Structure

```
Byte 0    : Report ID (0x07)
Byte 1    : Opcode (0x04 = Query, 0x07 = Write, 0x08 = Read, 0x09 = Response)
Byte 2..3 : Register Address (Big-Endian, e.g. 0x01, 0x01)
Byte 4    : Payload Length (N)
Byte 5..15: Data Bytes (Data[0..10])
Byte 16   : Frame Checksum
```

### Checksum Algorithms
- **Subsystem Commands**: `Checksum = XOR(Bytes[0..15])`
- **Register Memory Writes (`0x4C..0x52`, `0xA0`, `0x2C`)**: `Checksum = (0x55 - Value) & 0xFF`

### Crucial Register & Memory Map

| Register Address | Function | Payload Description |
|---|---|---|
| `0x01, 0x01` | Polling Rate | `0x01` = 1000Hz, `0x02` = 500Hz, `0x04` = 250Hz, `0x08` = 125Hz |
| `0x01, 0x02` | Active DPI Stage | `0x00` to `0x05` (0-indexed stages 1..6) |
| `0x04` (Opcode) | Battery Status Query | Response: `Byte 6` = Percentage (0–100%), `Byte 7` = Charging flag, `Bytes 8..9` = Voltage in mV |
| `0x0C + (stage * 4)` | Stage DPI Value | X & Y sensor register lookup index (see `SENSOR_3335` note below) |
| `0x2C + (stage * 4)` | Stage LED Color | `[R, G, B, 0x55 - (R + G + B)]` |
| `0x4C` | LED Mode | `0x00` = Off, `0x01` = Solid, `0x02` = Breathing, `0x03` = Neon |
| `0x4E` | LED Brightness | `0x00` (Off) to `0x0A` (100% Brightness) |
| `0x50` | LED Breathing Speed | `0x01` (Slowest) to `0x0A` (Fastest) |
| `0xA0` | Composite LED Packet | `[Mode, Brightness, Speed, R, G, B, Checksum]` |
| `0x01, 0x0B` | LOD Height | `0x01` = Low (1mm), `0x02` = High (2mm) |
| `0x01, 0x0C` | Angle Snapping | `0x01` = On, `0x00` = Off |
| `0x01, 0x0D` | Motion Sync | `0x01` = On, `0x00` = Off |
| `0x01, 0x0E` | Button Debounce | Value in milliseconds (`0`–`20` ms) |

### Note on `SENSOR_3335` vs PAW3395
In the extracted Windows OEM driver headers (`driver_sensor.h`), the lookup table is named `SENSOR_3335`. Even though the mouse physically integrates the flagship **PixArt PAW3395**, the OEM firmware uses this internal register index array (`0x00` to `0xCE`) to configure the PAW3395 sensor registers. This driver replicates this exact table to guarantee **100% hardware-level DPI fidelity**.

---

## 🏗️ Codebase Architecture

Designed according to modern, modular, and idiomatic Rust standards:

```
icarus-v2mini-linux/
├── src/
│   ├── lib.rs          # Core library exports
│   ├── main.rs         # CLI entry point, clap command routing, and error reporting
│   ├── device.rs       # hidapi device connection management, USB/wireless probes & I/O
│   ├── protocol.rs     # Packet serialization, register mapping, opcodes, and checksums
│   ├── config.rs       # Profile data structures, TOML serialization, and validation
│   ├── dpi.rs          # PixArt PAW3395 register lookup tables and bidirectional conversions
│   └── gui.rs          # egui / eframe desktop application, 3D visualizer & state machine
├── tests/
│   ├── test_config.rs   # Unit tests for profile loading, saving, and defaults
│   ├── test_dpi.rs      # Precision tests for PAW3395 lookup table mappings
│   └── test_protocol.rs # Integration tests for 17-byte HID packets & checksum verification
├── assets/             # Application icons and visual assets
├── examples/           # Sample profiles (profile-default.toml) & standalone probe tools
└── 99-icarus-mouse.rules # udev permissions for non-root hardware access
```

### Automated Testing

Run the full automated test suite with:

```bash
cargo test
```

Verify compliance with strict clippy guidelines:

```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

---

## 💡 Troubleshooting & FAQ

<details>
<summary><b>1. "Device not found" or "Permission Denied" error</b></summary>

- Verify your user belongs to the `input` or `plugdev` group.
- Ensure the udev rules are installed and reloaded:
  ```bash
  sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/
  sudo udevadm control --reload-rules && sudo udevadm trigger
  ```
- Unplug and reconnect the USB cable or wireless dongle.
</details>

<details>
<summary><b>2. Battery status shows "N/A" or "0%"</b></summary>

- When plugged in via direct USB cable (`PID 0xF511`), the mouse runs on VBUS power and the battery charging indicator will show `[⚡ Charging / USB Mode]` with live voltage (~4.8V).
- Battery percentage reporting is active in wireless 2.4G/4G modes (`PID 0xF512` / `0xF510`).
</details>

<details>
<summary><b>3. Wayland vs X11 Display issues</b></summary>

- The GUI natively supports both Wayland and X11 via `eframe` / `winit`.
- If you experience window scaling issues under Wayland, set `WINIT_UNIX_BACKEND=x11` before launching:
  ```bash
  WINIT_UNIX_BACKEND=x11 icarus-v2mini
  ```
</details>

---

## 🙏 Acknowledgments & Shout-out

### 🚀 Powered by [REA (Reverse Engineer Anything)](https://github.com/morluto/rea)
A massive shout-out and heartfelt gratitude to **[@morluto](https://github.com/morluto)** and the **[REA (Reverse Engineer Anything)](https://github.com/morluto/rea)** project!

Reverse-engineering proprietary gaming peripherals often requires wading through obfuscated installers, closed-source Windows DLLs (`HIDUsb.dll`), stripped PE binaries, and obscure USB HID report descriptors. **REA** made this entire workflow dramatically faster, clearer, and more structured:
- Disassembling and decompiling native PE binaries and DLL dispatch routines.
- Pinpointing vendor HID report handlers, opcodes (`0x04`, `0x07`, `0x08`, `0x09`), and checksum algorithms.
- Uncovering internal sensor register maps (`driver_sensor.h` / `SENSOR_3335`) and composite RGB lighting packet structures.
- Tracing runtime state transitions between wired USB and wireless dongle modes.

Without **REA**, bringing a 100% native, bloat-free Linux driver for the ICARUS V2 MINI to life would have taken exponentially more guesswork and time. If you do reverse engineering on Linux, definitely check out **[github.com/morluto/rea](https://github.com/morluto/rea)**!

### Special Thanks
- **PixArt Imaging**: For manufacturing the exceptional PAW3395 sensor.
- **Nordic Semiconductor**: For the nRF52840 wireless SoC.
- **Emilk and the `egui` community**: For making immediate-mode GUI programming in Rust a joy.

---

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

Copyright (c) 2026 **ayrazarya**
