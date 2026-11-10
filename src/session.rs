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

    #[test]
    fn very_long_run_starts_a_new_word() {
        let mut s = Session::new(LegacyMap::builtin());
        for _ in 0..200 {
            s.push('a');
        }
        assert!(s.word().len() <= 64);
    }
}
