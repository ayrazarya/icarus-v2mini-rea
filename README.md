# ICARUS V2 MINI — Linux Driver & Native GUI

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux_Native_(X11%20%2F%20Wayland)-green.svg)]()
[![Hardware](https://img.shields.io/badge/Sensor-PixArt_PAW3395-red.svg)]()
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe-blueviolet.svg)]()
[![Version](https://img.shields.io/badge/Release-v0.2.0-brightgreen.svg)](https://github.com/ayrazarya/icarus-v2mini-rea/releases)

A native, lightweight, and high-performance Linux driver, configuration utility, and desktop GUI for the **ICARUS V2 MINI** ultralight wireless gaming mouse, written from scratch in pure Rust.

Reverse-engineered directly from the official Windows vendor software (`ICARUS V2 MINI_v1.0.1.2_setup.exe` & `HIDUsb.dll`). It communicates directly with the mouse and 2.4G/4G wireless receivers via raw Linux HID endpoints (`hidapi`) without Wine, Windows VMs, Electron, or heavy runtime dependencies.

---

## Graphical User Interface (UI Previews)

The desktop application is built with `egui` and `eframe`, providing hardware-accelerated 60+ FPS rendering under both Wayland and X11 with minimal RAM consumption (< 30 MB).

### Tab 1: Button Remapping & Debounce Tester
Interactive 3D mouse visualizer with calibrated button callouts, action binding editor, hardware debounce slider (0–20 ms), and a live click tester with CPS counter, interval delta, and double-click detector.

<p align="center">
  <img src="assets/UI-Preview/Buttons.png" alt="Buttons and Debounce Tester" width="850">
</p>

### Tab 2: DPI, Sensor Calibration & Spectrum Bar
6-stage DPI adjustment with clickable milestone ticks, free-text custom DPI input (50–26,000 DPI in 50 DPI steps), Single DPI Lock (competitive FPS mode), quick esports presets, and sensor tuning (LOD, Polling Rate, Angle Snapping, Motion Sync).

<p align="center">
  <img src="assets/UI-Preview/DPI-Sensors.png" alt="DPI and Sensor Configuration" width="850">
</p>

### Tab 3: Lighting & Live Real-Time Auto-Apply
RGB lighting modes (Solid, Breathing, Neon, Off), 10-level brightness and speed sliders, sleep idle timeout, 14 reverse-engineered ColorBoard swatches (A–N), esports color presets, custom RGB sliders, and live instant preview that synchronizes with the physical mouse in real time.

<p align="center">
  <img src="assets/UI-Preview/Lighting.png" alt="Lighting and Live Auto-Apply" width="850">
</p>

### Tab 4: Macro Recording & Timeline Sequencer
Macro keystroke recorder and interactive visual timeline sequencer with reorderable events, delay adjustments, loop modes, and binding to mouse buttons.

<p align="center">
  <img src="assets/UI-Preview/Macro.png" alt="Macro Timeline Sequencer" width="850">
</p>

### Tab 5: Settings, 2.4G Dongle Pairing & Profile Management
2.4G wireless pairing tool, native Linux GTK file selection dialog (`zenity`) for custom profile import/export, Save As support, quick filename presets, and automatic udev rule generation.

<p align="center">
  <img src="assets/UI-Preview/Settings.png" alt="Settings and Profile Management" width="850">
</p>

---

## Keyboard Shortcuts

The GUI supports global keyboard shortcuts for fast navigation:

| Shortcut | Action | Description |
|---|---|---|
| `Ctrl + S` | **Apply Changes** | Writes active configuration to mouse hardware immediately without scrolling. |
| `Ctrl + R` | **Read from Mouse** | Refreshes hardware status, battery telemetry, and active configuration. |
| `1` .. `5` or `Ctrl + 1..5` | **Switch Tabs** | Fast navigation across Buttons (`1`), Sensor (`2`), Lighting (`3`), Macro (`4`), and Settings (`5`). |
| `Esc` | **Cancel / Unfocus** | Surrenders focus from active text fields or cancels profile renaming mode. |

---

## Features

### PixArt PAW3395 Sensor Accuracy
- Exact register mapping table (`0x00`–`0xCE` covering 50 to 26,000 DPI) reverse-engineered from vendor driver headers (`driver_sensor.h`).
- Eliminates sensitivity drift found in linear formulas (e.g., 6,500 DPI correctly maps to register `0x4D`).
- Granular 50 DPI steps with instant free-text typing.
- Clickable milestone ticks on the spectrum bar (`400`, `800`, `1.6k`, `3.2k`, `6.4k`, `12k`, `26k`).
- **Single DPI Lock**: 1-click toggle to lock to a single active DPI stage (disabling others), preventing accidental stage shifts during clutch gameplay.
- **Quick Presets**: Ready-to-use profiles including Esports Pro (400 - 800 - 1600 - 3200) and Ultra-High Res (800 - 1600 - 2400 - 3200 - 6400).

### Mouse Click & Debounce Tester
- Built-in interactive test pad to measure clicks (Left, Right, Middle, Back, Forward).
- Dynamic CPS (Clicks Per Second) counter calculated from recent click intervals.
- Real-time click delta measurement in milliseconds.
- Hardware double-click / switch chattering detector flagging intervals below the configured debounce threshold.

### Profile Renaming & Management
- Rename onboard profiles with custom names (e.g., "Valorant / CS2", "Dota 2 / MOBA", "Work / Daily").
- Custom names integrated into header profile dropdown.
- Native GTK/Zenity file picker dialog for importing and exporting `.toml` configuration files.

### Battery & Power Telemetry
- Real-time battery percentage read directly from the Nordic nRF52840 MCU and wireless receiver.
- Automatic USB wired mode detection (PID `0xF511` drawing VBUS power ~4.83V) with live voltage telemetry.
- Multi-tier battery indicators with low-battery warning alerts.

### 6-Button Remapping
- Button 1: Left Click
- Button 2: Right Click
- Button 3: Middle Click / Scroll Wheel
- Button 4: Forward Button (Front Side)
- Button 5: Back Button (Rear Side)
- Button 6: DPI Switch (Underside of mouse base)
- Supported actions: Standard clicks, DPI Cycle, DPI Up/Down, Polling Rate Cycle, Media controls, Keyboard scancodes.

### Performance Tuning
- Polling Rate: 125 Hz, 250 Hz, 500 Hz, 1000 Hz.
- Lift-Off Distance (LOD): 1 mm (Low) / 2 mm (High).
- Angle Snapping: Precision line straightening filter.
- Motion Sync: Synchronizes sensor reports with USB polling intervals.
- Ripple Control: High-DPI jitter smoothing filter.
- Debounce Time: Switch debounce filter adjustable from 0 ms to 20 ms.

### Complete RGB & Lighting Control
- Modes: Solid, Breathing, Neon / Rainbow Cycle, Off.
- 10-level brightness and 10-level breathing speed sliders.
- 14 reverse-engineered ColorBoard swatches (A–N) and 6 esports presets.
- **Live Auto-Apply**: Instant real-time preview updating the physical mouse on color, mode, or slider changes without manual Apply clicks.
- Configurable sleep timer (10s, 30s, 1m, 2m, 3m, 5m, or Never).

### Command-Line Interface (CLI)
Headless CLI tool for scripts, terminal users, and window manager keybinds (Hyprland, Sway, i3).

---

## Supported Hardware & Specifications

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

## Installation & Setup

### 1. Install System Dependencies

Make sure Rust (Cargo) and `libudev` development packages are installed:

```bash
# Ubuntu / Debian / Pop!_OS
sudo apt install -y build-essential libudev-dev pkg-config zenity

# Arch Linux / Manjaro / EndeavourOS
sudo pacman -S --needed base-devel systemd zenity

# Fedora / RHEL
sudo dnf install -y gcc systemd-devel zenity

# openSUSE
sudo zypper install -y gcc systemd-devel zenity
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

Alternatively, regenerate the udev rule directly from the binary:
```bash
icarus-v2mini udev-rule | sudo tee /etc/udev/rules.d/99-icarus-mouse.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

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

---

## CLI Command Reference

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
  Level: 100% [Charging / USB Mode] (4830 mV)
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

# Angle Snapping (precision line straightening)
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

## Protocol & Reverse Engineering Deep Dive

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

### Register & Memory Map

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

## Codebase Architecture

Designed according to idiomatic Rust standards:

```
icarus-v2mini-linux/
├── src/
│   ├── lib.rs          # Core library exports
│   ├── main.rs         # CLI entry point, clap routing, and error reporting
│   ├── device.rs       # hidapi device connection management, USB/wireless probes & I/O
│   ├── protocol.rs     # Packet serialization, register mapping, opcodes, and checksums
│   ├── config.rs       # Profile data structures, TOML serialization, and validation
│   ├── dpi.rs          # PixArt PAW3395 register lookup tables and bidirectional conversions
│   └── gui.rs          # egui / eframe desktop application, 3D visualizer & state machine
├── tests/
│   ├── test_config.rs   # Unit tests for profile loading, saving, and defaults
│   ├── test_dpi.rs      # Precision tests for PAW3395 lookup table mappings
│   └── test_protocol.rs # Integration tests for 17-byte HID packets & checksum verification
├── assets/
│   ├── UI-Preview/     # High-resolution screenshots of all GUI tabs
│   ├── app-icon.png    # Desktop application icon
│   └── Light/          # 14 reverse-engineered ColorBoard graphic assets
├── examples/           # Sample profiles (profile-default.toml)
└── 99-icarus-mouse.rules # udev permissions for non-root hardware access
```

### Automated Testing & Linting

```bash
# Run unit & integration tests
cargo test

# Check code formatting
cargo fmt --check

# Strict lint verification
cargo clippy --all-targets -- -D warnings
```

---

## Troubleshooting & FAQ

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

- When plugged in via direct USB cable (`PID 0xF511`), the mouse runs on VBUS power and the battery charging indicator will show `[Charging / USB Mode]` with live voltage (~4.8V).
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

## Acknowledgments

### Powered by [REA (Reverse Engineer Anything)](https://github.com/morluto/rea)
Special thanks and credit to **[@morluto](https://github.com/morluto)** and the **[REA (Reverse Engineer Anything)](https://github.com/morluto/rea)** project!

Reverse-engineering proprietary gaming peripherals often requires wading through obfuscated installers, closed-source Windows DLLs (`HIDUsb.dll`), stripped PE binaries, and obscure USB HID report descriptors. **REA** made this entire workflow dramatically faster, clearer, and more structured:
- Disassembling and decompiling native PE binaries and DLL dispatch routines.
- Pinpointing vendor HID report handlers, opcodes (`0x04`, `0x07`, `0x08`, `0x09`), and checksum algorithms.
- Uncovering internal sensor register maps (`driver_sensor.h` / `SENSOR_3335`) and composite RGB lighting packet structures.
- Tracing runtime state transitions between wired USB and wireless dongle modes.

Without **REA**, bringing a native, bloat-free Linux driver for the ICARUS V2 MINI to life would have taken significantly more guesswork and time. Check out **[github.com/morluto/rea](https://github.com/morluto/rea)**.

### Special Thanks
- **PixArt Imaging**: PAW3395 optical gaming sensor.
- **Nordic Semiconductor**: nRF52840 wireless SoC.
- **Emilk and the `egui` community**: Immediate-mode GUI framework in Rust.

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

Copyright (c) 2026 **ayrazarya**
