# Nagi

Lightning-fast video wallpaper daemon. Rust rewrite of [hidamari](https://github.com/jeffshee/hidamari).

Plays videos, streams, and web pages as your desktop wallpaper on Linux (X11) and Windows.

## Features

- Video wallpaper via mpv (hardware accelerated)
- Live stream support (YouTube, Twitch, etc. via yt-dlp)
- Multi-monitor support
- Pause when a window is fullscreen or maximized
- System tray with play/pause, mute, reload, shuffle
- GNOME static wallpaper save/restore
- GUI for video selection and settings

## Installation

### Linux (Debian/Ubuntu)

Download the `.deb` from [Releases](https://github.com/StarAtNyte/nagi/releases):

```bash
sudo apt install ./nagi_*_amd64.deb
```

**Dependencies:** `libmpv2`, `yt-dlp` (optional, for streams)

Enable as a user service:

```bash
systemctl --user enable --now nagi
```

### Windows

Download the `.msi` from [Releases](https://github.com/StarAtNyte/nagi/releases) and run it.

The installer bundles `libmpv.dll` — no separate install needed.

## Usage

```
nagi [OPTIONS]

Options:
  -b, --background   Run without opening the GUI
  -d, --debug        Enable debug logging
  -r, --reset        Reset config to defaults
  -p, --pause <SEC>  Delay startup by N seconds
  -h, --help         Print help
```

On first launch, the GUI opens. Drop videos into the Nagi folder (`~/Videos/Nagi` on Linux, `%USERPROFILE%\Videos\Nagi` on Windows), then select and apply.

## Building from Source

**Linux requirements:**

```bash
sudo apt install libmpv-dev libx11-dev libxcb1-dev
```

```bash
cargo build --release
```

Binary at `target/release/nagi`.

**Build `.deb`:**

```bash
cargo install cargo-deb
cargo deb
```

**Windows requirements:** MSVC toolchain + `libmpv.dll` in `wix/` (from [shinchiro's mpv builds](https://github.com/shinchiro/mpv-winbuild-cmake/releases)).

```bash
cargo build --release --target x86_64-pc-windows-msvc
dotnet tool install --global wix
wix build wix/main.wxs -o target/nagi-0.1.0-x64.msi
```

## Configuration

Config file: `~/.config/nagi/config.json` (Linux) / `%APPDATA%\nagi\config.json` (Windows)

| Key | Default | Description |
|-----|---------|-------------|
| `mode` | `"Null"` | `Null`, `Video`, `Stream`, `Webpage` |
| `is_mute` | `false` | Mute audio |
| `audio_volume` | `50` | Volume 0–100 |
| `is_pause_when_maximized` | `true` | Pause on fullscreen/maximize |
| `is_mute_when_maximized` | `false` | Mute on fullscreen/maximize |
| `is_static_wallpaper` | `true` | GNOME static wallpaper mode |
| `fade_duration_sec` | `1.5` | Fade transition duration |

## Features (Cargo)

| Feature | Default | Description |
|---------|---------|-------------|
| `player-mpv` | yes | mpv video playback |
| `gui` | no | egui settings window |
| `tray` | no | System tray icon |
| `static-wallpaper` | no | Static image support |
| `full` | no | All features |

```bash
cargo build --release --features full
```

## License

MIT
