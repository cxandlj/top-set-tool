use crate::tool::app_tool;

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
