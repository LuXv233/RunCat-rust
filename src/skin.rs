// skin.rs 皮肤定义模块：定义了主题模式、皮肤结构体以及具体的皮肤实例
use std::sync::{Arc, Mutex};
/// 主题模式（核心枚举）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeMode {
    Light,
    Dark,
    FollowSystem,
}

/// 单个皮肤定义
#[derive(Debug, Clone)]
pub struct Skin {
    pub name: &'static str,
    pub light_base_id: u16,   // 浅色图标起始 ID
    pub dark_base_id: u16,    // 深色图标起始 ID
    pub frame_count: u32,     // 该皮肤的动画帧数
}

impl Skin {
    /// 根据主题和帧索引，解析出实际的资源 ID
    pub fn resolve_icon_id(&self, theme: ThemeMode, frame: u32) -> u16 {
        let is_dark = match theme {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::FollowSystem => crate::utils::is_time_window_in_dark_mode(),
        };

        let base = if is_dark { self.dark_base_id } else { self.light_base_id };
        // 安全钳制帧索引，防止越界
        let safe_frame = frame.min(self.frame_count.saturating_sub(1));
        base + safe_frame as u16
    }
}

/// 运行时共享的皮肤状态（线程安全）
#[derive(Debug, Clone)]
pub struct SharedSkinState {
    pub skin: &'static Skin,
    pub theme: ThemeMode,
}

impl SharedSkinState {
    pub fn new(skin: &'static Skin, theme: ThemeMode) -> Self {
        Self { skin, theme }
    }

    /// 获取当前有效的主题模式（如果是 FollowSystem 则查询注册表）
    pub fn effective_theme(&self) -> ThemeMode {
        match self.theme {
            ThemeMode::FollowSystem => {
                if crate::utils::is_time_window_in_dark_mode() {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                }
            }
            other => other,
        }
    }

    /// 根据当前状态解析图标资源 ID
    pub fn resolve_icon_id(&self, frame: u32) -> u16 {
        self.skin.resolve_icon_id(self.effective_theme(), frame)
    }
}

pub type DynSkinState = Arc<Mutex<SharedSkinState>>;

pub const SKIN_RUNCAT: Skin = Skin {
    name: "RunCat",
    light_base_id: 116,  // light_cat_0 ~ light_cat_4
    dark_base_id: 111,   // dark_cat_0 ~ dark_cat_4
    frame_count: 5,
};

pub const SKIN_BUBBLE_KITTEN: Skin = Skin {
    name: "BubbleKitten",
    light_base_id: 101,  // BubbleKitten_0 ~ BubbleKitten_9
    dark_base_id: 101,   // ⚠️ 注意：rc 中 BubbleKitten 没有深色版，这里指向同一组作为 fallback
    frame_count: 10,
};