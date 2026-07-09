// 资源常量定义模块
use winapi::shared::minwindef::UINT;


pub const IDI_APP_ICON: u16 = 100; //logo
pub const IDI_BULLEKITTEN_0: u16 = 101; //首帧气泡小猫图标
// pub const IDI_DARKCAT_0: u16 = 111; //首帧深色图标
// pub const IDI_LIGHTCAT_0: u16 = 116; //首帧浅色图标
// pub const KITTEN_COUNT: usize = 10; // 猫咪动画的总帧数
// pub const RUNCAT_COUNT: usize = 5; // 猫咪动画的总帧数
//pub const FRAME_COUNT: usize = KITTEN_COUNT; // 动画帧数，


pub const WM_TRAYICON: u32 = winapi::um::winuser::WM_USER + 1; 




// 菜单项ID定义
pub const IDM_EXIT: u32 = 1001; // 退出
pub const IDM_START_SYSTEM: u32 = 1002; // 开机自启菜单项
pub const IDM_SHOW_TIME: u32 = 1003; // 显示时间窗口菜单项
// pub const IDM_SETTINGS: u32 = 1004; // 设置菜单项
pub const IDM_SKIN_AUTO: u32 = 1005; // 自动模式
pub const IDM_SKIN_DARK: u32 = 1006; // 深色模式
pub const IDM_SKIN_LIGHT: u32 = 1007; // 浅色模式
pub const IDM_SKIN_BUBBLEKITTEN: u32 = 1008; // 气泡小猫
pub const IDM_SKIN_RUNCAT: u32 = 1009; // 奔跑小猫
// pub const IDM_SKIN_CAT: u32 = 1010; // 浅色猫咪模式


pub const IDM_EDIT_MODE: UINT = 2010;