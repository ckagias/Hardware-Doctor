# Trouble

### A self-hosted desktop app for testing your PC's hardware, built with Tauri, React, and Rust

[About](#about) • [Features](#features) • [Modules](#modules) • [Installation](#installation) • [Building](#building) • [Dependencies](#dependencies) • [Contributing](#contributing) • [License](#license)

---

## About

A small desktop app for testing your PC's hardware: microphones, headphones/speakers, your keyboard, and your mouse. Windows' built-in troubleshooting for these is unreliable, and testing them online usually means hunting down a sketchy website. Trouble runs locally instead.

Built with Tauri, React, and TypeScript on the frontend, with the hardware logic (audio I/O, keyboard state, mouse state) handled in Rust.

If you find this useful, feel free to leave a star to help others find it!

---

## Features

- Live microphone level meter with record and playback
- Left/right/both channel test tones for headphones and speakers
- Full 100% on-screen keyboard layout with per-key press tracking
- Top-down mouse diagram with per-button press and tested-state tracking, including scroll and side buttons
- Live click counter for every mouse input, reset alongside the tested state
- Device picker with refresh for both audio modules
- All hardware state and device I/O handled in Rust, not the browser

---

## Modules


| Module                    | Description                                                                                                             |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Microphone test           | Pick an input device, watch a live level meter, record and play back a short clip                                       |
| Headphones / speakers     | Pick an output device, play a tone to the left channel, right channel, or both                                          |
| Keyboard test             | Press keys on your physical keyboard and watch them light up on a full 100% layout, with a running count of tested keys |
| Mouse test                | Click left/right/middle, scroll up/down, and press the side buttons, with a live per-input click counter                |
| Controller test (planned) | Button and stick visualization via the Gamepad API                                                                      |


The keyboard module can't capture the Windows/Super key or media keys, since the app intentionally avoids a system-wide key hook to read them.

---

## Installation

1. **Clone the repository**
  ```bash
   git clone https://github.com/ckagias/Hardware-Doctor.git
   cd Hardware-Doctor
  ```
2. **Install dependencies**
  ```bash
   npm install
  ```
3. **Run in development**
  ```bash
   npm run tauri dev
  ```

Requires [Node.js](https://nodejs.org) and the [Rust toolchain](https://www.rust-lang.org/tools/install). See [Tauri's prerequisites](https://tauri.app/start/prerequisites/) for platform-specific setup.

---

## Building

```bash
npm run tauri build
```

Produces a native installer/binary for your current platform in `src-tauri/target/release/bundle/`.

---

## Dependencies


| Package                                                 | Purpose                                 |
| ------------------------------------------------------- | --------------------------------------- |
| [Tauri](https://tauri.app/)                             | Desktop app shell and native bridge     |
| [React](https://react.dev/)                             | Frontend UI                             |
| [cpal](https://github.com/RustAudio/cpal)               | Cross-platform audio I/O                |
| [hound](https://github.com/ruuda/hound)                 | WAV encoding for mic recordings         |
| [parking_lot](https://github.com/Amanieu/parking_lot)   | Mutex for shared hardware state         |
| [base64](https://github.com/marshallpierce/rust-base64) | Encodes recorded clips for the frontend |


---

## Contributing

PRs adding new hardware test modules are welcome. Each module lives in `src/pages/` and is registered in `src/lib/modules.ts`; backend logic lives in `src-tauri/src/`.

---

## License

This project is licensed under the [MIT License](LICENSE).