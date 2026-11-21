//! Decides what each key press means. No Windows calls in here, so it can be tested anywhere.

// Windows virtual key codes.
pub const VK_BACK: u32 = 0x08;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_MENU: u32 = 0x12; // Alt
pub const VK_PAUSE: u32 = 0x13;
pub const VK_CAPITAL: u32 = 0x14; // Caps Lock
pub const VK_LWIN: u32 = 0x5B;
pub const VK_RWIN: u32 = 0x5C;
pub const VK_NUMLOCK: u32 = 0x90;
pub const VK_SCROLL: u32 = 0x91;
pub const VK_LSHIFT: u32 = 0xA0;
pub const VK_RSHIFT: u32 = 0xA1;
pub const VK_LCONTROL: u32 = 0xA2;
pub const VK_RCONTROL: u32 = 0xA3;
pub const VK_LMENU: u32 = 0xA4;
pub const VK_RMENU: u32 = 0xA5;

/// Used when settings.txt does not say otherwise.
/// Tap this key alone to switch Sinhala typing on or off.
pub const DEFAULT_ON_OFF_VK: u32 = VK_RCONTROL;
/// Tap this key alone to switch between Unicode and legacy output.
pub const DEFAULT_OUTPUT_VK: u32 = VK_RSHIFT;

#[derive(Debug, PartialEq, Eq)]
pub enum Key {
    /// A letter that belongs to the word being typed.
    Letter(char),
    /// Backspace. Edits the word if one is in progress.
    Backspace,
    /// A modifier or lock key. Changes nothing about the word.
    Ignore,
    /// Anything else (space, Enter, digits, arrows, shortcuts). The word is over.
    Break,
}

/// `shift` and `caps` decide the case of letters.
/// `shortcut` is true while Ctrl, Alt or Win is held. Those are never typing.
pub fn classify(vk: u32, shift: bool, caps: bool, shortcut: bool) -> Key {
    match vk {
        VK_SHIFT | VK_LSHIFT | VK_RSHIFT | VK_CONTROL | VK_LCONTROL | VK_RCONTROL | VK_MENU
        | VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN | VK_CAPITAL | VK_NUMLOCK | VK_SCROLL => {
            Key::Ignore
        }
        _ if shortcut => Key::Break,
        0x41..=0x5A => {
            let upper = vk as u8 as char; // 'A'..='Z'
            Key::Letter(if shift ^ caps { upper } else { upper.to_ascii_lowercase() })
        }
        VK_BACK => Key::Backspace,
        _ => Key::Break,
    }
}

/// What each digit key types while Shift is held, on a US keyboard: ) ! @ # $ % ^ & * (
const SHIFTED_DIGITS: &[u8; 10] = b")!@#$%^&*(";

/// The character a digit or punctuation key types on a US keyboard.
/// Returns None for every other key (letters, space, Enter, arrows, modifiers),
/// and for shortcuts, because Ctrl, Alt and Win never type.
///
/// Digits are included, because the shifted digits are marks (! @ # $ % ^ & * and brackets).
/// Only the US layout is known. On another layout the same key types another character.
pub fn punctuation(vk: u32, shift: bool, shortcut: bool) -> Option<char> {
    if shortcut {
        return None;
    }
    let pick = |plain: char, shifted: char| Some(if shift { shifted } else { plain });
    match vk {
        // Top row digits.
        0x30..=0x39 => {
            let digit = (vk - 0x30) as u8;
            Some(if shift { SHIFTED_DIGITS[digit as usize] as char } else { (b'0' + digit) as char })
        }
        // Number pad. Shift does not change these.
        0x60..=0x69 => Some((b'0' + (vk - 0x60) as u8) as char),
        0x6A => Some('*'),
        0x6B => Some('+'),
        0x6D => Some('-'),
        0x6E => Some('.'),
        0x6F => Some('/'),
        // Punctuation keys (VK_OEM_*).
        0xBA => pick(';', ':'),
        0xBB => pick('=', '+'),
        0xBC => pick(',', '<'),
        0xBD => pick('-', '_'),
        0xBE => pick('.', '>'),
        0xBF => pick('/', '?'),
        0xC0 => pick('`', '~'),
        0xDB => pick('[', '{'),
        0xDC => pick('\\', '|'),
        0xDD => pick(']', '}'),
        0xDE => pick('\'', '"'),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Toggle {
    OnOff,
    Output,
}

/// Detects a "tap": a toggle key pressed and released with no other key in between.
pub struct Taps {
    on_off: u32,
    output: u32,
    armed: Option<u32>,
}

impl Taps {
    pub fn new(on_off: u32, output: u32) -> Self {
        Self { on_off, output, armed: None }
    }

    pub fn key_down(&mut self, vk: u32) {
        self.armed = if vk == self.on_off || vk == self.output { Some(vk) } else { None };
    }

    /// Returns the toggle that was just tapped, if any.
    pub fn key_up(&mut self, vk: u32) -> Option<Toggle> {
        if self.armed != Some(vk) {
            return None;
        }
        self.armed = None;
        if vk == self.on_off {
            Some(Toggle::OnOff)
        } else {
            Some(Toggle::Output)
        }
    }

    /// A mouse click happened, so the tap is no longer a clean tap.
    pub fn cancel(&mut self) {
        self.armed = None;
    }
}

/// Uses the keys chosen in settings.txt.
impl Default for Taps {
    fn default() -> Self {
        let settings = crate::settings::get();
        Self::new(settings.on_off_key, settings.output_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taps() -> Taps {
        Taps::new(DEFAULT_ON_OFF_VK, DEFAULT_OUTPUT_VK)
    }

    #[test]
    fn letters_follow_shift_and_caps() {
        assert_eq!(classify(0x41, false, false, false), Key::Letter('a'));
        assert_eq!(classify(0x41, true, false, false), Key::Letter('A'));
        assert_eq!(classify(0x41, false, true, false), Key::Letter('A'));
        assert_eq!(classify(0x41, true, true, false), Key::Letter('a'));
        assert_eq!(classify(0x5A, false, false, false), Key::Letter('z'));
    }

    #[test]
    fn shortcuts_are_not_typing() {
        assert_eq!(classify(0x43, false, false, true), Key::Break); // Ctrl+C
        assert_eq!(classify(VK_BACK, false, false, true), Key::Break); // Ctrl+Backspace
    }

    #[test]
    fn modifiers_do_not_end_the_word() {
        for vk in [VK_SHIFT, VK_LSHIFT, VK_RCONTROL, VK_LMENU, VK_CAPITAL, VK_LWIN] {
            assert_eq!(classify(vk, false, false, false), Key::Ignore);
            assert_eq!(classify(vk, false, false, true), Key::Ignore);
        }
    }

    #[test]
    fn other_keys_end_the_word() {
        assert_eq!(classify(0x20, false, false, false), Key::Break); // space
        assert_eq!(classify(0x0D, false, false, false), Key::Break); // enter
        assert_eq!(classify(0x31, false, false, false), Key::Break); // 1
        assert_eq!(classify(0x25, false, false, false), Key::Break); // left arrow
        assert_eq!(classify(VK_BACK, false, false, false), Key::Backspace);
    }

    // ---------- punctuation keys (US keyboard) ----------

    #[test]
    fn the_six_marks_you_asked_about() {
        assert_eq!(punctuation(0xBF, true, false), Some('?')); // Shift + /
        assert_eq!(punctuation(0xBE, false, false), Some('.'));
        assert_eq!(punctuation(0xBA, true, false), Some(':')); // Shift + ;
        assert_eq!(punctuation(0x35, true, false), Some('%')); // Shift + 5
        assert_eq!(punctuation(0x36, true, false), Some('^')); // Shift + 6
        assert_eq!(punctuation(0x37, true, false), Some('&')); // Shift + 7
    }

    #[test]
    fn digits_with_and_without_shift() {
        for (i, plain) in "0123456789".chars().enumerate() {
            assert_eq!(punctuation(0x30 + i as u32, false, false), Some(plain));
        }
        for (i, marked) in ")!@#$%^&*(".chars().enumerate() {
            assert_eq!(punctuation(0x30 + i as u32, true, false), Some(marked));
        }
    }

    #[test]
    fn punctuation_keys_with_and_without_shift() {
        let keys = [
            (0xBA, ';', ':'), (0xBB, '=', '+'), (0xBC, ',', '<'), (0xBD, '-', '_'), (0xBE, '.', '>'),
            (0xBF, '/', '?'), (0xC0, '`', '~'), (0xDB, '[', '{'), (0xDC, '\\', '|'), (0xDD, ']', '}'),
            (0xDE, '\'', '"'),
        ];
        for (vk, plain, shifted) in keys {
            assert_eq!(punctuation(vk, false, false), Some(plain), "vk {vk:#X}");
            assert_eq!(punctuation(vk, true, false), Some(shifted), "vk {vk:#X} + Shift");
        }
    }

    #[test]
    fn number_pad_ignores_shift() {
        for shift in [false, true] {
            assert_eq!(punctuation(0x60, shift, false), Some('0'));
            assert_eq!(punctuation(0x69, shift, false), Some('9'));
            assert_eq!(punctuation(0x6A, shift, false), Some('*'));
            assert_eq!(punctuation(0x6B, shift, false), Some('+'));
            assert_eq!(punctuation(0x6D, shift, false), Some('-'));
            assert_eq!(punctuation(0x6E, shift, false), Some('.'));
            assert_eq!(punctuation(0x6F, shift, false), Some('/'));
        }
    }

    #[test]
    fn shortcuts_never_type_a_mark() {
        assert_eq!(punctuation(0xBF, true, true), None); // Ctrl + ?
        assert_eq!(punctuation(0x35, true, true), None);
        assert_eq!(punctuation(0xBE, false, true), None); // Ctrl + .
        assert_eq!(punctuation(0x31, false, true), None); // Alt + 1
    }

    #[test]
    fn letters_and_other_keys_are_not_punctuation() {
        for vk in 0x41..=0x5A {
            assert_eq!(punctuation(vk, false, false), None);
            assert_eq!(punctuation(vk, true, false), None);
        }
        for vk in [VK_BACK, 0x09, 0x0D, 0x1B, 0x20, 0x25, 0x26, 0x27, 0x28, 0x2E, 0x70, VK_SHIFT, VK_RCONTROL, VK_CAPITAL] {
            assert_eq!(punctuation(vk, false, false), None, "vk {vk:#X}");
        }
    }

    #[test]
    fn every_ascii_mark_can_be_typed() {
        let mut typable: Vec<char> = Vec::new();
        for vk in 0..=0xFF {
            for shift in [false, true] {
                if let Some(c) = punctuation(vk, shift, false) {
                    typable.push(c);
                }
            }
        }
        for mark in "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars() {
            assert!(typable.contains(&mark), "no key types {mark}");
        }
    }

    #[test]
    fn a_punctuation_key_is_always_a_break_for_the_word() {
        // The hook handles marks in the Break case, so no mark may be a letter or a modifier.
        for vk in 0..=0xFF {
            for shift in [false, true] {
                for caps in [false, true] {
                    if punctuation(vk, shift, false).is_some() {
                        assert_eq!(classify(vk, shift, caps, false), Key::Break, "vk {vk:#X}");
                    }
                }
            }
        }
    }

    #[test]
    fn tap_toggles() {
        let mut t = taps();
        t.key_down(DEFAULT_ON_OFF_VK);
        assert_eq!(t.key_up(DEFAULT_ON_OFF_VK), Some(Toggle::OnOff));

        t.key_down(DEFAULT_OUTPUT_VK);
        assert_eq!(t.key_up(DEFAULT_OUTPUT_VK), Some(Toggle::Output));
    }

    #[test]
    fn holding_with_another_key_is_not_a_tap() {
        let mut t = taps();
        t.key_down(DEFAULT_ON_OFF_VK);
        t.key_down(0x43); // C
        assert_eq!(t.key_up(0x43), None);
        assert_eq!(t.key_up(DEFAULT_ON_OFF_VK), None);

        t.key_down(DEFAULT_ON_OFF_VK);
        t.cancel(); // mouse click
        assert_eq!(t.key_up(DEFAULT_ON_OFF_VK), None);
    }

    #[test]
    fn repeat_while_held_still_counts_as_one_tap() {
        let mut t = taps();
        t.key_down(DEFAULT_ON_OFF_VK);
        t.key_down(DEFAULT_ON_OFF_VK);
        assert_eq!(t.key_up(DEFAULT_ON_OFF_VK), Some(Toggle::OnOff));
        assert_eq!(t.key_up(DEFAULT_ON_OFF_VK), None);
    }

    #[test]
    fn other_keys_can_be_chosen() {
        let mut t = Taps::new(VK_LSHIFT, VK_SCROLL);
        t.key_down(VK_LSHIFT);
        assert_eq!(t.key_up(VK_LSHIFT), Some(Toggle::OnOff));
        t.key_down(VK_SCROLL);
        assert_eq!(t.key_up(VK_SCROLL), Some(Toggle::Output));
        // the old defaults do nothing now
        t.key_down(VK_RCONTROL);
        assert_eq!(t.key_up(VK_RCONTROL), None);
    }
}