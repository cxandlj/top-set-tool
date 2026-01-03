use crate::constant;
use crate::tool::win_app_tool::{self, WindowInfo};

#[tauri::command]
pub fn list_windows(app: tauri::AppHandle) -> Vec<WindowInfo> {
    let hwnd_self =
        win_app_tool::get_win32_hwnd_by_tauri_hwnd(&app, constant::APP_MAIN_WINDOW_LABEL);
    win_app_tool::enum_windows(hwnd_self)
}

#[tauri::command]
pub fn toggle_topmost(hwnd: isize, enable: bool, app: tauri::AppHandle) {
    if enable {
        let hwnd_self =
            win_app_tool::get_win32_hwnd_by_tauri_hwnd(&app, constant::APP_MAIN_WINDOW_LABEL);
        let hwnd_self = match hwnd_self {
            Some(hwnd_self) => Some(hwnd_self.0),
            None => None,
        };
        win_app_tool::show_and_topmost(hwnd, hwnd_self);
    } else {
        win_app_tool::show_and_not_topmost(hwnd);
    }
}

#[tauri::command]
pub fn cancel_all_topmost(hwnd_list: Vec<isize>) {
    win_app_tool::cancel_all_topmost(hwnd_list);
}
