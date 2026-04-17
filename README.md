# Arduino TUI

A fast, keyboard-centric terminal user interface (TUI) for browsing, searching, and managing Arduino libraries. 

Built with Rust and [Ratatui](https://github.com/ratatui/ratatui), `arduino-tui` acts as a lightweight, non-blocking wrapper around the official `arduino-cli`, bringing a "Taproom-like" package manager experience directly to your terminal.

## Features

- ⚡ **Asynchronous & Non-blocking**: Uses `tokio` to run `arduino-cli` commands in the background. The UI never freezes while downloading or searching for libraries.
- ⌨️ **Vim-like Keybindings**: Navigate and manage your libraries entirely from the keyboard without reaching for the mouse.
- 🔍 **Fast Searching**: Instantly search the entire Arduino library registry.
- 📦 **Dependency Management**: Install and uninstall libraries with a single keystroke.
- ℹ️ **Detailed Views**: View library versions, authors, descriptions, and installation status at a glance.

## Prerequisites

`arduino-tui` requires the official **Arduino CLI** to be installed and available in your system's `PATH`.

- [Install Arduino CLI](https://arduino.github.io/arduino-cli/latest/installation/)

## Installation


### Option 1: Cargo (Recommended for Rust users)

If you have Rust installed, you can install the application directly from crates.io:

```bash
cargo install arduino-tui
```

### Option 2: Homebrew (macOS / Linux)

You can install `arduino-tui` and its dependency `arduino-cli` using Homebrew via our custom tap:

```bash
brew tap igmrrf/arduino-tui
brew install arduino-tui
```

### Option 3: Building from Source

To build and install directly from the GitHub repository:

```bash
git clone https://github.com/igmrrf/arduino-tui
cd arduino-tui
cargo install --path .
```

## Usage

Run the application from your terminal:

```bash
arduino-tui
```

### Keybindings

| Key | Action |
| :--- | :--- |
| `j` / `Down` | Move down the list |
| `k` / `Up` | Move up the list |
| `/` | Enter Search mode |
| `Enter` | Execute search (in Search mode) |
| `Esc` | Clear search / Return to installed libraries / Close Help |
| `i` | Install the currently selected library |
| `u` | Uninstall the currently selected library |
| `?` / `h` | Toggle the Help menu |
| `q` | Quit the application |

## Architecture

This project is built using:
- **[Rust](https://www.rust-lang.org/)**: For memory safety and performance.
- **[Ratatui](https://ratatui.rs/)**: For rendering the terminal user interface.
- **[Tokio](https://tokio.rs/)**: For asynchronous runtime and command execution.
- **[Serde](https://serde.rs/)**: For parsing JSON output from `arduino-cli`.

## License

This project is licensed under either of
- Apache License, Version 2.0
- MIT License
at your option.
