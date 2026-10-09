# orCAL

orCAL is a Rust calculator with a Tauri interface. The web UI is served by Tauri and calls the Rust calculation engine. It is software from the Colony project (https://github.com/Project-Colony/Colony).

## Features

- Operations: addition, subtraction, multiplication, division and decimal numbers
- Results are shown to 12 significant digits, so binary rounding noise never
  reaches the screen: `0.1+0.2` gives 0.3 and `1/8` gives 0.125
- Percent: `%` divides the number before it by 100 at full precision, so
  `200*0.5%` gives 1
- Scientific keypad (tablet layout): parentheses, powers (`^`, right-associative),
  factorials (`!`, whole numbers from 0 to 170), sin, cos and tan in degrees,
  ln, log (base 10), square root, the constants π and e, and ANS for the
  previous result
- Implicit multiplication: `2π`, `2(3)` and `(2)(3)` multiply
- Error handling: incomplete expression, invalid token, division by zero,
  undefined or infinite result, factorial out of range, and expressions nested
  more than 256 levels deep

Every expression, from either layout, is evaluated by the Rust engine in
`crates/orcal-core`. The interface never evaluates code itself, and the web view
runs under a content security policy that only allows the app's own scripts,
styles and IPC.

## Prerequisites

- Rust (stable toolchain) and Cargo installed
- System dependencies for Tauri (see the Tauri documentation for your OS)

## Installation

```bash
cargo build -p orcal-tauri
```

## Usage

To start the Tauri app in development mode:

```bash
cargo run -p orcal-tauri
```

## Development

- `cargo run -p orcal-tauri` to run the Tauri app
- `cargo build -p orcal-tauri --release` to compile in release mode

## Structure

- `crates/orcal-core`: parsing and evaluation logic
- `src-tauri`: Tauri application
- `ui/`: HTML/CSS/JS interface

## Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Windows builds are signed this way once the SignPath Foundation has accepted
the project; until then they ship without Authenticode. Every release asset,
on every platform, is always signed with the Project-Colony organisation's
ed25519 key, which Colony verifies before installing it.

- Committers and reviewers: [MotherSphere](https://github.com/MotherSphere)
- Approvers: [MotherSphere](https://github.com/MotherSphere)

### Privacy policy

This program will not transfer any information to other networked systems
unless specifically requested by the user or the person installing or
operating it.

orCAL contains no network code and no telemetry. Its interface is rendered by
the operating system's web view (Microsoft Edge WebView2 on Windows, WebKit on
macOS and Linux), which follows its vendor's own privacy policy.

## License

GPL-3.0-or-later. You may redistribute and modify orCAL under the terms of
version 3 of the GNU General Public License, or (at your option) any later
version. The full text is in [LICENSE](LICENSE).
