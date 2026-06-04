pub mod config;
pub mod monitor;
pub mod streaming;

#[cfg(feature = "player-mpv")]
pub mod player;

#[cfg(target_os = "linux")]
pub mod desktop_linux;
#[cfg(target_os = "windows")]
pub mod desktop_windows;

#[cfg(target_os = "linux")]
pub mod window_detect_linux;
#[cfg(target_os = "windows")]
pub mod window_detect_windows;

#[cfg(feature = "gui")]
pub mod gui;

#[cfg(feature = "tray")]
pub mod tray;
