# Swift Launcher

Swift Launcher is a keyboard-first launcher written in Rust + Slint.  
Press a global shortcut (for example `Super+Space`), type a few letters, and run actions instantly.

![screenshot](assets/screenshot.png)

## Goals

- Minimal latency: open → search → execute fast.
- Great UX: clean UI, solid keyboard navigation, sane defaults.
- Ship it as a solid native binary first, then add a Flatpak build later (if possible).

## Features

- [x] Fuzzy search (ranked) over actions.
- [ ] Shell command actions (optional working directory).
- [x] Desktop app actions (from `.desktop` entries).
- [ ] Project workflows (open folder, run dev server, run tests).
- [ ] Keyboard-only workflow (Up/Down select, Enter execute, Esc close).
- [ ] Optional system tray integration (toggle + quick status).
- [ ] Config file (TOML/JSON/RON), with optional hot-reload.
- [ ] (Future) Flatpak build and Flatpak-specific actions.

## Installation

### Native

```bash
git clone https://github.com/pezfisk/swift-launcher
cd swift-launcher
cargo build --release
./target/release/swift-launcher
````

#### CONFIGURE ON DESKTOP ENVIROMENTS
 
## Configuration (Not yet implemented, just for reference)
Default path:
- Native: ~/.config/swift-launcher/config.toml
- Flatpak: ~/.var/app/dev.invrs.swift/config/config.toml

### Example config

```toml
[[variables]]
editor = "vim"

[[actions]]
name = "Open dotfiles"
cwd = "/home/$USER/dotfiles"
exec = "$editor /home/$USER/dotfiles"
```

