use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowState {
    pub is_any_maximized: bool,
    pub is_any_fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self { is_any_maximized: false, is_any_fullscreen: false }
    }
}

pub fn start_window_monitor(tx: mpsc::Sender<WindowState>) {
    thread::spawn(move || {
        let (conn, _screen_num) = match x11rb::rust_connection::RustConnection::connect(None) {
            Ok(c) => c,
            Err(_) => {
                log::warn!("X11 not available for window monitoring");
                return;
            }
        };
        let screen = &conn.setup().roots[_screen_num];
        let root = screen.root;

        let atom_names = [
            "_NET_CLIENT_LIST",
            "_NET_WM_STATE",
            "_NET_WM_STATE_FULLSCREEN",
            "_NET_WM_STATE_MAXIMIZED_VERT",
            "_NET_WM_STATE_MAXIMIZED_HORZ",
        ];

        let atom_values: Vec<u32> = atom_names.iter().map(|&name| {
            conn.intern_atom(false, name.as_bytes())
                .ok()
                .and_then(|c| c.reply().ok())
                .map(|r| r.atom)
                .unwrap_or(0)
        }).collect();

        let [net_client_list, net_wm_state, net_wm_state_fullscreen,
             net_wm_state_maximized_vert, net_wm_state_maximized_horz] = atom_values[..5] else {
            return;
        };

        let mut prev_state = None;

        loop {
            let state = check_window_state(
                &conn, root,
                net_client_list, net_wm_state,
                net_wm_state_fullscreen,
                net_wm_state_maximized_vert,
                net_wm_state_maximized_horz,
            );

            if prev_state != Some(state) {
                prev_state = Some(state);
                let _ = tx.send(state);
            }

            thread::sleep(Duration::from_millis(500));
        }
    });
}

fn check_window_state(
    conn: &x11rb::rust_connection::RustConnection,
    root: u32,
    net_client_list: u32,
    net_wm_state: u32,
    net_wm_state_fullscreen: u32,
    net_wm_state_maximized_vert: u32,
    net_wm_state_maximized_horz: u32,
) -> WindowState {
    let client_list = match conn.get_property(
        false, root, net_client_list,
        x11rb::protocol::xproto::AtomEnum::WINDOW, 0, 1024,
    ) {
        Ok(r) => match r.reply() {
            Ok(reply) => reply,
            Err(_) => return WindowState::default(),
        },
        Err(_) => return WindowState::default(),
    };

    if client_list.value_len == 0 {
        return WindowState::default();
    }

    let windows: Vec<u32> = match client_list.value32() {
        Some(iter) => iter.collect(),
        None => return WindowState::default(),
    };

    let mut is_any_maximized = false;
    let mut is_any_fullscreen = false;

    for &win_id in &windows {
        let state_prop = match conn.get_property(
            false, win_id, net_wm_state,
            x11rb::protocol::xproto::AtomEnum::ATOM, 0, 1024,
        ) {
            Ok(r) => match r.reply() {
                Ok(reply) => reply,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if state_prop.value_len == 0 {
            continue;
        }

        let atoms: Vec<u32> = match state_prop.value32() {
            Some(iter) => iter.collect(),
            None => continue,
        };

        if atoms.contains(&net_wm_state_fullscreen) {
            is_any_fullscreen = true;
        }
        if atoms.contains(&net_wm_state_maximized_vert) &&
           atoms.contains(&net_wm_state_maximized_horz) {
            is_any_maximized = true;
        }
    }

    WindowState { is_any_maximized, is_any_fullscreen }
}
