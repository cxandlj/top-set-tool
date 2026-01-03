#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod constant;
mod controller;
mod tool;
extern crate dotenv;
use crate::tool::{app_tool, update_tool};
use anyhow::Context;
use controller::{app_ctr, update_ctr, win_app_ctr};
use dotenv::dotenv;
use std::env;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_positioner::{Position, WindowExt};
use windows::Win32::Foundation::HWND;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    dotenv().ok();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            app_ctr::minimize_window,
            app_ctr::close_window,
            update_ctr::fetch_update,
            update_ctr::install_update,
            win_app_ctr::list_windows,
            win_app_ctr::toggle_topmost,
            win_app_ctr::cancel_all_topmost
        ])
        .setup(|app| {
            let result: anyhow::Result<()> = (|| {
                #[cfg(desktop)]
                app.manage(update_tool::PendingUpdate(Mutex::new(None)));

                let _ = app
                    .handle()
                    .plugin(tauri_plugin_updater::Builder::new().build());

                let main_window =
                    app_tool::get_window_by_label(app.handle(), constant::APP_MAIN_WINDOW_LABEL)?;
                //将窗口置于右下角 任务栏上方
                let rect = app_tool::get_work_area_for_window(HWND(main_window.hwnd()?.0 as isize));
                main_window.set_position(tauri::Position::Physical(
                    tauri::PhysicalPosition::new(
                        rect.right - constant::APP_WINDOW_SIZE.0 - 10,
                        rect.bottom - constant::APP_WINDOW_SIZE.1 - 2,
                    ),
                ))?;
                //配置文件先配置隐藏 等位置设置好后再显示 防止窗口闪烁
                main_window.show()?;
                //tray
                let show_menu = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
                let check_update_menu =
                    MenuItem::with_id(app, "check_update", "检查更新", true, None::<&str>)?;
                let restart_menu = MenuItem::with_id(app, "restart", "重启", true, None::<&str>)?;
                let quit_menu = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let menu = Menu::with_items(
                    app,
                    &[&show_menu, &check_update_menu, &restart_menu, &quit_menu],
                )?;

                let _tray = TrayIconBuilder::new()
                    .tooltip(constant::APP_DISPLAY_NAME)
                    .icon(
                        app.default_window_icon()
                            .context("获取程序图标失败")?
                            .clone(),
                    )
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| {
                        let result: anyhow::Result<()> = (|| {
                            match event.id.as_ref() {
                                "show" => {
                                    let window = app_tool::get_window_by_label(
                                        app,
                                        constant::APP_MAIN_WINDOW_LABEL,
                                    )?;
                                    if window.is_minimized()? {
                                        window.unminimize()?;
                                    }
                                    window.show()?;
                                    window.set_focus()?;
                                }
                                "check_update" => {
                                    update_tool::check_update(app);
                                }
                                "restart" => {
                                    app.restart();
                                }
                                "quit" => {
                                    app.exit(0);
                                }
                                _ => {}
                            }
                            Ok(())
                        })();
                        if let Err(e) = result {
                            app_tool::send_error_to_frontend(app, e);
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        let app = tray.app_handle();
                        match event {
                            TrayIconEvent::DoubleClick {
                                button: MouseButton::Left,
                                ..
                            } => {
                                if let Some(window) = app.get_webview_window("main") {
                                    if let Ok(hide) = window.is_visible() {
                                        if !hide {
                                            let _ = window.show();
                                        }
                                    }
                                    if let Ok(min) = window.is_minimized() {
                                        if min {
                                            let _ = window.unminimize();
                                        }
                                    }
                                    let _ = window.set_focus();
                                } else {
                                }
                            }
                            _ => {}
                        }
                        tauri_plugin_positioner::on_tray_event(app, &event);
                    })
                    .build(app)?;
                Ok(())
            })();

            if let Err(e) = result {
                panic!("程序启动失败：{}", e.to_string());
            }
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                //点击关闭按钮不退出程序 隐藏主窗体
                api.prevent_close();
                let _ = window.hide();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("程序启动失败");
}
