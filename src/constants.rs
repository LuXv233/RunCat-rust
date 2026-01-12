// 资源常量定义模块

pub const IDI_APP_ICON: u16 = 100; //logo
pub const IDI_LIGHTCAT_0: u16 = 101; //首帧浅色图标
pub const IDI_DARKCAT_0: u16 = 106; //首帧深色图标
pub const FRAME_COUNT: usize = 5; // 猫咪动画的总帧数
pub const WM_TRAYICON: u32 = winapi::um::winuser::WM_USER + 1; 
pub const IDM_EXIT: u32 = 1001; // 退出菜单项
pub const IDM_START_SYSTEM: u32 = 1002; // 开机自启菜单项
pub const IDM_SHOW_TIME: u32 = 1003; // 显示时间窗口菜单项

pub const IDM_SKIN_AUTO: u32 = 1005; // 自动模式
pub const IDM_SKIN_DARK: u32 = 1006; // 深色模式
pub const IDM_SKIN_LIGHT: u32 = 1007; // 浅色模式
