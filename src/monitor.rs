use crate::config::MonitorInfo;

#[cfg(target_os = "linux")]
pub fn detect_monitors() -> Vec<MonitorInfo> {
    use x11rb::connection::Connection;
    use x11rb::protocol::randr::ConnectionExt as RandrExt;

    let (conn, screen_num) = match x11rb::rust_connection::RustConnection::connect(None) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to X11: {e}");
            return vec![MonitorInfo {
                name: "Default".into(),
                x: 0, y: 0, width: 1920, height: 1080, is_primary: true,
            }];
        }
    };

    let root = conn.setup().roots[screen_num].root;

    let resources = match conn.randr_get_screen_resources_current(root)
        .ok()
        .and_then(|c| c.reply().ok())
    {
        Some(r) => r,
        None => {
            log::warn!("RandR unavailable, falling back to screen root dimensions");
            let screen = &conn.setup().roots[screen_num];
            return vec![MonitorInfo {
                name: "Default".into(),
                x: 0, y: 0,
                width: screen.width_in_pixels as u32,
                height: screen.height_in_pixels as u32,
                is_primary: true,
            }];
        }
    };

    let timestamp = resources.config_timestamp;
    let mut monitors = Vec::new();
    let mut first = true;

    for &crtc in &resources.crtcs {
        let info = match conn.randr_get_crtc_info(crtc, timestamp)
            .ok()
            .and_then(|c| c.reply().ok())
        {
            Some(i) => i,
            None => continue,
        };

        if info.width == 0 || info.height == 0 || info.outputs.is_empty() {
            continue;
        }

        let name = info.outputs.first().and_then(|&out| {
            conn.randr_get_output_info(out, timestamp)
                .ok()
                .and_then(|c| c.reply().ok())
                .map(|o| String::from_utf8_lossy(&o.name).to_string())
        }).unwrap_or_else(|| format!("CRTC-{}", crtc));

        monitors.push(MonitorInfo {
            name,
            x: info.x as i32,
            y: info.y as i32,
            width: info.width as u32,
            height: info.height as u32,
            is_primary: first,
        });
        first = false;
    }

    if monitors.is_empty() {
        let screen = &conn.setup().roots[screen_num];
        monitors.push(MonitorInfo {
            name: "Default".into(),
            x: 0, y: 0,
            width: screen.width_in_pixels as u32,
            height: screen.height_in_pixels as u32,
            is_primary: true,
        });
    }

    monitors
}

#[cfg(target_os = "windows")]
pub fn detect_monitors() -> Vec<MonitorInfo> {
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, HDC, HMONITOR,
    };
    use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
    use std::sync::Mutex;

    static MONITORS: Mutex<Vec<MonitorInfo>> = Mutex::new(Vec::new());

    {
        if let Ok(mut m) = MONITORS.lock() {
            m.clear();
        }
    }

    extern "system" fn enum_proc(
        hmon: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        _lparam: LPARAM,
    ) -> BOOL {
        unsafe {
            let mut info: MONITORINFOEXW = std::mem::zeroed();
            info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
            if GetMonitorInfoW(hmon, &mut info.monitorInfo as *mut _ as *mut _).as_bool() {
                let rc = info.monitorInfo.rcMonitor;
                let name = String::from_utf16_lossy(&info.szDevice)
                    .trim_matches(char::from(0))
                    .to_string();
                let is_primary = info.monitorInfo.dwFlags & 1 != 0;
                if let Ok(mut m) = MONITORS.lock() {
                    m.push(MonitorInfo {
                        name,
                        x: rc.left,
                        y: rc.top,
                        width: (rc.right - rc.left) as u32,
                        height: (rc.bottom - rc.top) as u32,
                        is_primary,
                    });
                }
            }
        }
        BOOL(1)
    }

    unsafe {
        EnumDisplayMonitors(HDC::default(), None, Some(enum_proc), LPARAM(0));
    }

    let mut result = MONITORS.lock().map(|m| m.clone()).unwrap_or_default();
    if result.is_empty() {
        result.push(MonitorInfo {
            name: "Default".into(),
            x: 0, y: 0, width: 1920, height: 1080, is_primary: true,
        });
    }
    result
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn detect_monitors() -> Vec<MonitorInfo> {
    vec![MonitorInfo {
        name: "Default".into(),
        x: 0, y: 0, width: 1920, height: 1080, is_primary: true,
    }]
}
