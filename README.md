# orCAL

orCAL is a Rust calculator with a Tauri interface. The web UI is served by Tauri and calls the Rust calculation engine. It is software from the Colony project (https://github.com/Project-Colony/Colony).

## Features

- Operations: addition, subtraction, multiplication, division and decimal numbers
- Results are rounded to 12 significant digits (whole parts are never cut),
  and sums and differences to the 15 digits an f64 keeps, so `0.1+0.2` gives
  0.3, `0.1+0.2-0.3` gives 0 and `1/8` gives 0.125
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
`src/engine.rs`. The interface never evaluates code itself, and the web view
runs under a content security policy that only allows the app's own scripts,
styles and IPC.

## Prerequisites

- Rust 1.90 or newer, with Cargo
- System dependencies for Tauri (see the Tauri documentation for your OS)

## Installation

```bash
cargo build --release
```

The binary is `target/release/orCAL` (`orCAL.exe` on Windows).

## Usage

To start the Tauri app in development mode:

```bash
cargo run
```

`orCAL --version` prints the version and exits without opening a window.

## Where orCAL keeps its files

Like every Colony program, orCAL keeps its files under `Colony/orCAL/`, in the
local (never roaming) application folders:

| | Linux | Windows | macOS |
|---|---|---|---|
| Settings | `~/.config/Colony/orCAL/` | `%LOCALAPPDATA%\Colony\orCAL\` | `~/Library/Application Support/Colony/orCAL/` |
| Web view data | `~/.local/share/Colony/orCAL/webview/` | `%LOCALAPPDATA%\Colony\orCAL\webview\` | the system's WebKit location |

The settings folder holds `preferences/preferences.json` (theme, text size,
accessibility, language and startup layout, saved as soon as they change) and
`window-position.json`. A preferences file orCAL cannot read is kept as
`preferences.json.bak` and orCAL starts from the defaults.

Versions before the Colony layout kept these in `com.orcal.app` folders. The
first launch of a newer version copies the window position over and imports
the theme from the old web view storage; the launch after that switches to the
new web view folder. The old folders are left in place and can be deleted by
hand.

## Development

- `cargo run` to run the Tauri app
- `cargo test` to run the engine and command-line tests
- `cargo build --release` to compile in release mode

## Structure

- `src/main.rs`: entry point and Tauri setup
- `src/engine.rs`: parsing and evaluation logic
- `src/commands.rs`: the commands the interface calls to evaluate an expression and to fill the About page
- `src/window_state.rs`: remembers the window position between runs
- `src/paths.rs`: the Colony folder layout
- `src/preferences.rs`: `preferences.json` and the commands that read and write it
- `src/migrate.rs`: copies the settings of versions before the Colony layout
- `src/linux_display.rs`: the WebKitGTK workaround for Wayland
- `src/ui/`: HTML/CSS/JS interface; `src/ui/common/` is shared by both layouts
- `src/ui/vendor/colony/`: the Colony theme data, refreshed with
  `scripts/sync-colony-tokens.sh <commit>` and never edited by hand
- `tauri.conf.json`: window, content security policy and capability settings
- `assets/icons/`: application icons

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
