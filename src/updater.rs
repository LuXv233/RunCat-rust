// CPU监控和动画更新模块：根据CPU使用率更新托盘图标的动画速度

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};

use crate::constants::{FRAME_COUNT, IDI_LIGHTCAT_0, IDI_DARKCAT_0};

// 启动CPU监控更新线程
pub fn start_updater_thread(tx: mpsc::Sender<(u32, f32)>) {
    thread::spawn(move || {
        let mut system = System::new_all();
        let mut icon_index: usize = 0;
        
        loop {
            system.refresh_cpu();
            
            let cpu = system.cpus()
                .iter()
                .map(|c| c.cpu_usage())
                .sum::<f32>() / (system.cpus().len() as f32);
            
            let cpu_usage = cpu.max(0.0).min(100.0);
            let duration = calculate_duration(cpu_usage);
            
            icon_index = (icon_index + 1) % FRAME_COUNT;
            
            if crate::utils::is_skin_follow_system() {
                if crate::utils::is_time_window_in_dark_mode() {
                    let res_id = IDI_DARKCAT_0 as u32 + icon_index as u32;
                    let _ = tx.send((res_id, cpu_usage));
                } else {
                    let res_id = IDI_LIGHTCAT_0 as u32 + icon_index as u32;
                    let _ = tx.send((res_id, cpu_usage));
                } 
            } else if crate::utils::is_effective_dark_mode() {
                let res_id = IDI_DARKCAT_0 as u32 + icon_index as u32;
                let _ = tx.send((res_id, cpu_usage));
            } else {
                let res_id = IDI_LIGHTCAT_0 as u32 + icon_index as u32;
                let _ = tx.send((res_id, cpu_usage));
            } 
            //let res_id = IDI_LIGHTCAT_0 as u32 + icon_index as u32;
            
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
