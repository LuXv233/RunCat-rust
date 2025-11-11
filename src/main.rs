#![windows_subsystem = "windows"] 

use std::{ffi::OsString, ptr::null_mut, sync::mpsc, thread, time::Duration};
use std::os::windows::ffi::OsStrExt; 
use sysinfo::{CpuExt, System, SystemExt};
use winapi::{
    shared::{basetsd::UINT_PTR, minwindef::*, windef::*},
    um::{libloaderapi::*, shellapi::*, winuser::*},
};

// 资源
const IDI_APP_ICON: u16 = 100; 
const IDI_CAT_0: u16 = 101;
const FRAME_COUNT: usize = 5;
const WM_TRAYICON: u32 = WM_USER + 1;
const IDM_EXIT: u32 = 1001;

fn main() {
    let hinstance = unsafe { GetModuleHandleW(null_mut()) };

    let (tx, rx) = mpsc::channel();
    start_updater_thread(tx.clone());

    let class_name_w = to_wide_null(format!("RunCatClass{}", std::process::id()));
    register_class_and_create_window(hinstance, class_name_w.as_ptr()).unwrap_or_else(|e| {
        eprintln!("init window failed: {}", e);
        std::process::exit(1);
    });
    
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name_w.as_ptr(),
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
    };
   

    // 初始化托盘图标
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    set_nid_icon(&mut nid, hinstance, IDI_CAT_0);
    set_nid_tip(&mut nid, "RunCat Rust - CPU Monitor");

    if unsafe { Shell_NotifyIconW(NIM_ADD, &mut nid) } == 0 {
        eprintln!("Failed to add tray icon");
        return;
    }

    let mut msg: MSG = unsafe { std::mem::zeroed() };
    'msg_loop: loop {
        while unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) } != 0 {
            if msg.message == WM_QUIT {
                break 'msg_loop;
            }
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        while let Ok((res_id, cpu_usage)) = rx.try_recv() {
            apply_update(&mut nid, hinstance, res_id as u16, cpu_usage);
        }

        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok((res_id, cpu_usage)) => apply_update(&mut nid, hinstance, res_id as u16, cpu_usage),
            Err(mpsc::RecvTimeoutError::Disconnected) => break 'msg_loop,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }

    unsafe { Shell_NotifyIconW(NIM_DELETE, &mut nid) };
}

//退出
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            match lparam as UINT {
                WM_RBUTTONUP => {
                    let mut pt = POINT { x: 0, y: 0 };
                    GetCursorPos(&mut pt);
                    SetForegroundWindow(hwnd);
                    let hmenu = CreatePopupMenu();
                    let exit_text = to_wide_null("退出");
                    AppendMenuW(hmenu, MF_STRING, IDM_EXIT as UINT_PTR, exit_text.as_ptr());
                    TrackPopupMenu(
                        hmenu,
                        TPM_RIGHTALIGN | TPM_BOTTOMALIGN,
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
            0
        }
        WM_COMMAND => {
            match wparam as UINT {
                IDM_EXIT => PostQuitMessage(0),
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ---------- fn ----------

fn start_updater_thread(tx: mpsc::Sender<(u32, f32)>) {
    thread::spawn(move || {
        let mut system = System::new_all();
        let mut icon_index: usize = 0;
        loop {
            system.refresh_cpu();
            let cpu = system.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / (system.cpus().len() as f32);
            let cpu_usage = cpu.max(0.0).min(100.0);

            let duration = calculate_duration(cpu_usage);
            icon_index = (icon_index + 1) % FRAME_COUNT;
            let res_id = IDI_CAT_0 as u32 + icon_index as u32;
            let _ = tx.send((res_id, cpu_usage));
            thread::sleep(duration);
        }
    });
}
//图标，真的需要吗？
fn register_class_and_create_window(hinstance: HINSTANCE, class_name: *const u16) -> Result<(), &'static str> {
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: load_icon(hinstance, IDI_APP_ICON),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: class_name,
        hIconSm: load_icon(hinstance, IDI_APP_ICON),
    };
    if unsafe { RegisterClassExW(&wc) } == 0 {
        return Err("RegisterClassExW failed");
    }
    Ok(())
}

fn apply_update(nid: &mut NOTIFYICONDATAW, hinstance: HINSTANCE, res_id: u16, cpu_usage: f32) {
    set_nid_icon(nid, hinstance, res_id);
    set_nid_tip(nid, &format!("CPU: {:.1}%", cpu_usage));
    unsafe { Shell_NotifyIconW(NIM_MODIFY, nid) };
}

fn set_nid_icon(nid: &mut NOTIFYICONDATAW, hinstance: HINSTANCE, res_id: u16) {
    // 如果load_icon 资源缺失，终止进程
    let icon = load_icon(hinstance, res_id);
    nid.hIcon = icon;
}

fn set_nid_tip(nid: &mut NOTIFYICONDATAW, s: &str) {
    let wide = to_wide_null(s);
    nid.szTip = [0u16; 128];
    for (i, &c) in wide.iter().take(127).enumerate() {
        nid.szTip[i] = c;
    }
}

// CPU -> 动画间隔,越忙越快。
fn calculate_duration(cpu_usage: f32) -> Duration {
    let speed = cpu_usage / 5.0f32;
    let duration_ms = 500.0f32 / speed.max(0.01f32);
    Duration::from_millis(duration_ms.clamp(50.0, 200.0) as u64)
}


fn to_wide_null(s: impl AsRef<str>) -> Vec<u16> {
    let mut v: Vec<u16> = OsString::from(s.as_ref()).encode_wide().collect();
    if v.last().copied() != Some(0) {
        v.push(0);
    }
    v
}

// 从嵌入资源加载图标，若嵌入资源不存在，返回 NULL
fn load_icon(hinstance: HINSTANCE, res_id: u16) -> HICON {
    unsafe {
        let icon = LoadIconW(hinstance, MAKEINTRESOURCEW(res_id as WORD));
        if icon.is_null() {
            eprintln!("[RunCat] critical: embedded resource id {} missing; aborting because runtime fallback is disabled", res_id);
            std::process::exit(1);
        }
        icon
    }
}
