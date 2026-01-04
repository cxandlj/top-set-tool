#![allow(dead_code)]
use anyhow::Context;
use base64::{engine::general_purpose, Engine};
use serde::Serialize;
#[cfg(target_os = "windows")]
use std::mem::{size_of, zeroed};
use std::path::PathBuf;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr::null_mut};
use tauri::Manager;
use windows::core::{Interface, PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, BOOL, HWND, LPARAM, LRESULT, MAX_PATH, SIZE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Shell::{
    IShellItem, IShellItemImageFactory, IShellLinkW, SHCreateItemFromParsingName, ShellLink,
    SIIGBF_BIGGERSIZEOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongPtrW, GetWindowLongW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible, SendMessageTimeoutW, SetForegroundWindow, SetWindowPos, ShowWindow,
    GWL_EXSTYLE, GWL_STYLE, HWND_NOTOPMOST, HWND_TOPMOST, SMTO_ABORTIFHUNG, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW, SW_RESTORE, SW_SHOW, WM_GETTEXT, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_VISIBLE,
};

pub struct IconImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>, // RGBA8888
}
#[derive(Serialize)]
pub struct AppMeta {
    pub name: String,
    pub path: String,
    pub icon_png: String,
    pub display_name: String,
    pub is_top_most: bool,
}

pub fn load_exe_icon(path: &str, size: i32) -> anyhow::Result<IconImage> {
    unsafe {
        //初始化 COM（多次调用是安全的）
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        //路径转宽字符串
        let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();

        //创建 IShellItem
        let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None)?;

        //获取 ImageFactory
        let factory: IShellItemImageFactory = item.cast()?;

        //请求指定尺寸的 HBITMAP
        let hbitmap = factory.GetImage(
            SIZE { cx: size, cy: size },
            SIIGBF_BIGGERSIZEOK,
            // &mut hbitmap,
        )?;

        //HBITMAP → RGBA
        let mut bmp: BITMAP = zeroed();
        GetObjectW(
            hbitmap,
            size_of::<BITMAP>() as i32,
            Some(&mut bmp as *mut _ as _),
        );

        let width = bmp.bmWidth as u32;
        let height = bmp.bmHeight as u32;

        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = width as i32;
        bmi.bmiHeader.biHeight = -(height as i32); // top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0;

        let mut buf = vec![0u8; (width * height * 4) as usize];

        let hdc = GetDC(None);
        GetDIBits(
            hdc,
            hbitmap,
            0,
            height,
            Some(buf.as_mut_ptr() as _),
            &mut bmi,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, hdc);

        // BGRA → RGBA
        for px in buf.chunks_exact_mut(4) {
            px.swap(0, 2);
        }

        let _ = DeleteObject(hbitmap);

        Ok(IconImage {
            width,
            height,
            rgba: buf,
        })
    }
}

pub fn get_icon_base64(path: &str, size: i32) -> anyhow::Result<String> {
    let icon = load_exe_icon(path, size)?;
    let png_bytes = icon.rgba; // 你也可以转 png
    let b64 = general_purpose::STANDARD.encode(png_bytes);
    Ok(b64)
}
pub fn get_png_base64(path: &str, size: i32) -> anyhow::Result<String> {
    let icon = load_exe_icon(path, size)?;
    let img =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(icon.width, icon.height, icon.rgba)
            .ok_or_else(|| anyhow::anyhow!("image error"))?;

    let mut png = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)?;

    Ok(base64::engine::general_purpose::STANDARD.encode(png))
}
/// 表示解析到的快捷方式信息
#[derive(Debug, Clone)]
pub struct LnkInfo {
    pub name: String,
    pub target_path: Option<PathBuf>,
}

pub fn get_app_info_by_path(path: &str) -> anyhow::Result<AppMeta> {
    let ext = path.split(".").last().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "exe" => get_app_info(path),
        "lnk" => {
            let info = resolve_lnk(&PathBuf::from(path))?;
            if let Some(target) = info.clone().target_path {
                if let Some(path) = target.to_str() {
                    if path.ends_with(".exe") {
                        return get_app_info(path);
                    }
                }
            }
            return Err(anyhow::anyhow!("lnk解析失败"));
        }
        _ => {
            return Err(anyhow::anyhow!("只支持exe或lnk文件"));
        }
    }
}

pub unsafe fn get_app_info_by_hwnd(hwnd: isize) -> anyhow::Result<AppMeta> {
    // PID
    let mut pid = 0;
    GetWindowThreadProcessId(HWND(hwnd), Some(&mut pid));

    if let Some(path) = get_process_exe_path(pid) {
        let mut app_info = get_app_info_by_path(&path)?;
        let is_top_most = is_window_topmost(HWND(hwnd));
        app_info.is_top_most = is_top_most;
        return Ok(app_info);
    } else {
        return Err(anyhow::anyhow!("根据pid查询程序路径失败"));
    }
}

pub fn get_app_info(path: &str) -> anyhow::Result<AppMeta> {
    let exe_name = path.rsplit('\\').next().unwrap_or("").replace(".exe", "");
    let names = get_app_names(path);
    let display_name = names
        .file_description
        .or(names.product_name)
        .unwrap_or(exe_name.clone());
    let icon_base64 = get_png_base64(path, 256)?;
    Ok(AppMeta {
        name: exe_name,
        path: path.to_string(),
        icon_png: icon_base64,
        display_name,
        is_top_most: false,
    })
}

/// 解析单个 .lnk 文件 → exe 路径
pub fn resolve_lnk(lnk_path: &PathBuf) -> anyhow::Result<LnkInfo> {
    unsafe {
        // 初始化 COM（STA）
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let mut shell_link: Option<IShellLinkW> = None;
        CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .ok()
            .map(|sl| shell_link = Some(sl));

        let shell_link = shell_link.context("link信息获取失败")?;

        let persist_file: IPersistFile = shell_link.cast()?;

        let wide_path: Vec<u16> = OsStr::new(lnk_path).encode_wide().chain(Some(0)).collect();

        persist_file.Load(PCWSTR(wide_path.as_ptr()), STGM_READ)?;

        let mut buffer = [0u16; MAX_PATH as usize];
        shell_link.GetPath(&mut buffer, null_mut(), 0)?;
        let path = String::from_utf16_lossy(&buffer)
            .trim_end_matches('\0')
            .to_string();

        // 获取快捷方式名称
        let name = lnk_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        CoUninitialize();

        Ok(LnkInfo {
            name,
            target_path: if path.is_empty() {
                None
            } else {
                Some(PathBuf::from(path))
            },
        })
    }
}

#[derive(serde::Serialize, Debug, Default)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub name: String,
    pub display_name: String,
    pub icon_png: String,
    pub is_top_most: bool,
}

pub unsafe fn get_window_title_safe(hwnd: HWND) -> Option<String> {
    let mut buffer = [0u16; 256];

    let mut copied = 0usize;

    let result = SendMessageTimeoutW(
        hwnd,
        WM_GETTEXT,
        WPARAM(buffer.len()),
        LPARAM(buffer.as_mut_ptr() as isize),
        SMTO_ABORTIFHUNG,
        100, // 超时 100ms
        Some(&mut copied as *mut _),
    );

    if result == LRESULT(0) || copied == 0 {
        return None;
    }

    Some(String::from_utf16_lossy(
        &buffer[..copied.min(buffer.len())],
    ))
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }
    // 排除工具窗口 / 子窗口
    let style = GetWindowLongW(hwnd, GWL_STYLE);
    if style & WS_VISIBLE.0 as i32 == 0 {
        return BOOL(1);
    }

    // 排除没有标题栏的窗口
    let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
    if ex & WS_EX_TOOLWINDOW.0 as i32 != 0 {
        return BOOL(1);
    }
    let ctx = &mut *(lparam.0 as *mut EnumWindowsContext);
    if ctx.exclude_hwnd.is_some() && hwnd == ctx.exclude_hwnd.unwrap() {
        return BOOL(1);
    }

    let list = &mut *(ctx.list);
    if let Some(title) = get_window_title_safe(hwnd) {
        if let Ok(app_info) = get_app_info_by_hwnd(hwnd.0 as isize) {
            list.push(WindowInfo {
                hwnd: hwnd.0 as isize,
                title,
                name: app_info.name,
                display_name: app_info.display_name,
                icon_png: app_info.icon_png,
                is_top_most: app_info.is_top_most,
            });
        }
    }

    BOOL(1)
}

struct EnumWindowsContext {
    list: *mut Vec<WindowInfo>,
    exclude_hwnd: Option<HWND>,
}
pub fn enum_windows(hwnd_self: Option<HWND>) -> Vec<WindowInfo> {
    let mut list = Vec::new();

    let mut ctx = EnumWindowsContext {
        list: &mut list as *mut _,
        exclude_hwnd: hwnd_self,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_proc),
            // LPARAM(&mut list as *mut _ as isize),
            LPARAM(&mut ctx as *mut _ as isize),
        );
    }

    list
}

pub fn show_and_topmost(hwnd: isize, hwnd_self: Option<isize>) {
    unsafe {
        let hwnd = HWND(hwnd);

        //如果最小化，先还原
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }

        //设置为置顶窗口
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );

        //确保显示
        let _ = ShowWindow(hwnd, SW_SHOW);

        //请求前台（不保证 100% 成功）
        let _ = SetForegroundWindow(hwnd);

        //设置窗口再次置顶
        if let Some(hwnd) = hwnd_self {
            let _ = SetWindowPos(
                HWND(hwnd),
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
        }
    }
}

pub fn show_and_not_topmost(hwnd: isize) {
    unsafe {
        let hwnd = HWND(hwnd);

        let _ = SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
    }
}

pub fn cancel_all_topmost(hwnds: Vec<isize>) {
    for hwnd in hwnds {
        show_and_not_topmost(hwnd);
    }
}

unsafe fn get_process_exe_path(pid: u32) -> Option<String> {
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

    let mut buffer = [0u16; 260];
    let mut size = buffer.len() as u32;

    let ok = QueryFullProcessImageNameW(
        handle,
        PROCESS_NAME_FORMAT(0),
        PWSTR(&mut buffer as *mut _),
        &mut size,
    );

    let _ = CloseHandle(handle);

    ok.ok()?;

    Some(String::from_utf16_lossy(&buffer[..size as usize]))
}

pub struct AppNames {
    pub file_description: Option<String>,
    pub product_name: Option<String>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LangAndCodePage {
    lang: u16,
    codepage: u16,
}

pub fn get_app_names(exe_path: &str) -> AppNames {
    unsafe {
        let path: Vec<u16> = exe_path.encode_utf16().chain(Some(0)).collect();

        let mut handle = 0;
        let size = GetFileVersionInfoSizeW(PCWSTR(path.as_ptr()), Some(&mut handle));

        if size == 0 {
            return AppNames {
                file_description: None,
                product_name: None,
            };
        }

        let mut data = vec![0u8; size as usize];
        if GetFileVersionInfoW(PCWSTR(path.as_ptr()), 0, size, data.as_mut_ptr() as _).is_err() {
            return AppNames {
                file_description: None,
                product_name: None,
            };
        }
        //读取 Translation
        let mut trans_ptr: *mut LangAndCodePage = std::ptr::null_mut();
        let mut trans_len = 0u32;

        if VerQueryValueW(
            data.as_ptr() as _,
            PCWSTR(
                "\\VarFileInfo\\Translation\0"
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .as_ptr(),
            ),
            &mut trans_ptr as *mut _ as _,
            &mut trans_len,
        )
        .ok()
        .is_err()
        {
            return AppNames {
                file_description: None,
                product_name: None,
            };
        }

        let count = trans_len as usize / std::mem::size_of::<LangAndCodePage>();
        let translations = std::slice::from_raw_parts(trans_ptr, count);

        let mut zh_cn = Vec::new();
        let mut zh_tw = Vec::new();
        let mut others = Vec::new();

        for t in translations {
            if is_simplified_chinese(t.lang) {
                zh_cn.push(*t);
            } else if is_traditional_chinese(t.lang) {
                zh_tw.push(*t);
            } else {
                others.push(*t);
            }
        }

        zh_cn.append(&mut zh_tw);
        zh_cn.append(&mut others);

        let mut file_description = None;
        let mut product_name = None;

        //遍历语言块，优先中文
        for t in zh_cn {
            let keys = [
                format!(
                    "\\StringFileInfo\\{:04x}{:04x}\\FileDescription",
                    t.lang, t.codepage
                ),
                format!(
                    "\\StringFileInfo\\{:04x}{:04x}\\ProductName",
                    t.lang, t.codepage
                ),
            ];
            for key in keys {
                let mut ptr: *mut u16 = std::ptr::null_mut();
                let mut len = 0;

                if VerQueryValueW(
                    data.as_ptr() as _,
                    PCWSTR(
                        key.encode_utf16()
                            .chain(Some(0))
                            .collect::<Vec<_>>()
                            .as_ptr(),
                    ),
                    &mut ptr as *mut _ as _,
                    &mut len,
                )
                .as_bool()
                    && !ptr.is_null()
                {
                    let value =
                        String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len as usize))
                            .trim_end_matches("\0")
                            .to_string();
                    if key.contains("FileDescription") && file_description.is_none() {
                        file_description = Some(value.clone());
                    }
                    if key.contains("ProductName") && product_name.is_none() {
                        product_name = Some(value);
                    }
                }
            }
        }
        AppNames {
            file_description,
            product_name,
        }
    }
}

fn sub_lang(lang: u16) -> u16 {
    lang >> 10
}

fn primary_lang(lang: u16) -> u16 {
    lang & 0x3ff // 低 10 位
}

fn is_simplified_chinese(lang: u16) -> bool {
    primary_lang(lang) == 0x04 && sub_lang(lang) == 0x02
}

fn is_traditional_chinese(lang: u16) -> bool {
    primary_lang(lang) == 0x04 && sub_lang(lang) == 0x01
}
/// 判断窗口是否设置了“总在最前”
pub fn is_window_topmost(hwnd: HWND) -> bool {
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        (ex_style & WS_EX_TOPMOST.0) != 0
    }
}

pub fn get_win32_hwnd_by_tauri_hwnd(app: &tauri::AppHandle, window_label: &str) -> Option<HWND> {
    let hwnd_self: Option<HWND> = match app.get_webview_window(window_label) {
        Some(window) => match window.hwnd().ok() {
            Some(hwnd) => Some(HWND(hwnd.0 as isize)),
            None => None,
        },
        None => None,
    };
    hwnd_self
}
