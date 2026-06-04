use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread;
use std::time::Duration;
use crate::config::MonitorInfo;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongW, GetWindowRect, IsIconic, IsWindowVisible,
    GetDesktopWindow, GWL_EXSTYLE, GWL_STYLE,
};

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

pub fn start_window_monitor(tx: mpsc::Sender<WindowState>, monitors: Vec<MonitorInfo>) {
    thread::spawn(move || {
        let mut prev_state = None;
        loop {
            let state = check_window_state(&monitors);
            if prev_state != Some(state) {
                prev_state = Some(state);
                let _ = tx.send(state);
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
}

static MAXIMIZED: AtomicBool = AtomicBool::new(false);
static FULLSCREEN: AtomicBool = AtomicBool::new(false);
static DESKTOP_W: AtomicI32 = AtomicI32::new(1920);
static DESKTOP_H: AtomicI32 = AtomicI32::new(1080);

unsafe extern "system" fn enum_proc(hwnd: HWND, _: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
        return BOOL(1);
    }
    if hwnd == GetDesktopWindow() {
        return BOOL(1);
    }

    let style = GetWindowLongW(hwnd, GWL_STYLE);
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
    let has_caption = style & 0x00C00000 != 0;
    let has_thick = style & 0x00040000 != 0;
    let is_tool = ex_style & 0x00000080 != 0;

    if (!has_caption && !has_thick) || is_tool {
        return BOOL(1);
    }

    let mut wr = RECT::default();
    if GetWindowRect(hwnd, &mut wr).is_ok() {
        let ww = wr.right - wr.left;
        let wh = wr.bottom - wr.top;
        let dw = DESKTOP_W.load(Ordering::Relaxed);
        let dh = DESKTOP_H.load(Ordering::Relaxed);
        if ww >= dw && wh >= dh && wr.left <= 0 && wr.top <= 0 {
            FULLSCREEN.store(true, Ordering::Relaxed);
        }
    }

    // WS_MAXIMIZE = 0x01000000
    if style & 0x01000000 != 0 {
        MAXIMIZED.store(true, Ordering::Relaxed);
    }

    BOOL(1)
}

fn check_window_state(monitors: &[MonitorInfo]) -> WindowState {
    let dw = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap_or(1920);
    let dh = monitors.iter().map(|m| m.y + m.height as i32).max().unwrap_or(1080);

    DESKTOP_W.store(dw, Ordering::Relaxed);
    DESKTOP_H.store(dh, Ordering::Relaxed);
    MAXIMIZED.store(false, Ordering::Relaxed);
    FULLSCREEN.store(false, Ordering::Relaxed);

    unsafe { let _ = EnumWindows(Some(enum_proc), LPARAM(0)); }

    WindowState {
        is_any_maximized: MAXIMIZED.load(Ordering::Relaxed),
        is_any_fullscreen: FULLSCREEN.load(Ordering::Relaxed),
    }
}
