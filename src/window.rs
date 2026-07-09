// window.rs 窗口管理模块：负责Windows窗口的创建、注册和消息处理

use winapi::shared::minwindef::{HINSTANCE, LRESULT, UINT, WPARAM, LPARAM};
use winapi::shared::basetsd::UINT_PTR;
use winapi::shared::windef::HWND;
use winapi::um::winuser::*;
use winapi::shared::windef::POINT;
use std::ptr::null_mut;
use std::sync::OnceLock;
use crate::constants::{
    IDM_EXIT, IDM_SHOW_TIME,IDM_EDIT_MODE,
    IDM_SKIN_AUTO, IDM_SKIN_DARK, IDM_SKIN_LIGHT,
    IDM_SKIN_BUBBLEKITTEN, IDM_SKIN_RUNCAT,
    IDM_START_SYSTEM, WM_TRAYICON,
};
use crate::utils::{to_wide_null, load_cursor, load_icon};
use crate::skin::{ThemeMode, DynSkinState};

// 全局：共享皮肤状态（在 main 中创建，主窗口消息处理中读取/修改）
static G_SKIN_STATE: OnceLock<DynSkinState> = OnceLock::new();

pub fn set_skin_state(state: DynSkinState) {
    // 仅允许设置一次；如需多次更新，改用 Mutex/ArcSwap
    let _ = G_SKIN_STATE.set(state);
}

fn get_skin_state() -> DynSkinState {
    G_SKIN_STATE.get().expect("skin state not initialized").clone()
}

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
extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TRAYICON => handle_tray_icon_message(hwnd, lparam),
        WM_COMMAND => handle_command_message(wparam),
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

// 处理托盘图标消息（右键菜单）
fn handle_tray_icon_message(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    match lparam as UINT {
        WM_RBUTTONUP => {
            let mut pt = POINT { x: 0, y: 0 };

            // -------------------- 读取当前状态（纯安全代码）--------------------
            let skin_state = get_skin_state();
            let skin_state_guard = skin_state.lock().unwrap();
            let follow_system = skin_state_guard.theme == ThemeMode::FollowSystem;
            let effective_dark = skin_state_guard.effective_theme() == ThemeMode::Dark;
            let current_skin_name = skin_state_guard.skin.name;
            drop(skin_state_guard); // 提前释放锁

            // -------------------- 预构建所有宽字符串（避免临时值 UAF）--------------------
            let follow_text = to_wide_null("跟随系统");
            let dark_text = to_wide_null("深色模式");
            let light_text = to_wide_null("浅色模式");
            let bubble_text = to_wide_null("气泡小猫");
            let runcat_text = to_wide_null("奔跑小猫");
            let skin_name_text = to_wide_null(&format!("{}", current_skin_name));
            let color_mode_text = to_wide_null("颜色模式");
            let pet_text = to_wide_null("宠物");
            let time_window_visible = crate::timer::is_time_window_visible();
            let show_time_text = to_wide_null(if time_window_visible { "隐藏时间" } else { "显示时间" });
            let edit_text = to_wide_null("编辑模式");
            let start_text = to_wide_null("开机自启");
            let exit_text = to_wide_null("退出");

            // -------------------- 计算菜单标志位（纯安全代码）--------------------
            let auto_flags = MF_STRING | if follow_system { MF_CHECKED } else { 0 };
            let dark_flags = MF_STRING | if !follow_system && effective_dark { MF_CHECKED } else { 0 };
            let light_flags = MF_STRING | if !follow_system && !effective_dark { MF_CHECKED } else { 0 };
            let bubble_flags = MF_STRING | if current_skin_name == "BubbleKitten" { MF_CHECKED } else { 0 };
            let runcat_flags = MF_STRING | if current_skin_name != "BubbleKitten" { MF_CHECKED } else { 0 };
            let show_time_flags = MF_STRING | if time_window_visible { MF_CHECKED } else { 0 };
            let edit_flags = MF_STRING | if crate::timer::is_edit_mode() { MF_CHECKED } else { 0 };
            let start_enabled = crate::utils::is_startup_enabled();
            let start_flags = MF_STRING | if start_enabled { MF_CHECKED } else { 0 };

            // -------------------- 所有 FFI 调用统一在 unsafe 块中 --------------------
            unsafe {
                GetCursorPos(&mut pt);
                SetForegroundWindow(hwnd);
                let hmenu = CreatePopupMenu();
                let hsubmenu_color = CreatePopupMenu();
                let hsubmenu_skin = CreatePopupMenu();

                // 颜色模式子菜单
                AppendMenuW(hsubmenu_color, auto_flags, IDM_SKIN_AUTO as UINT_PTR, follow_text.as_ptr());
                AppendMenuW(hsubmenu_color, dark_flags, IDM_SKIN_DARK as UINT_PTR, dark_text.as_ptr());
                AppendMenuW(hsubmenu_color, light_flags, IDM_SKIN_LIGHT as UINT_PTR, light_text.as_ptr());

                // 皮肤切换子菜单
                AppendMenuW(hsubmenu_skin, bubble_flags, IDM_SKIN_BUBBLEKITTEN as UINT_PTR, bubble_text.as_ptr());
                AppendMenuW(hsubmenu_skin, runcat_flags, IDM_SKIN_RUNCAT as UINT_PTR, runcat_text.as_ptr());

                // 主菜单
                AppendMenuW(hmenu, MF_STRING | MF_DISABLED, 0, skin_name_text.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, null_mut());
                AppendMenuW(hmenu, MF_POPUP, hsubmenu_color as UINT_PTR, color_mode_text.as_ptr());
                AppendMenuW(hmenu, MF_POPUP, hsubmenu_skin as UINT_PTR, pet_text.as_ptr());
                AppendMenuW(hmenu, show_time_flags, IDM_SHOW_TIME as UINT_PTR, show_time_text.as_ptr());
                AppendMenuW(hmenu, edit_flags, IDM_EDIT_MODE as UINT_PTR, edit_text.as_ptr());
                AppendMenuW(hmenu, start_flags, IDM_START_SYSTEM as UINT_PTR, start_text.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, null_mut());
                AppendMenuW(hmenu, MF_STRING, IDM_EXIT as UINT_PTR, exit_text.as_ptr());

                // 显示并销毁菜单
                TrackPopupMenu(
                    hmenu,
                    TPM_LEFTALIGN | TPM_BOTTOMALIGN,
                    pt.x, pt.y,
                    0, hwnd, null_mut(),
                );
                PostMessageW(hwnd, WM_NULL, 0, 0); 
                DestroyMenu(hmenu);
            }
        }
        _ => {}
    }
    0
}

// 处理命令消息
fn handle_command_message(wparam: WPARAM) -> LRESULT {
    match wparam as UINT {
        IDM_SHOW_TIME => {
            if crate::timer::is_time_window_visible() {
                crate::timer::close_time_window();
                crate::utils::save_show_time(false);
            } else {
                let hinstance = crate::utils::get_module_handle();
                let hwnd = crate::timer::create_time_window(hinstance);
                if hwnd.is_null() {
                    eprintln!("创建时间窗口失败");
                } else {
                    crate::utils::save_show_time(true);
                }
            }
        }
        IDM_EDIT_MODE => {
            crate::timer::toggle_edit_mode();
        }
        IDM_START_SYSTEM => {
            crate::utils::toggle_startup();
        }

        // --- 颜色模式切换 ---
        IDM_SKIN_DARK => {
            let s_arc = get_skin_state();
            let mut s = s_arc.lock().unwrap();
            s.theme = ThemeMode::Dark;
            crate::utils::save_theme_mode(1);
            if crate::timer::is_time_window_visible() {
                crate::timer::redraw_time_window();
            }
        }

        IDM_SKIN_LIGHT => {
            let s_arc = get_skin_state();
            let mut s = s_arc.lock().unwrap();
            s.theme = ThemeMode::Light;
            crate::utils::save_theme_mode(2);
            if crate::timer::is_time_window_visible() {
                crate::timer::redraw_time_window();
            }
        }

        IDM_SKIN_AUTO => {
            let s_arc = get_skin_state();
            let mut s = s_arc.lock().unwrap();
            s.theme = ThemeMode::FollowSystem;
            crate::utils::save_theme_mode(0);
            if crate::timer::is_time_window_visible() {
                crate::timer::redraw_time_window();
            }
        }

        // --- 皮肤切换 ---
        IDM_SKIN_BUBBLEKITTEN => {
            let s_arc = get_skin_state();
            let mut s = s_arc.lock().unwrap();
            s.skin = &crate::skin::SKIN_BUBBLE_KITTEN;
            crate::utils::save_skin_type(0);
        }

        IDM_SKIN_RUNCAT => {
            let s_arc = get_skin_state();
            let mut s = s_arc.lock().unwrap();
            s.skin = &crate::skin::SKIN_RUNCAT;
            crate::utils::save_skin_type(1);
        }
        
        IDM_EXIT => {
            unsafe{PostQuitMessage(0);}
        }
        
        _ => {}
    }
    0
}
