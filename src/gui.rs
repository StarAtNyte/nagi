use eframe::egui::{self, Color32, Frame, Margin, RichText, Rounding, Stroke, Vec2};
use crate::config::{Config, Mode};

// ── Palette ──────────────────────────────────────────────────────────────────
const BG:       Color32 = Color32::from_rgb(11,  11,  15);
const SURFACE:  Color32 = Color32::from_rgb(20,  20,  27);
const SURFACE2: Color32 = Color32::from_rgb(28,  28,  38);
const BORDER:   Color32 = Color32::from_rgb(42,  42,  58);
const ACCENT:   Color32 = Color32::from_rgb(232, 100, 58);
const ACCENT_DIM:Color32= Color32::from_rgb(120, 50,  28);
const TEXT:     Color32 = Color32::from_rgb(220, 220, 235);
const MUTED:    Color32 = Color32::from_rgb(110, 110, 140);
const SUCCESS:  Color32 = Color32::from_rgb(74,  222, 128);

#[derive(Default)]
pub struct NagiGui {
    pub config: Config,
    pub video_paths: Vec<String>,
    pub selected_video: Option<usize>,
    pub stream_url: String,
    pub webpage_url: String,
    pub needs_save: bool,
    pub show_welcome: bool,
    pub active_tab: Tab,
}

#[derive(Default, PartialEq)]
pub enum Tab { #[default] Videos, Stream, Settings }

impl NagiGui {
    pub fn new(config: Config) -> Self {
        let show_welcome = config.is_first_time;
        let mut s = Self { config, show_welcome, ..Default::default() };
        s.refresh_videos();
        s
    }

    fn apply_theme(ctx: &egui::Context) {
        let mut v = egui::Visuals::dark();
        v.window_fill         = BG;
        v.panel_fill          = BG;
        v.faint_bg_color      = SURFACE;
        v.extreme_bg_color    = Color32::from_rgb(8, 8, 11);
        v.code_bg_color       = SURFACE2;
        v.window_stroke       = Stroke::new(1.0, BORDER);
        v.widgets.inactive.bg_fill     = SURFACE2;
        v.widgets.inactive.fg_stroke   = Stroke::new(1.0, MUTED);
        v.widgets.inactive.bg_stroke   = Stroke::new(1.0, BORDER);
        v.widgets.inactive.rounding    = Rounding::same(4.0);
        v.widgets.hovered.bg_fill      = Color32::from_rgb(38, 38, 52);
        v.widgets.hovered.fg_stroke    = Stroke::new(1.0, TEXT);
        v.widgets.hovered.bg_stroke    = Stroke::new(1.0, ACCENT);
        v.widgets.hovered.rounding     = Rounding::same(4.0);
        v.widgets.active.bg_fill       = ACCENT_DIM;
        v.widgets.active.fg_stroke     = Stroke::new(1.5, ACCENT);
        v.widgets.active.bg_stroke     = Stroke::new(1.0, ACCENT);
        v.widgets.active.rounding      = Rounding::same(4.0);
        v.widgets.open.bg_fill         = SURFACE2;
        v.selection.bg_fill            = Color32::from_rgba_premultiplied(232, 100, 58, 60);
        v.selection.stroke             = Stroke::new(1.0, ACCENT);
        v.slider_trailing_fill         = true;
        v.override_text_color          = Some(TEXT);
        ctx.set_visuals(v);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing    = Vec2::new(8.0, 6.0);
        style.spacing.window_margin   = Margin::same(0.0);
        style.spacing.button_padding  = Vec2::new(12.0, 6.0);
        style.spacing.slider_width    = 160.0;
        ctx.set_style(style);
    }

    fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
        Frame::none()
            .fill(SURFACE)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(16.0))
            .outer_margin(Margin { left: 16.0, right: 16.0, top: 0.0, bottom: 12.0 })
            .show(ui, add);
    }

    fn section_label(ui: &mut egui::Ui, text: &str) {
        ui.add_space(16.0);
        ui.add_space(2.0);
        Frame::none()
            .outer_margin(Margin { left: 16.0, right: 16.0, top: 0.0, bottom: 6.0 })
            .show(ui, |ui| {
                ui.label(RichText::new(text.to_uppercase())
                    .size(10.0)
                    .color(MUTED)
                    .strong());
            });
    }

    fn accent_btn(ui: &mut egui::Ui, label: &str) -> egui::Response {
        let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).size(13.0))
            .fill(ACCENT)
            .stroke(Stroke::NONE)
            .rounding(Rounding::same(6.0));
        ui.add(btn)
    }

    fn ghost_btn(ui: &mut egui::Ui, label: &str) -> egui::Response {
        let btn = egui::Button::new(RichText::new(label).color(TEXT).size(13.0))
            .fill(SURFACE2)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(Rounding::same(6.0));
        ui.add(btn)
    }

    fn status_dot(ui: &mut egui::Ui, active: bool) {
        let color = if active { SUCCESS } else { MUTED };
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
    }
}

impl eframe::App for NagiGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::apply_theme(ctx);

        if self.show_welcome {
            self.show_welcome_dialog(ctx);
        }

        // ── Header ────────────────────────────────────────────────
        egui::TopBottomPanel::top("header")
            .frame(Frame::none()
                .fill(SURFACE)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin { left: 20.0, right: 20.0, top: 14.0, bottom: 14.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("NAGI").size(16.0).color(ACCENT).strong());
                    ui.label(RichText::new("v0.1.0").size(11.0).color(MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let playing = self.config.mode != Mode::Null;
                        Self::status_dot(ui, playing);
                        let status = match self.config.mode {
                            Mode::Null    => "idle",
                            Mode::Video   => "playing",
                            Mode::Stream  => "streaming",
                            Mode::Webpage => "webpage",
                        };
                        ui.label(RichText::new(status).size(11.0).color(MUTED));
                    });
                });
            });

        // ── Tab bar ───────────────────────────────────────────────
        egui::TopBottomPanel::top("tabs")
            .frame(Frame::none()
                .fill(BG)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin { left: 16.0, right: 16.0, top: 0.0, bottom: 0.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (Tab::Videos,   "Videos"),
                        (Tab::Stream,   "Stream"),
                        (Tab::Settings, "Settings"),
                    ] {
                        let active = self.active_tab == tab;
                        let color  = if active { ACCENT } else { MUTED };
                        let btn = egui::Button::new(RichText::new(label).color(color).size(13.0))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                            .rounding(Rounding::ZERO);
                        let resp = ui.add(btn);
                        if active {
                            let r = resp.rect;
                            ui.painter().line_segment(
                                [r.left_bottom(), r.right_bottom()],
                                Stroke::new(2.0, ACCENT),
                            );
                        }
                        if resp.clicked() { self.active_tab = tab; }
                        ui.add_space(8.0);
                    }
                });
            });

        // ── Bottom bar ────────────────────────────────────────────
        egui::TopBottomPanel::bottom("footer")
            .frame(Frame::none()
                .fill(SURFACE)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin { left: 16.0, right: 16.0, top: 10.0, bottom: 10.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if Self::ghost_btn(ui, "🎲  Lucky").clicked() {
                        use rand::seq::SliceRandom;
                        let mut rng = rand::thread_rng();
                        if let Some(v) = self.video_paths.choose(&mut rng) {
                            self.config.set_mode(Mode::Video, Some(v), Some("Default"));
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let quit = egui::Button::new(RichText::new("Quit").color(MUTED).size(13.0))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE);
                        if ui.add(quit).clicked() { std::process::exit(0); }
                    });
                });
            });

        // ── Central panel ─────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(Frame::none().fill(BG))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);
                    match self.active_tab {
                        Tab::Videos   => self.show_videos(ui),
                        Tab::Stream   => self.show_stream(ui),
                        Tab::Settings => self.show_settings(ui),
                    }
                    ui.add_space(8.0);
                });
            });

        if self.needs_save {
            self.config.save();
            self.needs_save = false;
        }
    }
}

impl NagiGui {
    fn show_videos(&mut self, ui: &mut egui::Ui) {
        // ── Now playing ───────────────────────────────────────────
        if self.config.mode == Mode::Video {
            let src = self.config.data_source.get("Default").cloned().unwrap_or_default();
            if !src.is_empty() {
                let fname = std::path::Path::new(&src)
                    .file_name().map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(src.clone());
                Self::section_label(ui, "Now Playing");
                Self::card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("▶").color(ACCENT).size(14.0));
                        ui.label(RichText::new(&fname).color(TEXT).size(13.0));
                    });
                });
            }
        }

        // ── Volume ────────────────────────────────────────────────
        Self::section_label(ui, "Audio");
        Self::card(ui, |ui| {
            ui.horizontal(|ui| {
                let icon = if self.config.is_mute { "🔇" } else { "🔊" };
                ui.label(RichText::new(icon).size(14.0));
                let mut vol = self.config.audio_volume as f32;
                let slider = egui::Slider::new(&mut vol, 0.0..=100.0)
                    .show_value(false)
                    .trailing_fill(true);
                if ui.add(slider).changed() {
                    self.config.audio_volume = vol as u32;
                    self.needs_save = true;
                }
                ui.label(RichText::new(format!("{}%", self.config.audio_volume)).color(MUTED).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mute_label = if self.config.is_mute {
                        RichText::new("Unmute").color(ACCENT)
                    } else {
                        RichText::new("Mute").color(MUTED)
                    };
                    if ui.add(egui::Button::new(mute_label.size(12.0))
                        .fill(Color32::TRANSPARENT).stroke(Stroke::NONE)).clicked()
                    {
                        self.config.is_mute = !self.config.is_mute;
                        self.needs_save = true;
                    }
                });
            });
        });

        // ── Library ───────────────────────────────────────────────
        Self::section_label(ui, "Library");
        Self::card(ui, |ui| {
            ui.horizontal(|ui| {
                if Self::ghost_btn(ui, "Open Folder").clicked() {
                    let dir = crate::config::video_dir();
                    let _ = std::fs::create_dir_all(&dir);
                    open::that(&dir).ok();
                }
                if Self::ghost_btn(ui, "Refresh").clicked() {
                    self.refresh_videos();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} files", self.video_paths.len()))
                        .color(MUTED).size(12.0));
                });
            });

            if self.video_paths.is_empty() {
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No videos found").color(MUTED).size(13.0));
                    ui.label(RichText::new("Drop .mp4 / .webm / .mkv files in the folder above")
                        .color(Color32::from_rgb(70, 70, 90)).size(11.0));
                });
                ui.add_space(4.0);
            } else {
                ui.add_space(8.0);
                let paths = self.video_paths.clone();
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .id_source("video_list")
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for (i, path) in paths.iter().enumerate() {
                            let fname = std::path::Path::new(path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| path.clone());

                            let is_selected = self.selected_video == Some(i);
                            let is_playing = self.config.mode == Mode::Video
                                && self.config.data_source.get("Default")
                                    .map(|s| s == path).unwrap_or(false);

                            let bg = if is_selected { SURFACE2 } else { Color32::TRANSPARENT };
                            let border = if is_selected {
                                Stroke::new(1.0, ACCENT)
                            } else {
                                Stroke::new(1.0, Color32::TRANSPARENT)
                            };

                            Frame::none()
                                .fill(bg)
                                .stroke(border)
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin { left: 10.0, right: 8.0, top: 6.0, bottom: 6.0 })
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        if is_playing {
                                            ui.label(RichText::new("▶").color(ACCENT).size(11.0));
                                        } else {
                                            ui.add_space(16.0);
                                        }
                                        let label_color = if is_selected { TEXT } else { MUTED };
                                        if ui.add(egui::Label::new(
                                            RichText::new(&fname).color(label_color).size(13.0)
                                        ).sense(egui::Sense::click())).clicked() {
                                            self.selected_video = Some(i);
                                        }
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if is_selected {
                                                if Self::accent_btn(ui, "Play").clicked() {
                                                    self.config.set_mode(Mode::Video, Some(path), Some("Default"));
                                                    self.selected_video = None;
                                                }
                                            }
                                        });
                                    });
                                });
                        }
                    });
            }
        });
    }

    fn show_stream(&mut self, ui: &mut egui::Ui) {
        Self::section_label(ui, "Stream URL");
        Self::card(ui, |ui| {
            ui.label(RichText::new("YouTube, Twitch, Vimeo, and more via yt-dlp")
                .color(MUTED).size(11.0));
            ui.add_space(8.0);
            let url_input = egui::TextEdit::singleline(&mut self.stream_url)
                .hint_text("https://youtube.com/watch?v=...")
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace);
            ui.add(url_input);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if Self::accent_btn(ui, "▶  Play Stream").clicked() && !self.stream_url.is_empty() {
                    self.config.set_mode(Mode::Stream, Some(&self.stream_url.clone()), Some("Default"));
                }
                if self.config.mode == Mode::Stream {
                    if Self::ghost_btn(ui, "Stop").clicked() {
                        self.config.set_mode(Mode::Null, None, None);
                    }
                }
            });
        });

        Self::section_label(ui, "Webpage");
        Self::card(ui, |ui| {
            ui.label(RichText::new("Load a webpage as wallpaper")
                .color(MUTED).size(11.0));
            ui.add_space(8.0);
            let url_input = egui::TextEdit::singleline(&mut self.webpage_url)
                .hint_text("https://...")
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace);
            ui.add(url_input);
            ui.add_space(8.0);
            if Self::accent_btn(ui, "Open Webpage").clicked() && !self.webpage_url.is_empty() {
                self.config.set_mode(Mode::Webpage, Some(&self.webpage_url.clone()), Some("Default"));
            }
        });

        if self.config.mode != Mode::Null {
            Self::section_label(ui, "Playback");
            Self::card(ui, |ui| {
                ui.horizontal(|ui| {
                    Self::status_dot(ui, true);
                    ui.label(RichText::new(format!("{}", self.config.mode)).color(TEXT).size(13.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Self::ghost_btn(ui, "Stop All").clicked() {
                            self.config.set_mode(Mode::Null, None, None);
                        }
                    });
                });
            });
        }
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        Self::section_label(ui, "Behavior");
        Self::card(ui, |ui| {
            self.toggle_row(ui, "Pause when maximized", &mut self.config.is_pause_when_maximized.clone(), |v| v);
            let mut pause = self.config.is_pause_when_maximized;
            if Self::toggle_setting(ui, "Pause when maximized",
                "Pause video when any window is fullscreen or maximized", pause)
            {
                pause = !pause;
                self.config.is_pause_when_maximized = pause;
                self.needs_save = true;
            }

            ui.add_space(4.0);
            ui.add(egui::Separator::default().spacing(8.0));
            ui.add_space(4.0);

            let mut mute_max = self.config.is_mute_when_maximized;
            if Self::toggle_setting(ui, "Mute when maximized",
                "Mute audio when any window is fullscreen or maximized", mute_max)
            {
                mute_max = !mute_max;
                self.config.is_mute_when_maximized = mute_max;
                self.needs_save = true;
            }
        });

        Self::section_label(ui, "Wallpaper");
        Self::card(ui, |ui| {
            let mut static_wp = self.config.is_static_wallpaper;
            if Self::toggle_setting(ui, "Static wallpaper mode",
                "Set a static screenshot as GNOME wallpaper behind the video", static_wp)
            {
                static_wp = !static_wp;
                self.config.is_static_wallpaper = static_wp;
                self.needs_save = true;
            }

            if self.config.is_static_wallpaper {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Blur radius").color(MUTED).size(12.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{}", self.config.static_wallpaper_blur_radius))
                            .color(TEXT).size(12.0).monospace());
                    });
                });
                ui.add_space(4.0);
                let mut blur = self.config.static_wallpaper_blur_radius as f32;
                if ui.add(egui::Slider::new(&mut blur, 0.0..=20.0)
                    .show_value(false).trailing_fill(true)).changed()
                {
                    self.config.static_wallpaper_blur_radius = blur as u32;
                    self.needs_save = true;
                }
            }
        });

        Self::section_label(ui, "Timing");
        Self::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Fade duration").color(MUTED).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:.1}s", self.config.fade_duration_sec))
                        .color(TEXT).size(12.0).monospace());
                });
            });
            ui.add_space(4.0);
            let mut fade = self.config.fade_duration_sec as f32;
            if ui.add(egui::Slider::new(&mut fade, 0.0..=5.0)
                .show_value(false).trailing_fill(true)).changed()
            {
                self.config.fade_duration_sec = fade as f64;
                self.needs_save = true;
            }
        });

        Self::section_label(ui, "Danger Zone");
        Self::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Reset all settings to defaults").color(MUTED).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn = egui::Button::new(RichText::new("Reset").color(Color32::from_rgb(220, 80, 80)).size(12.0))
                        .fill(SURFACE2)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(80, 30, 30)))
                        .rounding(Rounding::same(6.0));
                    if ui.add(btn).clicked() {
                        let monitors = crate::monitor::detect_monitors();
                        self.config = Config::default_config(&monitors);
                        self.needs_save = true;
                    }
                });
            });
        });
    }

    fn toggle_row(&mut self, _ui: &mut egui::Ui, _label: &str, _val: &mut bool, _f: impl Fn(bool) -> bool) {}

    fn toggle_setting(ui: &mut egui::Ui, label: &str, desc: &str, value: bool) -> bool {
        let mut clicked = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(label).color(TEXT).size(13.0));
                ui.label(RichText::new(desc).color(MUTED).size(11.0));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (fill, knob_x) = if value {
                    (ACCENT, 28.0_f32)
                } else {
                    (Color32::from_rgb(45, 45, 60), 12.0_f32)
                };
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(42.0, 22.0), egui::Sense::click());
                let painter = ui.painter();
                painter.rect_filled(rect, Rounding::same(11.0), fill);
                painter.circle_filled(
                    egui::pos2(rect.left() + knob_x, rect.center().y),
                    9.0,
                    Color32::WHITE,
                );
                if resp.clicked() { clicked = true; }
            });
        });
        clicked
    }

    fn show_welcome_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Welcome to Nagi")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .frame(Frame::none()
                .fill(SURFACE)
                .stroke(Stroke::new(1.0, BORDER))
                .rounding(Rounding::same(12.0))
                .inner_margin(Margin::same(24.0)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("NAGI").size(28.0).color(ACCENT).strong());
                    ui.label(RichText::new("Video Wallpaper Daemon").size(13.0).color(MUTED));
                    ui.add_space(16.0);
                    ui.label(RichText::new("1. Open the Videos tab").color(TEXT).size(13.0));
                    ui.label(RichText::new("2. Click 'Open Folder' and drop your videos in").color(MUTED).size(12.0));
                    ui.label(RichText::new("3. Hit Refresh, select a video, and press Play").color(MUTED).size(12.0));
                    ui.add_space(20.0);
                    if Self::accent_btn(ui, "Get Started").clicked() {
                        self.show_welcome = false;
                        self.config.is_first_time = false;
                        self.needs_save = true;
                    }
                });
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
