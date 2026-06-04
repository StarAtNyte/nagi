use eframe::egui;
use crate::config::{Config, Mode};

#[derive(Default)]
pub struct NagiGui {
    pub config: Config,
    pub video_paths: Vec<String>,
    pub selected_video: Option<usize>,
    pub stream_url: String,
    pub webpage_url: String,
    pub needs_save: bool,
    pub show_welcome: bool,
}

impl NagiGui {
    pub fn new(config: Config) -> Self {
        let show_welcome = config.is_first_time;
        let mut gui = Self {
            config,
            show_welcome,
            ..Default::default()
        };
        gui.refresh_videos();
        gui
    }
}

impl eframe::App for NagiGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.show_welcome {
            self.show_welcome_dialog(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Nagi");

                ui.separator();

                let mut vol = self.config.audio_volume as f32;
                if ui.add(egui::Slider::new(&mut vol, 0.0..=100.0).text("Volume")).changed() {
                    self.config.audio_volume = vol as u32;
                    self.needs_save = true;
                }

                let mut mute = self.config.is_mute;
                if ui.checkbox(&mut mute, "Mute Audio").changed() {
                    self.config.is_mute = mute;
                    self.needs_save = true;
                }

                ui.separator();

                let mut static_wp = self.config.is_static_wallpaper;
                if ui.checkbox(&mut static_wp, "Static Wallpaper (GNOME)").changed() {
                    self.config.is_static_wallpaper = static_wp;
                    self.needs_save = true;
                }
                if self.config.is_static_wallpaper {
                    let mut blur = self.config.static_wallpaper_blur_radius as f32;
                    if ui.add(egui::Slider::new(&mut blur, 0.0..=20.0).text("Blur Radius")).changed() {
                        self.config.static_wallpaper_blur_radius = blur as u32;
                        self.needs_save = true;
                    }
                }

                ui.separator();

                {
                    let mut pause = self.config.is_pause_when_maximized;
                    if ui.checkbox(&mut pause, "Pause When Maximized").changed() {
                        self.config.is_pause_when_maximized = pause;
                        self.needs_save = true;
                    }
                }
                {
                    let mut mute_max = self.config.is_mute_when_maximized;
                    if ui.checkbox(&mut mute_max, "Mute When Maximized").changed() {
                        self.config.is_mute_when_maximized = mute_max;
                        self.needs_save = true;
                    }
                }

                ui.separator();

                ui.heading("Local Videos");
                if ui.button("Open Video Folder").clicked() {
                    let dir = crate::config::video_dir();
                    let _ = std::fs::create_dir_all(&dir);
                    open::that(&dir).ok();
                }
                if ui.button("Refresh").clicked() {
                    self.refresh_videos();
                }

                let paths = self.video_paths.clone();
                let mut selected = self.selected_video;
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for (i, path) in paths.iter().enumerate() {
                        let fname = std::path::Path::new(path)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.clone());
                        if ui.selectable_label(selected == Some(i), &fname).clicked() {
                            selected = Some(i);
                        }
                    }
                });
                self.selected_video = selected;

                if self.selected_video.is_some() && ui.button("Apply").clicked() {
                    if let Some(idx) = self.selected_video {
                        if let Some(path) = self.video_paths.get(idx) {
                            self.config.set_mode(Mode::Video, Some(path), Some("Default"));
                        }
                    }
                }

                ui.separator();

                ui.heading("Stream URL");
                ui.text_edit_singleline(&mut self.stream_url);
                if ui.button("Play Stream").clicked() {
                    if !self.stream_url.is_empty() {
                        self.config.set_mode(Mode::Stream, Some(&self.stream_url), Some("Default"));
                    }
                }

                ui.heading("Webpage");
                ui.text_edit_singleline(&mut self.webpage_url);
                if ui.button("Open Webpage").clicked() {
                    if !self.webpage_url.is_empty() {
                        self.config.set_mode(Mode::Webpage, Some(&self.webpage_url), Some("Default"));
                    }
                }

                ui.separator();

                if ui.button("I'm Feeling Lucky").clicked() {
                    use rand::seq::SliceRandom;
                    let mut rng = rand::thread_rng();
                    if let Some(video) = self.video_paths.choose(&mut rng) {
                        self.config.set_mode(Mode::Video, Some(video), Some("Default"));
                    }
                }

                if ui.button("Quit Nagi").clicked() {
                    std::process::exit(0);
                }
            });
        });

        if self.needs_save {
            self.config.save();
            self.needs_save = false;
        }
    }
}

impl NagiGui {
    fn show_welcome_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Welcome to Nagi")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Quickstart for adding local videos:");
                ui.label("  - Click the folder icon to open the Nagi folder");
                ui.label("  - Put your videos there");
                ui.label("  - Click the refresh button");
                if ui.button("OK").clicked() {
                    self.show_welcome = false;
                    self.config.is_first_time = false;
                    self.needs_save = true;
                }
            });
    }

    fn refresh_videos(&mut self) {
        let dir = crate::config::video_dir();
        let _ = std::fs::create_dir_all(&dir);
        let mut paths = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| {
                    ["mp4", "webm", "mkv", "avi", "mov", "gif"]
                        .contains(&ext.to_string_lossy().to_lowercase().as_str())
                }) {
                    paths.push(path.to_string_lossy().to_string());
                }
            }
        }
        paths.sort();
        self.video_paths = paths;
    }
}
