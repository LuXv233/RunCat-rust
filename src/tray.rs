// 系统托盘图标管理模块：负责系统托盘图标的创建、更新和删除

use winapi::shared::windef::HWND;
use winapi::um::shellapi::{NOTIFYICONDATAW, Shell_NotifyIconW, NIM_ADD, NIM_MODIFY, NIM_DELETE};
use winapi::shared::minwindef::HINSTANCE;

use crate::utils::{to_wide_null, load_icon};

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
