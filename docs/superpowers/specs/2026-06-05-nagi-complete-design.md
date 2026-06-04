# Nagi — Complete Implementation Design

## Overview

Make `nagi` production-ready on Linux and Windows: wire up all existing stub/unused code, fix 16 compiler warnings, finish the Win32 wallpaper backend, and add `.deb` + `.msi` packaging with GitHub Actions CI.

---

## Section 1: Component Completion

### Linux

**Multi-monitor (RandR)**
- Current: `monitor::detect_monitors` reads X11 screen root dimensions — single monitor only.
- Fix: use `x11rb::randr` extension to enumerate outputs. Each active output becomes a `MonitorInfo` with correct x/y/width/height.
- Crate addition: `x11rb` already in deps; enable `randr` feature.

**Tray**
- Current: `TrayHandle` struct in `tray.rs` is never instantiated.
- Fix: in `main.rs`, under `#[cfg(feature = "tray")]`, construct `TrayHandle::new(config.mode.clone())` after player init. Spawn event loop thread to handle menu item clicks (show GUI, toggle mute, toggle play/pause, reload, lucky, quit).

**Static wallpaper**
- Current: `set_static_wallpaper_gnome`, `restore_original_wallpaper`, `get_gnome_wallpaper`, `gnome_desktop_icon_workaround` exist in `desktop_linux.rs` but are never called.
- Fix: call `get_gnome_wallpaper()` at startup to save original. When config mode is `Mode::Null` or on exit, call `restore_original_wallpaper`. Call `set_static_wallpaper_gnome` + `gnome_desktop_icon_workaround` when static wallpaper mode activates.

**Unused warnings**
- 16 warnings: dead functions, unused variables. Prefix unused locals with `_`, add `#[allow(dead_code)]` where function is intentionally kept for future use, or wire up as above.

---

### Windows

**Win32 desktop window (WorkerW trick)**
- Current: `desktop_windows.rs::DesktopHandle` stores monitor metadata but creates no actual windows.
- Fix: implement actual Win32 wallpaper window using the Wallpaper Engine approach:
  1. `FindWindow("Progman", NULL)` to get the shell's Program Manager window.
  2. `SendMessage(progman, 0x052C, 0, 0)` to force WorkerW creation.
  3. `EnumWindows` to find the WorkerW window (sibling of SHELLDLL_DefView).
  4. `CreateWindowEx` with `WS_CHILD | WS_VISIBLE` style, parent = WorkerW HWND.
  5. Return HWND as `u64` for mpv `wid` option (same as Linux).
- All unsafe Win32 calls via the existing `windows` crate.

**mpv on Windows**
- Same `mpv` crate, same `PlayerManager`/`PlayerInstance` logic.
- Requires `libmpv.dll` at runtime (see Packaging section).
- Main wires up Windows desktop windows same as Linux path: collect `(name, hwnd_as_u64)` pairs, pass to `PlayerManager::init_players`.

**Window detect (Windows)**
- Current: `window_detect_windows.rs::start_window_monitor` exists but never called in `main.rs`.
- Fix: wire up in `main.rs` under `#[cfg(all(feature = "player-mpv", target_os = "windows"))]`, same pattern as Linux branch.
- Note: current fullscreen detection hardcodes 1920×1080. Improve to check against actual monitor dimensions from `detect_monitors()`.

---

## Section 2: Packaging

### Linux `.deb` (cargo-deb)

Add to `Cargo.toml`:

```toml
[package.metadata.deb]
maintainer = "Nagi Contributors"
copyright = "2024"
license-file = ["LICENSE", "0"]
extended-description = "Lightning-fast video wallpaper daemon. Rust rewrite of hidamari."
depends = "$auto, libmpv2"
section = "multimedia"
priority = "optional"
assets = [
    ["target/release/nagi", "usr/bin/", "755"],
    ["packaging/linux/nagi.desktop", "usr/share/applications/", "644"],
    ["packaging/linux/nagi.service", "lib/systemd/user/", "644"],
]
```

Files to create:
- `packaging/linux/nagi.desktop` — XDG desktop entry (Name, Exec, Icon, Categories=AudioVideo)
- `packaging/linux/nagi.service` — systemd user unit (`ExecStart=/usr/bin/nagi -b`, `WantedBy=default.target`)

Build command: `cargo deb` → outputs `target/debian/nagi_0.1.0_amd64.deb`

---

### Windows `.msi` (WiX v4)

Files to create:
- `wix/main.wxs` — WiX source: Product, Package, Feature, Component for `nagi.exe` + `libmpv.dll`
- `wix/libmpv.dll` — NOT committed to git. CI downloads from mpv-player/mpv GitHub releases (Windows build). Documented in README.

WiX configuration:
- Install dir: `Program Files\Nagi`
- Start menu shortcut
- Add install dir to `PATH` via Environment element
- GUID-based component ids (generate once, hardcode)

Build steps:
```
cargo build --release --target x86_64-pc-windows-msvc
wix build wix/main.wxs -o target/nagi-0.1.0-x64.msi
```

---

### GitHub Actions (`.github/workflows/release.yml`)

Trigger: push to `main` OR tag matching `v*`.

**Job: build-linux** (ubuntu-latest)
1. `sudo apt-get install libmpv-dev cargo-deb`
2. `cargo deb`
3. Upload `target/debian/nagi_*.deb` as artifact + release asset

**Job: build-windows** (windows-latest)
1. Download `libmpv.dll` from mpv-player/mpv latest release (PowerShell)
2. Copy `libmpv.dll` to `wix/`
3. `cargo build --release --target x86_64-pc-windows-msvc`
4. Install WiX v4 (`dotnet tool install --global wix`)
5. `wix build wix/main.wxs -o target/nagi-0.1.0-x64.msi`
6. Upload `.msi` as artifact + release asset

---

## File Change Summary

| File | Change |
|------|--------|
| `Cargo.toml` | Add `randr` feature to x11rb, add `[package.metadata.deb]` |
| `src/monitor.rs` | Replace screen-root detection with RandR enumeration |
| `src/desktop_windows.rs` | Implement WorkerW Win32 window creation |
| `src/main.rs` | Wire tray, static wallpaper, Windows desktop+player+window-detect |
| `src/tray.rs` | Add click event handling (menu item actions) |
| `src/window_detect_windows.rs` | Fix fullscreen check to use actual monitor dims |
| `packaging/linux/nagi.desktop` | New file |
| `packaging/linux/nagi.service` | New file |
| `wix/main.wxs` | New file |
| `.github/workflows/release.yml` | New file |

---

## Out of Scope

- Wayland support (X11 only for Linux)
- macOS
- Webpage mode backend (config value exists, no renderer)
- yt-dlp streaming integration (functions exist in `streaming.rs`, not wired to player)
