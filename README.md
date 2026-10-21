# icarus-v2mini — Native Linux Configuration Tool for ICARUS V2 MINI

A native, high-performance Linux CLI and configuration tool for the **ICARUS V2 MINI** gaming mouse, written in Rust.

Reverse-engineered directly from the official Windows driver (`ICARUS V2 MINI_v1.0.1.2_2024.10.28_setup.exe`). It communicates directly with the mouse via raw USB/HID on Linux **without Wine, Windows, or runtime dependencies**.

---

## Hardware Specifications

| Component | Specification |
|---|---|
| **Vendor ID (VID)** | `0x3554` |
| **Product ID (USB Wired)** | `0xF511` |
| **Product ID (2.4G Wireless)** | `0xF512` |
| **Product ID (4G Wireless)** | `0xF510` |
| **Sensor** | PixArt PAW3395 (50 – 26,000 DPI) |
| **Mouse MCU** | Nordic nRF52840 |
| **2.4G Dongle MCU** | CX52650N |
| **4G Dongle MCU** | WCH CH32V305 |
| **HID Interface** | Interface 1 (Vendor-Specific HID, Report ID 0x07) |

---

## Features

- **Device Information**: Query firmware version, active connection mode, polling rate, current DPI stage, LOD, angle snap, motion sync, debounce, and battery level.
- **Polling Rate Configuration**: 125 Hz, 250 Hz, 500 Hz, 1000 Hz.
- **DPI Configuration**:
  - Cycle or set active DPI stage (1–8)
  - Configure DPI values with PAW3395 sensor register translation (50–26,000 DPI)
  - Set per-stage LED indicators
- **Sensor Features**:
  - Lift-Off Distance (LOD): 1mm (Low) / 2mm (High)
  - Angle Snapping (Angle Straightening) toggle
  - Motion Sync toggle
  - Click Debounce filter (0–20 ms)
- **LED Lighting Effects**:
  - Modes: `Off`, `Solid`, `Breathing`
  - Brightness control (0–10)
  - Breathing speed control (1–10)
  - Custom RGB colors
- **Battery Status**: Check wireless battery percentage and visual charge bar.
- **Profile Management**: Export current mouse settings to a human-readable TOML file or apply saved profiles.
- **udev Rule Generator**: Integrated non-root permission setup.

---

## Installation & Setup

### 1. Build from Source

```bash
git clone https://github.com/your-username/icarus-v2mini-linux.git
cd icarus-v2mini-linux
cargo build --release
```

The compiled binary will be located at `target/release/icarus-v2mini`.

### 2. Configure udev Permissions (Non-root Access)

By default, Linux requires root permissions to access raw HID devices. Run the built-in generator to install the udev rule:

```bash
# Generate and install the udev rule
target/release/icarus-v2mini udev-rule | sudo tee /etc/udev/rules.d/99-icarus-mouse.rules

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Alternatively, copy the provided `99-icarus-mouse.rules`:
```bash
sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

---

## Usage Guide

### General Help & Device Status

```bash
# View general help
icarus-v2mini --help

# List connected ICARUS devices (USB, 2.4G, 4G)
icarus-v2mini list

# Show full device info & current settings
icarus-v2mini info
```

### Polling Rate

```bash
# Query current polling rate
icarus-v2mini rate

# Set polling rate to 1000 Hz (supported: 125, 250, 500, 1000)
icarus-v2mini rate 1000
```

### DPI Settings

```bash
# Display current DPI stage & sensor configuration
icarus-v2mini dpi

# Switch to DPI stage 2 (1-indexed)
icarus-v2mini dpi --stage 2

# Set DPI value for stage 2 to 1600 DPI
icarus-v2mini dpi --stage 2 --value 1600
```

### Sensor Tuning

```bash
# Set Lift-Off Distance to 1mm (low) or 2mm (high)
icarus-v2mini lod low
icarus-v2mini lod high

# Enable or disable Angle Snapping
icarus-v2mini angle-snap on
icarus-v2mini angle-snap off

# Enable or disable Motion Sync
icarus-v2mini motion-sync on
icarus-v2mini motion-sync off

# Set button debounce filter (e.g. 4ms)
icarus-v2mini debounce 4
```

### LED Controls

```bash
# Set DPI LED to solid white at full brightness
icarus-v2mini led --mode solid --color "#FFFFFF" --brightness 10

# Set DPI LED to breathing cyan
icarus-v2mini led --mode breathing --color "#00FFFF" --speed 5

# Turn off LED
icarus-v2mini led --mode off
```

### Battery Status

```bash
icarus-v2mini battery
```

### Profile Import & Export

```bash
# Save current mouse settings to a TOML file
icarus-v2mini save my-gaming-profile.toml

# Apply settings from a TOML file
icarus-v2mini apply my-gaming-profile.toml
```

See [examples/profile-default.toml](examples/profile-default.toml) for an example profile template.

---

## Protocol Architecture

The ICARUS V2 MINI uses a 17-byte HID command packet (prepended by report ID `0x07` when sent across the wire).

### Packet Format (17 Bytes Payload)

| Offset | Field | Description |
|---|---|---|
| `0x00` | Opcode | `0x08` = Read, `0x07` = Write, `0x09` = Response |
| `0x01` | Addr High | Sub-system / Register high byte |
| `0x02` | Addr Low | Register low byte |
| `0x03` | Length | Data payload length (1–12 bytes) |
| `0x04..0x0F`| Data | Payload bytes (zero-padded) |
| `0x10` | Checksum | Bitwise XOR of bytes `0x00..0x0F` |

### Key Register Map

| Register (Hi, Lo) | Function | Description |
|---|---|---|
| `(0x01, 0x01)` | Report Rate | `0x08`=125Hz, `0x04`=250Hz, `0x02`=500Hz, `0x01`=1000Hz |
| `(0x01, 0x02)` | Current DPI | Active stage index (0-based) |
| `(0x01, 0x03)` | DPI LED | Mode, brightness, breath speed, RGB colors |
| `(0x01, 0x04)` | LED Bar | Per-stage LED bar RGB colors |
| `(0x01, 0x05)` | Battery | Wireless battery percentage (0–100%) |
| `(0x01, 0x06)` | Version | Firmware version string |
| `(0x02, 0x00)` | Flash All | Full configuration data map |

---

## Testing

Run the automated test suite:

```bash
cargo test
```

---

## License

MIT License
