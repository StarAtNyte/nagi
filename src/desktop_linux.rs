use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub struct X11Desktop {
    pub conn: RustConnection,
    pub screen_num: usize,
    pub windows: Vec<u32>,
}

impl X11Desktop {
    pub fn new() -> Option<Self> {
        let (conn, screen_num) = RustConnection::connect(None).ok()?;
        Some(Self { conn, screen_num, windows: Vec::new() })
    }

    pub fn create_desktop_window(
        &mut self,
        x: i32, y: i32,
        w: u32, h: u32,
        _name: &str,
    ) -> Option<u32> {
        let screen = &self.conn.setup().roots[self.screen_num];
        let win = self.conn.generate_id().ok()?;

        let aux = CreateWindowAux::new()
            .background_pixel(screen.black_pixel)
            .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY);

        self.conn.create_window(
            screen.root_depth,
            win,
            screen.root,
            x as i16, y as i16,
            w as u16, h as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &aux,
        ).ok()?;

        // Set _NET_WM_WINDOW_TYPE = DESKTOP
        let desktop_atom = self.intern("_NET_WM_WINDOW_TYPE_DESKTOP");
        let wm_type_atom = self.intern("_NET_WM_WINDOW_TYPE");
        if desktop_atom != 0 && wm_type_atom != 0 {
            let data = desktop_atom.to_ne_bytes();
            let _ = self.conn.change_property(
                PropMode::REPLACE, win, wm_type_atom,
                AtomEnum::ATOM, 32, 1, &data,
            );
        }

        // Set _NET_WM_NAME
        let wm_name = self.intern("_NET_WM_NAME");
        let utf8 = self.intern("UTF8_STRING");
        if wm_name != 0 && utf8 != 0 {
            let _ = self.conn.change_property(
                PropMode::REPLACE, win, wm_name, utf8, 8,
                b"Nagi".len() as u32, b"Nagi",
            );
        }

        // Set _NET_WM_DESKTOP = all desktops (0xFFFFFFFF)
        let wm_desktop = self.intern("_NET_WM_DESKTOP");
        if wm_desktop != 0 {
            let all = 0xFFFFFFFFu32;
            let data = all.to_ne_bytes();
            let _ = self.conn.change_property(
                PropMode::REPLACE, win, wm_desktop,
                AtomEnum::CARDINAL, 32, 1, &data,
            );
        }

        let _ = self.conn.map_window(win);
        let _ = self.conn.flush();

        self.windows.push(win);
        Some(win)
    }

    fn intern(&self, name: &str) -> u32 {
        self.conn.intern_atom(false, name.as_bytes())
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| r.atom)
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn destroy_all(&mut self) {
        for &win in &self.windows {
            let _ = self.conn.destroy_window(win);
        }
        let _ = self.conn.flush();
        self.windows.clear();
    }
}

#[allow(dead_code)]
pub fn set_static_wallpaper_gnome(image_path: &str) {
    let uri = format!("file://{}", image_path);
    let _ = std::process::Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri", &uri])
        .output();
    let _ = std::process::Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri-dark", &uri])
        .output();
}

#[allow(dead_code)]
pub fn restore_original_wallpaper(original_uri: &Option<String>, original_uri_dark: &Option<String>) {
    if let Some(uri) = original_uri {
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.desktop.background", "picture-uri", uri])
            .output();
    }
    if let Some(uri) = original_uri_dark {
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.desktop.background", "picture-uri-dark", uri])
            .output();
    }
}

pub fn get_gnome_wallpaper() -> Option<(String, String)> {
    let light = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-uri"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().trim_matches('\'').to_string());

    let dark = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-uri-dark"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().trim_matches('\'').to_string());

    match (light, dark) {
        (Some(l), Some(d)) => Some((l, d)),
        _ => None,
    }
}

pub fn gnome_desktop_icon_workaround() {
    let extensions = [
        "ding@rastersoft.com",
        "desktopicons-neo@darkdemon",
        "gtk4-ding@smedius.gitlab.com",
        "zorin-desktop-icons@zorinos.com",
    ];
    for ext in &extensions {
        let _ = std::process::Command::new("gnome-extensions")
            .args(["disable", ext])
            .output();
        let _ = std::process::Command::new("gnome-extensions")
            .args(["enable", ext])
            .output();
    }
}
