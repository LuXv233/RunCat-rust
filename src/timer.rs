// timer.rs 时间显示模块：支持编辑模式拖动与彩虹炫彩皮肤

use winapi::shared::minwindef::{HINSTANCE, LRESULT, UINT, WPARAM, LPARAM, FALSE};
use winapi::shared::basetsd::UINT_PTR;
use winapi::shared::windef::{HWND, HDC, RECT, HFONT, POINT, SIZE};
use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};
use std::ptr::null_mut;
use std::mem;

use crate::utils::{to_wide_null, load_cursor, load_icon, get_windows_time};

const WINDOW_CLASS_NAME: &str = "TimeDisplayWindow";
const WINDOW_TITLE: &str = "时间显示";
const WINDOW_WIDTH: i32 = 300;
const WINDOW_HEIGHT: i32 = 100;
const TIMER_ID: UINT_PTR = 1;
const ANIMATION_INTERVAL: u32 = 16; // ~60FPS

static mut G_CLASS_REGISTERED: bool = false;

/// 获取高精度时间（微秒），用于丝滑颜色动画
fn query_performance_counter_us() -> u64 {
    unsafe {
        let mut counter: i64 = 0;
        let mut frequency: i64 = 0;
        QueryPerformanceCounter(&mut counter as *mut i64 as *mut _);
        QueryPerformanceFrequency(&mut frequency as *mut i64 as *mut _);
        // 转换为微秒
        (counter as u64 * 1_000_000) / (frequency as u64)
    }
}

/// 时间窗口内部状态
struct TimeWindowState {
    hwnd: HWND,
    visible: bool,
    edit_mode: bool,
    last_pos: Option<(i32, i32)>, // 记忆最后位置
}

// 使用 Option 包装，避免 static mut 的未初始化 UB
static mut G_TIME_STATE: Option<TimeWindowState> = None;

/// 安全地访问并修改时间窗口状态
fn with_time_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut TimeWindowState) -> R,
{
    unsafe {
        let state_ptr = std::ptr::addr_of_mut!(G_TIME_STATE);
        (*state_ptr).as_mut().map(f)
    }
}

// ==================== 公共 API ====================

pub fn create_time_window(hinstance: HINSTANCE) -> HWND {
    // 如果窗口已存在，直接显示
    if let Some(hwnd) = with_time_state(|s| {
        if !s.hwnd.is_null() && s.visible {
            unsafe { ShowWindow(s.hwnd, SW_SHOW); }
            Some(s.hwnd)
        } else {
            None
        }
    }).flatten() {
        return hwnd;
    }

    let class_name_w = to_wide_null(WINDOW_CLASS_NAME);
    let window_title_w = to_wide_null(WINDOW_TITLE);

    // 只注册一次窗口类
    if !unsafe { G_CLASS_REGISTERED } {
        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(time_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: load_icon(hinstance, crate::constants::IDI_APP_ICON),
            hCursor: load_cursor() as *mut _,
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name_w.as_ptr(),
            hIconSm: load_icon(hinstance, crate::constants::IDI_APP_ICON),
        };

        if unsafe { RegisterClassExW(&wc) } == 0 {
            eprintln!("注册时间窗口类失败");
            return null_mut();
        }
        unsafe { G_CLASS_REGISTERED = true; }
    }

    // 使用保存的位置、记忆的位置或居中计算
    let pos = crate::utils::load_time_window_pos()
        .or_else(|| with_time_state(|s| s.last_pos).flatten())
        .unwrap_or_else(|| {
            let mut screen_rect: RECT = unsafe { mem::zeroed() };
            unsafe { GetWindowRect(GetDesktopWindow(), &mut screen_rect); }
            let x = (screen_rect.right - screen_rect.left - WINDOW_WIDTH) / 2;
            let y = (screen_rect.bottom - screen_rect.top - WINDOW_HEIGHT) / 2;
            (x, y)
        });

    // ✅ 初始创建带 WS_EX_TRANSPARENT，完全穿透
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,
            class_name_w.as_ptr(),
            window_title_w.as_ptr(),
            WS_POPUP,
            pos.0, pos.1, WINDOW_WIDTH, WINDOW_HEIGHT,
            null_mut(), null_mut(), hinstance, null_mut(),
        )
    };

    if hwnd.is_null() {
        eprintln!("创建时间窗口失败");
        return null_mut();
    }

    unsafe {
        G_TIME_STATE = Some(TimeWindowState {
            hwnd,
            visible: true,
            edit_mode: false,
            last_pos: Some(pos),
        });

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        // 提高定时器精度以支撑流畅动画
        SetTimer(hwnd, TIMER_ID, ANIMATION_INTERVAL, None);
    }

    hwnd
}

pub fn close_time_window() {
    with_time_state(|s| {
        s.visible = false;
        s.edit_mode = false;
        unsafe { ShowWindow(s.hwnd, SW_HIDE); }
    });
}

pub fn is_time_window_visible() -> bool {
    with_time_state(|s| s.visible).unwrap_or(false)
}

pub fn is_edit_mode() -> bool {
    with_time_state(|s| s.edit_mode).unwrap_or(false)
}

/// 切换编辑模式：动态增删 WS_EX_TRANSPARENT
pub fn toggle_edit_mode() {
    with_time_state(|state| {
        state.edit_mode = !state.edit_mode;

        unsafe {
            let ex_style = GetWindowLongPtrW(state.hwnd, GWL_EXSTYLE) as u32;
            let new_style = if state.edit_mode {
                ex_style & !WS_EX_TRANSPARENT
            } else {
                ex_style | WS_EX_TRANSPARENT
            };
            SetWindowLongPtrW(state.hwnd, GWL_EXSTYLE, new_style as isize);

            // ⚠️ 修改扩展样式后必须带 SWP_FRAMECHANGED 才能立即生效
            SetWindowPos(
                state.hwnd,
                null_mut(),
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );

            // 切换后立即重绘以显示/隐藏编辑边框
            InvalidateRect(state.hwnd, null_mut(), FALSE);
        }
    });
}

/// 重绘时间窗口（主题/皮肤切换时调用，不闪烁）
pub fn redraw_time_window() {
    with_time_state(|s| {
        if !s.hwnd.is_null() {
            unsafe { InvalidateRect(s.hwnd, null_mut(), FALSE); }
        }
    });
}

/// 重新创建时间窗口（主题切换时调用）
pub fn recreate_time_window() {
    let old_hwnd = with_time_state(|s| {
        let hwnd = s.hwnd;
        s.hwnd = null_mut();
        s.visible = false;
        s.edit_mode = false;
        hwnd
    }).unwrap_or(null_mut());

    if !old_hwnd.is_null() {
        unsafe { DestroyWindow(old_hwnd); }
    }

    let hinstance = crate::utils::get_module_handle();
    let new_hwnd = create_time_window(hinstance);
    if new_hwnd.is_null() {
        eprintln!("重新创建时间窗口失败");
    }
}

// ==================== 窗口过程 ====================

unsafe extern "system" fn time_window_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        // ✅ 双模式命中测试
        WM_NCHITTEST => {
            let is_edit = with_time_state(|s| s.edit_mode).unwrap_or(false);
            if !is_edit {
                HTTRANSPARENT as LRESULT
            } else {
                HTCAPTION as LRESULT
            }
        }
        
        // 监听窗口移动消息，记录新位置并保存到注册表
        WM_MOVE => {
            let x = lparam as i32 & 0xFFFF;
            let y = (lparam >> 16) as i32;
            with_time_state(|s| {
                s.last_pos = Some((x, y));
            });
            crate::utils::save_time_window_pos(x, y);
            0
        }

        WM_PAINT => {
            paint_time_window(hwnd);
            0
        }

        WM_ERASEBKGND => 1, // 跳过背景擦除，防止闪烁

        WM_TIMER => {
            InvalidateRect(hwnd, null_mut(), FALSE);
            0
        }

        WM_DESTROY => {
            KillTimer(hwnd, TIMER_ID);
            G_TIME_STATE = None;
            0
        }

        WM_CLOSE => {
            with_time_state(|s| {
                s.visible = false;
                s.edit_mode = false;
            });
            ShowWindow(hwnd, SW_HIDE);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ==================== 渲染引擎 ====================

unsafe fn paint_time_window(hwnd: HWND) {
    let mut ps: PAINTSTRUCT = mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);
    if hdc.is_null() {
        return;
    }

    let mut rect: RECT = mem::zeroed();
    GetClientRect(hwnd, &mut rect);
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        EndPaint(hwnd, &ps);
        return;
    }

    // 双缓冲准备
    let mem_dc = CreateCompatibleDC(hdc);
    let mut bmi: BITMAPINFO = mem::zeroed();
    bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height; // top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB as u32;

    let mut bits: *mut ::std::ffi::c_void = null_mut();
    let hbitmap = CreateDIBSection(
        mem_dc, &bmi, DIB_RGB_COLORS,
        &mut bits as *mut _ as *mut _,
        null_mut(), 0,
    );
    let old_bitmap = SelectObject(mem_dc, hbitmap as *mut _);

    // 清零 DIB
    if !bits.is_null() {
        let total = (width as usize) * (height as usize) * 4;
        std::ptr::write_bytes(bits as *mut u8, 0, total);
    }

    // ✅ 编辑模式下：填充整个客户区最小 alpha + 绘制虚线边框
    let is_edit = with_time_state(|s| s.edit_mode).unwrap_or(false);
    if is_edit && !bits.is_null() {
        // 先填充整个客户区最小 alpha，使全部区域可点击拖动
        fill_entire_hit_area(bits as *mut u8, width as usize, height as usize);
        draw_edit_border(bits as *mut u8, width as usize, height as usize);
    }

    // ✅ 丝滑彩虹渐变 - 逐字符流动效果
    let (_, _, _, hour, minute, second, _, _) = get_windows_time();
    let time_str = format!("{:02}:{:02}:{:02}", hour, minute, second);

    // 使用高精度计时器
    let current_time_us = query_performance_counter_us();

    // 色相旋转：每秒 8°，更慢更柔和
    let hue_speed = 8.0_f64 / 1_000_000.0; // 度/微秒
    let base_hue = (current_time_us as f64 * hue_speed) % 360.0;

    // 绘制文字 - 逐字符彩虹渐变
    let font = create_large_font(mem_dc, 60);
    let old_font = SelectObject(mem_dc, font as *mut _);
    SetBkMode(mem_dc, TRANSPARENT as i32);

    let chars: Vec<char> = time_str.chars().collect();
    let char_hue_offset = 8.0_f64; // 字符间色相偏移，更小跳跃更小

    let mut total_width = 0;
    let mut char_widths = Vec::with_capacity(chars.len());
    for c in &chars {
        let c_str = c.to_string();
        let mut c_rect = rect;
        DrawTextW(
            mem_dc,
            to_wide_null(&c_str).as_ptr(),
            -1,
            &mut c_rect,
            DT_CALCRECT | DT_SINGLELINE,
        );
        let w = c_rect.right - c_rect.left;
        char_widths.push(w);
        total_width += w;
    }

    let start_x = (width - total_width) / 2;
    let mid_y = height / 2;

    for (i, (c, &w)) in chars.iter().zip(char_widths.iter()).enumerate() {
        let char_hue = (base_hue + i as f64 * char_hue_offset) % 360.0;
        let (r, g, b) = hsv_to_rgb(char_hue as f32, 0.85, 0.95);
        let color_ref = (b as u32) << 16 | (g as u32) << 8 | (r as u32);
        SetTextColor(mem_dc, color_ref);

        let c_str = c.to_string();
        let mut c_rect = RECT {
            left: start_x + char_widths[..i].iter().sum::<i32>(),
            top: mid_y - 30,
            right: start_x + char_widths[..=i].iter().sum::<i32>(),
            bottom: mid_y + 30,
        };
        DrawTextW(
            mem_dc,
            to_wide_null(&c_str).as_ptr(),
            -1,
            &mut c_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
    }

    SelectObject(mem_dc, old_font);
    DeleteObject(font as *mut _);

    // ✅ Alpha 通道生成：文字像素取 RGB 最大值作为 Alpha
    if !bits.is_null() {
        let px = bits as *mut u8;
        let pixels = (width as usize) * (height as usize);
        for i in 0..pixels {
            let off = i * 4;
            let b_val = *px.add(off);
            let g_val = *px.add(off + 1);
            let r_val = *px.add(off + 2);
            let a = r_val.max(g_val).max(b_val);
            // 编辑边框已有 alpha，取较大值避免被覆盖
            let existing_a = *px.add(off + 3);
            *px.add(off + 3) = a.max(existing_a);
        }
    }

    // 提交分层窗口
    let pt_src = POINT { x: 0, y: 0 };
    let size = SIZE { cx: width, cy: height };
    let mut blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA,
    };
    UpdateLayeredWindow(
        hwnd, null_mut(),
        null_mut(),
        &size as *const SIZE as *mut SIZE,
        mem_dc,
        &pt_src as *const POINT as *mut POINT,
        0, &mut blend, ULW_ALPHA,
    );

    // 清理 GDI 资源
    SelectObject(mem_dc, old_bitmap);
    DeleteObject(hbitmap as *mut _);
    DeleteDC(mem_dc);
    EndPaint(hwnd, &ps);
}

/// 绘制编辑模式下的虚线边框（直接在 DIB 上操作）
unsafe fn draw_edit_border(buf: *mut u8, width: usize, height: usize) {
    let border_color: [u8; 4] = [200, 200, 200, 120]; // BGRA, 半透明白色
    let dash_len = 8usize;

    let set_pixel = |buf: *mut u8, x: usize, y: usize, w: usize| {
        if x < w && y < height {
            let off = (y * w + x) * 4;
            unsafe {
                // Alpha 混合
                let src_a = border_color[3] as f32 / 255.0;
                let dst_a = *buf.add(off + 3) as f32 / 255.0;
                let out_a = src_a + dst_a * (1.0 - src_a);
                if out_a > 0.0 {
                    for c in 0..3 {
                        let src = border_color[c] as f32;
                        let dst = *buf.add(off + c) as f32;
                        *buf.add(off + c) = ((src * src_a + dst * dst_a * (1.0 - src_a)) / out_a) as u8;
                    }
                    *buf.add(off + 3) = (out_a * 255.0) as u8;
                }
            }
        }
    };

    // 上边 & 下边
    for x in 0..width {
        if (x / dash_len) % 2 == 0 {
            set_pixel(buf, x, 0, width);
            set_pixel(buf, x, height.saturating_sub(1), width);
        }
    }
    // 左边 & 右边
    for y in 0..height {
        if (y / dash_len) % 2 == 0 {
            set_pixel(buf, 0, y, width);
            set_pixel(buf, width.saturating_sub(1), y, width);
        }
    }
}

/// 填充整个客户区的 alpha，使分层窗口全部区域可点击拖动
unsafe fn fill_entire_hit_area(buf: *mut u8, width: usize, height: usize) {
    let min_alpha: u8 = 1;
    let pixels = width * height;
    for i in 0..pixels {
        let off = i * 4;
        if *buf.add(off + 3) < min_alpha {
            *buf.add(off + 3) = min_alpha;
        }
    }
}

unsafe fn create_large_font(_hdc: HDC, size: i32) -> HFONT {
    unsafe {
        CreateFontW(
            -size, 0, 0, 0,
            FW_BOLD,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            FF_DONTCARE,
            to_wide_null("微软雅黑").as_ptr(),
        )
    }
}

unsafe fn calculate_centered_rect(hdc: HDC, window_rect: &RECT, font: HFONT, text: &str) -> RECT {
    let mut rect = *window_rect;
    unsafe {
        let old_font = SelectObject(hdc, font as *mut _);

        DrawTextW(
            hdc,
            to_wide_null(text).as_ptr(),
            -1,
            &mut rect,
            DT_CALCRECT | DT_SINGLELINE,
        );

        SelectObject(hdc, old_font);
    }

    let tw = rect.right - rect.left;
    let th = rect.bottom - rect.top;
    let ww = window_rect.right - window_rect.left;
    let wh = window_rect.bottom - window_rect.top;

    rect.left = (ww - tw) / 2;
    rect.top = (wh - th) / 2;
    rect.right = rect.left + tw;
    rect.bottom = rect.top + th;
    rect
}

/// HSV → RGB 颜色转换
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}