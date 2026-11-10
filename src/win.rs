//! Windows only. A global keyboard hook that turns Singlish keystrokes into
//! Sinhala in whatever app has focus.
//!
//! How it works:
//!   1. A low-level keyboard hook sees every key before the app does.
//!   2. A letter is swallowed. The `Session` works out what the word should
//!      look like now, and `SendInput` types the difference into the app
//!      (some Backspaces, then the new text as Unicode characters).
//!   3. Space, Enter, digits, arrows and shortcuts are passed through and
//!      end the word. A mouse click or a change of window also ends the word.
//!
//! No external crates. The few Win32 functions are declared by hand below.

use std::cell::RefCell;
use std::mem::size_of;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::null;
use std::thread::JoinHandle;

use crate::engine::Edit;
use crate::keymap::*;
use crate::legacy::LegacyMap;
use crate::session::{Output, Session};
use crate::state;

// ---------------------------------------------------------------- Win32 bits

const WH_KEYBOARD_LL: i32 = 13;
const WH_MOUSE_LL: i32 = 14;
const HC_ACTION: i32 = 0;

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_RBUTTONDOWN: u32 = 0x0204;
const WM_MBUTTONDOWN: u32 = 0x0207;

const LLKHF_INJECTED: u32 = 0x10;

const INPUT_KEYBOARD: u32 = 1;
const KEYEVENTF_KEYUP: u32 = 0x0002;
const KEYEVENTF_UNICODE: u32 = 0x0004;

/// Put on every key we send, so they can be recognised in a debugger.
const OUR_EXTRA_INFO: usize = 0x5347_4C53;

type HookProc = unsafe extern "system" fn(i32, usize, isize) -> isize;

#[repr(C)]
struct KbdLlHookStruct {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    extra_info: usize,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
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
#[derive(Clone, Copy)]
struct KeybdInput {
    vk: u16,
    scan: u16,
    flags: u32,
    time: u32,
    extra: usize,
}

// Only used to make the union as big as the real INPUT union.
// SendInput refuses the call if the size we pass is wrong.
#[repr(C)]
#[derive(Clone, Copy)]
struct MouseInput {
    dx: i32,
    dy: i32,
    data: u32,
    flags: u32,
    time: u32,
    extra: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
union InputUnion {
    mi: MouseInput,
    ki: KeybdInput,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Input {
    kind: u32,
    u: InputUnion,
}

// Checked while compiling. These are the sizes Windows expects.
const _: () = {
    let wide = size_of::<usize>() == 8;
    assert!(size_of::<Input>() == if wide { 40 } else { 28 });
    assert!(size_of::<KbdLlHookStruct>() == if wide { 24 } else { 20 });
    assert!(size_of::<Msg>() == if wide { 48 } else { 28 });
};

#[link(name = "user32")]
extern "system" {
    fn SetWindowsHookExW(id_hook: i32, hook: Option<HookProc>, module: isize, thread_id: u32) -> isize;
    fn UnhookWindowsHookEx(hook: isize) -> i32;
    fn CallNextHookEx(hook: isize, code: i32, wparam: usize, lparam: isize) -> isize;
    fn GetMessageW(msg: *mut Msg, hwnd: isize, filter_min: u32, filter_max: u32) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn GetAsyncKeyState(vk: i32) -> i16;
    fn GetKeyState(vk: i32) -> i16;
    fn GetForegroundWindow() -> isize;
    fn SendInput(count: u32, inputs: *const Input, size: i32) -> u32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> isize;
}

// --------------------------------------------------------------------- state

struct State {
    session: Session,
    taps: Taps,
    /// Keys whose key-down we swallowed. Their key-up is swallowed too.
    swallowed: [bool; 256],
    last_window: isize,
    caps_on: bool,
    caps_held: bool,
}

thread_local! {
    // The hook runs on the thread that installed it, so thread-local storage is enough.
    static STATE: RefCell<Option<State>> = RefCell::new(None);
}

fn held(vk: u32) -> bool {
    (unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000) != 0
}

// -------------------------------------------------------------- sending text

fn key_event(vk: u16, scan: u16, flags: u32) -> Input {
    Input {
        kind: INPUT_KEYBOARD,
        u: InputUnion {
            ki: KeybdInput { vk, scan, flags, time: 0, extra: OUR_EXTRA_INFO },
        },
    }
}

/// Types the edit into the focused app: Backspaces first, then the new text.
fn apply_edit(edit: &Edit) {
    let mut events: Vec<Input> = Vec::new();
    for _ in 0..edit.backspaces {
        events.push(key_event(VK_BACK as u16, 0, 0));
        events.push(key_event(VK_BACK as u16, 0, KEYEVENTF_KEYUP));
    }
    for unit in edit.text.encode_utf16() {
        events.push(key_event(0, unit, KEYEVENTF_UNICODE));
        events.push(key_event(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
    }
    if events.is_empty() {
        return;
    }
    let sent = unsafe { SendInput(events.len() as u32, events.as_ptr(), size_of::<Input>() as i32) };
    if sent as usize != events.len() {
        state::log(format!(
            "SendInput sent {sent} of {} events. The app may be running as administrator.",
            events.len()
        ));
    }
}

// ------------------------------------------------------------- key handling

fn toggle_on_off(st: &mut State) {
    let on = !state::enabled();
    state::set_enabled(on);
    st.session.reset();
    state::log(if on { "Sinhala typing ON" } else { "Sinhala typing OFF" });
}

fn toggle_output(st: &mut State) {
    let legacy = !state::legacy();
    state::set_legacy(legacy);
    st.session.reset();
    state::log(if legacy { "Output: legacy (FM Abhaya style)" } else { "Output: Unicode" });
}

fn log_edit(st: &State, key: &str, edit: &Edit) {
    state::log(format!(
        "{key}  word \"{}\"  screen \"{}\"  edit: delete {}, type \"{}\"",
        st.session.word(),
        st.session.shown(),
        edit.backspaces,
        edit.text
    ));
}

/// Returns true when the key must be swallowed (the app must not see it).
fn on_key(st: &mut State, msg: u32, k: &KbdLlHookStruct) -> bool {
    let vk = k.vk_code;
    let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
    let up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

    // Track Caps Lock ourselves. Reading it from inside a hook is not reliable.
    // Holding the key repeats key-down but only the first one toggles.
    if vk == VK_CAPITAL {
        if down && !st.caps_held {
            st.caps_on = !st.caps_on;
            st.caps_held = true;
        }
        if up {
            st.caps_held = false;
        }
    }

    // Keys sent by SendInput (ours or another program's) are not typing.
    if k.flags & LLKHF_INJECTED != 0 {
        return false;
    }

    if up {
        match st.taps.key_up(vk) {
            Some(Toggle::OnOff) => toggle_on_off(st),
            Some(Toggle::Output) => toggle_output(st),
            None => {}
        }
        let slot = (vk & 0xFF) as usize;
        let was_swallowed = st.swallowed[slot];
        st.swallowed[slot] = false;
        return was_swallowed;
    }
    if !down {
        return false;
    }

    st.taps.key_down(vk);

    if !state::enabled() {
        st.session.reset();
        return false;
    }

    st.session
        .set_mode(if state::legacy() { Output::Legacy } else { Output::Unicode });

    // A different window means the cursor is somewhere else: new word.
    let window = unsafe { GetForegroundWindow() };
    if window != st.last_window {
        st.last_window = window;
        st.session.reset();
    }

    let shift = held(VK_SHIFT);
    let shortcut = held(VK_CONTROL) || held(VK_MENU) || held(VK_LWIN) || held(VK_RWIN);
    let slot = (vk & 0xFF) as usize;

    match classify(vk, shift, st.caps_on, shortcut) {
        Key::Ignore => false,
        Key::Break => {
            st.session.reset();
            false
        }
        Key::Letter(ch) => {
            let edit = st.session.push(ch);
            log_edit(st, &format!("'{ch}'"), &edit);
            apply_edit(&edit);
            st.swallowed[slot] = true;
            true
        }
        Key::Backspace => match st.session.pop() {
            Some(edit) => {
                log_edit(st, "Backspace", &edit);
                apply_edit(&edit);
                st.swallowed[slot] = true;
                true
            }
            None => false,
        },
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: usize, lparam: isize) -> isize {
    if code == HC_ACTION {
        let info = &*(lparam as *const KbdLlHookStruct);
        // A panic must never cross into Windows. On any problem, let the key through.
        let swallow = catch_unwind(AssertUnwindSafe(|| {
            STATE.with(|cell| match cell.try_borrow_mut() {
                Ok(mut guard) => match guard.as_mut() {
                    Some(st) => on_key(st, wparam as u32, info),
                    None => false,
                },
                Err(_) => false,
            })
        }))
        .unwrap_or(false);
        if swallow {
            return 1;
        }
    }
    CallNextHookEx(0, code, wparam, lparam)
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: usize, lparam: isize) -> isize {
    if code == HC_ACTION && matches!(wparam as u32, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN) {
        // A click moves the cursor, so the word in progress is over.
        let _ = catch_unwind(AssertUnwindSafe(|| {
            STATE.with(|cell| {
                if let Ok(mut guard) = cell.try_borrow_mut() {
                    if let Some(st) = guard.as_mut() {
                        st.session.reset();
                        st.taps.cancel();
                    }
                }
            })
        }));
    }
    CallNextHookEx(0, code, wparam, lparam)
}

// ------------------------------------------------------------------- startup

fn hook_thread(map: LegacyMap) {
    let caps_on = (unsafe { GetKeyState(VK_CAPITAL as i32) } & 1) != 0;
    STATE.with(|cell| {
        *cell.borrow_mut() = Some(State {
            session: Session::new(map),
            taps: Taps::default(),
            swallowed: [false; 256],
            last_window: 0,
            caps_on,
            caps_held: false,
        });
    });

    unsafe {
        let module = GetModuleHandleW(null());
        let keyboard = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
        if keyboard == 0 {
            state::log("Could not install the keyboard hook.");
            return;
        }
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);

        state::log(format!(
            "Hook installed. Sinhala typing is {}. Output: {}.",
            if state::enabled() { "ON" } else { "OFF" },
            if state::legacy() { "legacy" } else { "Unicode" }
        ));

        // Low-level hooks only work while this thread keeps reading messages.
        let mut msg: Msg = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(keyboard);
        if mouse != 0 {
            UnhookWindowsHookEx(mouse);
        }
    }
}

/// Starts the hook on its own thread. Set `state::set_legacy` before calling if
/// you want to begin with legacy output.
pub fn start(map: LegacyMap) -> JoinHandle<()> {
    std::thread::spawn(move || hook_thread(map))
}
