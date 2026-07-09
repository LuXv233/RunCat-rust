//utils.rs 工具函数模块：提供Windows API相关的辅助函数

use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use winapi::um::winuser::{LoadIconW, LoadCursorW, IDC_ARROW};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::minwinbase::SYSTEMTIME;
use winapi::um::sysinfoapi::GetLocalTime;
use winapi::shared::minwindef::{HINSTANCE, WORD};
use winapi::shared::windef::HICON;
use std::ptr::null_mut;
use winreg::RegKey;
use winreg::enums::*;
use std::env;

// Windows宽字符字符串
pub fn to_wide_null(s: impl AsRef<str>) -> Vec<u16> {
    let wide: Vec<u16> = OsString::from(s.as_ref())
        .encode_wide()
        .chain(Some(0))
        .collect();
    wide
}

// 加载图标资源
pub fn load_icon(hinstance: HINSTANCE, res_id: u16) -> HICON {
    unsafe {
        LoadIconW(hinstance, res_id as WORD as usize as *const u16)
    }
}

// 加载系统默认光标资源
pub fn load_cursor() -> HICON {
    unsafe {
        LoadCursorW(null_mut(), IDC_ARROW as usize as *const u16) as HICON
    }
}

// 获取当前模块的句柄

pub fn get_module_handle() -> HINSTANCE {
    unsafe {
        GetModuleHandleW(null_mut())
    }
}




pub fn toggle_startup() {
    let app_name = "RunCat";

    let exe_path = match env::current_exe() {
        Ok(p) => p.display().to_string(),
        Err(e) => {
            eprintln!("无法获取可执行文件路径: {}", e);
            return;
        }
    };
    let exe_quoted = format!("\"{}\"", exe_path);

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    match hkcu.open_subkey_with_flags(run_subkey, KEY_READ | KEY_WRITE) {
        Ok(key) => {
            match key.get_value::<String, _>(app_name) {
                Ok(_) => {
                    if let Err(e) = key.delete_value(app_name) {
                        eprintln!("删除启动项失败: {}", e);
                    } else {
                        println!("已禁用开机自启");
                    }
                }
                Err(_) => {
                    if let Err(e) = key.set_value(app_name, &exe_quoted) {
                        eprintln!("设置启动项失败: {}", e);
                    } else {
                        println!("已启用开机自启");
                    }
                }
            }
        }
        Err(_) => {
            match hkcu.create_subkey(run_subkey) {
                Ok((key, _disp)) => {
                    if let Err(e) = key.set_value(app_name, &exe_quoted) {
                        eprintln!("创建并设置启动项失败: {}", e);
                    } else {
                        println!("已启用开机自启 (新建 Run 子键)");
                    }
                }
                Err(e) => eprintln!("无法打开或创建 Run 注册表项: {}", e),
            }
        }
    }
}


// 检查是否已启用开机自启
pub fn is_startup_enabled() -> bool {
    let app_name = "RunCat";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    match hkcu.open_subkey_with_flags(run_subkey, KEY_READ) {
        Ok(key) => match key.get_value::<String, _>(app_name) {
            Ok(_) => true,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

// 获取Windows本地时间
pub fn get_windows_time() -> (u16, u16, u16, u16, u16, u16, u16, u16) {
    let mut system_time: SYSTEMTIME = unsafe { std::mem::zeroed() };
    
    unsafe {
        GetLocalTime(&mut system_time);
    }
    
    (
        system_time.wYear,
        system_time.wMonth,
        system_time.wDay,
        system_time.wHour,
        system_time.wMinute,
        system_time.wSecond,
        system_time.wMilliseconds,
        system_time.wDayOfWeek,
    )
}




pub fn is_time_window_in_dark_mode() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let personalize_subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";

    match hkcu.open_subkey_with_flags(personalize_subkey, KEY_READ) {
        Ok(key) => match key.get_value::<u32, _>("AppsUseLightTheme") {
            Ok(value) => value == 0,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

// ==================== 配置持久化 ====================

const SETTINGS_SUBKEY: &str = "Software\\RunCat\\Settings";

fn open_settings_key(flags: u32) -> Option<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags(SETTINGS_SUBKEY, flags)
        .ok()
        .or_else(|| {
            hkcu.create_subkey(SETTINGS_SUBKEY).ok().map(|(k, _)| k)
        })
}

fn read_setting_u32(name: &str, default: u32) -> u32 {
    open_settings_key(KEY_READ)
        .and_then(|key| key.get_value::<u32, _>(name).ok())
        .unwrap_or(default)
}

fn write_setting_u32(name: &str, value: u32) {
    if let Some(key) = open_settings_key(KEY_WRITE) {
        let _ = key.set_value(name, &value);
    }
}

/// 主题模式: 0=FollowSystem, 1=Dark, 2=Light
pub fn load_theme_mode() -> u32 {
    read_setting_u32("ThemeMode", 0)
}

pub fn save_theme_mode(mode: u32) {
    write_setting_u32("ThemeMode", mode);
}

/// 宠物皮肤: 0=BubbleKitten, 1=RunCat
pub fn load_skin_type() -> u32 {
    read_setting_u32("SkinType", 0)
}

pub fn save_skin_type(skin: u32) {
    write_setting_u32("SkinType", skin);
}

/// 时间窗口是否显示: 0=隐藏, 1=显示
pub fn load_show_time() -> bool {
    read_setting_u32("ShowTime", 0) == 1
}

pub fn save_show_time(show: bool) {
    write_setting_u32("ShowTime", if show { 1 } else { 0 });
}

/// 时间窗口位置
pub fn load_time_window_pos() -> Option<(i32, i32)> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey_with_flags(SETTINGS_SUBKEY, KEY_READ) {
        let x: u32 = key.get_value("TimeWinX").unwrap_or(u32::MAX);
        let y: u32 = key.get_value("TimeWinY").unwrap_or(u32::MAX);
        if x != u32::MAX && y != u32::MAX {
            return Some((x as i32, y as i32));
        }
    }
    None
}

pub fn save_time_window_pos(x: i32, y: i32) {
    if let Some(key) = open_settings_key(KEY_WRITE) {
        let _ = key.set_value("TimeWinX", &(x as u32));
        let _ = key.set_value("TimeWinY", &(y as u32));
    }
}