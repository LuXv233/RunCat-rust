// 工具函数模块：提供Windows API相关的辅助函数

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



const RUNCAT_SETTINGS_SUBKEY: &str = "Software\\RunCat\\Settings";

/// 应用层：是否跟随系统主题（默认 true）
pub fn is_skin_follow_system() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey_with_flags(RUNCAT_SETTINGS_SUBKEY, KEY_READ) {
        Ok(key) => match key.get_value::<u32, _>("FollowSystemTheme") {
            Ok(v) => v == 1,
            Err(_) => true,
        },
        Err(_) => true,
    }
}




pub fn set_skin_follow_system(enable: bool) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let value: u32 = if enable { 1 } else { 0 };
    match hkcu.open_subkey_with_flags(RUNCAT_SETTINGS_SUBKEY, KEY_WRITE) {
        Ok(key) => {
            if let Err(e) = key.set_value("FollowSystemTheme", &value) {
                eprintln!("写入 FollowSystemTheme 失败: {}", e);
            }
        }
        Err(_) => {
            match hkcu.create_subkey(RUNCAT_SETTINGS_SUBKEY) {
                Ok((key, _)) => {
                    if let Err(e) = key.set_value("FollowSystemTheme", &value) {
                        eprintln!("创建并写入 FollowSystemTheme 失败: {}", e);
                    }
                }
                Err(e) => eprintln!("无法创建 RunCat 设置子键: {}", e),
            }
        }
    }
}
    pub fn set_app_force_dark(enable: bool) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let value: u32 = if enable { 1 } else { 0 };
    match hkcu.open_subkey_with_flags(RUNCAT_SETTINGS_SUBKEY, KEY_WRITE) {
        Ok(key) => {
            if let Err(e) = key.set_value("ForceDark", &value) {
                eprintln!("写入 ForceDark 失败: {}", e);
            }
        }
        Err(_) => {
            match hkcu.create_subkey(RUNCAT_SETTINGS_SUBKEY) {
                Ok((key, _)) => {
                    if let Err(e) = key.set_value("ForceDark", &value) {
                        eprintln!("创建并写入 ForceDark 失败: {}", e);
                    }
                }
                Err(e) => eprintln!("无法创建 RunCat 设置子键: {}", e),
            }
        }
    }
}

/// 应用层：强制深色（当不跟随系统时使用）；默认 false（浅色）
pub fn is_app_force_dark() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey_with_flags(RUNCAT_SETTINGS_SUBKEY, KEY_READ) {
        Ok(key) => match key.get_value::<u32, _>("ForceDark") {
            Ok(v) => v == 1,
            Err(_) => false,
        },
        Err(_) => false,
    }
}



/// 返回当前生效的深色模式状态
pub fn is_effective_dark_mode() -> bool {
    if is_skin_follow_system() {
        is_time_window_in_dark_mode()
    } else {
        is_app_force_dark()
    }
}
