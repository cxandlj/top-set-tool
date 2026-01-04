#![allow(dead_code)]
use anyhow::Context;
use tauri::{AppHandle, Manager};
use tauri::{Emitter, Monitor};
use tauri_plugin_autostart::ManagerExt;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};

pub fn get_screen_info(app: &AppHandle) -> Option<Monitor> {
    let window = app.get_webview_window("main")?;
    // 获取当前窗口所在的显示器
    match window.current_monitor() {
        Ok(Some(monitor)) => Some(monitor),
        _ => None,
    }
}

pub fn set_auto_start(app: &AppHandle, enable: bool) {
    // 获取自动启动管理器
    let autostart_manager = app.autolaunch();
    if enable {
        // 启用 autostart
        let _ = autostart_manager.enable();
    } else {
        let _ = autostart_manager.disable();
    }
}

pub fn get_work_area_for_window(hwnd: HWND) -> RECT {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        let _ = GetMonitorInfoW(monitor, &mut info);
        info.rcWork
    }
}

pub fn send_error_to_frontend(app: &AppHandle, err: anyhow::Error) {
    let _ = app.emit("sys_error", err.to_string());
}

pub fn get_window_by_label(app: &AppHandle, label: &str) -> anyhow::Result<tauri::WebviewWindow> {
    let window = app.get_webview_window(label).context("获取窗口失败")?;
    Ok(window)
}
