use crate::config::Config;
use std::collections::HashMap;

/// libmpv API is fully thread-safe per its documentation.
/// The `MpvHandler` wrapping `*mut mpv_handle` just lacks the marker.
unsafe impl Send for PlayerInstance {}
unsafe impl Sync for PlayerInstance {}

pub struct PlayerInstance {
    pub handler: mpv::MpvHandler,
    pub is_playing: bool,
    pub volume: u32,
    pub is_mute: bool,
    #[allow(dead_code)]
    pub wid: u64,
}

impl PlayerInstance {
    pub fn new_for_window(wid: u64) -> Option<Self> {
        let mut builder = mpv::MpvHandlerBuilder::new().ok()?;
        let _ = builder.set_option("wid", wid as i64);
        let _ = builder.set_option("input-cursor", "no");
        let _ = builder.set_option("input-vo-keyboard", "no");
        let _ = builder.set_option("no-osc", "yes");
        let _ = builder.set_option("no-window-dragging", "yes");
        let _ = builder.set_option("loop-file", "inf");
        let _ = builder.set_option("keepaspect-window", "no");
        let _ = builder.try_hardware_decoding();
        let mut handler = builder.build().ok()?;
        let _ = handler.set_property("pause", true);
        Some(PlayerInstance {
            handler,
            is_playing: false,
            volume: 50,
            is_mute: false,
            wid,
        })
    }

    #[allow(dead_code)]
    pub fn load_file(&mut self, path: &str) {
        let _ = self.handler.command(&["loadfile", path]);
    }

    #[allow(dead_code)]
    pub fn play(&mut self) {
        let _ = self.handler.set_property("pause", false);
        self.is_playing = true;
    }

    #[allow(dead_code)]
    pub fn pause(&mut self) {
        let _ = self.handler.set_property("pause", true);
        self.is_playing = false;
    }

    pub fn set_volume(&mut self, vol: u32) {
        self.volume = vol.clamp(0, 100);
        let _ = self.handler.set_property("volume", self.volume as i64);
    }

    pub fn set_mute(&mut self, mute: bool) {
        self.is_mute = mute;
        let _ = self.handler.set_property("mute", mute);
    }

    #[allow(dead_code)]
    pub fn stop(&mut self) {
        let _ = self.handler.command(&["stop"]);
        self.is_playing = false;
    }
}

pub struct PlayerManager {
    pub players: HashMap<String, PlayerInstance>,
    pub config: Config,
}

impl PlayerManager {
    pub fn new(config: Config) -> Self {
        Self { players: HashMap::new(), config }
    }

    pub fn init_players(
        &mut self,
        monitors: &[crate::config::MonitorInfo],
        xwins: &[(String, u64)],
    ) {
        for (m, wid) in monitors.iter().zip(xwins.iter()) {
            if !self.players.contains_key(&m.name) {
                if let Some(player) = PlayerInstance::new_for_window(wid.1) {
                    self.players.insert(m.name.clone(), player);
                }
            }
        }
        let source = self.config.data_source.get("Default").cloned();
        if let Some(ref src) = source {
            if !src.is_empty() {
                self.load_source(src);
            }
        }
    }

    pub fn load_source(&mut self, source: &str) {
        for (_name, player) in &mut self.players {
            let _ = player.handler.command(&["loadfile", source]);
            player.set_volume(self.config.audio_volume);
            player.set_mute(self.config.is_mute);
        }
        for (_name, player) in &mut self.players {
            let _ = player.handler.set_property("pause", false);
            player.is_playing = true;
        }
    }

    pub fn start_playback(&mut self) {
        for (_name, player) in &mut self.players {
            let _ = player.handler.set_property("pause", false);
            player.is_playing = true;
        }
    }

    pub fn pause_playback(&mut self) {
        for (_name, player) in &mut self.players {
            let _ = player.handler.set_property("pause", true);
            player.is_playing = false;
        }
    }

    #[allow(dead_code)]
    pub fn set_volume_all(&mut self, vol: u32) {
        for (_name, player) in &mut self.players {
            player.set_volume(vol);
        }
    }

    #[allow(dead_code)]
    pub fn set_mute_all(&mut self, mute: bool) {
        for (_name, player) in &mut self.players {
            player.set_mute(mute);
        }
    }

    #[allow(dead_code)]
    pub fn cleanup(&mut self) {
        for (_name, player) in &mut self.players {
            player.stop();
        }
        self.players.clear();
    }
}
