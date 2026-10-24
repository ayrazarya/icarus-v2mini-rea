# Coding Standards & Contributing Guide

Thank you for your interest in contributing to the **ICARUS V2 MINI Linux Driver & Native GUI**! This guide explains the coding conventions, architectural rules, step-by-step feature workflows, contributor expectations, and Quality Assurance (QA) checks enforced across the repository.

---

## 1. Nice to Have (If You Want to Contribute)

### Contributor Mindset & Tooling
- **Can Use an Agentic AI Very Well**: We strongly value contributors who are proficient at steering Agentic AI coding assistants, MCP reverse-engineering tools (such as [REA](https://github.com/morluto/rea)), and automated verification workflows—or who write clean, deterministic code that pairs seamlessly with strict compiler/linter gates without introducing AI hallucinations, bloat, or untested assumptions.
- **Note on AI Over-Reliance ("Can't Use an Agentic AI Very Well" Anti-Pattern)**: Please do **not** submit unverified, copy-pasted AI output that ignores hardware register invariants, breaks the `0x55 - sum` checksum protocol, introduces `unsafe` blocks, or bypasses Clippy lints. Every contribution must be grounded in actual hardware protocol evidence and pass all automated tests.
- **QA & Systems Programming Mindset**: Always think about edge cases—sleeping 2.4GHz wireless mice, USB hot-unplugging mid-transaction, multi-byte UTF-8 input in fixed-size EEPROM buffers, and zero-allocation hot paths.

### Nice-to-Have Roadmap Features (Open for Contributions)
1. **Linux Distribution Packaging**:
   - Native package manifests for **Arch Linux AUR (`PKGBUILD`)**, **Flatpak**, **Debian/Ubuntu (`.deb`)**, **Fedora/RPM (`.rpm`)**, and **NixOS (`flake.nix`)**.
2. **System Tray / StatusNotifierItem Applet**:
   - Lightweight desktop system tray icon displaying live battery percentage, charging state, and a quick-switch menu for the 5 on-board profiles.
3. **Extended Wayland Compositor Support for Auto-Profile**:
   - Native active-window detectors for **KDE Plasma (KWin D-Bus)**, **GNOME Shell (Mutter)**, **Niri**, and **COSMIC** alongside existing Hyprland, Sway, X11, and `/proc` scanners.
4. **Live Polling Rate (Hz) & Sensor Jitter Graph**:
   - Real-time mouse movement frequency analyzer and Hz stability graph inside the GUI Sensor tab.
5. **Standalone Macro Import/Export (`.json` / `.toml`)**:
   - Shareable single-macro preset files in addition to full-profile `.toml` exports.

---

## 2. Rust 2024 Coding Standards

This repository enforces strict compiler and Clippy lints in [`Cargo.toml`](Cargo.toml):

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

### Mandatory Engineering Rules
1. **Zero `unsafe` Code (`unsafe_code = "forbid"`)**:
   - Use checked slice indexing (`.get(...)`), pattern matching, and standard endianness helpers (`u16::from_le_bytes`, `u16::from_be_bytes`).
   - When truncating UTF-8 strings for fixed-length hardware buffers (such as 30-byte EEPROM macro names), always slice at valid character boundaries using `.char_indices()`.
2. **Compile-Time Evaluation (`const fn` & `#[must_use]`)**:
   - Pure functions that compute register indices, encode/decode bytes, or construct `Command` packets must be marked `#[must_use] pub const fn`.
3. **Zero Unnecessary Allocations**:
   - Return `&'static str` or `Cow<'static, str>` rather than allocating `String` on hot paths.
   - Use `std::mem::take` when moving `Vec` fields across configuration structs instead of `.clone()`.
   - Prefer `.unwrap_or_else(|| ...)` over `.unwrap_or(...)` whenever the fallback performs an allocation or function call.
4. **Explicit, Layered Error Handling (`thiserror` & `anyhow`)**:
   - **Never use `.unwrap()` or `.expect()`** in production library or binary code (outside `tests/`).
   - Use `ProtocolError` (`thiserror`) inside `src/protocol/` and `src/device/`, and `anyhow::Result` with `.context(...)` at CLI/application boundaries.
   - Always check `dev.read_device_online()?` before multi-packet wireless operations so sleeping mice fail fast rather than stalling for seconds on sequential USB timeouts.
5. **Documentation Integrity**:
   - Document all public types, traits, and functions with `///` doc-comments. Preserve existing comments and architectural explanations when refactoring.

---

## 3. Step-by-Step Guide to Adding Features

### A. Adding a New Hardware Register or Command
1. **Define the Register/Opcode**:
   - Add the EEPROM address constant in [`src/protocol/addr.rs`](src/protocol/addr.rs) or the opcode constant in [`src/protocol/mod.rs`](src/protocol/mod.rs).
2. **Create a `const fn` Packet Constructor**:
   - Add a `#[must_use] pub const fn` builder method on `Command` in [`src/protocol/command.rs`](src/protocol/command.rs).
3. **Implement the Driver Method in `src/device/`**:
   - Add the read/write method in [`src/device/registers.rs`](src/device/registers.rs) or [`src/device/composite.rs`](src/device/composite.rs).
   - Remember to call `dev.commit_config(1)?` in standalone CLI commands if the register requires an EEPROM commit to take effect immediately.
4. **Add Protocol Unit Tests**:
   - Verify exact byte layout and checksum calculation in [`tests/test_protocol.rs`](tests/test_protocol.rs).

### B. Adding a New CLI Subcommand
1. Add the subcommand variant to `Commands` in [`src/cli/args.rs`](src/cli/args.rs).
2. Implement the `cmd_*` handler function in the appropriate module under [`src/cli/commands/`](src/cli/commands/mod.rs).
3. Wire the command dispatch in [`src/cli/mod.rs`](src/cli/mod.rs).

### C. Adding a New GUI Control & Bilingual Translation
1. **Add Bilingual Strings (`EN` & `ID`)**:
   - Add the translation method to `AppLanguage` in the corresponding submodule under [`src/i18n/`](src/i18n/mod.rs), providing complete English and Indonesian strings with zero language mixing.
2. **Update the GUI Tab**:
   - Add the widget in [`src/gui/tabs/`](src/gui/tabs/mod.rs) and use `queue_lighting_sync()` / `queue_dongle_sync()` if the control is a real-time slider.
3. **Add Localization Tests**:
   - Verify the new translation method in [`tests/test_i18n.rs`](tests/test_i18n.rs).

---

## 4. Pre-Commit QA Verification Checklist

Before opening a Pull Request or pushing changes, run the full verification pipeline from the repository root and ensure zero warnings and zero failures:

```bash
# 1. Verify code formatting
cargo fmt --all --check

# 2. Run strict Clippy static analysis across all targets & features (0 warnings allowed)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Run the complete unit & integration test suite (51+ tests)
cargo test --workspace --all-features

# 4. Verify optimized release build compilation
cargo build --release
```
