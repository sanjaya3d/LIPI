//! Windows only. The main window and the "Show Logs" popup.
//!
//! The main window has its own title bar, cards, buttons and icons, all drawn by hand
//! with only the colours in `theme.rs` (near black, lime, white and a few greys). No image files, no extra libraries.
//! Edges are made smooth by drawing the rounded corners (and the small icons) three times bigger
//! in a tiny hidden bitmap and shrinking them, so the program still uses very little memory.
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
use crate::autostart;
use crate::hints::{self, Tab};
use crate::icons::{self, Icon, Ink, Piece};
use crate::layout::{Hit, Layout, Rect, SettingCard, View};
use crate::legacy::LegacyMap;
use crate::settings;
use crate::state;
use crate::theme::{ACCENT, ACCENT_DIM, ACCENT_TINT, BG, CARD, CHIP, DIM, EDGE, LINE, MUTED, TEXT, WELL};

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
/// Message a second copy of the program sends to the first one: "show your window" (WM_APP + 2).
const WM_SHOW_LIPI: u32 = 0x8002;
const WM_NULL: u32 = 0x0000;
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

// The tray menu and the "already running" message.
const MF_STRING: u32 = 0x0000;
const MF_CHECKED: u32 = 0x0008;
const MF_SEPARATOR: u32 = 0x0800;
const TPM_RIGHTBUTTON: u32 = 0x0002;
const TPM_BOTTOMALIGN: u32 = 0x0020;
const TPM_RETURNCMD: u32 = 0x0100;
const MB_ICONINFORMATION: u32 = 0x0040;
const MB_SETFOREGROUND: u32 = 0x0001_0000;
const ERROR_ALREADY_EXISTS: u32 = 183;
/// Lets any program bring a window to the front. Used for the copy that is already running.
const ASFW_ANY: u32 = u32::MAX;
const MENU_OPEN: usize = 1;
const MENU_TYPING: usize = 2;
const MENU_EXIT: usize = 3;

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
    fn CreatePopupMenu() -> isize;
    fn AppendMenuW(menu: isize, flags: u32, id: usize, item: *const u16) -> i32;
    fn TrackPopupMenu(menu: isize, flags: u32, x: i32, y: i32, reserved: i32, hwnd: isize, rect: *const Rect) -> i32;
    fn DestroyMenu(menu: isize) -> i32;
    fn PostMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> i32;
    fn FindWindowW(class: *const u16, title: *const u16) -> isize;
    fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, flags: u32) -> i32;
    fn AllowSetForegroundWindow(process_id: u32) -> i32;
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
    fn CreateMutexW(attributes: *const c_void, initial_owner: i32, name: *const u16) -> isize;
    fn GetLastError() -> u32;
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
static FONT_LABEL: AtomicIsize = AtomicIsize::new(0);
static FONT_SMALL_BOLD: AtomicIsize = AtomicIsize::new(0);
static FONT_LOGO_SMALL: AtomicIsize = AtomicIsize::new(0);
static FONT_HINT: AtomicIsize = AtomicIsize::new(0);
static FONT_HINT_SI: AtomicIsize = AtomicIsize::new(0);
static FONT_NOTE: AtomicIsize = AtomicIsize::new(0);
/// Consolas, for key chips and the version badge.
static FONT_MONO: AtomicIsize = AtomicIsize::new(0);
/// Nirmala UI 13, for the Sinhala line and the status line on the main page.
static FONT_SUB: AtomicIsize = AtomicIsize::new(0);
static FONT_LOG: AtomicIsize = AtomicIsize::new(0);
/// Nirmala UI 12 bold, for the smart quotes text and its button.
static FONT_TIP_SMALL: AtomicIsize = AtomicIsize::new(0);
/// Nirmala UI 11, the smallest text. Used only when a line would not fit in its place at the normal size.
static FONT_NOTE_SMALL: AtomicIsize = AtomicIsize::new(0);
/// 0 is the main page, 1 is the key hint page, 2 is the Settings page.
static VIEW: AtomicU8 = AtomicU8::new(0);
/// Settings page: true while Windows will start LIPI at sign in. Read from the registry
/// each time the page opens, and again after every change, so it is never a guess.
static STARTUP_ON: AtomicBool = AtomicBool::new(false);
/// Settings page: true when LIPI starts hidden in the tray (start_in_tray in settings.txt).
static TRAY_START_ON: AtomicBool = AtomicBool::new(false);
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

/// A rounded card with an optional fill and an optional border (colour, width).
///
/// The whole shape is drawn with plain Windows calls first. Then only the four corners, small
/// squares, are drawn three times bigger and shrunk, which makes them smooth. A big card
/// never needs a big hidden bitmap, so the program keeps using very little memory.
/// `behind` is the colour already under the card. Without a fill, the inside of each
/// corner square is painted with `behind` too, so only use that where the card sits on `behind`.
unsafe fn card(hdc: isize, rect: Rect, radius: i32, fill_colour: Option<u32>, border: Option<(u32, i32)>, behind: u32) {
    // Windows leaves out the last row and column of a filled shape that has no border.
    let extra = if border.is_none() && fill_colour.is_some() { 1 } else { 0 };
    let plain = Rect { right: rect.right + extra, bottom: rect.bottom + extra, ..rect };
    shape(hdc, plain, radius, fill_colour, border);

    let stroke = border.map_or(0, |(_, w)| w);
    let size = radius + stroke.max(1) + 1;
    for (left, top) in [
        (rect.left, rect.top),
        (rect.right - size, rect.top),
        (rect.left, rect.bottom - size),
        (rect.right - size, rect.bottom - size),
    ] {
        let dest = Rect { left, top, right: left + size, bottom: top + size };
        smooth(hdc, dest, behind, |m| {
            // The whole card, moved so that this corner lands on the small bitmap.
            let area = Rect {
                left: (rect.left - left) * SS,
                top: (rect.top - top) * SS,
                right: (rect.right - left) * SS + extra,
                bottom: (rect.bottom - top) * SS + extra,
            };
            shape(m, area, radius * SS, fill_colour, border.map(|(c, w)| (c, w * SS)));
        });
    }
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

/// The first font in `fonts` in which `s` is no wider than `width`. When none is narrow enough,
/// the last one (the smallest) is used.
unsafe fn fit_font(hdc: isize, s: &str, width: i32, fonts: &[isize]) -> isize {
    for &font in fonts {
        if text_size(hdc, font, s).0 <= width {
            return font;
        }
    }
    fonts.last().copied().unwrap_or(0)
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

/// A rounded button on a surface of the colour `behind` (a card, for example), so the smooth
/// corners blend into that surface. A chosen one is filled lime. Used for "Check for updates".
unsafe fn pill_on(hdc: isize, rect: Rect, label: &str, selected: bool, font: isize, behind: u32, lay: &Layout) {
    let radius = rect.height() / 2;
    let flags = DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
    if selected {
        card(hdc, rect, radius, Some(ACCENT), None, behind);
        text(hdc, label, rect, font, BG, flags);
    } else {
        card(hdc, rect, radius, None, Some((ACCENT, lay.scale(2))), behind);
        text(hdc, label, rect, font, TEXT, flags);
    }
}

/// A tab of the key hint page. The chosen one is lime with dark text. The others are dark cards
/// with a thin line and light text.
unsafe fn tab_pill(hdc: isize, rect: Rect, label: &str, chosen: bool, font: isize, lay: &Layout) {
    let radius = rect.height() / 2;
    let flags = DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
    if chosen {
        card(hdc, rect, radius, Some(ACCENT), None, BG);
        text(hdc, label, rect, font, BG, flags);
    } else {
        card(hdc, rect, radius, Some(CARD), Some((LINE, lay.scale(1).max(1))), BG);
        text(hdc, label, rect, font, MUTED, flags);
    }
}

/// The Show Logs and Keys buttons: a dark card with a thin line, a lime icon and a white label.
unsafe fn tool_button(hdc: isize, rect: Rect, icon: &Icon, label: &str, font: isize, lay: &Layout) {
    card(hdc, rect, lay.scale(12), Some(CARD), Some((LINE, lay.scale(1))), BG);
    let label_width = text_size(hdc, font, label).0;
    let icon_width = lay.scale(icon.w);
    let gap = lay.scale(10);
    let left = rect.left + (rect.width() - (icon_width + gap + label_width)) / 2;
    let icon_slot = Rect { left, top: rect.top, right: left + icon_width, bottom: rect.bottom };
    draw_icon(hdc, icon, icon_slot, ACCENT, CARD, CARD, lay);
    let label_area = Rect { left: left + icon_width + gap, top: rect.top, right: rect.right, bottom: rect.bottom };
    text(hdc, label, label_area, font, TEXT, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
}

/// One of the two output buttons on the Target Encoding card. The chosen one is lime with dark
/// text. The other one is a dark well with a thin line. An icon and the name on the first line,
/// what the output is for on the second line.
unsafe fn encoding_button(hdc: isize, rect: Rect, icon: &Icon, name: &str, note: &str, chosen: bool, lay: &Layout) {
    let (fill_colour, border, name_colour, note_colour) = if chosen {
        (ACCENT, None, BG, BG)
    } else {
        (WELL, Some((LINE, lay.scale(1))), TEXT, DIM)
    };
    card(hdc, rect, lay.scale(12), Some(fill_colour), border, CARD);

    let name_font = FONT_LABEL.load(Relaxed);
    let pad = lay.scale(8);
    let first = Rect { left: rect.left, top: rect.top + pad, right: rect.right, bottom: rect.top + pad + lay.scale(24) };
    let second = Rect { left: rect.left, top: first.bottom, right: rect.right, bottom: rect.bottom - pad };

    let name_width = text_size(hdc, name_font, name).0;
    let icon_width = lay.scale(icon.w);
    let gap = lay.scale(8);
    let left = rect.left + (rect.width() - (icon_width + gap + name_width)) / 2;
    let icon_slot = Rect { left, top: first.top, right: left + icon_width, bottom: first.bottom };
    draw_icon(hdc, icon, icon_slot, name_colour, fill_colour, fill_colour, lay);
    let name_area = Rect { left: left + icon_width + gap, ..first };
    text(hdc, name, name_area, name_font, name_colour, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
    // The line under the name uses the normal small size, and the smaller one when it would not fit.
    let note_area = Rect { left: rect.left + pad, right: rect.right - pad, ..second };
    let note_font = fit_font(hdc, note, note_area.width(), &[FONT_NOTE.load(Relaxed), FONT_NOTE_SMALL.load(Relaxed)]);
    text(hdc, note, note_area, note_font, note_colour, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);
}

/// A key chip ("Right Ctrl") followed by a small word ("toggle"), the pair right-aligned so that
/// it ends at `right`, on the middle line of `row`. Returns the left edge of the pair, so the
/// text on the left knows how far it may go. `behind` is the colour under the chip.
unsafe fn key_group(hdc: isize, row: Rect, right: i32, key: &str, word: &str, behind: u32, lay: &Layout) -> i32 {
    let mono = FONT_MONO.load(Relaxed);
    let word_font = FONT_HINT.load(Relaxed);
    let word_width = text_size(hdc, word_font, word).0;
    let chip_width = text_size(hdc, mono, key).0 + lay.scale(12);
    let chip_height = lay.scale(22);
    let gap = lay.scale(6);

    let word_area = Rect { left: right - word_width, top: row.top, right, bottom: row.bottom };
    let chip_left = right - word_width - gap - chip_width;
    let chip_top = (row.top + row.bottom - chip_height) / 2;
    let chip = Rect { left: chip_left, top: chip_top, right: chip_left + chip_width, bottom: chip_top + chip_height };
    card(hdc, chip, lay.scale(6), Some(CHIP), Some((LINE, lay.scale(1))), behind);
    text(hdc, key, chip, mono, TEXT, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    text(hdc, word, word_area, word_font, DIM, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);
    chip_left
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
    match VIEW.load(Relaxed) {
        1 => View::Keys,
        2 => View::Settings,
        _ => View::Main,
    }
}

/// What every page shares: the background, the lime outline and the title bar.
unsafe fn draw_frame(hdc: isize, width: i32, height: i32, lay: &Layout) {
    let centred = DT_CENTER | DT_VCENTER | DT_SINGLELINE;
    let left_middle = DT_LEFT | DT_VCENTER | DT_SINGLELINE;

    let full = Rect { left: 0, top: 0, right: width, bottom: height };
    fill(hdc, full, BG);
    shape(hdc, full, lay.scale(8), None, Some((ACCENT, lay.scale(2))));
    outline_corners(hdc, width, height, lay);

    // Small logo, name, settings gear, minimise and close, then a thin line under the bar.
    // The gear is lime while the Settings page is open.
    if view() != View::Main {
        // The Settings page and the key hint page: a back arrow, then the logo, the name and the name of the page.
        draw_icon(hdc, &icons::BACK, lay.title_back, TEXT, BG, BG, lay);
        smooth_shape(hdc, lay.crumb_logo, lay.scale(6), Some(ACCENT), None, BG);
        text(hdc, "\u{0D85}", lay.crumb_logo, FONT_LOGO_SMALL.load(Relaxed), BG, centred);
        text(hdc, about::NAME, lay.crumb_text, FONT_SMALL_BOLD.load(Relaxed), TEXT, left_middle);
        let page_name = if view() == View::Keys { about::KEYS_PAGE_NAME } else { about::PAGE_NAME };
        let page = format!("\u{00B7}  {page_name}");
        text(hdc, &page, lay.crumb_page, FONT_HINT.load(Relaxed), MUTED, left_middle | DT_END_ELLIPSIS);
    } else {
        smooth_shape(hdc, lay.title_logo, lay.scale(6), Some(ACCENT), None, BG);
        text(hdc, "\u{0D85}", lay.title_logo, FONT_LOGO_SMALL.load(Relaxed), BG, centred);
        text(hdc, about::NAME, lay.title_text, FONT_SMALL_BOLD.load(Relaxed), TEXT, left_middle);
        // The version badge: lime text on a faint lime fill with a thin lime line.
        card(hdc, lay.title_badge, lay.scale(6), Some(ACCENT_TINT), Some((ACCENT_DIM, lay.scale(1))), BG);
        text(hdc, &about::short_version(), lay.title_badge, FONT_MONO.load(Relaxed), ACCENT, centred);
    }
    let gear_colour = if view() == View::Settings { ACCENT } else { TEXT };
    draw_icon(hdc, &icons::GEAR, lay.settings_button, gear_colour, BG, BG, lay);
    draw_icon(hdc, &icons::MINIMISE, lay.minimise_button, TEXT, BG, BG, lay);
    draw_icon(hdc, &icons::CLOSE, lay.close_button, TEXT, BG, BG, lay);
    fill(hdc, lay.title_rule, LINE);
}

/// An on/off switch: a lime track and a knob on the right when on, a grey track and a knob on the left when off.
/// `behind` is the colour under the switch (the card it sits on).
unsafe fn draw_switch(hdc: isize, track: Rect, on: bool, behind: u32, lay: &Layout) {
    let track_colour = if on { ACCENT } else { EDGE };
    smooth_shape(hdc, track, track.height() / 2, Some(track_colour), None, behind);
    let pad = lay.scale(4);
    let knob = track.height() - 2 * pad;
    let knob_left = if on { track.right - pad - knob } else { track.left + pad };
    let knob_rect = Rect { left: knob_left, top: track.top + pad, right: knob_left + knob, bottom: track.top + pad + knob };
    smooth_shape(hdc, knob_rect, knob / 2, Some(BG), None, track_colour);
}

unsafe fn draw_main(hdc: isize, lay: &Layout) {
    let label_font = FONT_LABEL.load(Relaxed);
    let hint_font = FONT_HINT.load(Relaxed);
    let sub_font = FONT_SUB.load(Relaxed);
    let small_font = FONT_TIP_SMALL.load(Relaxed);
    let bold_font = FONT_SMALL_BOLD.load(Relaxed);
    let left_middle = DT_LEFT | DT_VCENTER | DT_SINGLELINE;
    let enabled = state::enabled();
    let legacy = state::legacy();
    let chosen = settings::get();
    let card_radius = lay.scale(14);
    let inner_radius = lay.scale(10);
    let thin = lay.scale(1).max(1);

    // ---- Sinhala Typing: a tile, the name, a Sinhala line, ON or OFF, the switch,
    // and under them a row that names the key which turns typing on and off.
    card(hdc, lay.typing_card, card_radius, Some(CARD), Some((LINE, thin)), BG);
    card(hdc, lay.typing_icon, inner_radius, Some(CHIP), Some((LINE, thin)), CARD);
    draw_icon(hdc, &icons::KEYBOARD, lay.typing_icon, ACCENT, CHIP, CHIP, lay);
    text(hdc, "Sinhala Typing", lay.typing_label, label_font, TEXT, left_middle | DT_END_ELLIPSIS);
    // Drawn with TextOut (inside `rich`), so tall Sinhala marks are never cut off.
    let typing_line = if enabled { about::TYPING_ON_SI } else { about::TYPING_OFF_SI };
    rich(hdc, lay.typing_sub, &[Part { text: typing_line, font: sub_font, colour: DIM }], false);
    text(
        hdc,
        if enabled { "ON" } else { "OFF" },
        lay.typing_state,
        label_font,
        if enabled { ACCENT } else { DIM },
        DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
    );
    draw_switch(hdc, lay.switch_track, enabled, CARD, lay);

    card(hdc, lay.shortcut_row, inner_radius, Some(WELL), Some((LINE, thin)), CARD);
    draw_icon(hdc, &icons::LINK, lay.shortcut_icon, ACCENT, WELL, WELL, lay);
    let group_left = key_group(
        hdc,
        lay.shortcut_row,
        lay.shortcut_row.right - lay.scale(12),
        settings::key_name(chosen.on_off_key),
        "toggle",
        WELL,
        lay,
    );
    let label_area = Rect { right: group_left - lay.scale(8), ..lay.shortcut_label };
    let label_font = fit_font(
        hdc,
        "Global switch shortcut",
        label_area.width(),
        &[hint_font, FONT_NOTE.load(Relaxed), FONT_NOTE_SMALL.load(Relaxed)],
    );
    text(hdc, "Global switch shortcut", label_area, label_font, TEXT, left_middle | DT_END_ELLIPSIS);

    // ---- Target Encoding: a heading with the key that switches the output, two buttons,
    // and one line about what LIPI is sending now.
    card(hdc, lay.encoding_card, card_radius, Some(CARD), Some((LINE, thin)), BG);
    draw_icon(hdc, &icons::OUTPUT, lay.encoding_icon, ACCENT, CARD, CARD, lay);
    let group_left = key_group(
        hdc,
        lay.encoding_row,
        lay.encoding_row.right,
        settings::key_name(chosen.output_key),
        "to switch",
        CARD,
        lay,
    );
    let heading_area = Rect { right: group_left - lay.scale(8), ..lay.encoding_label };
    text(hdc, "Target Encoding", heading_area, label_font, TEXT, left_middle | DT_END_ELLIPSIS);
    encoding_button(hdc, lay.unicode_button, &icons::GLOBE, about::UNICODE_NAME, about::UNICODE_FOR, !legacy, lay);
    encoding_button(hdc, lay.legacy_button, &icons::LETTER_A, about::LEGACY_NAME, about::LEGACY_FOR, legacy, lay);
    let tick_colour = if enabled { ACCENT } else { DIM };
    draw_icon(hdc, &icons::CHECK_CIRCLE, lay.encoding_status_icon, tick_colour, CARD, CARD, lay);
    text(hdc, about::encoding_status(enabled, legacy), lay.encoding_status, sub_font, MUTED, DT_LEFT | DT_WORDBREAK);

    // ---- Smart quotes warning. In legacy mode the full stop is a straight apostrophe, and Adobe
    // and Microsoft apps turn it into a curly quote. The button opens the page that explains it.
    // The text is wrapped by measuring it, so it never runs under the button, and it takes three
    // lines of 20 pixels. Each line is drawn with TextOut (inside `rich`), so tall Sinhala
    // marks are never cut off. The words "smart quotes" are lime.
    card(hdc, lay.tip_card, lay.scale(12), Some(CARD), Some((ACCENT_DIM, thin)), BG);
    let full = about::tip_full();
    let phrase = full.find(about::TIP_PHRASE).map_or((0, 0), |at| (at, at + about::TIP_PHRASE.len()));
    let room = lay.tip_text;
    let fonts = [small_font, FONT_NOTE_SMALL.load(Relaxed)];
    let mut tip_font = fonts[0];
    let mut lines = about::wrap_words(&full, room.width(), |t| text_size(hdc, fonts[0], t).0);
    if lines.len() > 3 {
        // Wider letters than expected on this computer: try the smaller size.
        tip_font = fonts[1];
        lines = about::wrap_words(&full, room.width(), |t| text_size(hdc, fonts[1], t).0);
    }
    let line_height = room.height() / 3;
    let shown = lines.len().min(3) as i32;
    let block_top = room.top + (room.height() - shown * line_height) / 2;
    for (i, &line) in lines.iter().take(3).enumerate() {
        let area = Rect {
            left: room.left,
            top: block_top + i as i32 * line_height,
            right: room.right,
            bottom: block_top + (i as i32 + 1) * line_height,
        };
        let parts: Vec<Part> = about::pieces(line, phrase)
            .into_iter()
            .map(|(from, to, lime)| Part { text: &full[from..to], font: tip_font, colour: if lime { ACCENT } else { MUTED } })
            .collect();
        rich(hdc, area, &parts, false);
    }
    card(hdc, lay.tip_button, lay.tip_button.height() / 2, Some(ACCENT), None, CARD);
    rich(hdc, lay.tip_button, &[Part { text: about::TIP_BUTTON, font: small_font, colour: BG }], true);

    // ---- Show Logs and Keys
    tool_button(hdc, lay.logs_button, &icons::DOCUMENT, "Show Logs", label_font, lay);
    tool_button(hdc, lay.keys_button, &icons::KEYBOARD_SOLID, "Keys", label_font, lay);

    // ---- Footer: a thin line, then a line, the developer name with a cube, a line
    fill(hdc, lay.footer_rule, LINE);
    let dev = lay.dev_row;
    let dev_label_width = text_size(hdc, hint_font, "DEV: ").0;
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
        &[Part { text: "DEV: ", font: hint_font, colour: DIM }, Part { text: about::DEVELOPER, font: bold_font, colour: ACCENT }],
        false,
    );
    let line_top = (dev.top + dev.bottom - thin) / 2;
    let margin = lay.scale(12);
    fill(hdc, Rect { left: dev.left, top: line_top, right: dev_left - margin, bottom: line_top + thin }, LINE);
    fill(hdc, Rect { left: dev_left + dev_group + margin, top: line_top, right: dev.right, bottom: line_top + thin }, LINE);

    // Contact buttons: GitHub, Facebook, YouTube and the portfolio. Clicking one opens its page in the browser.
    for (button, icon) in [
        (lay.github_button, &icons::GITHUB),
        (lay.facebook_button, &icons::FACEBOOK),
        (lay.youtube_button, &icons::YOUTUBE),
        (lay.portfolio_button, &icons::PORTFOLIO),
    ] {
        draw_icon(hdc, icon, button, ACCENT, BG, BG, lay);
    }
}

/// The heading of the Settings page and the key hint page: a lime dot and the section name on
/// the left, a small grey tag on the right.
unsafe fn page_heading(hdc: isize, lay: &Layout, name: &str, tag: &str) {
    let bold_font = FONT_SMALL_BOLD.load(Relaxed);
    let heading = lay.page_title;
    let dot_size = lay.scale(6);
    let dot_top = (heading.top + heading.bottom - dot_size) / 2;
    let dot = Rect { left: heading.left + lay.scale(2), top: dot_top, right: heading.left + lay.scale(2) + dot_size, bottom: dot_top + dot_size };
    smooth_shape(hdc, dot, dot_size / 2, Some(ACCENT), None, BG);
    text(hdc, name, Rect { left: heading.left + lay.scale(16), ..heading }, bold_font, ACCENT, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
    text(hdc, tag, lay.page_tag, bold_font, DIM, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);
}

/// The key hint page: a heading, four tabs, a grid of letters with their keys, a note.
unsafe fn draw_keys(hdc: isize, lay: &Layout) {
    let tab = Tab::from_index(HINT_TAB.load(Relaxed));
    let page = hints::page(tab);
    let key_font = FONT_HINT.load(Relaxed);
    let glyph_font = FONT_HINT_SI.load(Relaxed);
    let thin = lay.scale(1).max(1);

    page_heading(hdc, lay, about::KEYS_HEADING, about::KEYS_TAG);
    for (i, t) in hints::TABS.iter().enumerate() {
        tab_pill(hdc, lay.hint_tab(i), t.label(), *t == tab, key_font, lay);
    }

    // The grid: every letter in a dark card with a thin line, the Sinhala in lime and its keys in white.
    let rows = page.rows();
    let radius = lay.scale(8);
    for (i, c) in page.cells.iter().enumerate() {
        let cell = lay.hint_cell(page.columns, rows, i);
        card(hdc, cell, radius, Some(CARD), Some((LINE, thin)), BG);
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
    text(hdc, &page.note, note_area, FONT_NOTE.load(Relaxed), DIM, DT_LEFT | DT_WORDBREAK);
}

/// One switch card on the Settings page: a tile with an icon, the name, a short note and the
/// switch, then a thin line and a detail row with a tick, a line of text and a status word.
/// The icon and the tick are lime while the setting is on, grey while it is off.
unsafe fn setting_card(
    hdc: isize,
    c: &SettingCard,
    icon: &Icon,
    (title, note, detail, status): (&str, &str, &str, &str),
    on: bool,
    lay: &Layout,
) {
    let left_middle = DT_LEFT | DT_VCENTER | DT_SINGLELINE;
    let thin = lay.scale(1).max(1);
    let tint = if on { ACCENT } else { DIM };

    card(hdc, c.card, lay.scale(14), Some(CARD), Some((LINE, thin)), BG);
    card(hdc, c.tile, lay.scale(10), Some(CHIP), Some((LINE, thin)), CARD);
    draw_icon(hdc, icon, c.tile, tint, CHIP, CHIP, lay);
    text(hdc, title, c.title, FONT_LABEL.load(Relaxed), TEXT, left_middle | DT_END_ELLIPSIS);
    text(hdc, note, c.note, FONT_NOTE.load(Relaxed), DIM, DT_LEFT | DT_WORDBREAK);
    draw_switch(hdc, c.control, on, CARD, lay);

    fill(hdc, c.rule, LINE);
    draw_icon(hdc, &icons::CHECK_CIRCLE, c.detail_icon, tint, CARD, CARD, lay);
    text(hdc, detail, c.detail_text, FONT_HINT.load(Relaxed), DIM, left_middle | DT_END_ELLIPSIS);
    text(hdc, status, c.detail_state, FONT_SMALL_BOLD.load(Relaxed), tint, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);
}

/// The Settings page: a heading, one card for each switch, the update card,
/// a bar that says changes are saved, and a footer.
unsafe fn draw_settings(hdc: isize, lay: &Layout) {
    let label_font = FONT_LABEL.load(Relaxed);
    let note_font = FONT_NOTE.load(Relaxed);
    let hint_font = FONT_HINT.load(Relaxed);
    let bold_font = FONT_SMALL_BOLD.load(Relaxed);
    let left_middle = DT_LEFT | DT_VCENTER | DT_SINGLELINE;
    let thin = lay.scale(1).max(1);
    let card_radius = lay.scale(14);

    // ---- Heading (the same one the key hint page has), and one line of text under it.
    page_heading(hdc, lay, about::SETTINGS_HEADING, about::SETTINGS_TAG);
    text(hdc, about::SETTINGS_LEAD, lay.page_lead, note_font, DIM, left_middle | DT_END_ELLIPSIS);

    // ---- The two switches
    let startup_on = STARTUP_ON.load(Relaxed);
    let tray_on = TRAY_START_ON.load(Relaxed);
    setting_card(
        hdc,
        &lay.startup,
        &icons::POWER,
        (about::STARTUP_TITLE, about::STARTUP_NOTE, about::STARTUP_DETAIL, about::startup_status(startup_on)),
        startup_on,
        lay,
    );
    setting_card(
        hdc,
        &lay.tray,
        &icons::TRAY,
        (about::TRAY_TITLE, about::TRAY_NOTE, about::TRAY_DETAIL, about::tray_status(tray_on)),
        tray_on,
        lay,
    );

    // ---- Updates: the version in a chip, and a button that opens the newest release on GitHub
    // in the browser. LIPI itself never goes online. This card has a lime line, like the
    // card that stands out in the design.
    let u = &lay.update;
    card(hdc, u.card, card_radius, Some(CARD), Some((ACCENT_DIM, thin)), BG);
    card(hdc, u.tile, lay.scale(10), Some(CHIP), Some((ACCENT_DIM, thin)), CARD);
    draw_icon(hdc, &icons::DOWNLOAD, u.tile, ACCENT, CHIP, CHIP, lay);
    text(hdc, about::UPDATE_TITLE, u.title, label_font, TEXT, left_middle | DT_END_ELLIPSIS);
    text(hdc, about::UPDATE_NOTE, u.note, note_font, DIM, DT_LEFT | DT_WORDBREAK);
    card(hdc, u.chip, lay.scale(6), Some(ACCENT_TINT), Some((ACCENT_DIM, thin)), CARD);
    text(hdc, &about::version_chip(), u.chip, FONT_MONO.load(Relaxed), ACCENT, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    fill(hdc, u.rule, LINE);
    pill_on(hdc, u.button, about::UPDATE_BUTTON, true, label_font, CARD, lay);

    // ---- A bar: changes are saved by themselves, and where the file is
    card(hdc, lay.saved_bar, lay.scale(12), Some(CARD), Some((LINE, thin)), BG);
    smooth_shape(hdc, lay.saved_dot, lay.saved_dot.width() / 2, Some(ACCENT), None, CARD);
    text(hdc, about::SAVED_NOTE, lay.saved_text, hint_font, TEXT, left_middle | DT_END_ELLIPSIS);
    text(hdc, about::SAVED_PATH, lay.saved_path, FONT_MONO.load(Relaxed), DIM, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);

    // ---- Footer: a lime dot, the engine version and the developer on the left, a GitHub link on the right.
    let dot_size = lay.footer_dot.width();
    smooth_shape(hdc, lay.footer_dot, dot_size / 2, Some(ACCENT), None, BG);
    let engine = about::engine_line();
    rich(
        hdc,
        lay.footer_text,
        &[
            Part { text: &engine, font: hint_font, colour: DIM },
            Part { text: "  \u{00B7}  ", font: hint_font, colour: DIM },
            Part { text: "DEV: ", font: hint_font, colour: DIM },
            Part { text: about::DEVELOPER, font: bold_font, colour: MUTED },
        ],
        false,
    );
    let link = lay.footer_github;
    let label_width = text_size(hdc, hint_font, about::GITHUB_LABEL).0;
    let icon_width = lay.scale(icons::CODE.w);
    let gap = lay.scale(6);
    let left = link.right - (icon_width + gap + label_width);
    let icon_slot = Rect { left, top: link.top, right: left + icon_width, bottom: link.bottom };
    draw_icon(hdc, &icons::CODE, icon_slot, ACCENT, BG, BG, lay);
    text(hdc, about::GITHUB_LABEL, Rect { left: left + icon_width + gap, ..link }, hint_font, MUTED, left_middle);
}

unsafe fn draw(hdc: isize, width: i32, height: i32) {
    let lay = Layout::new(DPI.load(Relaxed));
    draw_frame(hdc, width, height, &lay);
    match view() {
        View::Main => draw_main(hdc, &lay),
        View::Keys => draw_keys(hdc, &lay),
        View::Settings => draw_settings(hdc, &lay),
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
        "{}: {}, {}\nLeft click: open. Right click: menu.",
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

/// The tray icon's right-click menu: Open, Sinhala typing (ticked when on), Exit.
/// It is made when the icon is clicked and thrown away straight after, so it costs no memory while idle.
unsafe fn tray_menu(hwnd: isize) {
    let menu = CreatePopupMenu();
    if menu == 0 {
        return;
    }
    let typing_flags = if state::enabled() { MF_STRING | MF_CHECKED } else { MF_STRING };
    AppendMenuW(menu, MF_STRING, MENU_OPEN, wide("Open LIPI").as_ptr());
    AppendMenuW(menu, typing_flags, MENU_TYPING, wide("Sinhala typing").as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    AppendMenuW(menu, MF_STRING, MENU_EXIT, wide("Exit").as_ptr());

    let mut point = Point { x: 0, y: 0 };
    GetCursorPos(&mut point);
    // Windows needs the window in front, or the menu does not close when the user clicks elsewhere.
    SetForegroundWindow(hwnd);
    let choice = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_BOTTOMALIGN,
        point.x,
        point.y,
        0,
        hwnd,
        null(),
    );
    PostMessageW(hwnd, WM_NULL, 0, 0); // the other half of the menu close fix
    DestroyMenu(menu);

    match choice as usize {
        MENU_OPEN => show_window(hwnd),
        MENU_TYPING => {
            let on = !state::enabled();
            state::set_enabled(on);
            state::log(if on { "Sinhala typing ON (tray)" } else { "Sinhala typing OFF (tray)" });
            refresh(hwnd);
        }
        // The same as the close button: the window closes, the tray icon goes, the program ends.
        MENU_EXIT => {
            state::log("Exit (tray)");
            SendMessageW(hwnd, WM_CLOSE, 0, 0);
        }
        _ => {}
    }
    SetTimer(hwnd, TRIM_TIMER_ID, 3000, null()); // give the memory back a moment later
}

/// True if another copy of LIPI is already running for this Windows user.
/// The name is kept open until the program ends, so the second copy always sees it.
unsafe fn another_copy_is_running() -> bool {
    let name = wide("Local\\LIPI-single-instance");
    let handle = CreateMutexW(null(), 0, name.as_ptr());
    // Read the error straight away, before anything else can change it.
    let already = GetLastError() == ERROR_ALREADY_EXISTS;
    // If Windows would not make the name at all, do not block the user: just carry on.
    handle != 0 && already
}

/// Asks the first copy to show its window. Returns false if that could not be done.
unsafe fn show_first_copy() -> bool {
    // The first copy may still be starting up, so look a few times.
    for _ in 0..10 {
        let window = FindWindowW(wide(MAIN_CLASS).as_ptr(), null());
        if window != 0 {
            AllowSetForegroundWindow(ASFW_ANY);
            // This fails when the first copy runs as administrator and this one does not.
            return PostMessageW(window, WM_SHOW_LIPI, 0, 0) != 0;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    false
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

/// Settings page: turns "Run on startup" on or off in the registry.
fn toggle_run_on_startup() {
    let want = !STARTUP_ON.load(Relaxed);
    match autostart::set_enabled(want) {
        Ok(()) => state::log(if want { "Run on startup ON (settings)" } else { "Run on startup OFF (settings)" }),
        Err(e) => state::log(format!("Could not change run on startup: {e}")),
    }
    // Show what Windows really has, not what was asked for.
    let now = autostart::is_enabled();
    STARTUP_ON.store(now, Relaxed);
    if want && !now {
        state::log("Windows still has LIPI switched off. Turn it on in Task Manager, Startup apps.");
    }
}

/// Settings page: saves "Start in tray" in settings.txt. Only that line of the file changes.
fn toggle_start_in_tray() {
    let want = !TRAY_START_ON.load(Relaxed);
    match settings::save_start_in_tray(&settings::current_folder(), want) {
        Ok(()) => {
            TRAY_START_ON.store(want, Relaxed);
            state::log(if want { "Start in tray ON (settings)" } else { "Start in tray OFF (settings)" });
        }
        Err(e) => state::log(format!("Could not save settings.txt: {e}")),
    }
}

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
        // The gear opens Settings, and closes it again when it is already open.
        Hit::Settings => {
            if view() == View::Settings {
                VIEW.store(0, Relaxed);
            } else {
                STARTUP_ON.store(autostart::is_enabled(), Relaxed); // what Windows has right now
                VIEW.store(2, Relaxed);
            }
        }
        Hit::StartupCard => toggle_run_on_startup(),
        Hit::TrayCard => toggle_start_in_tray(),
        Hit::Tab(i) => HINT_TAB.store(i, Relaxed),
        Hit::TipButton => {
            open_link(about::TIP_URL);
            return;
        }
        // No network code in LIPI: the browser opens the release page, the user compares the numbers.
        Hit::UpdateButton => {
            state::log("Opened the GitHub releases page (check for updates)");
            open_link(about::RELEASES_URL);
            return;
        }
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
        Hit::Portfolio => {
            open_link(about::PORTFOLIO_URL);
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
            // Esc closes the key hint page and the Settings page.
            WM_KEYDOWN => {
                if wparam == VK_ESCAPE && view() != View::Main {
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
                    WM_RBUTTONUP => tray_menu(hwnd),
                    _ => {}
                }
                Some(0)
            }
            // Someone started LIPI a second time. This copy shows its window instead.
            WM_SHOW_LIPI => {
                show_window(hwnd);
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
    // Only one LIPI at a time. This must come before the keyboard hook starts, or two hooks
    // would both type Sinhala letters for each key.
    unsafe {
        if another_copy_is_running() {
            if !show_first_copy() {
                MessageBoxW(
                    0,
                    wide("LIPI is already running.\nPlease check Hidden Icons (the ^ arrow near the clock).").as_ptr(),
                    wide(about::NAME).as_ptr(),
                    MB_ICONINFORMATION | MB_SETFOREGROUND,
                );
            }
            return;
        }
    }
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
        FONT_LABEL.store(make_font(lay.scale(16), 700, "Segoe UI"), Relaxed);
        FONT_SMALL_BOLD.store(make_font(lay.scale(14), 700, "Segoe UI"), Relaxed);
        FONT_LOGO_SMALL.store(make_font(lay.scale(15), 700, "Nirmala UI"), Relaxed);
        FONT_HINT.store(make_font(lay.scale(13), 600, "Segoe UI"), Relaxed);
        FONT_HINT_SI.store(make_font(lay.scale(19), 700, "Nirmala UI"), Relaxed);
        FONT_NOTE.store(make_font(lay.scale(12), 600, "Nirmala UI"), Relaxed);
        FONT_MONO.store(make_font(lay.scale(12), 700, "Consolas"), Relaxed);
        FONT_SUB.store(make_font(lay.scale(13), 600, "Nirmala UI"), Relaxed);
        FONT_LOG.store(make_font(lay.scale(16), 400, "Nirmala UI"), Relaxed);
        // The smart quotes card: Nirmala UI draws both the Sinhala and the English words.
        FONT_TIP_SMALL.store(make_font(lay.scale(12), 700, "Nirmala UI"), Relaxed);
        FONT_NOTE_SMALL.store(make_font(lay.scale(11), 600, "Nirmala UI"), Relaxed);

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
        TRAY_START_ON.store(settings::get().start_in_tray, Relaxed);
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
