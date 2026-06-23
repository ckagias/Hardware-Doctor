# Trouble

### A self-hosted desktop app for testing your PC's hardware, built in pure Rust

[About](#about) • [Features](#features) • [Modules](#modules) • [Download](#download) • [Installation](#installation) • [Building](#building) • [Releasing](#releasing) • [Dependencies](#dependencies) • [License](#license)

---

## About

A small desktop app for testing your PC's hardware: microphones, headphones/speakers, your keyboard, and your mouse. Windows' built-in troubleshooting for these is unreliable, and testing them online usually means hunting down a sketchy website. Trouble runs locally instead.

Built entirely in Rust with [eframe](https://github.com/emilk/egui)/[egui](https://github.com/emilk/egui) for the UI, no browser or JS runtime involved.

If you find this useful, feel free to leave a star to help others find it!

---

## Features

- Live microphone level meter with record and playback
- Left/right/both channel test tones for headphones and speakers
- Full 100% on-screen keyboard layout with per-key press tracking
- Top-down mouse diagram with per-button press and tested-state tracking, including scroll and side buttons
- Live click counter for every mouse input, reset alongside the tested state
- Generic gamepad diagram with live analog stick and trigger visualization, plus a per-input press counter
- Device picker with refresh for both audio modules
- All hardware state and device I/O handled in Rust, not the browser

---

## Modules


| Module                | Description                                                                                                             |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Microphone test       | Pick an input device, watch a live level meter, record and play back a short clip                                       |
| Headphones / speakers | Pick an output device, play a tone to the left channel, right channel, or both                                          |
| Keyboard test         | Press keys on your physical keyboard and watch them light up on a full 100% layout, with a running count of tested keys |
| Mouse test            | Click left/right/middle, scroll up/down, and press the side buttons, with a live per-input click counter                |
| Controller test       | Generic gamepad diagram with button, D-pad, stick, and trigger visualization via gilrs (XInput on Windows)              |


While Trouble's window has focus, the keyboard module reads keys through egui's own input events, which can't distinguish left/right Shift/Ctrl/Alt and has no separate Caps Lock, Windows/Super key, NumLock, ScrollLock, PrintScreen, Pause, ContextMenu, or numpad-distinct-from-main-row keys. Those specific keys are only testable while the window is unfocused, via a global low-level keyboard hook (`rdev`) that picks up the slack.

---

## Download

Prebuilt Windows binaries are published on the [Releases](https://github.com/ckagias/Hardware-Doctor/releases) page:

- **`TroubleSetup-<version>.exe`**: installer, recommended. Installs Trouble, adds a Start Menu shortcut, and installs the Visual C++ Redistributable if it's missing.
- **`trouble.exe`**: portable binary, no installer. Requires the [Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe) to already be installed, otherwise it fails to launch with error 126.

Windows SmartScreen may warn about an unrecognized publisher since the app isn't code-signed. Click **More info → Run anyway** to proceed.

---

## Installation

1. **Clone the repository**
  ```bash
   git clone https://github.com/ckagias/Hardware-Doctor.git
   cd Hardware-Doctor
  ```
2. **Run in development**
  ```bash
   cargo run
  ```

Requires the [Rust toolchain](https://www.rust-lang.org/tools/install). No other dependencies needed, everything else is pulled in by `cargo`.

A debug build (`cargo run` / `cargo build`) shows a console window alongside the app for log output; release builds don't.

---

## Building

```bash
cargo build --release
```

Produces `target/release/trouble.exe`. That binary alone is fully self-contained and can be run directly without installing anything.

---

## Releasing

1. Bump `version` in `Cargo.toml`.
2. Build the release binary and installer:
  ```bash
   cargo build --release
   makensis /DVERSION=<version> installer\trouble.nsi
  ```
   ([makensis](https://nsis.sourceforge.io/Download) must be installed; this produces `TroubleSetup-<version>.exe` in the repo root, alongside `target/release/trouble.exe`.)
3. Tag the release and publish it on GitHub:
  ```bash
   git tag v<version>
   git push origin v<version>
   gh release create v<version> TroubleSetup-<version>.exe target/release/trouble.exe --title "Trouble v<version>"
  ```

---

## Dependencies


| Package                                               | Purpose                                                   |
| ----------------------------------------------------- | --------------------------------------------------------- |
| [eframe](https://github.com/emilk/egui)               | Native window/app shell (winit + glow)                    |
| [egui](https://github.com/emilk/egui)                 | Immediate-mode UI used for every panel                    |
| [cpal](https://github.com/RustAudio/cpal)             | Cross-platform audio I/O                                  |
| [rdev](https://github.com/narsil/rdev)                | Global keyboard hook, used while the window is unfocused  |
| [gilrs](https://gitlab.com/gilrs-project/gilrs)       | Gamepad input (XInput on Windows, evdev/libudev on Linux) |
| [parking_lot](https://github.com/Amanieu/parking_lot) | Mutex for shared hardware state                           |
| [image](https://github.com/image-rs/image)            | Decodes the embedded app icon                             |


---

## License

This project is licensed under the [MIT License](LICENSE).