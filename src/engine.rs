//! Singlish to Sinhala Unicode engine.
//!
//! `transliterate` converts a whole buffer of Latin keystrokes into Sinhala.
//! `Composer` wraps it for real-time typing. It remembers what is on screen
//! and, after every key press, returns the smallest edit needed
//! (some backspaces, then some new text).

const HAL: char = '\u{0DCA}'; // al-lakuna, removes the inherent vowel
const ZWJ: char = '\u{200D}'; // joins yansaya and rakaransaya
const YA: char = 'ය';
const RA: char = 'ර';

/// Standalone vowels. Used at the start of a word or after another vowel.
static VOWELS: &[(&str, char)] = &[
    ("a", 'අ'),
    ("aa", 'ආ'),
    ("A", 'ඇ'),
    ("Aa", 'ඈ'),
    ("AA", 'ඈ'),
    ("i", 'ඉ'),
    ("ii", 'ඊ'),
    ("u", 'උ'),
    ("uu", 'ඌ'),
    ("R", 'ඍ'),
    ("Ru", 'ඎ'),
    ("e", 'එ'),
    ("ee", 'ඒ'),
    ("ai", 'ඓ'),
    ("o", 'ඔ'),
    ("oo", 'ඕ'),
    ("au", 'ඖ'),
    ("ou", 'ඖ'),
];

/// Vowel signs. Used straight after a consonant. `None` means the
/// consonant keeps its inherent "a" and no sign is written.
/// Only the keys of the reference sheet are here. "R", "Ru" and "ou" are vowel keys
/// only (ඍ, ඎ, ඖ), so "kR", "kRu" and "kou" are not vowel signs.
static SIGNS: &[(&str, Option<char>)] = &[
    ("a", None),
    ("aa", Some('\u{0DCF}')),  // ා
    ("A", Some('\u{0DD0}')),   // ැ
    ("Aa", Some('\u{0DD1}')),  // ෑ
    ("AA", Some('\u{0DD1}')),  // ෑ
    ("i", Some('\u{0DD2}')),   // ි
    ("ii", Some('\u{0DD3}')),  // ී
    ("u", Some('\u{0DD4}')),   // ු
    ("uu", Some('\u{0DD6}')),  // ූ
    ("ru", Some('\u{0DD8}')),  // ෘ  (kru)
    ("ruu", Some('\u{0DF2}')), // ෲ  (kruu)
    ("e", Some('\u{0DD9}')),   // ෙ
    ("ee", Some('\u{0DDA}')),  // ේ
    ("ai", Some('\u{0DDB}')),  // ෛ
    ("o", Some('\u{0DDC}')),   // ො
    ("oo", Some('\u{0DDD}')),  // ෝ
    ("au", Some('\u{0DDE}')),  // ෞ
];

/// Consonants, written without their inherent vowel ("k", not "ka").
static CONSONANTS: &[(&str, char)] = &[
    ("k", 'ක'),
    ("kh", 'ඛ'),
    ("g", 'ග'),
    ("gh", 'ඝ'),
    ("zg", 'ඟ'),
    ("ch", 'ච'),
    ("chh", 'ඡ'),
    ("j", 'ජ'),
    ("zj", 'ඦ'),
    ("zk", 'ඤ'),
    ("zh", 'ඥ'),
    ("t", 'ට'),
    ("T", 'ඨ'),
    ("d", 'ඩ'),
    ("D", 'ඪ'),
    ("N", 'ණ'),
    ("zd", 'ඬ'),
    ("th", 'ත'),
    ("thh", 'ථ'),
    ("dh", 'ද'),
    ("q", 'ද'),
    ("dhh", 'ධ'),
    ("n", 'න'),
    ("zdh", 'ඳ'),
    ("zq", 'ඳ'),
    ("p", 'ප'),
    ("ph", 'ඵ'),
    ("b", 'බ'),
    ("bh", 'භ'),
    ("m", 'ම'),
    ("B", 'ඹ'),
    ("y", 'ය'),
    ("r", 'ර'),
    ("l", 'ල'),
    ("L", 'ළ'),
    ("w", 'ව'),
    ("v", 'ව'),
    ("sh", 'ශ'),
    ("Sh", 'ෂ'),
    ("S", 'ෂ'),
    ("s", 'ස'),
    ("h", 'හ'),
    ("f", 'ෆ'),
];

/// Signs and letters that sit after whatever came before and never take
/// a hal or a vowel of their own ("kax" gives කං, "kaX" gives කඞ).
/// The reference sheet has no key for them on their own, so they only work
/// after a Sinhala letter or sign. At the start of a word they stay as typed.
static MARKS: &[(&str, char)] = &[
    ("x", '\u{0D82}'),  // ං anusvara
    ("zn", '\u{0D82}'), // ං anusvara
    ("H", '\u{0D83}'),  // ඃ visarga
    ("X", '\u{0D9E}'),  // ඞ
];

#[derive(Clone, Copy)]
enum Token {
    Vowel(char),
    Sign(Option<char>),
    Consonant(char),
    Mark(char),
}

/// Longest key in `table` that `s` starts with. Returns (key length, value).
/// All keys are ASCII, so key length in bytes equals length in characters.
fn longest<T: Copy>(table: &[(&str, T)], s: &str) -> Option<(usize, T)> {
    table
        .iter()
        .filter(|entry| s.starts_with(entry.0))
        .max_by_key(|entry| entry.0.len())
        .map(|entry| (entry.0.len(), entry.1))
}

/// Reads the next token from the start of `rest`.
/// After a consonant we look for vowel signs, otherwise for standalone vowels.
/// The longest match wins, so "kruu" is one sign and "chha" is one letter.
/// Marks (x, zn, H, X) are looked for only when `marks_allowed` is true.
fn next_token(rest: &str, after_consonant: bool, marks_allowed: bool) -> Option<(usize, Token)> {
    let vowel_like = if after_consonant {
        longest(SIGNS, rest).map(|(n, s)| (n, Token::Sign(s)))
    } else {
        longest(VOWELS, rest).map(|(n, v)| (n, Token::Vowel(v)))
    };
    [
        vowel_like,
        longest(CONSONANTS, rest).map(|(n, c)| (n, Token::Consonant(c))),
        longest(MARKS, rest).filter(|_| marks_allowed).map(|(n, m)| (n, Token::Mark(m))),
    ]
    .into_iter()
    .flatten()
    .max_by_key(|(n, _)| *n)
}

/// True if some key of the scheme contains the capital letter `c`.
/// Those capitals (A, R, T, D, N, L, S, B, H, X) have a meaning of their own and are
/// never read as lowercase.
fn capital_is_used(c: char) -> bool {
    VOWELS
        .iter()
        .map(|e| e.0)
        .chain(SIGNS.iter().map(|e| e.0))
        .chain(CONSONANTS.iter().map(|e| e.0))
        .chain(MARKS.iter().map(|e| e.0))
        .any(|key| key.contains(c))
}

/// Converts Singlish text to Sinhala Unicode.
/// Anything that is not part of the scheme (space, digits, punctuation)
/// is passed through unchanged.
pub fn transliterate(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    // A consonant that has been written but is still waiting to learn
    // whether it gets a vowel sign or a hal.
    let mut pending: Option<char> = None;
    // True while the last thing written was Sinhala. Marks (x, H, X) need that.
    let mut after_sinhala = false;
    let mut rest = input;

    while let Some(first) = rest.chars().next() {
        let mut hit = next_token(rest, pending.is_some(), after_sinhala);

        // A capital letter that no key uses ("Kamal") is read as lowercase.
        // Capitals that have a meaning (A, R, T, D, N, L, S, B, H, X) are never changed.
        if hit.is_none() && first.is_ascii_uppercase() && !capital_is_used(first) {
            let lowered = format!("{}{}", first.to_ascii_lowercase(), &rest[1..]);
            hit = next_token(&lowered, pending.is_some(), after_sinhala);
        }
        after_sinhala = hit.is_some();

        match hit {
            Some((len, token)) => {
                match token {
                    Token::Vowel(v) => {
                        out.push(v);
                        pending = None;
                    }
                    Token::Sign(sign) => {
                        if let Some(s) = sign {
                            out.push(s);
                        }
                        pending = None;
                    }
                    Token::Mark(m) => {
                        // "kax" gives ක + ං. No hal is needed before a mark.
                        out.push(m);
                        pending = None;
                    }
                    Token::Consonant(c) => {
                        if let Some(prev) = pending {
                            // Two consonants in a row: the first loses its vowel.
                            out.push(HAL);
                            // y and r after a consonant join as yansaya / rakaransaya.
                            if (c == YA || c == RA) && prev != YA && prev != RA {
                                out.push(ZWJ);
                            }
                        }
                        out.push(c);
                        pending = Some(c);
                    }
                }
                rest = &rest[len..];
            }
            None => {
                if pending.take().is_some() {
                    out.push(HAL);
                }
                out.push(first);
                rest = &rest[first.len_utf8()..];
            }
        }
    }

    if pending.is_some() {
        out.push(HAL); // "k" at the end gives ක්
    }
    out
}

/// Keys that can be typed, grouped by what they give, in table order.
/// The first key of a group is the main one. The others are different ways to type the same thing.
pub type KeyGroups = Vec<(char, Vec<&'static str>)>;

fn grouped(table: impl Iterator<Item = (&'static str, char)>) -> KeyGroups {
    let mut out: KeyGroups = Vec::new();
    for (key, value) in table {
        match out.iter_mut().find(|(v, _)| *v == value) {
            Some((_, keys)) => keys.push(key),
            None => out.push((value, vec![key])),
        }
    }
    out
}

/// Standalone vowels and the keys for them (used by the key hint page).
pub fn vowel_keys() -> KeyGroups {
    grouped(VOWELS.iter().copied())
}

/// Consonants, written without their vowel ("k" gives ක්, "ka" gives ක).
pub fn consonant_keys() -> KeyGroups {
    grouped(CONSONANTS.iter().copied())
}

/// Vowel signs that follow a consonant. The plain "a" has no sign, so it is not listed.
pub fn sign_keys() -> KeyGroups {
    grouped(SIGNS.iter().filter_map(|&(key, sign)| sign.map(|s| (key, s))))
}

/// Anusvara, visarga and the letter ඞ.
pub fn mark_keys() -> KeyGroups {
    grouped(MARKS.iter().copied())
}

/// What to do on screen: delete `backspaces` characters, then type `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub backspaces: usize,
    pub text: String,
}

/// Smallest edit that turns `old` into `new`. Counts are in Unicode code points.
pub fn diff(old: &str, new: &str) -> Edit {
    let common = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .count();
    Edit {
        backspaces: old.chars().count() - common,
        text: new.chars().skip(common).collect(),
    }
}

/// A word being typed. Feed it keys, apply the edits it returns.
#[derive(Default)]
pub struct Composer {
    buf: String,   // Latin letters typed so far in this word
    shown: String, // Sinhala currently on screen for this word
}

impl Composer {
    /// Longest word we keep. A longer run starts a fresh word.
    const MAX_LEN: usize = 64;

    pub fn new() -> Self {
        Self::default()
    }

    /// True when no word is in progress.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// A letter was typed.
    pub fn push(&mut self, c: char) -> Edit {
        if self.buf.len() >= Self::MAX_LEN {
            self.reset();
        }
        self.buf.push(c);
        self.refresh()
    }

    /// Backspace was pressed. `None` means no word is in progress,
    /// so the real Backspace key should be passed to the app.
    pub fn pop(&mut self) -> Option<Edit> {
        self.buf.pop()?;
        Some(self.refresh())
    }

    /// The word ended (space, punctuation, Enter, mouse click, arrow keys...).
    pub fn reset(&mut self) {
        self.buf.clear();
        self.shown.clear();
    }

    fn refresh(&mut self) -> Edit {
        let new = transliterate(&self.buf);
        let edit = diff(&self.shown, &new);
        self.shown = new;
        edit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            assert_eq!(transliterate(input), *expected, "input: {input}");
        }
    }

    #[test]
    fn standalone_vowels() {
        check(&[
            ("a", "අ"), ("aa", "ආ"), ("A", "ඇ"), ("Aa", "ඈ"), ("AA", "ඈ"),
            ("i", "ඉ"), ("ii", "ඊ"), ("u", "උ"), ("uu", "ඌ"),
            ("R", "ඍ"), ("Ru", "ඎ"),
            ("e", "එ"), ("ee", "ඒ"), ("ai", "ඓ"),
            ("o", "ඔ"), ("oo", "ඕ"), ("au", "ඖ"), ("ou", "ඖ"),
        ]);
    }

    #[test]
    fn consonants() {
        check(&[
            ("ka", "ක"), ("ga", "ග"), ("cha", "ච"), ("ja", "ජ"), ("ta", "ට"),
            ("da", "ඩ"), ("tha", "ත"), ("dha", "ද"), ("qa", "ද"), ("na", "න"),
            ("Na", "ණ"), ("pa", "ප"), ("ba", "බ"), ("ma", "ම"), ("ya", "ය"),
            ("ra", "ර"), ("la", "ල"), ("La", "ළ"), ("wa", "ව"), ("va", "ව"),
            ("sa", "ස"), ("sha", "ශ"), ("Sa", "ෂ"), ("Sha", "ෂ"), ("ha", "හ"),
            ("fa", "ෆ"),
        ]);
    }

    #[test]
    fn aspirated_and_special_letters() {
        check(&[
            ("kha", "ඛ"), ("gha", "ඝ"), ("chha", "ඡ"), ("Ta", "ඨ"), ("Da", "ඪ"),
            ("thha", "ථ"), ("dhha", "ධ"), ("pha", "ඵ"), ("bha", "භ"),
            ("zga", "ඟ"), ("zja", "ඦ"), ("zda", "ඬ"), ("zdha", "ඳ"), ("zqa", "ඳ"),
            ("zka", "ඤ"), ("zha", "ඥ"), ("Ba", "ඹ"), ("Lu", "ළු"),
        ]);
    }

    #[test]
    fn vowel_signs_on_ka() {
        check(&[
            ("k", "ක්"), ("ka", "ක"), ("kaa", "කා"), ("kA", "කැ"),
            ("kAa", "කෑ"), ("kAA", "කෑ"), ("ki", "කි"), ("kii", "කී"),
            ("ku", "කු"), ("kuu", "කූ"), ("kru", "කෘ"), ("kruu", "කෲ"),
            ("ke", "කෙ"), ("kee", "කේ"), ("kai", "කෛ"), ("ko", "කො"),
            ("koo", "කෝ"), ("kau", "කෞ"),
        ]);
    }

    #[test]
    fn marks_and_conjuncts() {
        check(&[
            ("kaH", "කඃ"),
            ("kax", "කං"),
            ("kazn", "කං"),
            ("kaX", "කඞ"),
            ("kya", "ක\u{0DCA}\u{200D}ය"),
            ("kra", "ක\u{0DCA}\u{200D}ර"),
        ]);
    }

    #[test]
    fn real_words() {
        check(&[
            ("ballaa", "බල්ලා"),
            ("ammaa", "අම්මා"),
            ("mama", "මම"),
            ("karu", "කරු"),
            ("sixhala", "සිංහල"),
            ("koLaBa", "කොළඹ"),
            ("raajya", "රාජ\u{0DCA}\u{200D}ය"),
            ("shrii laxkaa", "ශ\u{0DCA}\u{200D}රී ලංකා"),
            ("Kamal", "කමල්"),
            ("rr", "ර්ර්"),
        ]);
    }

    #[test]
    fn unknown_characters_pass_through() {
        check(&[("k.", "ක්."), ("12 ka", "12 ක"), ("ka-ma", "ක-ම")]);
    }

    /// Applies an edit to a pretend screen.
    fn apply(screen: &mut String, edit: &Edit) {
        for _ in 0..edit.backspaces {
            screen.pop();
        }
        screen.push_str(&edit.text);
    }

    #[test]
    fn composer_screen_always_matches_engine() {
        let word = "shrii laxkaaballaakoLaBa";
        let mut composer = Composer::new();
        let mut screen = String::new();
        for (i, c) in word.chars().enumerate() {
            if c == ' ' {
                composer.reset();
                screen.push(' ');
                continue;
            }
            apply(&mut screen, &composer.push(c));
            // The last word on screen must equal the engine output for the typed prefix.
            let start = word[..=i].rfind(' ').map_or(0, |p| p + 1);
            let expected_tail = transliterate(&word[start..=i]);
            assert!(screen.ends_with(&expected_tail), "after typing {:?}", &word[..=i]);
        }
    }

    #[test]
    fn composer_edits_are_small() {
        let mut c = Composer::new();
        assert_eq!(c.push('b'), Edit { backspaces: 0, text: "බ්".into() });
        assert_eq!(c.push('a'), Edit { backspaces: 1, text: "".into() });
        assert_eq!(c.push('l'), Edit { backspaces: 0, text: "ල්".into() });
        assert_eq!(c.push('l'), Edit { backspaces: 0, text: "ල්".into() });
        assert_eq!(c.push('a'), Edit { backspaces: 1, text: "".into() });
        assert_eq!(c.push('a'), Edit { backspaces: 0, text: "ා".into() });
    }

    #[test]
    fn key_groups_keep_table_order_and_list_aliases_after_the_main_key() {
        let vowels = vowel_keys();
        assert_eq!(vowels.len(), 16); // 18 keys, two of them are aliases
        assert_eq!(vowels[0], ('අ', vec!["a"]));
        assert!(vowels.contains(&('ඈ', vec!["Aa", "AA"])));
        assert!(vowels.contains(&('ඖ', vec!["au", "ou"])));

        let consonants = consonant_keys();
        assert_eq!(consonants.len(), 39);
        assert!(consonants.contains(&('ද', vec!["dh", "q"])));
        assert!(consonants.contains(&('ෂ', vec!["Sh", "S"])));

        let signs = sign_keys();
        assert_eq!(signs.len(), 15); // the plain "a" has no sign
        assert!(signs.contains(&('\u{0DD8}', vec!["ru"])));
        assert!(signs.contains(&('\u{0DF2}', vec!["ruu"])));
        assert!(signs.contains(&('\u{0DD1}', vec!["Aa", "AA"])));
        assert!(signs.iter().all(|(_, keys)| !keys.contains(&"a")));
        // R, Ru and ou are vowel keys only. They are never vowel signs.
        assert!(signs.iter().all(|(_, keys)| keys.iter().all(|k| !["R", "Ru", "ou"].contains(k))));

        let marks = mark_keys();
        assert!(marks.contains(&('\u{0D82}', vec!["x", "zn"])));
        assert!(marks.contains(&('ඞ', vec!["X"])));
    }

    #[test]
    fn every_listed_key_really_types_what_the_groups_say() {
        for (glyph, keys) in vowel_keys() {
            for key in keys {
                assert_eq!(transliterate(key), glyph.to_string(), "vowel key {key}");
            }
        }
        for (glyph, keys) in consonant_keys() {
            for key in keys {
                assert_eq!(transliterate(&format!("{key}a")), glyph.to_string(), "consonant key {key}");
                assert_eq!(transliterate(key), format!("{glyph}\u{0DCA}"), "consonant key {key} alone");
            }
        }
        for (sign, keys) in sign_keys() {
            for key in keys {
                assert_eq!(transliterate(&format!("k{key}")), format!("ක{sign}"), "sign key {key}");
            }
        }
    }

    #[test]
    fn r_ru_and_ou_are_vowels_only_not_vowel_signs() {
        check(&[("R", "ඍ"), ("Ru", "ඎ"), ("ou", "ඖ"), ("au", "ඖ")]);
        // The sheet gives ru, ruu and au after a consonant. It does not give kR, kRu or kou.
        check(&[("kru", "කෘ"), ("kruu", "කෲ"), ("kau", "කෞ")]);
        check(&[("kR", "ක්R"), ("kRu", "ක්Rඋ"), ("kou", "කොඋ")]);
    }

    #[test]
    fn marks_need_a_sinhala_letter_or_sign_before_them() {
        // The sheet shows x, zn, H and X only after ක. On their own they stay as typed.
        check(&[("x", "x"), ("H", "H"), ("X", "X"), ("zn", "zන්"), ("x1", "x1"), ("k x", "ක් x")]);
        check(&[("kax", "කං"), ("kazn", "කං"), ("kaH", "කඃ"), ("kaX", "කඞ")]);
        // After other letters and signs they still work.
        check(&[("aaH", "ආඃ"), ("kiH", "කිඃ"), ("naax", "නාං"), ("sixhala", "සිංහල")]);
    }

    #[test]
    fn capitals_that_have_a_meaning_are_never_read_as_lowercase() {
        for c in "ARTDNLSBHX".chars() {
            assert!(capital_is_used(c), "{c} should count as used");
            let lower = c.to_ascii_lowercase().to_string();
            assert_ne!(transliterate(&c.to_string()), transliterate(&lower), "capital {c}");
        }
        check(&[("A", "ඇ"), ("T", "ඨ්"), ("D", "ඪ්"), ("N", "ණ්"), ("L", "ළ්"), ("S", "ෂ්"), ("B", "ඹ්")]);
        // A capital that no key uses is still read as lowercase, so a name typed with a capital works.
        assert!(!capital_is_used('K'));
        check(&[("Kamal", "කමල්"), ("Pa", "ප")]);
    }

    #[test]
    fn composer_backspace() {
        let mut c = Composer::new();
        assert_eq!(c.pop(), None); // nothing typed, real Backspace passes through
        let mut screen = String::new();
        for ch in "ballaa".chars() {
            apply(&mut screen, &c.push(ch));
        }
        assert_eq!(screen, "බල්ලා");
        apply(&mut screen, &c.pop().unwrap());
        assert_eq!(screen, "බල්ල");
        for _ in 0..5 {
            apply(&mut screen, &c.pop().unwrap());
        }
        assert_eq!(screen, "");
        assert!(c.is_empty());
        assert_eq!(c.pop(), None);
    }
}