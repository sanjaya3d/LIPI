//! Windows only. The main window and the "Show Logs" popup.
//!
//! The main window has its own title bar, cards, buttons and icons, all drawn by hand
//! with only the five colours in `theme.rs` (black, lime, white and two greys). No image files, no extra libraries.
//! Edges are made smooth by drawing each shape three times bigger in a small hidden
//! bitmap and shrinking it, so the program still uses very little memory.
//! The only other window is the log popup.
//! The Win32 functions are declared by hand below.

use std::collections::VecDeque;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, AtomicU32, AtomicU8, Ordering::Relaxed};
use std::sync::Mutex;

use crate::about;
use crate::hints::{self, Tab};
use crate::icons::{self, Icon, Ink, Piece};
use crate::layout::{Hit, Layout, Rect, View};
use crate::legacy::LegacyMap;
use crate::settings;
use crate::state;
use crate::theme::{ACCENT, BG, EDGE, MUTED, TEXT};

// ---------------------------------------------------------------- Win32 bits

const WM_DESTROY: u32 = 0x0002;
const WM_SIZE: u32 = 0x0005;
const WM_PAINT: u32 = 0x000F;
const WM_CLOSE: u32 = 0x0010;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_SETCURSOR: u32 = 0x0020;
const WM_SETFONT: u32 = 0x0030;
const WM_SETICON: u32 = 0x0080;
const WM_NCHITTEST: u32 = 0x0084;
const WM_KEYDOWN: u32 = 0x0100;
const WM_TIMER: u32 = 0x0113;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_RBUTTONUP: u32 = 0x0205;
/// Message the tray icon sends to the main window (WM_APP + 1).
const WM_TRAY: u32 = 0x8001;
const SIZE_MINIMIZED: usize = 1;
const SETICON_SMALL: usize = 0;
const SETICON_BIG: usize = 1;
const VK_ESCAPE: usize = 0x1B;
const HTCLIENT: isize = 1;
const HTCAPTION: isize = 2;

const NIM_ADD: u32 = 0;
const NIM_MODIFY: u32 = 1;
const NIM_DELETE: u32 = 2;
const NIF_MESSAGE: u32 = 0x1;
const NIF_ICON: u32 = 0x2;
const NIF_TIP: u32 = 0x4;

const EM_SETSEL: u32 = 0x00B1;
const EM_SCROLLCARET: u32 = 0x00B7;
const EM_SETLIMITTEXT: u32 = 0x00C5;

const WS_POPUP: u32 = 0x8000_0000;
const WS_CHILD: u32 = 0x4000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_VSCROLL: u32 = 0x0020_0000;
const WS_SYSMENU: u32 = 0x0008_0000;
const WS_MINIMIZEBOX: u32 = 0x0002_0000;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
const ES_MULTILINE: u32 = 0x0004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_READONLY: u32 = 0x0800;

const CW_USEDEFAULT: i32 = i32::MIN;
const SW_HIDE: i32 = 0;
const SW_SHOWNORMAL: i32 = 1;
const SW_SHOW: i32 = 5;
const SW_MINIMIZE: i32 = 6;
const SW_RESTORE: i32 = 9;
const IDC_ARROW: usize = 32512;
const IDC_HAND: usize = 32649;
const IDI_APPLICATION: usize = 32512;
const LOGPIXELSX: i32 = 88;
const SM_CXICON: i32 = 11;
const SM_CXSMICON: i32 = 49;

const DT_LEFT: u32 = 0x0000;
const DT_CENTER: u32 = 0x0001;
const DT_RIGHT: u32 = 0x0002;
const DT_VCENTER: u32 = 0x0004;
const DT_WORDBREAK: u32 = 0x0010;
const DT_SINGLELINE: u32 = 0x0020;
const DT_NOPREFIX: u32 = 0x0800;
const DT_END_ELLIPSIS: u32 = 0x8000;

const SRCCOPY: u32 = 0x00CC_0020;
const WHITENESS: u32 = 0x00FF_0062;
const BLACKNESS: u32 = 0x0000_0042;
const HALFTONE: i32 = 4;
const TRANSPARENT: i32 = 1;
const NULL_BRUSH: i32 = 5;
const NULL_PEN: i32 = 8;
const PS_SOLID: i32 = 0;
const PS_INSIDEFRAME: i32 = 6;
const TA_TOP: u32 = 0;
const TA_BASELINE: u32 = 24;

const COINIT_APARTMENTTHREADED: u32 = 0x2;
const COINIT_DISABLE_OLE1DDE: u32 = 0x4;

type WndProc = unsafe extern "system" fn(isize, u32, usize, isize) -> isize;

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Size {
    cx: i32,
    cy: i32,
}

#[repr(C)]
struct Msg {
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: Point,
}

#[repr(C)]
struct PaintStruct {
    hdc: isize,
    erase: i32,
    paint: Rect,
    restore: i32,
    update: i32,
    reserved: [u8; 32],
}

#[repr(C)]
struct WndClassExW {
    size: u32,
    style: u32,
    wnd_proc: Option<WndProc>,
    class_extra: i32,
    window_extra: i32,
    instance: isize,
    icon: isize,
    cursor: isize,
    background: isize,
    menu_name: *const u16,
    class_name: *const u16,
    icon_small: isize,
}

/// Font measurements (TEXTMETRICW). Only `ascent` and `descent` are used.
#[repr(C)]
struct TextMetricW {
    height: i32,
    ascent: i32,
    descent: i32,
    internal_leading: i32,
    external_leading: i32,
    ave_char_width: i32,
    max_char_width: i32,
    weight: i32,
    overhang: i32,
    digitized_aspect_x: i32,
    digitized_aspect_y: i32,
    first_char: u16,
    last_char: u16,
    default_char: u16,
    break_char: u16,
    italic: u8,
    underlined: u8,
    struck_out: u8,
    pitch_and_family: u8,
    charset: u8,
}

/// Describes the tray icon to Windows (NOTIFYICONDATAW).
#[repr(C)]
struct NotifyIconData {
    size: u32,
    hwnd: isize,
    id: u32,
    flags: u32,
    callback_message: u32,
    icon: isize,
    tip: [u16; 128],
    item_state: u32,
    item_state_mask: u32,
    info: [u16; 256],
    timeout_or_version: u32,
    info_title: [u16; 64],
    info_flags: u32,
    guid: [u32; 4],
    balloon_icon: isize,
}

/// Describes an icon to be made from two bitmaps (ICONINFO).
#[repr(C)]
struct IconInfo {
    is_icon: i32,
    hotspot_x: u32,
    hotspot_y: u32,
    mask: isize,
    colour: isize,
}

// Checked while compiling. These are the sizes Windows expects.
const _: () = {
    let wide = size_of::<usize>() == 8;
    assert!(size_of::<Rect>() == 16);
    assert!(size_of::<Point>() == 8);
    assert!(size_of::<Size>() == 8);
    assert!(size_of::<TextMetricW>() == 60);
    assert!(size_of::<Msg>() == if wide { 48 } else { 28 });
    assert!(size_of::<PaintStruct>() == if wide { 72 } else { 64 });
    assert!(size_of::<WndClassExW>() == if wide { 80 } else { 48 });
    assert!(size_of::<NotifyIconData>() == if wide { 976 } else { 956 });
    assert!(size_of::<IconInfo>() == if wide { 32 } else { 20 });
};

#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(class: *const WndClassExW) -> u16;
    fn CreateWindowExW(
        ex_style: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: isize,
        menu: isize,
        instance: isize,
        param: *const c_void,
    ) -> isize;
    fn DefWindowProcW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn ShowWindow(hwnd: isize, cmd: i32) -> i32;
    fn UpdateWindow(hwnd: isize) -> i32;
    fn IsWindow(hwnd: isize) -> i32;
    fn PostQuitMessage(code: i32);
    fn GetMessageW(msg: *mut Msg, hwnd: isize, filter_min: u32, filter_max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn BeginPaint(hwnd: isize, ps: *mut PaintStruct) -> isize;
    fn EndPaint(hwnd: isize, ps: *const PaintStruct) -> i32;
    fn GetClientRect(hwnd: isize, rect: *mut Rect) -> i32;
    fn InvalidateRect(hwnd: isize, rect: *const Rect, erase: i32) -> i32;
    fn SetTimer(hwnd: isize, id: usize, millis: u32, proc_: *const c_void) -> usize;
    fn KillTimer(hwnd: isize, id: usize) -> i32;
    fn LoadCursorW(instance: isize, name: *const u16) -> isize;
    fn SetCursor(cursor: isize) -> isize;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn ScreenToClient(hwnd: isize, point: *mut Point) -> i32;
    fn GetDC(hwnd: isize) -> isize;
    fn ReleaseDC(hwnd: isize, hdc: isize) -> i32;
    fn FillRect(hdc: isize, rect: *const Rect, brush: isize) -> i32;
    fn DrawTextW(hdc: isize, text: *const u16, len: i32, rect: *mut Rect, format: u32) -> i32;
    fn SetForegroundWindow(hwnd: isize) -> i32;
    fn SendMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn SetWindowTextW(hwnd: isize, text: *const u16) -> i32;
    fn GetWindowTextLengthW(hwnd: isize) -> i32;
    fn MoveWindow(hwnd: isize, x: i32, y: i32, width: i32, height: i32, repaint: i32) -> i32;
    fn SetWindowRgn(hwnd: isize, region: isize, redraw: i32) -> i32;
    fn SetProcessDPIAware() -> i32;
    fn GetSystemMetrics(index: i32) -> i32;
    fn CreateIconIndirect(info: *const IconInfo) -> isize;
    fn LoadIconW(instance: isize, name: *const u16) -> isize;
    fn RegisterWindowMessageW(name: *const u16) -> u32;
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(color: u32) -> isize;
    fn CreatePen(style: i32, width: i32, color: u32) -> isize;
    fn DeleteObject(object: isize) -> i32;
    fn SelectObject(hdc: isize, object: isize) -> isize;
    fn GetStockObject(index: i32) -> isize;
    fn CreateCompatibleDC(hdc: isize) -> isize;
    fn CreateCompatibleBitmap(hdc: isize, width: i32, height: i32) -> isize;
    fn CreateBitmap(width: i32, height: i32, planes: u32, bits_per_pixel: u32, bits: *const c_void) -> isize;
    fn CreateRoundRectRgn(left: i32, top: i32, right: i32, bottom: i32, corner_w: i32, corner_h: i32) -> isize;
    fn DeleteDC(hdc: isize) -> i32;
    fn BitBlt(dst: isize, x: i32, y: i32, w: i32, h: i32, src: isize, sx: i32, sy: i32, rop: u32) -> i32;
    fn StretchBlt(
        dst: isize,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        src: isize,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        rop: u32,
    ) -> i32;
    fn PatBlt(hdc: isize, x: i32, y: i32, width: i32, height: i32, rop: u32) -> i32;
    fn SetStretchBltMode(hdc: isize, mode: i32) -> i32;
    fn SetBrushOrgEx(hdc: isize, x: i32, y: i32, old: *mut Point) -> i32;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike_out: u32,
        charset: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face: *const u16,
    ) -> isize;
    fn SetTextColor(hdc: isize, color: u32) -> u32;
    fn SetBkColor(hdc: isize, color: u32) -> u32;
    fn SetBkMode(hdc: isize, mode: i32) -> i32;
    fn SetTextAlign(hdc: isize, mode: u32) -> u32;
    fn TextOutW(hdc: isize, x: i32, y: i32, text: *const u16, len: i32) -> i32;
    fn GetTextExtentPoint32W(hdc: isize, text: *const u16, len: i32, size: *mut Size) -> i32;
    fn GetTextMetricsW(hdc: isize, metrics: *mut TextMetricW) -> i32;
    fn RoundRect(hdc: isize, left: i32, top: i32, right: i32, bottom: i32, corner_w: i32, corner_h: i32) -> i32;
    fn Ellipse(hdc: isize, left: i32, top: i32, right: i32, bottom: i32) -> i32;
    fn Polyline(hdc: isize, points: *const Point, count: i32) -> i32;
    fn Polygon(hdc: isize, points: *const Point, count: i32) -> i32;
    fn GetDeviceCaps(hdc: isize, index: i32) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> isize;
    fn GetCurrentProcess() -> isize;
    fn SetProcessWorkingSetSize(process: isize, minimum: usize, maximum: usize) -> i32;
}

#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(hwnd: isize, attribute: u32, value: *const c_void, size: u32) -> i32;
}

#[link(name = "shell32")]
extern "system" {
    fn Shell_NotifyIconW(message: u32, data: *const NotifyIconData) -> i32;
    fn ShellExecuteW(
        hwnd: isize,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show: i32,
    ) -> isize;
}

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *const c_void, flags: u32) -> i32;
}

// --------------------------------------------------------------------- state

const MAIN_CLASS: &str = "LipiMain";
const LOG_CLASS: &str = "LipiLogs";
const TIMER_ID: usize = 1;
const TRIM_TIMER_ID: usize = 2;
const MAX_LINES: usize = 400;
/// Shapes are drawn this many times bigger in a hidden bitmap, then shrunk, for smooth edges.
const SS: i32 = 3;
const TRAY_ID: u32 = 1;

static DPI: AtomicI32 = AtomicI32::new(96);
static LOG_WINDOW: AtomicIsize = AtomicIsize::new(0);
static LOG_EDIT: AtomicIsize = AtomicIsize::new(0);
static BG_BRUSH: AtomicIsize = AtomicIsize::new(0);
static FONT_TITLE: AtomicIsize = AtomicIsize::new(0);
static FONT_TITLE_SI: AtomicIsize = AtomicIsize::new(0);
static FONT_LABEL: AtomicIsize = AtomicIsize::new(0);
static FONT_SMALL: AtomicIsize = AtomicIsize::new(0);
static FONT_SMALL_BOLD: AtomicIsize = AtomicIsize::new(0);
static FONT_LOGO: AtomicIsize = AtomicIsize::new(0);
static FONT_LOGO_SMALL: AtomicIsize = AtomicIsize::new(0);
static FONT_HEAD: AtomicIsize = AtomicIsize::new(0);
static FONT_HINT: AtomicIsize = AtomicIsize::new(0);
static FONT_HINT_SI: AtomicIsize = AtomicIsize::new(0);
static FONT_NOTE: AtomicIsize = AtomicIsize::new(0);
static FONT_LOG: AtomicIsize = AtomicIsize::new(0);
/// 0 is the main page, 1 is the key hint page.
static VIEW: AtomicU8 = AtomicU8::new(0);
/// The tab showing on the key hint page, counted from 0.
static HINT_TAB: AtomicU8 = AtomicU8::new(0);
static SEEN_ENABLED: AtomicBool = AtomicBool::new(true);
static SEEN_LEGACY: AtomicBool = AtomicBool::new(false);
static ICON_ON: AtomicIsize = AtomicIsize::new(0);
static ICON_OFF: AtomicIsize = AtomicIsize::new(0);
static ICON_LARGE: AtomicIsize = AtomicIsize::new(0);
/// True while the tray icon exists. Without it the window must not hide itself.
static TRAY_ADDED: AtomicBool = AtomicBool::new(false);
/// Windows sends this message when the taskbar is rebuilt (after Explorer restarts).
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
static COM_READY: AtomicBool = AtomicBool::new(false);
static LOG_LINES: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

// ------------------------------------------------------------------- drawing

unsafe fn fill(hdc: isize, rect: Rect, color: u32) {
    let brush = CreateSolidBrush(color);
    FillRect(hdc, &rect, brush);
    DeleteObject(brush);
}

/// A rounded rectangle with an optional fill and an optional border (colour, width).
/// The border is drawn inside the rectangle.
unsafe fn shape(hdc: isize, rect: Rect, radius: i32, fill_colour: Option<u32>, border: Option<(u32, i32)>) {
    let brush = match fill_colour {
        Some(c) => CreateSolidBrush(c),
        None => GetStockObject(NULL_BRUSH),
    };
    let pen = match border {
        Some((c, w)) => CreatePen(PS_INSIDEFRAME, w, c),
        None => GetStockObject(NULL_PEN),
    };
    let old_brush = SelectObject(hdc, brush);
    let old_pen = SelectObject(hdc, pen);
    RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, radius * 2, radius * 2);
    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);
    if fill_colour.is_some() {
        DeleteObject(brush);
    }
    if border.is_some() {
        DeleteObject(pen);
    }
}

/// A solid circle or oval inside `rect`.
unsafe fn disc(hdc: isize, rect: Rect, colour: u32) {
    let brush = CreateSolidBrush(colour);
    let old_brush = SelectObject(hdc, brush);
    let old_pen = SelectObject(hdc, GetStockObject(NULL_PEN));
    Ellipse(hdc, rect.left, rect.top, rect.right, rect.bottom);
    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);
    DeleteObject(brush);
}

/// A line through the points, with round ends and corners.
unsafe fn polyline(hdc: isize, points: &[Point], colour: u32, width: i32) {
    let pen = CreatePen(PS_SOLID, width.max(1), colour);
    let old_pen = SelectObject(hdc, pen);
    Polyline(hdc, points.as_ptr(), points.len() as i32);
    SelectObject(hdc, old_pen);
    DeleteObject(pen);
}

/// A solid shape with straight sides through the points.
unsafe fn polygon(hdc: isize, points: &[Point], colour: u32) {
    let brush = CreateSolidBrush(colour);
    let old_brush = SelectObject(hdc, brush);
    let old_pen = SelectObject(hdc, GetStockObject(NULL_PEN));
    Polygon(hdc, points.as_ptr(), points.len() as i32);
    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);
    DeleteObject(brush);
}

/// Draws something SS times bigger in a small hidden bitmap, then shrinks it onto `dest`.
/// Plain GDI shapes have rough edges. Shrinking a bigger drawing makes them smooth.
/// `draw_big` draws on the hidden bitmap. Its (0, 0) is the top left of `dest`.
/// `background` is the colour already under `dest`, so the edges blend into it.
unsafe fn smooth(hdc: isize, dest: Rect, background: u32, draw_big: impl FnOnce(isize)) {
    let (w, h) = (dest.width(), dest.height());
    if w <= 0 || h <= 0 {
        return;
    }
    let memory = CreateCompatibleDC(hdc);
    let bitmap = CreateCompatibleBitmap(hdc, w * SS, h * SS);
    if memory != 0 && bitmap != 0 {
        let old = SelectObject(memory, bitmap);
        fill(memory, Rect { left: 0, top: 0, right: w * SS, bottom: h * SS }, background);
        draw_big(memory);
        SetStretchBltMode(hdc, HALFTONE);
        SetBrushOrgEx(hdc, 0, 0, null_mut()); // Windows asks for this after choosing HALFTONE
        StretchBlt(hdc, dest.left, dest.top, w, h, memory, 0, 0, w * SS, h * SS, SRCCOPY);
        SelectObject(memory, old);
    }
    if bitmap != 0 {
        DeleteObject(bitmap);
    }
    if memory != 0 {
        DeleteDC(memory);
    }
}

/// `shape`, with smooth edges.
unsafe fn smooth_shape(
    hdc: isize,
    rect: Rect,
    radius: i32,
    fill_colour: Option<u32>,
    border: Option<(u32, i32)>,
    background: u32,
) {
    // Windows leaves out the last row and column of a filled shape that has no border.
    let extra = if border.is_none() { 1 } else { 0 };
    smooth(hdc, rect, background, |m| {
        let big = Rect { left: 0, top: 0, right: rect.width() * SS + extra, bottom: rect.height() * SS + extra };
        shape(m, big, radius * SS, fill_colour, border.map(|(c, w)| (c, w * SS)));
    });
}

/// Draws an icon from `icons.rs`, centred in `slot`, with smooth edges.
/// `main` is the icon colour, `back` the colour for marks cut out of it,
/// `background` the colour under the icon.
unsafe fn draw_icon(hdc: isize, icon: &Icon, slot: Rect, main: u32, back: u32, background: u32, lay: &Layout) {
    let w = (icon.w * lay.dpi + 95) / 96;
    let h = (icon.h * lay.dpi + 95) / 96;
    let left = slot.left + (slot.width() - w) / 2;
    let top = slot.top + (slot.height() - h) / 2;
    let dest = Rect { left, top, right: left + w, bottom: top + h };

    smooth(hdc, dest, background, |m| {
        let big = |v: i32| v * SS * lay.dpi / 96;
        let ink = |i: Ink| if i == Ink::Main { main } else { back };
        for piece in icon.pieces {
            match *piece {
                Piece::Lines { points, width, ink: i } => {
                    let pts: Vec<Point> = points.iter().map(|&(x, y)| Point { x: big(x), y: big(y) }).collect();
                    polyline(m, &pts, ink(i), big(width));
                }
                Piece::Frame { x, y, w, h, r, width, ink: i } => {
                    let area = Rect { left: big(x), top: big(y), right: big(x + w), bottom: big(y + h) };
                    shape(m, area, big(r), None, Some((ink(i), big(width))));
                }
                Piece::Block { x, y, w, h, ink: i } => {
                    fill(m, Rect { left: big(x), top: big(y), right: big(x + w), bottom: big(y + h) }, ink(i));
                }
                Piece::Panel { x, y, w, h, r, ink: i } => {
                    // Windows leaves out the last row and column of a filled shape that has no border.
                    let area = Rect { left: big(x), top: big(y), right: big(x + w) + 1, bottom: big(y + h) + 1 };
                    shape(m, area, big(r), Some(ink(i)), None);
                }
                Piece::Disc { x, y, d, ink: i } => {
                    disc(m, Rect { left: big(x), top: big(y), right: big(x + d), bottom: big(y + d) }, ink(i));
                }
                Piece::Poly { points, ink: i } => {
                    let pts: Vec<Point> = points.iter().map(|&(x, y)| Point { x: big(x), y: big(y) }).collect();
                    polygon(m, &pts, ink(i));
                }
            }
        }
    });
}

unsafe fn text(hdc: isize, s: &str, rect: Rect, font: isize, color: u32, flags: u32) {
    let mut rect = rect;
    let chars: Vec<u16> = s.encode_utf16().collect();
    SelectObject(hdc, font);
    SetTextColor(hdc, color);
    SetBkMode(hdc, TRANSPARENT);
    DrawTextW(hdc, chars.as_ptr(), chars.len() as i32, &mut rect, flags | DT_NOPREFIX);
}

/// Width, ascent and descent of `s` in `font`.
unsafe fn text_size(hdc: isize, font: isize, s: &str) -> (i32, i32, i32) {
    let chars: Vec<u16> = s.encode_utf16().collect();
    SelectObject(hdc, font);
    let mut size = Size { cx: 0, cy: 0 };
    GetTextExtentPoint32W(hdc, chars.as_ptr(), chars.len() as i32, &mut size);
    let mut metrics: TextMetricW = zeroed();
    GetTextMetricsW(hdc, &mut metrics);
    (size.cx, metrics.ascent, metrics.descent)
}

/// One piece of a line of text.
struct Part<'a> {
    text: &'a str,
    font: isize,
    colour: u32,
}

/// Draws several pieces of text one after another on the same baseline, in the middle
/// of `area` (up and down). Pieces may use different fonts. Left aligned or centred.
unsafe fn rich(hdc: isize, area: Rect, parts: &[Part], centred: bool) {
    let mut widths = Vec::with_capacity(parts.len());
    let (mut total, mut ascent, mut descent) = (0, 0, 0);
    for part in parts {
        let (w, a, d) = text_size(hdc, part.font, part.text);
        widths.push(w);
        total += w;
        ascent = ascent.max(a);
        descent = descent.max(d);
    }
    let mut x = if centred { area.left + ((area.width() - total) / 2).max(0) } else { area.left };
    let baseline = area.top + (area.height() - (ascent + descent)) / 2 + ascent;

    SetBkMode(hdc, TRANSPARENT);
    SetTextAlign(hdc, TA_BASELINE);
    for (part, width) in parts.iter().zip(widths) {
        let chars: Vec<u16> = part.text.encode_utf16().collect();
        SelectObject(hdc, part.font);
        SetTextColor(hdc, part.colour);
        TextOutW(hdc, x, baseline, chars.as_ptr(), chars.len() as i32);
        x += width;
    }
    SetTextAlign(hdc, TA_TOP);
}

/// A rounded button. The chosen one is filled, the others are outlines.
/// Used for the output choice and for the tabs of the key hint page.
unsafe fn pill(hdc: isize, rect: Rect, label: &str, selected: bool, font: isize, lay: &Layout) {
    let radius = rect.height() / 2;
    let flags = DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
    if selected {
        smooth_shape(hdc, rect, radius, Some(ACCENT), None, BG);
        text(hdc, label, rect, font, BG, flags);
    } else {
        smooth_shape(hdc, rect, radius, None, Some((ACCENT, lay.scale(2))), BG);
        text(hdc, label, rect, font, TEXT, flags);
    }
}

/// An outlined button with an icon and a label side by side. The pair is centred in the button.
unsafe fn icon_button(hdc: isize, rect: Rect, icon: &Icon, label: &str, font: isize, radius: i32, lay: &Layout) {
    smooth_shape(hdc, rect, radius, None, Some((ACCENT, lay.scale(2))), BG);
    let label_width = text_size(hdc, font, label).0;
    let icon_width = lay.scale(icon.w);
    let gap = lay.scale(10);
    let left = rect.left + (rect.width() - (icon_width + gap + label_width)) / 2;
    let icon_slot = Rect { left, top: rect.top, right: left + icon_width, bottom: rect.bottom };
    draw_icon(hdc, icon, icon_slot, ACCENT, BG, BG, lay);
    let label_area = Rect { left: left + icon_width + gap, top: rect.top, right: rect.right, bottom: rect.bottom };
    text(hdc, label, label_area, font, TEXT, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
}

/// Smooths the four rounded corners of the window outline. The straight parts of the
/// outline are plain GDI lines. Only small squares at the corners are drawn big and shrunk.
unsafe fn outline_corners(hdc: isize, width: i32, height: i32, lay: &Layout) {
    let radius = lay.scale(8);
    let stroke = lay.scale(2);
    let size = radius + stroke;
    for (left, top) in [(0, 0), (width - size, 0), (0, height - size), (width - size, height - size)] {
        let dest = Rect { left, top, right: left + size, bottom: top + size };
        smooth(hdc, dest, BG, |m| {
            // The whole outline, moved so that this corner lands on the small bitmap.
            let area = Rect { left: -left * SS, top: -top * SS, right: (width - left) * SS, bottom: (height - top) * SS };
            shape(m, area, radius * SS, None, Some((ACCENT, stroke * SS)));
        });
    }
}

/// Which page is showing.
fn view() -> View {
    if VIEW.load(Relaxed) == 1 {
        View::Keys
    } else {
        View::Main
    }
}

/// What both pages share: the background, the lime outline and the title bar.
unsafe fn draw_frame(hdc: isize, width: i32, height: i32, lay: &Layout) {
    let centred = DT_CENTER | DT_VCENTER | DT_SINGLELINE;
    let left_middle = DT_LEFT | DT_VCENTER | DT_SINGLELINE;

    let full = Rect { left: 0, top: 0, right: width, bottom: height };
    fill(hdc, full, BG);
    shape(hdc, full, lay.scale(8), None, Some((ACCENT, lay.scale(2))));
    outline_corners(hdc, width, height, lay);

    // Small logo, name, minimise and close, then a lime line under the bar
    smooth_shape(hdc, lay.title_logo, lay.scale(6), Some(ACCENT), None, BG);
    text(hdc, "\u{0D85}", lay.title_logo, FONT_LOGO_SMALL.load(Relaxed), BG, centred);
    text(hdc, about::NAME, lay.title_text, FONT_SMALL_BOLD.load(Relaxed), TEXT, left_middle);
    draw_icon(hdc, &icons::MINIMISE, lay.minimise_button, TEXT, BG, BG, lay);
    draw_icon(hdc, &icons::CLOSE, lay.close_button, TEXT, BG, BG, lay);
    fill(hdc, lay.title_rule, ACCENT);
}

unsafe fn draw_main(hdc: isize, lay: &Layout) {
    let title_font = FONT_TITLE.load(Relaxed);
    let title_si_font = FONT_TITLE_SI.load(Relaxed);
    let label_font = FONT_LABEL.load(Relaxed);
    let small_font = FONT_SMALL.load(Relaxed);
    let bold_font = FONT_SMALL_BOLD.load(Relaxed);
    let left_middle = DT_LEFT | DT_VCENTER | DT_SINGLELINE;
    let centred = DT_CENTER | DT_VCENTER | DT_SINGLELINE;
    let enabled = state::enabled();
    let legacy = state::legacy();
    let card_radius = lay.scale(12);
    let button_radius = lay.scale(14);
    let stroke = lay.scale(2);
    let part = |text: &'static str, font: isize, colour: u32| Part { text, font, colour };

    // Header: logo tile, a thin divider, the name, and one line about what it does
    smooth_shape(hdc, lay.logo_tile, lay.scale(14), Some(ACCENT), None, BG);
    text(hdc, "\u{0D85}", lay.logo_tile, FONT_LOGO.load(Relaxed), BG, centred);
    fill(hdc, lay.logo_divider, ACCENT);
    rich(
        hdc,
        lay.name,
        &[
            part(about::NAME, title_font, TEXT),
            part(" ", title_font, TEXT),
            part(about::NAME_SINHALA, title_si_font, TEXT),
        ],
        false,
    );
    text(hdc, about::TAGLINE, lay.tagline, small_font, TEXT, left_middle | DT_END_ELLIPSIS);
    fill(hdc, lay.rule, ACCENT);

    // Sinhala typing on or off
    smooth_shape(hdc, lay.typing_card, card_radius, None, Some((ACCENT, stroke)), BG);
    draw_icon(hdc, &icons::KEYBOARD, lay.typing_icon, ACCENT, BG, BG, lay);
    text(hdc, "Sinhala typing", lay.typing_label, label_font, TEXT, left_middle);
    text(
        hdc,
        if enabled { "ON" } else { "OFF" },
        lay.typing_state,
        label_font,
        if enabled { ACCENT } else { MUTED },
        DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
    );
    let track = lay.switch_track;
    let track_colour = if enabled { ACCENT } else { EDGE };
    smooth_shape(hdc, track, track.height() / 2, Some(track_colour), None, BG);
    let pad = lay.scale(4);
    let knob = track.height() - 2 * pad;
    let knob_left = if enabled { track.right - pad - knob } else { track.left + pad };
    let knob_rect = Rect { left: knob_left, top: track.top + pad, right: knob_left + knob, bottom: track.top + pad + knob };
    smooth_shape(hdc, knob_rect, knob / 2, Some(BG), None, track_colour);

    // Output type
    draw_icon(hdc, &icons::OUTPUT, lay.output_icon, ACCENT, BG, BG, lay);
    text(hdc, "Output", lay.output_label, label_font, TEXT, left_middle);
    pill(hdc, lay.unicode_button, "Unicode", !legacy, label_font, lay);
    pill(hdc, lay.legacy_button, "Legacy (FM Abhaya)", legacy, label_font, lay);

    // Key hints. They name the keys chosen in settings.txt.
    smooth_shape(hdc, lay.info_panel, card_radius, None, Some((EDGE, stroke)), BG);
    draw_icon(hdc, &icons::INFO, lay.info_icon, ACCENT, BG, BG, lay);
    let chosen = settings::get();
    let on_off_key = settings::key_name(chosen.on_off_key);
    let output_key = settings::key_name(chosen.output_key);
    rich(
        hdc,
        lay.info_line1,
        &[
            part("Tap ", small_font, MUTED),
            part(on_off_key, bold_font, ACCENT),
            part(": Sinhala typing on or off", small_font, MUTED),
        ],
        false,
    );
    rich(
        hdc,
        lay.info_line2,
        &[
            part("Tap ", small_font, MUTED),
            part(output_key, bold_font, ACCENT),
            part(": Unicode or Legacy output", small_font, MUTED),
        ],
        false,
    );

    // Show Logs and Keys
    icon_button(hdc, lay.logs_button, &icons::DOCUMENT, "Show Logs", label_font, button_radius, lay);
    icon_button(hdc, lay.keys_button, &icons::KEYBOARD_SOLID, "Keys", label_font, button_radius, lay);

    // Footer: a line, the developer name with a cube, a line
    let dev = lay.dev_row;
    let dev_label_width = text_size(hdc, small_font, "DEV: ").0;
    let dev_name_width = text_size(hdc, bold_font, about::DEVELOPER).0;
    let cube_width = lay.scale(icons::CUBE.w);
    let cube_gap = lay.scale(8);
    let dev_group = cube_width + cube_gap + dev_label_width + dev_name_width;
    let dev_left = dev.left + (dev.width() - dev_group) / 2;
    let cube_slot = Rect { left: dev_left, top: dev.top, right: dev_left + cube_width, bottom: dev.bottom };
    draw_icon(hdc, &icons::CUBE, cube_slot, ACCENT, BG, BG, lay);
    let dev_text_area = Rect { left: dev_left + cube_width + cube_gap, top: dev.top, right: dev.right, bottom: dev.bottom };
    rich(
        hdc,
        dev_text_area,
        &[part("DEV: ", small_font, MUTED), part(about::DEVELOPER, bold_font, ACCENT)],
        false,
    );
    let line_top = (dev.top + dev.bottom - stroke) / 2;
    let margin = lay.scale(12);
    fill(hdc, Rect { left: dev.left, top: line_top, right: dev_left - margin, bottom: line_top + stroke }, ACCENT);
    fill(hdc, Rect { left: dev_left + dev_group + margin, top: line_top, right: dev.right, bottom: line_top + stroke }, ACCENT);

    // Contact buttons: GitHub, Facebook and YouTube. Clicking one opens its page in the browser.
    for (button, icon) in [
        (lay.github_button, &icons::GITHUB),
        (lay.facebook_button, &icons::FACEBOOK),
        (lay.youtube_button, &icons::YOUTUBE),
    ] {
        draw_icon(hdc, icon, button, ACCENT, BG, BG, lay);
    }
}

/// The key hint page: a Back button, four tabs, a grid of letters with their keys, a note.
unsafe fn draw_keys(hdc: isize, lay: &Layout) {
    let tab = Tab::from_index(HINT_TAB.load(Relaxed));
    let page = hints::page(tab);
    let key_font = FONT_HINT.load(Relaxed);
    let glyph_font = FONT_HINT_SI.load(Relaxed);
    let thin = lay.scale(1).max(1);

    icon_button(hdc, lay.back_button, &icons::BACK, "Back", FONT_LABEL.load(Relaxed), lay.back_button.height() / 2, lay);
    text(hdc, "Key hints", lay.hint_title, FONT_HEAD.load(Relaxed), TEXT, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

    for (i, t) in hints::TABS.iter().enumerate() {
        pill(hdc, lay.hint_tab(i), t.label(), *t == tab, key_font, lay);
    }

    let rows = page.rows();
    let radius = lay.scale(8);
    for (i, c) in page.cells.iter().enumerate() {
        let cell = lay.hint_cell(page.columns, rows, i);
        smooth_shape(hdc, cell, radius, None, Some((EDGE, thin)), BG);
        let wide = cell.width() > lay.scale(120);
        let pad = lay.scale(if wide { 10 } else { 4 });
        let inner = Rect { left: cell.left + pad, top: cell.top, right: cell.right - pad, bottom: cell.bottom };
        if wide {
            // Wide cells (the examples): the Sinhala on the left, the keys on the right.
            text(hdc, &c.glyph, inner, glyph_font, ACCENT, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
            text(hdc, &c.keys, inner, key_font, TEXT, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);
        } else {
            // Narrow cells: the Sinhala and its keys as one centred pair.
            rich(
                hdc,
                inner,
                &[
                    Part { text: &c.glyph, font: glyph_font, colour: ACCENT },
                    Part { text: " ", font: key_font, colour: TEXT },
                    Part { text: &c.keys, font: key_font, colour: TEXT },
                ],
                true,
            );
        }
    }

    let note_area = lay.hint_note_at(page.columns, rows);
    text(hdc, &page.note, note_area, FONT_NOTE.load(Relaxed), MUTED, DT_LEFT | DT_WORDBREAK);
}

unsafe fn draw(hdc: isize, width: i32, height: i32) {
    let lay = Layout::new(DPI.load(Relaxed));
    draw_frame(hdc, width, height, &lay);
    match view() {
        View::Main => draw_main(hdc, &lay),
        View::Keys => draw_keys(hdc, &lay),
    }
}

/// Draws into a memory bitmap first, then copies it to the window, so nothing flickers.
unsafe fn paint(hwnd: isize) {
    let mut ps: PaintStruct = zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut client = Rect::default();
    GetClientRect(hwnd, &mut client);
    let (w, h) = (client.right, client.bottom);

    let memory = CreateCompatibleDC(hdc);
    let bitmap = CreateCompatibleBitmap(hdc, w, h);
    let old = SelectObject(memory, bitmap);
    draw(memory, w, h);
    BitBlt(hdc, 0, 0, w, h, memory, 0, 0, SRCCOPY);
    SelectObject(memory, old);
    DeleteObject(bitmap);
    DeleteDC(memory);

    EndPaint(hwnd, &ps);
}

// ------------------------------------------------------- icons and tray icon

/// Draws a small rounded square with the letter අ on it, using only palette colours.
/// `on` picks the look: a lime square while Sinhala typing is on, a grey one while off.
unsafe fn make_icon(size: i32, on: bool) -> isize {
    let screen = GetDC(0);
    let memory = CreateCompatibleDC(screen);
    let colour_bitmap = CreateCompatibleBitmap(screen, size, size);
    let mask_bitmap = CreateBitmap(size, size, 1, 1, null());
    ReleaseDC(0, screen);

    let mut icon = 0;
    if memory != 0 && colour_bitmap != 0 && mask_bitmap != 0 {
        let area = Rect { left: 0, top: 0, right: size, bottom: size };
        // Windows leaves out the right and bottom edge when the pen is null.
        let round = Rect { left: 0, top: 0, right: size + 1, bottom: size + 1 };
        let radius = size / 4;

        // The colour picture: black outside the square, the letter on the square.
        let original_bitmap = SelectObject(memory, colour_bitmap);
        PatBlt(memory, 0, 0, size, size, BLACKNESS);
        shape(memory, round, radius, Some(if on { ACCENT } else { EDGE }), None);
        let font = make_font(size * 7 / 10, 700, "Nirmala UI");
        let original_font = SelectObject(memory, font);
        text(memory, "\u{0D85}", area, font, if on { BG } else { MUTED }, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        SelectObject(memory, original_font);
        DeleteObject(font);

        // The mask: white (1) means see-through, black (0) means solid.
        SelectObject(memory, mask_bitmap);
        PatBlt(memory, 0, 0, size, size, WHITENESS);
        shape(memory, round, radius, Some(0), None);
        SelectObject(memory, original_bitmap);

        let info = IconInfo { is_icon: 1, hotspot_x: 0, hotspot_y: 0, mask: mask_bitmap, colour: colour_bitmap };
        icon = CreateIconIndirect(&info);
    }
    if colour_bitmap != 0 {
        DeleteObject(colour_bitmap);
    }
    if mask_bitmap != 0 {
        DeleteObject(mask_bitmap);
    }
    if memory != 0 {
        DeleteDC(memory);
    }
    icon
}

/// The three icons, made once at start. If Windows refuses, the plain default icon is used.
unsafe fn make_icons() {
    let small = GetSystemMetrics(SM_CXSMICON).max(16);
    let large = GetSystemMetrics(SM_CXICON).max(32);
    let fallback = LoadIconW(0, IDI_APPLICATION as *const u16);
    let or_fallback = |icon: isize| if icon != 0 { icon } else { fallback };
    ICON_ON.store(or_fallback(make_icon(small, true)), Relaxed);
    ICON_OFF.store(or_fallback(make_icon(small, false)), Relaxed);
    ICON_LARGE.store(or_fallback(make_icon(large, true)), Relaxed);
}

fn tray_tip() -> String {
    format!(
        "{}: {}, {}\nLeft click: open. Right click: typing on or off.",
        about::NAME,
        if state::enabled() { "ON" } else { "OFF" },
        if state::legacy() { "Legacy" } else { "Unicode" }
    )
}

unsafe fn tray_data(hwnd: isize) -> NotifyIconData {
    let mut data: NotifyIconData = zeroed();
    data.size = size_of::<NotifyIconData>() as u32;
    data.hwnd = hwnd;
    data.id = TRAY_ID;
    data.flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.callback_message = WM_TRAY;
    data.icon = if state::enabled() { ICON_ON.load(Relaxed) } else { ICON_OFF.load(Relaxed) };
    // The last slot stays 0, which ends the text.
    let tip: Vec<u16> = tray_tip().encode_utf16().take(data.tip.len() - 1).collect();
    data.tip[..tip.len()].copy_from_slice(&tip);
    data
}

unsafe fn tray_add(hwnd: isize) {
    let added = Shell_NotifyIconW(NIM_ADD, &tray_data(hwnd)) != 0;
    TRAY_ADDED.store(added, Relaxed);
    if !added {
        state::log("Could not add the tray icon. Minimise will keep the window on the taskbar.");
    }
}

unsafe fn tray_update(hwnd: isize) {
    if TRAY_ADDED.load(Relaxed) {
        Shell_NotifyIconW(NIM_MODIFY, &tray_data(hwnd));
    }
}

unsafe fn tray_remove(hwnd: isize) {
    if TRAY_ADDED.swap(false, Relaxed) {
        Shell_NotifyIconW(NIM_DELETE, &tray_data(hwnd));
    }
}

/// Redraws the window and brings the tray icon and tooltip up to date.
unsafe fn refresh(hwnd: isize) {
    InvalidateRect(hwnd, null(), 0);
    tray_update(hwnd);
}

/// Brings the main window back from the tray.
unsafe fn show_window(hwnd: isize) {
    ShowWindow(hwnd, SW_RESTORE);
    SetForegroundWindow(hwnd);
}

/// Gives back memory the program is not using right now. Windows brings back
/// whatever is needed again by itself, so nothing changes for the user.
unsafe fn trim_memory() {
    SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
}

/// Opens a web address in the default browser.
unsafe fn open_link(url: &str) {
    // Windows asks for COM to be started before ShellExecute. It is done once, on the first click.
    if !COM_READY.swap(true, Relaxed) {
        CoInitializeEx(null(), COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
    }
    let result = ShellExecuteW(0, wide("open").as_ptr(), wide(url).as_ptr(), null(), null(), SW_SHOWNORMAL);
    if result > 32 {
        state::log(format!("Opened {url}"));
    } else {
        state::log(format!("Could not open {url}"));
    }
}

// ---------------------------------------------------------------- the popup

/// Makes the title bar match the palette. Works on Windows 11 only.
/// Older Windows ignores it and keeps its normal title bar.
unsafe fn theme_title_bar(hwnd: isize) {
    for (attribute, colour) in [(35u32, BG), (36, TEXT), (34, ACCENT)] {
        // 35 caption colour, 36 caption text colour, 34 border colour
        DwmSetWindowAttribute(hwnd, attribute, &colour as *const u32 as *const c_void, 4);
    }
}

/// Rounds the corners of the main window. Windows 11 does it by itself when asked.
/// Older Windows cannot, so the window is cut to a rounded shape instead.
unsafe fn round_window(hwnd: isize, width: i32, height: i32, radius: i32) {
    let preference: u32 = 2; // DWMWCP_ROUND
    let asked = DwmSetWindowAttribute(hwnd, 33, &preference as *const u32 as *const c_void, 4);
    if asked != 0 {
        let region = CreateRoundRectRgn(0, 0, width + 1, height + 1, radius * 2, radius * 2);
        SetWindowRgn(hwnd, region, 1); // Windows owns the region from here on
    }
}

/// Puts all remembered log lines into the popup and scrolls to the end.
unsafe fn refresh_log_text() {
    let edit = LOG_EDIT.load(Relaxed);
    if edit == 0 {
        return;
    }
    let all = match LOG_LINES.lock() {
        Ok(lines) => lines.iter().cloned().collect::<Vec<_>>().join("\r\n"),
        Err(_) => String::new(),
    };
    SetWindowTextW(edit, wide(&all).as_ptr());
    let len = GetWindowTextLengthW(edit);
    SendMessageW(edit, EM_SETSEL, len as usize, len as isize);
    SendMessageW(edit, EM_SCROLLCARET, 0, 0);
}

unsafe fn open_logs(owner: isize) {
    let existing = LOG_WINDOW.load(Relaxed);
    if existing != 0 && IsWindow(existing) != 0 {
        ShowWindow(existing, SW_RESTORE);
        SetForegroundWindow(existing);
        return;
    }

    let hinst = GetModuleHandleW(null());
    let dpi = DPI.load(Relaxed);
    let (w, h) = (640 * dpi / 96, 420 * dpi / 96);
    let title = format!("{} Logs", about::NAME);
    let window = CreateWindowExW(
        0,
        wide(LOG_CLASS).as_ptr(),
        wide(&title).as_ptr(),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        w,
        h,
        owner,
        0,
        hinst,
        null(),
    );
    if window == 0 {
        return;
    }
    let edit = CreateWindowExW(
        0,
        wide("EDIT").as_ptr(),
        wide("").as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL,
        0,
        0,
        w,
        h,
        window,
        0,
        hinst,
        null(),
    );
    SendMessageW(edit, WM_SETFONT, FONT_LOG.load(Relaxed) as usize, 1);
    SendMessageW(edit, EM_SETLIMITTEXT, 0, 0); // 0 means the largest size allowed

    LOG_WINDOW.store(window, Relaxed);
    LOG_EDIT.store(edit, Relaxed);
    theme_title_bar(window);

    let mut client = Rect::default();
    GetClientRect(window, &mut client);
    MoveWindow(edit, 0, 0, client.right, client.bottom, 1);
    refresh_log_text();
}

unsafe extern "system" fn log_proc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    // A panic must never cross into Windows.
    let handled = catch_unwind(AssertUnwindSafe(|| match msg {
        WM_SIZE => {
            let edit = LOG_EDIT.load(Relaxed);
            if edit != 0 {
                MoveWindow(edit, 0, 0, (lparam & 0xFFFF) as i32, ((lparam >> 16) & 0xFFFF) as i32, 1);
            }
            Some(0)
        }
        // A read-only edit box asks for its colours as a "static" control.
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            let hdc = wparam as isize;
            SetTextColor(hdc, TEXT);
            SetBkColor(hdc, BG);
            Some(BG_BRUSH.load(Relaxed))
        }
        WM_ERASEBKGND => Some(1),
        WM_DESTROY => {
            LOG_EDIT.store(0, Relaxed);
            LOG_WINDOW.store(0, Relaxed);
            Some(0)
        }
        _ => None,
    }))
    .unwrap_or(None);
    match handled {
        Some(result) => result,
        None => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// -------------------------------------------------------------- main window

unsafe fn on_click(hwnd: isize, lparam: isize) {
    let x = (lparam & 0xFFFF) as i16 as i32;
    let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
    let lay = Layout::new(DPI.load(Relaxed));
    match lay.hit(view(), x, y) {
        Hit::Minimise => {
            ShowWindow(hwnd, SW_MINIMIZE); // goes to the tray, see WM_SIZE
            return;
        }
        Hit::Close => {
            SendMessageW(hwnd, WM_CLOSE, 0, 0);
            return;
        }
        Hit::TypingCard => {
            let on = !state::enabled();
            state::set_enabled(on);
            state::log(if on { "Sinhala typing ON (window)" } else { "Sinhala typing OFF (window)" });
        }
        Hit::UnicodeButton => {
            state::set_legacy(false);
            state::log("Output: Unicode (window)");
        }
        Hit::LegacyButton => {
            state::set_legacy(true);
            state::log("Output: legacy (FM Abhaya style) (window)");
        }
        Hit::LogsButton => open_logs(hwnd),
        Hit::KeysButton => VIEW.store(1, Relaxed),
        Hit::Back => VIEW.store(0, Relaxed),
        Hit::Tab(i) => HINT_TAB.store(i, Relaxed),
        Hit::GitHub => {
            open_link(about::GITHUB_URL);
            return;
        }
        Hit::Facebook => {
            open_link(about::FACEBOOK_URL);
            return;
        }
        Hit::YouTube => {
            open_link(about::YOUTUBE_URL);
            return;
        }
        Hit::Caption | Hit::None => return,
    }
    refresh(hwnd);
}

/// Runs every 100 ms. Redraws when the hotkeys changed something and
/// collects new log lines for the popup.
unsafe fn on_timer(hwnd: isize) {
    let enabled = state::enabled();
    let legacy = state::legacy();
    let enabled_changed = SEEN_ENABLED.swap(enabled, Relaxed) != enabled;
    let legacy_changed = SEEN_LEGACY.swap(legacy, Relaxed) != legacy;
    if enabled_changed || legacy_changed {
        refresh(hwnd);
    }

    let fresh = state::take_logs();
    if !fresh.is_empty() {
        if let Ok(mut lines) = LOG_LINES.lock() {
            for line in fresh {
                if lines.len() >= MAX_LINES {
                    lines.pop_front();
                }
                lines.push_back(line);
            }
        }
        refresh_log_text();
    }
}

unsafe extern "system" fn main_proc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    // A panic must never cross into Windows.
    let handled = catch_unwind(AssertUnwindSafe(|| {
        // Explorer restarted and the taskbar is new, so the tray icon must be added again.
        let taskbar_created = TASKBAR_CREATED.load(Relaxed);
        if taskbar_created != 0 && msg == taskbar_created {
            tray_add(hwnd);
            return Some(0);
        }
        match msg {
            WM_PAINT => {
                paint(hwnd);
                Some(0)
            }
            WM_ERASEBKGND => Some(1), // we paint the whole window ourselves
            // There is no Windows title bar, so tell Windows our own title bar is one.
            // That is what lets the window be dragged by it.
            WM_NCHITTEST => {
                let mut point = Point { x: (lparam & 0xFFFF) as i16 as i32, y: ((lparam >> 16) & 0xFFFF) as i16 as i32 };
                ScreenToClient(hwnd, &mut point);
                let lay = Layout::new(DPI.load(Relaxed));
                if lay.hit(view(), point.x, point.y) == Hit::Caption {
                    Some(HTCAPTION)
                } else {
                    None
                }
            }
            // A hand pointer over buttons and links.
            WM_SETCURSOR => {
                if (lparam & 0xFFFF) == HTCLIENT {
                    let mut point = Point { x: 0, y: 0 };
                    GetCursorPos(&mut point);
                    ScreenToClient(hwnd, &mut point);
                    let lay = Layout::new(DPI.load(Relaxed));
                    if lay.hit(view(), point.x, point.y).clickable() {
                        SetCursor(LoadCursorW(0, IDC_HAND as *const u16));
                        return Some(1);
                    }
                }
                None
            }
            WM_LBUTTONDOWN => {
                on_click(hwnd, lparam);
                Some(0)
            }
            // Esc closes the key hint page.
            WM_KEYDOWN => {
                if wparam == VK_ESCAPE && view() == View::Keys {
                    VIEW.store(0, Relaxed);
                    refresh(hwnd);
                    Some(0)
                } else {
                    None
                }
            }
            WM_SIZE => {
                // Minimise goes to the tray, but only if the tray icon exists to bring it back.
                if wparam == SIZE_MINIMIZED && TRAY_ADDED.load(Relaxed) {
                    ShowWindow(hwnd, SW_HIDE);
                    trim_memory();
                    Some(0)
                } else {
                    None
                }
            }
            WM_TRAY => {
                match (lparam & 0xFFFF) as u32 {
                    WM_LBUTTONUP => show_window(hwnd),
                    WM_RBUTTONUP => {
                        let on = !state::enabled();
                        state::set_enabled(on);
                        state::log(if on { "Sinhala typing ON (tray)" } else { "Sinhala typing OFF (tray)" });
                        refresh(hwnd);
                    }
                    _ => {}
                }
                Some(0)
            }
            WM_TIMER => {
                if wparam == TRIM_TIMER_ID {
                    KillTimer(hwnd, TRIM_TIMER_ID);
                    trim_memory();
                } else {
                    on_timer(hwnd);
                }
                Some(0)
            }
            WM_DESTROY => {
                tray_remove(hwnd);
                KillTimer(hwnd, TIMER_ID);
                PostQuitMessage(0);
                Some(0)
            }
            _ => None,
        }
    }))
    .unwrap_or(None);
    match handled {
        Some(result) => result,
        None => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ------------------------------------------------------------------- startup

unsafe fn register_class(name: &str, proc_: WndProc, hinst: isize) {
    let class_name = wide(name);
    let class = WndClassExW {
        size: size_of::<WndClassExW>() as u32,
        style: 0,
        wnd_proc: Some(proc_),
        class_extra: 0,
        window_extra: 0,
        instance: hinst,
        icon: 0,
        cursor: LoadCursorW(0, IDC_ARROW as *const u16),
        background: BG_BRUSH.load(Relaxed),
        menu_name: null(),
        class_name: class_name.as_ptr(),
        icon_small: 0,
    };
    RegisterClassExW(&class);
}

unsafe fn make_font_style(height: i32, weight: i32, underline: bool, face: &str) -> isize {
    // charset 1 = default, quality 5 = ClearType
    CreateFontW(-height, 0, 0, 0, weight, 0, underline as u32, 0, 1, 0, 0, 5, 0, wide(face).as_ptr())
}

unsafe fn make_font(height: i32, weight: i32, face: &str) -> isize {
    make_font_style(height, weight, false, face)
}

/// Opens the window, starts the keyboard hook, and runs until the window is closed.
pub fn run(map: LegacyMap, legacy_output: bool) {
    state::set_legacy(legacy_output);
    SEEN_LEGACY.store(legacy_output, Relaxed);
    SEEN_ENABLED.store(state::enabled(), Relaxed);
    let _hook = crate::win::start(map);

    unsafe {
        SetProcessDPIAware(); // sharp text on high-DPI screens
        let screen = GetDC(0);
        let dpi = GetDeviceCaps(screen, LOGPIXELSX).max(96);
        ReleaseDC(0, screen);
        DPI.store(dpi, Relaxed);
        let lay = Layout::new(dpi);

        BG_BRUSH.store(CreateSolidBrush(BG), Relaxed);
        // Nirmala UI has Sinhala letters. Segoe UI is used for everything else.
        FONT_TITLE.store(make_font(lay.scale(32), 700, "Segoe UI"), Relaxed);
        FONT_TITLE_SI.store(make_font(lay.scale(30), 700, "Nirmala UI"), Relaxed);
        FONT_LABEL.store(make_font(lay.scale(16), 700, "Segoe UI"), Relaxed);
        FONT_SMALL.store(make_font(lay.scale(14), 600, "Segoe UI"), Relaxed);
        FONT_SMALL_BOLD.store(make_font(lay.scale(14), 700, "Segoe UI"), Relaxed);
        FONT_LOGO.store(make_font(lay.scale(34), 700, "Nirmala UI"), Relaxed);
        FONT_LOGO_SMALL.store(make_font(lay.scale(15), 700, "Nirmala UI"), Relaxed);
        FONT_HEAD.store(make_font(lay.scale(22), 700, "Segoe UI"), Relaxed);
        FONT_HINT.store(make_font(lay.scale(13), 600, "Segoe UI"), Relaxed);
        FONT_HINT_SI.store(make_font(lay.scale(19), 700, "Nirmala UI"), Relaxed);
        FONT_NOTE.store(make_font(lay.scale(12), 600, "Nirmala UI"), Relaxed);
        FONT_LOG.store(make_font(lay.scale(16), 400, "Nirmala UI"), Relaxed);

        make_icons();
        TASKBAR_CREATED.store(RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()), Relaxed);

        let hinst = GetModuleHandleW(null());
        register_class(MAIN_CLASS, main_proc, hinst);
        register_class(LOG_CLASS, log_proc, hinst);

        // No Windows title bar: the window draws its own. It can still be minimised.
        let style = WS_POPUP | WS_MINIMIZEBOX | WS_SYSMENU;
        let (w, h) = lay.client_size();
        let x = (GetSystemMetrics(0) - w) / 2;
        let y = (GetSystemMetrics(1) - h) / 2;

        let window = CreateWindowExW(
            0,
            wide(MAIN_CLASS).as_ptr(),
            wide(about::NAME).as_ptr(),
            style,
            x,
            y,
            w,
            h,
            0,
            0,
            hinst,
            null(),
        );
        if window == 0 {
            return;
        }
        theme_title_bar(window);
        round_window(window, w, h, lay.scale(8));
        SendMessageW(window, WM_SETICON, SETICON_SMALL, ICON_ON.load(Relaxed));
        SendMessageW(window, WM_SETICON, SETICON_BIG, ICON_LARGE.load(Relaxed));

        tray_add(window);
        // Start hidden only if the tray icon is there to open the window again.
        if settings::get().start_in_tray && TRAY_ADDED.load(Relaxed) {
            state::log("Started in the tray. Left click the tray icon to open the window.");
        } else {
            ShowWindow(window, SW_SHOW);
            UpdateWindow(window);
        }
        SetTimer(window, TIMER_ID, 100, null());
        SetTimer(window, TRIM_TIMER_ID, 3000, null()); // once, a few seconds after start

        let mut msg: Msg = zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
