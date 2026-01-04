#![allow(dead_code)]
use anyhow::Context;
use tauri::menu::Menu;
use tauri::{AppHandle, Manager};
use tauri::{Emitter, Monitor};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_positioner::{Position, WindowExt};
use tauri_utils::config::WindowConfig;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};

const APP_SETTING_FILE: &str = ".app_settings.json";
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum AppExitType {
    Exit,
    Minimize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub auto_start: bool,
    pub app_exit_type: AppExitType,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            auto_start: false,
            app_exit_type: AppExitType::Minimize,
        }
    }
}

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

pub fn show_window(app: &AppHandle, config: WindowConfig) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window(&config.label) {
        if window.is_minimized()? {
            window.unminimize()?
        }
        window.show()?;
        window.set_focus()?;
    } else {
        let empty_menu = Menu::new(app)?;
        let window = tauri::WebviewWindowBuilder::from_config(app, &config)?
            .menu(empty_menu)
            .build()?;
        if config.x.is_none() {
            window.move_window(Position::Center)?;
        }
        if !config.visible {
            window.show()?;
        }
    }
    Ok(())
}

pub fn close_window(app: &AppHandle, label: &str) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.close()?;
    }
    Ok(())
}

pub fn destroy_window(app: &AppHandle, label: &str) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.destroy()?;
    }
    Ok(())
}

pub fn load_setting(app: &AppHandle) -> anyhow::Result<AppSettings> {
    let app_dir = app.path().app_config_dir()?;
    let setting_path = app_dir.join(APP_SETTING_FILE);

    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }
    if !setting_path.exists() {
        let setting = AppSettings::default();
        save_setting(app, &setting)?;
        return Ok(setting);
    }

    let file = std::fs::File::open(setting_path)?;
    let reader = std::io::BufReader::new(file);
    let settings = serde_json::from_reader(reader)?;
    Ok(settings)
}

pub fn save_setting(app: &AppHandle, settings: &AppSettings) -> anyhow::Result<()> {
    let app_dir = app.path().app_config_dir()?;
    let setting_path = app_dir.join(APP_SETTING_FILE);

    let file = std::fs::File::create(setting_path)?;
    serde_json::to_writer_pretty(file, settings)?;

    Ok(())
}
