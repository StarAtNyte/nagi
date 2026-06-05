use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub mode: Mode,
    pub data_source: HashMap<String, String>,
    pub is_mute: bool,
    pub audio_volume: u32,
    pub is_static_wallpaper: bool,
    pub static_wallpaper_blur_radius: u32,
    pub is_pause_when_maximized: bool,
    pub is_mute_when_maximized: bool,
    pub fade_duration_sec: f64,
    pub fade_interval: f64,
    pub is_show_systray: bool,
    pub is_first_time: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Mode {
    Null,
    Video,
    Stream,
    Webpage,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Null => write!(f, "MODE_NULL"),
            Mode::Video => write!(f, "MODE_VIDEO"),
            Mode::Stream => write!(f, "MODE_STREAM"),
            Mode::Webpage => write!(f, "MODE_WEBPAGE"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_config(&[])
    }
}

impl Config {
    pub fn default_config(monitors: &[MonitorInfo]) -> Self {
        let mut data_source = HashMap::new();
        for m in monitors {
            data_source.insert(m.name.clone(), String::new());
        }
        data_source.insert("Default".into(), String::new());

        Config {
            version: CONFIG_VERSION,
            mode: Mode::Null,
            data_source,
            is_mute: false,
            audio_volume: 50,
            is_static_wallpaper: true,
            static_wallpaper_blur_radius: 5,
            is_pause_when_maximized: true,
            is_mute_when_maximized: false,
            fade_duration_sec: 1.5,
            fade_interval: 0.1,
            is_show_systray: false,
            is_first_time: true,
        }
    }

    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => log::warn!("Failed to parse config: {e}"),
                    }
                }
                Err(e) => log::warn!("Failed to read config: {e}"),
            }
        }
        let monitors = crate::monitor::detect_monitors();
        let config = Config::default_config(&monitors);
        config.save();
        config
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, &content) {
                    log::error!("Failed to save config: {e}");
                }
            }
            Err(e) => log::error!("Failed to serialize config: {e}"),
        }
    }

    #[allow(dead_code)]
    pub fn set_mode(&mut self, mode: Mode, source: Option<&str>, monitor: Option<&str>) {
        self.mode = mode.clone();
        if let (Some(source), Some(monitor_name)) = (source, monitor) {
            self.data_source.insert(monitor_name.to_string(), source.to_string());
        }
        if let Some(source) = source {
            self.data_source.insert("Default".into(), source.to_string());
        }
        self.save();
    }
}

fn config_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let xdg = std::env::var("XDG_CONFIG_HOME")
            .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap_or_default()));
        PathBuf::from(xdg).join("nagi")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".into());
        PathBuf::from(appdata).join("nagi")
    }
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn video_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let xdg_videos = std::env::var("XDG_VIDEOS_DIR")
            .or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                let videos = PathBuf::from(&home).join("Videos");
                if videos.exists() {
                    Ok(videos.to_string_lossy().to_string())
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            })
            .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_default());
        PathBuf::from(xdg_videos).join("Nagi")
    }
    #[cfg(target_os = "windows")]
    {
        let videos = std::env::var("USERPROFILE")
            .unwrap_or_else(|_| "C:\\Users\\Default".into());
        PathBuf::from(videos).join("Videos").join("Nagi")
    }
}

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    #[allow(dead_code)]
    pub is_primary: bool,
}
