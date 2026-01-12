// 窗口管理模块：负责Windows窗口的创建、注册和消息处理

use winapi::shared::minwindef::{HINSTANCE, LRESULT, UINT, WPARAM, LPARAM};
use winapi::shared::basetsd::UINT_PTR;
use winapi::shared::windef::HWND;
use winapi::um::winuser::*;
use winapi::shared::windef::POINT;
use std::ptr::null_mut;

use crate::constants::{WM_TRAYICON, IDM_EXIT, IDM_START_SYSTEM, IDM_SHOW_TIME, IDM_SKIN_DARK, IDM_SKIN_LIGHT, IDM_SKIN_AUTO};
use crate::utils::{to_wide_null, load_cursor, load_icon};


// 注册窗口类并创建窗口
pub fn register_class_and_create_window(hinstance: HINSTANCE, class_name: *const u16) -> Result<(), &'static str> {
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: load_icon(hinstance, crate::constants::IDI_APP_ICON),
        hCursor: load_cursor() as *mut _, 
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: class_name,
        hIconSm: load_icon(hinstance, crate::constants::IDI_APP_ICON),
    };
    
    if unsafe { RegisterClassExW(&wc) } == 0 {
        return Err("RegisterClassExW failed");
    }
    
    // 注册成功
    Ok(())
}

// 创建消息窗口
pub fn create_message_window(hinstance: HINSTANCE, class_name: *const u16) -> HWND {
    unsafe {
        CreateWindowExW(
            0,
            class_name,
            to_wide_null("RunCat").as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            hinstance,
            null_mut(),
        )
    }
}

// 窗口消息处理函数
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TRAYICON => handle_tray_icon_message(hwnd, lparam),
        WM_COMMAND => handle_command_message(wparam),
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        
        // 其他消息使用默认处理
        // DefWindowProcW是Windows提供的默认窗口过程函数
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// 处理托盘图标消息
unsafe fn handle_tray_icon_message(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    match lparam as UINT {
        WM_RBUTTONUP => {
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            
            SetForegroundWindow(hwnd);
            let hmenu = CreatePopupMenu();
            
            let time_window_visible = crate::timer::is_time_window_visible();
            
            let show_time_text = if time_window_visible {
                "隐藏时间"
            } else {
                "显示时间"
            };
            let start_system = to_wide_null("开机自启");
            let exit_text = to_wide_null("退出");

            let start_enabled = crate::utils::is_startup_enabled();
            let mut start_flags = MF_STRING;
            if start_enabled {
                start_flags |= MF_CHECKED;
            }

            let mut show_time_flags = MF_STRING;
            if time_window_visible {
                show_time_flags |= MF_CHECKED;
            }
            let hsubmenu_skin = CreatePopupMenu();
            
            let mut auto_flags = MF_STRING;
            let mut dark_flags = MF_STRING;
            let mut light_flags = MF_STRING;
            let follow_system = crate::utils::is_skin_follow_system();
            let effective_dark = crate::utils::is_effective_dark_mode();
            if follow_system {
                auto_flags |= MF_CHECKED;
            } else if effective_dark {
                dark_flags |= MF_CHECKED;
            } else {
                light_flags |= MF_CHECKED;
            }

            let auto_text = to_wide_null("跟随系统");
            let dark_text = to_wide_null("深色模式");
            let light_text = to_wide_null("浅色模式");

            AppendMenuW(
                hsubmenu_skin,
                auto_flags,
                IDM_SKIN_AUTO as UINT_PTR,
                auto_text.as_ptr(),
            );
            AppendMenuW(
                hsubmenu_skin,
                dark_flags,
                IDM_SKIN_DARK as UINT_PTR,
                dark_text.as_ptr(),
            );
            AppendMenuW(
                hsubmenu_skin,
                light_flags,
                IDM_SKIN_LIGHT as UINT_PTR,
                light_text.as_ptr(),
            );

            // 将子菜单添加到主弹出菜单
            AppendMenuW(
                hmenu,
                MF_POPUP,
                hsubmenu_skin as UINT_PTR,
                to_wide_null("颜色模式").as_ptr(),
            );
            // 添加"显示时间"或"隐藏时间"菜单项到菜单
            AppendMenuW(
                hmenu,
                show_time_flags,
                IDM_SHOW_TIME as UINT_PTR,
                to_wide_null(show_time_text).as_ptr(),
            );

            // 添加"开机自启"菜单项到菜单
            AppendMenuW(
                hmenu,
                start_flags,
                IDM_START_SYSTEM as UINT_PTR,
                start_system.as_ptr(),
            );

            AppendMenuW(hmenu, MF_SEPARATOR, 0, null_mut());

            AppendMenuW(
                hmenu,
                MF_STRING,
                IDM_EXIT as UINT_PTR,
                exit_text.as_ptr(),
            );
            
            

            
            TrackPopupMenu(
                hmenu,
                TPM_LEFTALIGN | TPM_BOTTOMALIGN,
                pt.x,
                pt.y,
                0,
                hwnd,
                null_mut(),
            );
            
            PostMessageW(hwnd, WM_NULL, 0, 0);
            DestroyMenu(hmenu);
        }
        
        _ => {}
    }
    
    // 返回0表示消息已处理
    0
}

// 处理命令消息
unsafe fn handle_command_message(wparam: WPARAM) -> LRESULT {
    match wparam as UINT {
        IDM_SHOW_TIME => {
            if crate::timer::is_time_window_visible() {
                crate::timer::close_time_window();
            } else {
                let hinstance = crate::utils::get_module_handle();
                let hwnd = crate::timer::create_time_window(hinstance);
                
                if hwnd.is_null() {
                    eprintln!("创建时间窗口失败");
                }
            }
        }
        
        IDM_START_SYSTEM => {
            crate::utils::toggle_startup();
        }

        IDM_SKIN_DARK => {
            crate::utils::set_skin_follow_system(false);
            crate::utils::set_app_force_dark(true);
            if crate::timer::is_time_window_visible() {
                crate::timer::close_time_window();
                let hinstance = crate::utils::get_module_handle();
                let hwnd = crate::timer::create_time_window(hinstance);
                if hwnd.is_null() {
                    eprintln!("创建时间窗口失败");
                }
            }
        }

        IDM_SKIN_LIGHT => {
            crate::utils::set_skin_follow_system(false);
            crate::utils::set_app_force_dark(false);
            if crate::timer::is_time_window_visible() {
                crate::timer::close_time_window();
                let hinstance = crate::utils::get_module_handle();
                let hwnd = crate::timer::create_time_window(hinstance);
                if hwnd.is_null() {
                    eprintln!("创建时间窗口失败");
                }
            }
        }

        IDM_SKIN_AUTO => {
            crate::utils::set_skin_follow_system(true);
            if crate::timer::is_time_window_visible() {
                crate::timer::close_time_window();
                let hinstance = crate::utils::get_module_handle();
                let hwnd = crate::timer::create_time_window(hinstance);
                if hwnd.is_null() {
                    eprintln!("创建时间窗口失败");
                }
            }
        }
        
        IDM_EXIT => {
            PostQuitMessage(0);
        }
        
        _ => {}
    }
    
    0
    
}
