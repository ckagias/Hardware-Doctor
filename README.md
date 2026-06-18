# Trouble

A small, self-hosted desktop app for testing your PC's hardware — starting with
microphones and headphones/speakers, since Windows' built-in troubleshooting for
these is unreliable and testing them usually means hunting down a sketchy website.

Built with [Tauri](https://tauri.app) + React + TypeScript, so it's a single
lightweight installable app instead of a browser tab.

## Features (v1)

- **Microphone test** — pick an input device, watch a live level meter while you
  talk, and record/play back a short clip to hear exactly what your mic captures.
- **Headphones / speakers test** — pick an output device and play a tone to the
  left channel, right channel, or both, to confirm everything is wired correctly.

## Roadmap

- Keyboard tester (key press visualization)
- Mouse tester (clicks, movement, scroll)
- Controller tester (button/stick visualization, via the Gamepad API)

## Development

Requires [Node.js](https://nodejs.org) and the
[Rust toolchain](https://www.rust-lang.org/tools/install) (Tauri's prerequisites:
https://tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev
```

## Building

```bash
npm run tauri build
```

Produces a native installer/binary for your current platform in
`src-tauri/target/release/bundle/`.

## Contributing

This is an open-source project — PRs adding new hardware test modules
(keyboard, mouse, controller, etc.) are welcome. Each module lives in
`src/pages/` and is registered in `src/lib/modules.ts`.
