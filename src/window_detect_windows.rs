use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::config::MonitorInfo;

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

fn check_window_state(monitors: &[MonitorInfo]) -> WindowState {
    use windows::Win32::Foundation::{BOOL, HWND, RECT};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowRect, IsWindowVisible, IsIconic,
        GetDesktopWindow, GetWindowLongW, GWL_STYLE, GWL_EXSTYLE,
    };

    let desktop_w = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap_or(1920);
    let desktop_h = monitors.iter().map(|m| m.y + m.height as i32).max().unwrap_or(1080);

    use std::sync::atomic::{AtomicBool, Ordering};
    static MAXIMIZED: AtomicBool = AtomicBool::new(false);
    static FULLSCREEN: AtomicBool = AtomicBool::new(false);

    MAXIMIZED.store(false, Ordering::SeqCst);
    FULLSCREEN.store(false, Ordering::SeqCst);

    // Pass desktop size via thread-local to avoid global mutation race
    thread_local! {
        static DW: std::cell::Cell<i32> = std::cell::Cell::new(1920);
        static DH: std::cell::Cell<i32> = std::cell::Cell::new(1080);
    }
    DW.with(|c| c.set(desktop_w));
    DH.with(|c| c.set(desktop_h));

    extern "system" fn enum_proc(hwnd: HWND, _lparam: isize) -> BOOL {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                return BOOL(1);
            }
            if hwnd == GetDesktopWindow() {
                return BOOL(1);
            }
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);

            let has_caption = style & 0x00C00000 != 0;
            let has_thickframe = style & 0x00040000 != 0;
            let is_tool = ex_style & 0x00000080 != 0;

            if (!has_caption && !has_thickframe) || is_tool {
                return BOOL(1);
            }

            let mut wr = RECT::default();
            if GetWindowRect(hwnd, &mut wr).as_bool() {
                let ww = wr.right - wr.left;
                let wh = wr.bottom - wr.top;
                let dw = DW.with(|c| c.get());
                let dh = DH.with(|c| c.get());
                if ww >= dw && wh >= dh && wr.left <= 0 && wr.top <= 0 {
                    FULLSCREEN.store(true, Ordering::SeqCst);
                }
            }

            // WS_MAXIMIZE = 0x01000000
            if style & 0x01000000 != 0 {
                MAXIMIZED.store(true, Ordering::SeqCst);
            }
        }
        BOOL(1)
    }

    unsafe { EnumWindows(Some(enum_proc), isize::default()); }

    WindowState {
        is_any_maximized: MAXIMIZED.load(Ordering::SeqCst),
        is_any_fullscreen: FULLSCREEN.load(Ordering::SeqCst),
    }
}
