// main.rs 主程序入口文件：负责初始化应用程序并运行消息循环
#![windows_subsystem = "windows"]

mod constants;
mod utils;
mod window;
mod tray;
mod timer;
mod skin;


use std::{ptr::null_mut, sync::{mpsc, Arc, Mutex}, time::Duration};
use winapi::um::winuser::{PeekMessageW, TranslateMessage, DispatchMessageW, MSG, WM_QUIT, PM_REMOVE};
use winapi::um::shellapi::NOTIFYICONDATAW;
use winapi::shared::minwindef::HINSTANCE;

use constants::IDI_BULLEKITTEN_0;
use utils::get_module_handle;
use window::{register_class_and_create_window, create_message_window};
use tray::{create_notify_icon_data, add_tray_icon, remove_tray_icon};
use tray::start_updater_thread;
use skin::{ThemeMode, SharedSkinState, DynSkinState};

fn main() {
    let hinstance = get_module_handle();
    let (tx, rx) = mpsc::channel();

    // 加载保存的设置
    let saved_theme = match utils::load_theme_mode() {
        1 => ThemeMode::Dark,
        2 => ThemeMode::Light,
        _ => ThemeMode::FollowSystem,
    };
    let saved_skin = if utils::load_skin_type() == 1 {
        &skin::SKIN_RUNCAT
    } else {
        &skin::SKIN_BUBBLE_KITTEN
    };

    // 创建共享的皮肤状态（使用保存的设置）
    let skin_state: DynSkinState = Arc::new(Mutex::new(
        SharedSkinState::new(saved_skin, saved_theme)
    ));

    // 把 Arc 传给主窗口的消息处理，方便菜单切换时更新
    window::set_skin_state(skin_state.clone());

    // 启动后台线程，传递 Arc<Mutex<SharedSkinState>>
    start_updater_thread(tx, skin_state.clone());

    let class_name_w = utils::to_wide_null(format!("RunCatClass{}", std::process::id()));
    register_class_and_create_window(hinstance, class_name_w.as_ptr()).unwrap_or_else(|e| {
        eprintln!("init window failed: {}", e);
        std::process::exit(1);
    });

    let hwnd = create_message_window(hinstance, class_name_w.as_ptr());
    let mut nid = create_notify_icon_data(hwnd, hinstance, IDI_BULLEKITTEN_0);

    if !add_tray_icon(&mut nid) {
        eprintln!("Failed to add tray icon");
        return;
    }

    // 根据保存的设置显示时间窗口
    if utils::load_show_time() {
        crate::timer::create_time_window(hinstance);
    }

    run_message_loop(&rx, &mut nid, hinstance, &skin_state);
    remove_tray_icon(&mut nid);
}

// 运行主消息循环
fn run_message_loop(
    rx: &mpsc::Receiver<(u32, f32)>,
    nid: &mut NOTIFYICONDATAW,
    _hinstance: HINSTANCE,
    skin_state: &DynSkinState,
) {
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

        // 非阻塞尝试接收
        while let Ok((frame, cpu_usage)) = rx.try_recv() {
            update_tray_from_cpu(nid, frame, cpu_usage, skin_state);
        }

        // 阻塞等待（超时 250ms）
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok((frame, cpu_usage)) => update_tray_from_cpu(nid, frame, cpu_usage, skin_state),
            Err(mpsc::RecvTimeoutError::Disconnected) => break 'msg_loop,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

// 更新托盘图标和CPU信息显示
fn update_tray_from_cpu(
    nid: &mut NOTIFYICONDATAW,
    frame: u32,
    cpu_usage: f32,
    skin_state: &DynSkinState,
) {
    let hinstance = get_module_handle();

    // 通过共享状态解析真正的图标资源 ID
    let icon_id = {
        let state = skin_state.lock().unwrap();
        state.resolve_icon_id(frame)
    };

    let icon = utils::load_icon(hinstance, icon_id);
    nid.hIcon = icon;

    let tip = format!("CPU: {:.0}%", cpu_usage);
    let wide = utils::to_wide_null(&tip);
    
    nid.szTip = [0u16; 128];
    
    for (i, &c) in wide.iter().take(127).enumerate() {
        nid.szTip[i] = c;
    }

    tray::update_tray_icon(nid);
}

