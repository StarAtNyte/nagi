use clap::Parser;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

mod config;
mod monitor;
mod streaming;

#[cfg(feature = "player-mpv")]
mod player;

#[cfg(target_os = "linux")]
mod desktop_linux;
#[cfg(target_os = "windows")]
mod desktop_windows;

#[cfg(target_os = "linux")]
mod window_detect_linux;
#[cfg(target_os = "windows")]
mod window_detect_windows;

#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "tray")]
mod tray;

#[derive(Parser, Debug)]
#[command(name = "nagi", version = "0.1.0", about = "Lightning-fast video wallpaper daemon")]
struct Args {
    #[arg(short = 'p', long, default_value_t = 0)]
    pause: u64,
    #[arg(short = 'b', long)]
    background: bool,
    #[arg(short = 'd', long)]
    debug: bool,
    #[arg(short = 'r', long)]
    reset: bool,
}

fn main() {
    let args = Args::parse();

    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
    if args.pause > 0 {
        thread::sleep(Duration::from_secs(args.pause));
    }

    log::info!("Starting Nagi v{}", env!("CARGO_PKG_VERSION"));

    let video_dir = config::video_dir();
    let _ = std::fs::create_dir_all(&video_dir);

    let monitors = monitor::detect_monitors();
    let mut config = config::Config::load();
    if args.reset {
        config = config::Config::default_config(&monitors);
        config.save();
    }

    // ── Linux: X11 desktop windows ──────────────────────────────
    #[cfg(target_os = "linux")]
    let mut x11_desktop = desktop_linux::X11Desktop::new();

    #[cfg(target_os = "linux")]
    let xwins: Vec<(String, u64)> = {
        let mut xw = Vec::new();
        if let Some(ref mut d) = x11_desktop {
            for m in &monitors {
                if let Some(win) = d.create_desktop_window(m.x, m.y, m.width, m.height, &m.name) {
                    log::info!("Desktop {} {}x{} @{},{} => XID={}", m.name, m.width, m.height, m.x, m.y, win);
                    xw.push((m.name.clone(), win as u64));
                }
            }
        }
        xw
    };

    // ── Windows: Win32 WorkerW wallpaper windows ─────────────────
    #[cfg(target_os = "windows")]
    let win32_desktop = desktop_windows::DesktopHandle::create_wallpaper_windows(&monitors);

    #[cfg(target_os = "windows")]
    let xwins: Vec<(String, u64)> = win32_desktop.as_wid_pairs();

    // ── Fallback for other platforms ──────────────────────────────
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let xwins: Vec<(String, u64)> = monitors.iter().map(|m| (m.name.clone(), 0)).collect();

    // ── Static wallpaper (Linux/GNOME) ───────────────────────────
    #[cfg(target_os = "linux")]
    let original_wallpaper = desktop_linux::get_gnome_wallpaper();

    #[cfg(target_os = "linux")]
    if config.is_static_wallpaper {
        if let Some(ref wp) = original_wallpaper {
            log::info!("Saved original wallpaper: {}", wp.0);
        }
        desktop_linux::gnome_desktop_icon_workaround();
    }

    // ── Player (mpv per monitor) ─────────────────────────────────
    #[cfg(feature = "player-mpv")]
    let player_mgr = {
        let mut pm = player::PlayerManager::new(config.clone());
        if config.mode != config::Mode::Null && !xwins.is_empty() {
            pm.init_players(&monitors, &xwins);
            log::info!("Player: {} windows", pm.players.len());
        }
        Arc::new(Mutex::new(pm))
    };

    // ── Window detection: pause on fullscreen (Linux) ────────────
    #[cfg(all(feature = "player-mpv", target_os = "linux"))]
    {
        let (tx, rx) = mpsc::channel();
        window_detect_linux::start_window_monitor(tx);
        let pm = player_mgr.clone();
        let pause = config.is_pause_when_maximized;
        thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if let Ok(mut p) = pm.lock() {
                    if pause && (state.is_any_maximized || state.is_any_fullscreen) {
                        p.pause_playback();
                    } else if pause {
                        p.start_playback();
                    }
                }
            }
        });
    }

    // ── Window detection: pause on fullscreen (Windows) ──────────
    #[cfg(all(feature = "player-mpv", target_os = "windows"))]
    {
        let (tx, rx) = mpsc::channel();
        window_detect_windows::start_window_monitor(tx, monitors.clone());
        let pm = player_mgr.clone();
        let pause = config.is_pause_when_maximized;
        thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                if let Ok(mut p) = pm.lock() {
                    if pause && (state.is_any_maximized || state.is_any_fullscreen) {
                        p.pause_playback();
                    } else if pause {
                        p.start_playback();
                    }
                }
            }
        });
    }

    // ── GUI ──────────────────────────────────────────────────────
    #[cfg(feature = "gui")]
    if !args.background {
        let cfg_gui = config.clone();
        thread::spawn(move || {
            let opts = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 600.0]),
                ..Default::default()
            };
            let _ = eframe::run_native("Nagi", opts, Box::new(|_cc| Box::new(gui::NagiGui::new(cfg_gui))));
        });
    }

    // ── System tray ──────────────────────────────────────────────
    #[cfg(feature = "tray")]
    let tray_handle = tray::TrayHandle::new();

    log::info!("Nagi running. Ctrl+C to quit.");

    loop {
        // ── Tray event loop ──────────────────────────────────────
        #[cfg(all(feature = "tray", feature = "player-mpv"))]
        if let Some(ref tray) = tray_handle {
            if let Some(action) = tray.poll_action() {
                match action {
                    tray::TrayAction::ToggleMute => {
                        if let Ok(mut pm) = player_mgr.lock() {
                            let mute = !pm.config.is_mute;
                            pm.config.is_mute = mute;
                            pm.set_mute_all(mute);
                            pm.config.save();
                        }
                    }
                    tray::TrayAction::TogglePlayPause => {
                        if let Ok(mut pm) = player_mgr.lock() {
                            let playing = pm.players.values().any(|p| p.is_playing);
                            if playing { pm.pause_playback(); } else { pm.start_playback(); }
                        }
                    }
                    tray::TrayAction::Reload => {
                        if let Ok(mut pm) = player_mgr.lock() {
                            let src = pm.config.data_source.get("Default").cloned().unwrap_or_default();
                            if !src.is_empty() {
                                pm.load_source(&src);
                            }
                        }
                    }
                    tray::TrayAction::Lucky => {
                        use rand::seq::SliceRandom;
                        let dir = config::video_dir();
                        let videos: Vec<_> = std::fs::read_dir(&dir)
                            .into_iter()
                            .flatten()
                            .flatten()
                            .filter(|e| e.path().is_file() && e.path().extension()
                                .map(|x| ["mp4","webm","mkv","avi","mov","gif"].contains(&x.to_string_lossy().to_lowercase().as_str()))
                                .unwrap_or(false))
                            .map(|e| e.path().to_string_lossy().to_string())
                            .collect();
                        if let Some(v) = videos.choose(&mut rand::thread_rng()) {
                            if let Ok(mut pm) = player_mgr.lock() {
                                pm.load_source(v);
                            }
                        }
                    }
                    tray::TrayAction::Quit | tray::TrayAction::ShowGui => {
                        if matches!(action, tray::TrayAction::Quit) {
                            log::info!("Quit requested via tray");
                            #[cfg(target_os = "linux")]
                            if config.is_static_wallpaper {
                                desktop_linux::restore_original_wallpaper(
                                    &original_wallpaper.as_ref().map(|w| w.0.clone()),
                                    &original_wallpaper.as_ref().map(|w| w.1.clone()),
                                );
                            }
                            std::process::exit(0);
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}
