// tray.rs 系统托盘图标管理模块：负责系统托盘图标的创建、更新和删除

use winapi::shared::windef::HWND;
use winapi::um::shellapi::{NOTIFYICONDATAW, Shell_NotifyIconW, NIM_ADD, NIM_MODIFY, NIM_DELETE};
use winapi::shared::minwindef::HINSTANCE;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};

use crate::utils::{to_wide_null, load_icon};
use crate::skin::SharedSkinState;

const NIF_ICON: u32 = 0x00000002;
const NIF_MESSAGE: u32 = 0x00000001;
const NIF_TIP: u32 = 0x00000004;

// 初始化托盘图标数据结构
pub fn create_notify_icon_data(hwnd: HWND, hinstance: HINSTANCE, res_id: u16) -> NOTIFYICONDATAW {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = crate::constants::WM_TRAYICON;
    
    set_nid_icon(&mut nid, hinstance, res_id);
    set_nid_tip(&mut nid, "RunCat Rust - CPU Monitor");

    nid
}

// 添加托盘图标到系统托盘区域
pub fn add_tray_icon(nid: &mut NOTIFYICONDATAW) -> bool {
    unsafe { Shell_NotifyIconW(NIM_ADD, nid) != 0 }
}

// 更新托盘图标的显示
pub fn update_tray_icon(nid: &mut NOTIFYICONDATAW) {
    unsafe { Shell_NotifyIconW(NIM_MODIFY, nid); }
}

// 从系统托盘中移除托盘图标
pub fn remove_tray_icon(nid: &mut NOTIFYICONDATAW) {
    unsafe { Shell_NotifyIconW(NIM_DELETE, nid); }
}

// 设置托盘图标的图标
fn set_nid_icon(nid: &mut NOTIFYICONDATAW, hinstance: HINSTANCE, res_id: u16) {
    let icon = load_icon(hinstance, res_id);
    nid.hIcon = icon;
}

// 设置托盘图标的提示文本
fn set_nid_tip(nid: &mut NOTIFYICONDATAW, s: &str) {
    let wide = to_wide_null(s);
    nid.szTip = [0u16; 128];
    for (i, &c) in wide.iter().take(127).enumerate() {
        nid.szTip[i] = c;
    }
}

/// 启动后台 CPU 监控和动画帧更新线程。
///
/// 通过 `SharedSkinState` 共享当前皮肤和主题，主线程修改后线程自动感知。
/// 向主线程发送 `(icon_index, cpu_usage)`，由主线程调用 `skin.resolve_icon_id()` 解析资源 ID。
pub fn start_updater_thread(
    tx: mpsc::Sender<(u32, f32)>,
    skin_state: Arc<Mutex<SharedSkinState>>,
) {
    thread::spawn(move || {
        let mut system = System::new_all();
        let mut icon_index: u32 = 0;
        
        loop {
            system.refresh_cpu();
            
            let cpu_usage = system.cpus()
                .iter()
                .map(|c| c.cpu_usage())
                .sum::<f32>() 
                / system.cpus().len() as f32;
            
            let cpu_usage = cpu_usage.clamp(0.0, 100.0);
            let duration = calculate_duration(cpu_usage);
            
            // 从共享状态获取当前皮肤的帧数，安全推进
            let frame_count = {
                let state = skin_state.lock().unwrap();
                state.skin.frame_count
            };
            icon_index = (icon_index + 1) % frame_count;
            
            // 只发送帧索引和 CPU 使用率，不包含皮肤/主题引用
            let _ = tx.send((icon_index, cpu_usage));
            
            thread::sleep(duration);
        }
    });
}

// 计算动画帧的切换间隔
pub fn calculate_duration(cpu_usage: f32) -> Duration {
    let speed = cpu_usage / 5.0f32;
    let duration_ms = 500.0f32 / speed.max(0.01f32);
    Duration::from_millis(duration_ms.clamp(50.0, 200.0) as u64)
}