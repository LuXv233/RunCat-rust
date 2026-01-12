// 屏幕绘制模块

use winapi::shared::minwindef::{HINSTANCE, LRESULT, UINT, WPARAM, LPARAM, TRUE, FALSE};
use winapi::shared::basetsd::UINT_PTR;
use winapi::shared::windef::{HWND, HDC, RECT, HFONT, POINT, SIZE};
use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use std::ptr::null_mut;
use std::mem;

use crate::utils::{to_wide_null, load_cursor, load_icon, get_windows_time};

const WINDOW_CLASS_NAME: &str = "TimeDisplayWindow";
const WINDOW_TITLE: &str = "时间显示";
const WINDOW_WIDTH: i32 = 300;
const WINDOW_HEIGHT: i32 = 100;
const TIMER_ID: UINT_PTR = 1;
const TIMER_INTERVAL: u32 = 1000;

// 全局变量
static mut G_HWND: Option<HWND> = None;
static mut G_TIME_WINDOW_VISIBLE: bool = false;

// 创建并显示时间显示窗口
pub fn create_time_window(hinstance: HINSTANCE) -> HWND {
    unsafe {
        if let Some(hwnd) = G_HWND {
            ShowWindow(hwnd, SW_SHOW);
            G_TIME_WINDOW_VISIBLE = true;
            return hwnd;
        }
    }
    
    let class_name_w = to_wide_null(WINDOW_CLASS_NAME);
    let window_title_w = to_wide_null(WINDOW_TITLE);
    let wc = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(time_window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: load_icon(hinstance, crate::constants::IDI_APP_ICON),
        hCursor: load_cursor() as *mut _,
        hbrBackground: null_mut(), // 透明背景，不使用画刷
        lpszMenuName: null_mut(),
        lpszClassName: class_name_w.as_ptr(),
        hIconSm: load_icon(hinstance, crate::constants::IDI_APP_ICON),
    };

    if unsafe { RegisterClassExW(&wc) } == 0 {
        eprintln!("注册窗口类失败");
        return null_mut();
    }
    
    let mut screen_rect: RECT = unsafe { mem::zeroed() };
    unsafe {
        GetWindowRect(GetDesktopWindow(), &mut screen_rect);
    }
    let screen_width = screen_rect.right - screen_rect.left;
    let screen_height = screen_rect.bottom - screen_rect.top;
    let x = (screen_width - WINDOW_WIDTH) / 2;
    let y = (screen_height - WINDOW_HEIGHT) / 2;
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,  // 扩展样式：分层、最前、透明、工具窗口
            class_name_w.as_ptr(),           // 窗口类名
            window_title_w.as_ptr(),         // 窗口标题
            WS_POPUP,                       // 窗口样式：弹出窗口
            x,                               // 水平位置
            y,                               // 垂直位置
            WINDOW_WIDTH,                     // 窗口宽度
            WINDOW_HEIGHT,                    // 窗口高度
            null_mut(),                      // 父窗口句柄
            null_mut(),                      // 菜单句柄
            hinstance,                       // 实例句柄
            null_mut(),                      // 创建参数
        )
    };
    
    if hwnd.is_null() {
        eprintln!("创建窗口失败");
        return null_mut();
    }
    
    unsafe {
        G_HWND = Some(hwnd);
        G_TIME_WINDOW_VISIBLE = true;
    }
    
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        SetTimer(hwnd, TIMER_ID, TIMER_INTERVAL, None);
    }
    
    hwnd
}

unsafe extern "system" fn time_window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            paint_time_window(hwnd);
            0
        }
        
        WM_ERASEBKGND => {
            1
        }
        
        WM_TIMER => {
            InvalidateRect(hwnd, null_mut(), TRUE);
            0
        }
        
        WM_DESTROY => {
            KillTimer(hwnd, TIMER_ID);
            
            G_HWND = None;
            G_TIME_WINDOW_VISIBLE = false;
            PostQuitMessage(0);
            0
        }
        
        WM_CLOSE => {
            ShowWindow(hwnd, SW_HIDE);
            G_TIME_WINDOW_VISIBLE = false;
            0
        }
        
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

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

    let (_, _, _, hour, minute, second, _, _) = get_windows_time();
    let time_str = format!("{:02}:{:02}:{:02}", hour, minute, second);
    let mem_dc = CreateCompatibleDC(hdc);
    let mut bmi: BITMAPINFO = mem::zeroed();
    bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height; // 负值 => top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB as u32;

    let mut bits: *mut ::std::ffi::c_void = null_mut();
    let hbitmap = CreateDIBSection(mem_dc, &mut bmi, DIB_RGB_COLORS, &mut bits as *mut _ as *mut _, null_mut(), 0);
    let old_bitmap = SelectObject(mem_dc, hbitmap as *mut _);

    if !bits.is_null() {
        let buf = bits as *mut u8;
        let total = (width as usize * height as usize) * 4;
        for i in 0..total {
            *buf.add(i) = 0;
        }
    }

    let font = create_large_font(mem_dc, 60);
    let old_font = SelectObject(mem_dc, font as *mut _);
    SetTextColor(mem_dc, 0x00FFFFFF);
    SetBkMode(mem_dc, TRANSPARENT as i32);
    let mut text_rect = calculate_centered_rect(mem_dc, &rect, font, &time_str);
    DrawTextW(
        mem_dc,
        to_wide_null(&time_str).as_ptr(),
        -1,
        &mut text_rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );
    SelectObject(mem_dc, old_font);
    DeleteObject(font as *mut _);

    if !bits.is_null() {
        let px = bits as *mut u8;
        let pixels = (width as usize * height as usize) as usize;
        for i in 0..pixels {
            let off = i * 4;
            let b = *px.add(off);
            let g = *px.add(off + 1);
            let r = *px.add(off + 2);
            let a = r.max(g).max(b);
            *px.add(off + 3) = a;
        }
    }

    let pt_src = POINT { x: 0, y: 0 };
    let pt_dst = POINT { x: rect.left, y: rect.top };
    let size = SIZE { cx: width, cy: height };
    let mut blend: BLENDFUNCTION = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA,
    };
    UpdateLayeredWindow(
        hwnd,
        null_mut(),
        &pt_dst as *const POINT as *mut POINT,
        &size as *const SIZE as *mut SIZE,
        mem_dc,
        &pt_src as *const POINT as *mut POINT,
        0,
        &mut blend,
        ULW_ALPHA,
    );

    SelectObject(mem_dc, old_bitmap);
    DeleteObject(hbitmap as *mut _);
    DeleteDC(mem_dc);
    EndPaint(hwnd, &ps);
}

unsafe fn create_large_font(_hdc: HDC, size: i32) -> HFONT {
    CreateFontW(
        -size,                             // 字体高度（负值表示字符高度）
        0,                                 // 字体宽度（0表示使用默认宽度）
        0,                                 // 文本倾斜角度
        0,                                 // 字符倾斜角度
        FW_BOLD,                           // 字体粗细（粗体）
        FALSE as u32,                       // 斜体
        FALSE as u32,                       // 下划线
        FALSE as u32,                       // 删除线
        DEFAULT_CHARSET,                    // 字符集
        OUT_DEFAULT_PRECIS,                 // 输出精度
        CLIP_DEFAULT_PRECIS,               // 裁剪精度
        ANTIALIASED_QUALITY,                // 输出质量（灰度抗锯齿，避免 ClearType 彩色边缘）
        FF_DONTCARE,                       // 字体族
        to_wide_null("微软雅黑").as_ptr(),    // 字体名称
    )
}

unsafe fn calculate_centered_rect(hdc: HDC, window_rect: &RECT, font: HFONT, text: &str) -> RECT {
    let mut rect = *window_rect;

    let old_font = SelectObject(hdc, font as *mut _);

    DrawTextW(
        hdc,
        to_wide_null(text).as_ptr(),
        -1,
        &mut rect,
        DT_CALCRECT | DT_SINGLELINE,
    );

    SelectObject(hdc, old_font);

    let text_width = rect.right - rect.left;
    let text_height = rect.bottom - rect.top;
    let window_width = window_rect.right - window_rect.left;
    let window_height = window_rect.bottom - window_rect.top;

    rect.left = (window_width - text_width) / 2;
    rect.top = (window_height - text_height) / 2;
    rect.right = rect.left + text_width;
    rect.bottom = rect.top + text_height;

    rect
}

pub fn close_time_window() {
    unsafe {
        if let Some(hwnd) = G_HWND {
            ShowWindow(hwnd, SW_HIDE);
            G_TIME_WINDOW_VISIBLE = false;
        }
    }
}

pub fn is_time_window_visible() -> bool {
    unsafe { G_TIME_WINDOW_VISIBLE }
}
