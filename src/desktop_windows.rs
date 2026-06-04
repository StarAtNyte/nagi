use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, HBRUSH, BLACK_BRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, EnumWindows, FindWindowExW, FindWindowW,
    RegisterClassExW, SendMessageTimeoutW, ShowWindow,
    CS_HREDRAW, CS_VREDRAW, SMTO_NORMAL, SW_SHOW, WNDCLASSEXW, WS_CHILD, WS_VISIBLE,
};
use windows::core::PCWSTR;

static WORKER_W: AtomicIsize = AtomicIsize::new(0);

unsafe extern "system" fn wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn find_worker_w() -> Option<HWND> {
    unsafe {
        let progman_cls = to_wide("Progman");
        let progman = FindWindowW(PCWSTR(progman_cls.as_ptr()), PCWSTR::null())
            .unwrap_or_default();
        if progman.0.is_null() {
            return None;
        }

        let _ = SendMessageTimeoutW(
            progman, 0x052C, WPARAM(0xD), LPARAM(0x1), SMTO_NORMAL, 1000, None,
        );

        WORKER_W.store(0, Ordering::SeqCst);

        unsafe extern "system" fn enum_proc(hwnd: HWND, _: LPARAM) -> BOOL {
            let shell_cls = to_wide("SHELLDLL_DefView");
            let worker_cls = to_wide("WorkerW");
            let shell = FindWindowExW(hwnd, HWND::default(), PCWSTR(shell_cls.as_ptr()), PCWSTR::null())
                .unwrap_or_default();
            if !shell.0.is_null() {
                let worker = FindWindowExW(HWND::default(), hwnd, PCWSTR(worker_cls.as_ptr()), PCWSTR::null())
                    .unwrap_or_default();
                WORKER_W.store(worker.0 as isize, Ordering::SeqCst);
            }
            BOOL(1)
        }

        let _ = EnumWindows(Some(enum_proc), LPARAM(0));

        let raw = WORKER_W.load(Ordering::SeqCst);
        if raw != 0 { Some(HWND(raw as *mut c_void)) } else { None }
    }
}

pub struct DesktopHandle {
    pub windows: Vec<(String, HWND)>,
}

impl DesktopHandle {
    pub fn create_wallpaper_windows(monitors: &[crate::config::MonitorInfo]) -> Self {
        let worker = match find_worker_w() {
            Some(w) => w,
            None => {
                log::error!("Failed to find WorkerW — wallpaper windows unavailable");
                return Self { windows: Vec::new() };
            }
        };

        let class_name = to_wide("NagiWallpaper");
        let hinstance = unsafe { GetModuleHandleW(PCWSTR::null()).unwrap_or_default() };

        unsafe {
            let brush = HBRUSH(GetStockObject(BLACK_BRUSH).0);
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinstance.into(),
                hbrBackground: brush,
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            RegisterClassExW(&wc);
        }

        let mut windows = Vec::new();

        for m in monitors {
            let result = unsafe {
                CreateWindowExW(
                    Default::default(),
                    PCWSTR(class_name.as_ptr()),
                    PCWSTR::null(),
                    WS_CHILD | WS_VISIBLE,
                    m.x, m.y,
                    m.width as i32, m.height as i32,
                    worker,
                    None,
                    hinstance,
                    None,
                )
            };

            match result {
                Ok(hwnd) if !hwnd.0.is_null() => {
                    unsafe { ShowWindow(hwnd, SW_SHOW); }
                    log::info!("Wallpaper window {} {}x{} @{},{}", m.name, m.width, m.height, m.x, m.y);
                    windows.push((m.name.clone(), hwnd));
                }
                _ => log::error!("Failed to create wallpaper window for {}", m.name),
            }
        }

        Self { windows }
    }

    pub fn as_wid_pairs(&self) -> Vec<(String, u64)> {
        self.windows.iter().map(|(name, hwnd)| (name.clone(), hwnd.0 as u64)).collect()
    }
}
