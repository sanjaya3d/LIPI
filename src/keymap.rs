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
