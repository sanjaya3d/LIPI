//! The word being typed right now, for either Unicode or legacy output.
//!
//! The screen always shows `shown`. After each key, `Session` works out the
//! smallest edit (backspaces plus new text) that makes the screen match the
//! new result. Counting is done on what is really on screen, so it is right
//! for legacy text too.

use crate::engine::{diff, transliterate, Edit};
use crate::legacy::LegacyMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Unicode,
    Legacy,
}

pub struct Session {
    buf: String,   // Latin letters typed in this word
    shown: String, // what is on screen for this word
    mode: Output,
    map: LegacyMap,
}

impl Session {
    /// Longest word we keep. A longer run starts a fresh word.
    const MAX_LEN: usize = 64;

    pub fn new(map: LegacyMap) -> Self {
        Self { buf: String::new(), shown: String::new(), mode: Output::Unicode, map }
    }

    /// Changing the output type ends the current word.
    pub fn set_mode(&mut self, mode: Output) {
        if self.mode != mode {
            self.reset();
            self.mode = mode;
        }
    }

    /// Latin letters typed in this word.
    pub fn word(&self) -> &str {
        &self.buf
    }

    /// What is on screen for this word.
    pub fn shown(&self) -> &str {
        &self.shown
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn push(&mut self, c: char) -> Edit {
        if self.buf.len() >= Self::MAX_LEN {
            self.reset();
        }
        self.buf.push(c);
        self.refresh()
    }

    /// `None` means no word is in progress, so the real Backspace should go through.
    pub fn pop(&mut self) -> Option<Edit> {
        self.buf.pop()?;
        Some(self.refresh())
    }

    pub fn reset(&mut self) {
        self.buf.clear();
        self.shown.clear();
    }

    /// A digit or punctuation mark was typed. In legacy output the font has its own glyphs at
    /// the codes of some marks (the key `?` would show a Sinhala letter), so the mark must be
    /// typed as its legacy code instead. Returns the text to type, or `None` when the key
    /// already gives the right glyph (or the output is Unicode) and should pass through.
    pub fn mark(&self, ch: char) -> Option<String> {
        if self.mode != Output::Legacy {
            return None;
        }
        let key = ch.to_string();
        let text = self.map.convert(&key).text;
        if text == key {
            None
        } else {
            Some(text)
        }
    }

    fn refresh(&mut self) -> Edit {
        let unicode = transliterate(&self.buf);
        let new = match self.mode {
            Output::Unicode => unicode,
            Output::Legacy => self.map.convert(&unicode).text,
        };
        let edit = diff(&self.shown, &new);
        self.shown = new;
        edit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(screen: &mut String, edit: &Edit) {
        for _ in 0..edit.backspaces {
            screen.pop();
        }
        screen.push_str(&edit.text);
    }

    fn type_word(session: &mut Session, screen: &mut String, word: &str) {
        for c in word.chars() {
            apply(screen, &session.push(c));
        }
    }

    #[test]
    fn unicode_output() {
        let mut s = Session::new(LegacyMap::builtin());
        let mut screen = String::new();
        type_word(&mut s, &mut screen, "ballaa");
        assert_eq!(screen, "බල්ලා");
    }

    #[test]
    fn legacy_output() {
        let mut s = Session::new(LegacyMap::builtin());
        s.set_mode(Output::Legacy);
        let mut screen = String::new();
        type_word(&mut s, &mut screen, "ballaa");
        assert_eq!(screen, "n,a,d");
    }

    #[test]
    fn legacy_kombuva_jumps_in_front_while_typing() {
        let mut s = Session::new(LegacyMap::builtin());
        s.set_mode(Output::Legacy);
        let mut screen = String::new();
        type_word(&mut s, &mut screen, "k");
        assert_eq!(screen, "la");
        type_word(&mut s, &mut screen, "e");
        assert_eq!(screen, "fl");
        type_word(&mut s, &mut screen, "e");
        assert_eq!(screen, "fla");
    }

    #[test]
    fn backspace_removes_one_typed_letter() {
        for mode in [Output::Unicode, Output::Legacy] {
            let mut s = Session::new(LegacyMap::builtin());
            s.set_mode(mode);
            let mut screen = String::new();
            type_word(&mut s, &mut screen, "ballaa");
            apply(&mut screen, &s.pop().unwrap());
            let mut expected = String::new();
            let mut fresh = Session::new(LegacyMap::builtin());
            fresh.set_mode(mode);
            type_word(&mut fresh, &mut expected, "balla");
            assert_eq!(screen, expected);
            assert_eq!(s.word(), "balla");
        }
    }

    #[test]
    fn backspace_with_no_word_goes_to_the_app() {
        let mut s = Session::new(LegacyMap::builtin());
        assert!(s.pop().is_none());
    }

    #[test]
    fn changing_output_starts_a_new_word() {
        let mut s = Session::new(LegacyMap::builtin());
        let mut screen = String::new();
        type_word(&mut s, &mut screen, "ka");
        s.set_mode(Output::Legacy);
        assert!(s.is_empty());
        type_word(&mut s, &mut screen, "ka");
        assert_eq!(screen, "කl"); // the first word stays, the second is legacy
    }

    // ---------- marks (? . : % ^ & and the rest) ----------

    #[test]
    fn unicode_output_never_changes_a_mark() {
        let s = Session::new(LegacyMap::builtin());
        for ch in "?.:%^&,!()".chars() {
            assert_eq!(s.mark(ch), None, "{ch}");
        }
    }

    #[test]
    fn legacy_output_types_the_code_from_your_table() {
        let mut s = Session::new(LegacyMap::builtin());
        s.set_mode(Output::Legacy);
        let table = [
            ('?', "@"), ('.', "'"), (':', "("), ('%', "]"), (',', "\""), ('!', "æ"), ('(', "^"), (')', "&"),
            ('\'', "z"), ('"', "—"), ('/', "$"), ('=', "}"), ('+', "¬"), (';', "¦"), ('~', "~&"),
        ];
        for (key, code) in table {
            assert_eq!(s.mark(key).as_deref(), Some(code), "{key}");
        }
    }

    #[test]
    fn a_mark_that_is_already_right_passes_through() {
        // ^ and & keep their own codes in the table. Digits and letters have no entry at all.
        let mut s = Session::new(LegacyMap::builtin());
        s.set_mode(Output::Legacy);
        for ch in "^&#$*-<>@[\\]_`{|}0123456789".chars() {
            assert_eq!(s.mark(ch), None, "{ch}");
        }
    }

    #[test]
    fn a_mark_after_a_word_leaves_the_word_alone() {
        // "mama" then "?" : the word is finished, the mark is typed on its own.
        let mut s = Session::new(LegacyMap::builtin());
        s.set_mode(Output::Legacy);
        let mut screen = String::new();
        type_word(&mut s, &mut screen, "mama");
        let word_on_screen = screen.clone();
        s.reset(); // what the hook does for every Break key
        screen.push_str(&s.mark('?').unwrap());
        assert_eq!(screen, format!("{word_on_screen}@"));
        assert!(s.is_empty());
    }

    #[test]
    fn a_mark_follows_the_output_mode_when_it_changes() {
        let mut s = Session::new(LegacyMap::builtin());
        s.set_mode(Output::Legacy);
        assert_eq!(s.mark('?').as_deref(), Some("@"));
        s.set_mode(Output::Unicode);
        assert_eq!(s.mark('?'), None);
    }

    #[test]
    fn every_key_that_types_a_mark_is_either_converted_or_passed_through_cleanly() {
        // Run every character a US keyboard can type through the hook's rule.
        let mut s = Session::new(LegacyMap::builtin());
        s.set_mode(Output::Legacy);
        let mut converted = 0;
        for vk in 0..=0xFFu32 {
            for shift in [false, true] {
                if let Some(ch) = crate::keymap::punctuation(vk, shift, false) {
                    if let Some(text) = s.mark(ch) {
                        converted += 1;
                        assert!(!text.is_empty(), "{ch}");
                        assert_ne!(text, ch.to_string(), "{ch}");
                    }
                }
            }
        }
        assert!(converted >= 15, "only {converted} keys were converted");
    }

    #[test]
    fn very_long_run_starts_a_new_word() {
        let mut s = Session::new(LegacyMap::builtin());
        for _ in 0..200 {
            s.push('a');
        }
        assert!(s.word().len() <= 64);
    }
}
