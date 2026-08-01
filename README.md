# orCAL

orCAL is a Rust calculator with a Tauri interface. The web UI is served by Tauri and calls the Rust calculation engine. It is software from the Colony project (https://github.com/MotherSphere/Colony).

## Features

- Operations: addition, subtraction, multiplication, division
- Parentheses and decimal numbers
- Error handling (incomplete expression, division by zero, invalid token)

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
- `tasks/`: future instructions and work plan
- `docs/`: documentation and compliance

## License

GPL-3.0-or-later. You may redistribute and modify orCAL under the terms of
version 3 of the GNU General Public License, or (at your option) any later
version. The full text is in [LICENSE](LICENSE).
