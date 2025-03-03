# Young Shell

A template for a Rust application that's using [Slint](https://slint.rs/) for the user interface.

## About

This template helps you get started developing a Rust application with Slint as toolkit
for the user interface. It demonstrates the integration between the `.slint` UI markup and
Rust code, how to react to callbacks, get and set properties, and use basic widgets.


## Features
- Top status bar (`astal toggle top`)
  - [ ] Weather
  - [ ] Network status
  - [ ] Time
  - [ ] Wallpaper switcher
- Right music bar (`astal toggle right`)
  - [ ] Music controls (Uses MPRIS)
  - [ ] Lyrics (requires [sptlrx](https://github.com/raitonoberu/sptlrx) in mpris mode)
  - [ ] Music visualizer (uses [CAVA](https://github.com/karlstav/cava))
- Notifications
  - Uses regular wayland protocols for notifications
- Launcher
  - [ ] Fuzzy app launcher
  - [ ] Calculator (requires [libqalculate](https://qalculate.github.io/). `= ` prefix)
  - [ ] Journal entry (requires Obsidian + Thino Pro plugin. I will add support for other journaling apps soon™. `; ` prefix)
  - [ ] Task taking (will use ticktick. I will add support for other task managers soon™)

- [ ] App Launcher
- [ ] Power Menu
- [ ] Wallpaper Selector
- [ ] System Tray
- [ ] Notifications
- [ ] Terminal
- [ ] Pins
- [ ] Kanban Board
- [ ] Calendar (Incomplete)
- [ ] Color Picker
- [ ] Dashboard
- [ ] Network Manager
- [ ] Bluetooth Manager
- [ ] Power Manager
- [ ] Settings
- [ ] Screenshot Tool
- [ ] Screen Recorder
- [ ] Clipboard Manager
- [ ] Dock
- [ ] Workspaces Overview
- [ ] Multimodal AI Assistant
- [ ] Vertical Layout

## Usage

1. Install Rust by following its [getting-started guide](https://www.rust-lang.org/learn/get-started).
   Once this is done, you should have the `rustc` compiler and the `cargo` build system installed in your `PATH`.
2. Download and extract the [ZIP archive of this repository](https://github.com/slint-ui/slint-rust-template/archive/refs/heads/main.zip).
3. Rename the extracted directory and change into it:
    ```
    mv slint-rust-template-main my-project
    cd my-project    
    ```
4. Build with `cargo`:
    ```
    cargo build
    ```
5. Run the application binary:
    ```
    cargo run
