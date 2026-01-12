// 设置系统为"windows"，这样程序运行时不会显示控制台窗口
#![windows_subsystem = "windows"]

mod constants;
mod utils;
mod window;
mod tray;
mod updater;
mod timer;

use std::{ptr::null_mut, sync::mpsc, time::Duration};
use winapi::um::winuser::{PeekMessageW, TranslateMessage, DispatchMessageW, MSG, WM_QUIT, PM_REMOVE};
use winapi::um::shellapi::NOTIFYICONDATAW;
use winapi::shared::minwindef::HINSTANCE;

use constants::IDI_LIGHTCAT_0;
use utils::get_module_handle;
use window::register_class_and_create_window;
use tray::{create_notify_icon_data, add_tray_icon, remove_tray_icon};
use updater::start_updater_thread;

fn main() {
    let hinstance = get_module_handle();
    let (tx, rx) = mpsc::channel();
    
    start_updater_thread(tx);

    let class_name_w = utils::to_wide_null(format!("RunCatClass{}", std::process::id()));
    register_class_and_create_window(hinstance, class_name_w.as_ptr()).unwrap_or_else(|e| {
        eprintln!("init window failed: {}", e);
        std::process::exit(1);
    });

    let hwnd = window::create_message_window(hinstance, class_name_w.as_ptr());
    let mut nid = create_notify_icon_data(hwnd, hinstance,IDI_LIGHTCAT_0);
    
    if !add_tray_icon(&mut nid) {
        eprintln!("Failed to add tray icon");
        return;
    }

    run_message_loop(&rx, &mut nid, hinstance);
    remove_tray_icon(&mut nid);
}

// 运行主消息循环

// # 参数
// * `rx`: 消息通道的接收端，用于接收来自后台更新线程的CPU使用率数据
// * `nid`: 可变的系统托盘图标数据结构，用于更新托盘图标的显示
// * `_hinstance`: 应用程序实例句柄（当前未使用，保留用于未来扩展）
fn run_message_loop(rx: &mpsc::Receiver<(u32, f32)>, nid: &mut NOTIFYICONDATAW, _hinstance: HINSTANCE) {
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
            update_tray_from_cpu(nid, res_id as u16, cpu_usage);
        }

        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok((res_id, cpu_usage)) => update_tray_from_cpu(nid, res_id as u16, cpu_usage),
            Err(mpsc::RecvTimeoutError::Disconnected) => break 'msg_loop,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

// 更新托盘图标和CPU信息显示

// # 参数
// * `nid`: 可变的系统托盘图标数据结构，包含图标、提示文本等配置
// * `res_id`: 要显示的图标资源ID，对应不同CPU使用率的猫咪动画帧
// * `cpu_usage`: 当前的CPU使用率（百分比）
fn update_tray_from_cpu(nid: &mut NOTIFYICONDATAW, res_id: u16, cpu_usage: f32) {
    let hinstance = get_module_handle();
    let icon = utils::load_icon(hinstance, res_id);
    
    nid.hIcon = icon;

    let tip = format!("CPU: {:.0}%", cpu_usage);
    let wide = utils::to_wide_null(&tip);
    
    nid.szTip = [0u16; 128];
    
    for (i, &c) in wide.iter().take(127).enumerate() {
        nid.szTip[i] = c;
    }

    tray::update_tray_icon(nid);
}
