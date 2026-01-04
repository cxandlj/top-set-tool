use crate::tool::app_tool::{self, AppSettings};
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub fn close_window(window_label: String, app: tauri::AppHandle) -> Result<(), String> {
    let result: anyhow::Result<()> = (|| {
        app_tool::get_window_by_label(&app, &window_label)?.close()?;
        Ok(())
    })();
    if let Err(e) = result {
        return Err(e.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn destroy_window(window_label: String, app: tauri::AppHandle) -> Result<(), String> {
    let result: anyhow::Result<()> = (|| {
        app_tool::get_window_by_label(&app, &window_label)?.destroy()?;
        Ok(())
    })();
    if let Err(e) = result {
        return Err(e.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn minimize_window(window_label: String, app: tauri::AppHandle) -> Result<(), String> {
    let result: anyhow::Result<()> = (|| {
        app_tool::get_window_by_label(&app, &window_label)?.minimize()?;
        Ok(())
    })();
    if let Err(e) = result {
        return Err(e.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_app_setting(setting: State<'_, Mutex<AppSettings>>) -> Result<AppSettings, String> {
    let setting = setting.lock().unwrap();
    Ok(setting.clone())
}

#[tauri::command]
pub fn save_app_setting(
    old_setting: State<'_, Mutex<AppSettings>>,
    settings: AppSettings,
    app: tauri::AppHandle,
) -> Result<(), String> {
    println!("settings:{:?}", settings);
    let mut old_setting = old_setting.lock().unwrap();
    old_setting.auto_start = settings.auto_start;
    if old_setting.app_exit_type != settings.app_exit_type {
        let autostart_manager = app.autolaunch();
        if settings.auto_start {
            // 启用 autostart
            let _ = autostart_manager.enable();
        } else {
            let _ = autostart_manager.disable();
        }
        old_setting.app_exit_type = settings.app_exit_type.clone();
    }
    app_tool::save_setting(&app, &settings).map_err(|err| err.to_string())
}
