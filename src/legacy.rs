//! Sinhala Unicode to legacy font text (FM Abhaya).
//!
//! A legacy font stores one glyph per character and does no shaping, so this
//! converter does what a shaping engine would do. It reads the text one
//! syllable at a time. A syllable is
//!
//!     letter   [conjunct ...]   [vowel sign]   [anusvara / visarga]
//!
//! and it is written in this order of priority:
//!
//!   1. A ligature that starts the cluster (ක්ෂ, dra, repaya + ය, repaya + ණ) is one glyph.
//!   2. If the WHOLE syllable has its own entry (letter + sign, or letter +
//!      conjunct + sign), that code is used. These are the special FM Abhaya
//!      glyphs, for example ඩ් = â, ඩි = ä, ච් = É, මි = ñ, වි = ú.
//!   3. Otherwise the syllable is built from its pieces. Kombuva signs go in
//!      front of the letter, the rest go after it. ේ uses the hal glyph of the
//!      letter when there is one (බේ = f + í).
//!
//! Nothing depends on the order of lines in `legacy_map.txt`. The longest
//! match wins, so adding a line can never be hidden by another line.
//!
//! Split vowel signs (ෙ + ා for ො and so on) are joined first, so text from
//! any source gives the same result.

use std::collections::HashMap;

const HAL: char = '\u{0DCA}';
const ZWJ: char = '\u{200D}';
const ZWNJ: char = '\u{200C}';
const AELA: char = '\u{0DCF}';
const KOMBUVA: char = '\u{0DD9}';
const KOMBUVA_HAL: char = '\u{0DDA}'; // ේ
const KOMBUVA_KOMBUVA: char = '\u{0DDB}'; // ෛ
const KOMBUVA_AELA: char = '\u{0DDC}'; // ො
const KOMBUVA_AELA_HAL: char = '\u{0DDD}'; // ෝ
const KOMBUVA_GAYA: char = '\u{0DDE}'; // ෞ
const GAYANUKITTA: char = '\u{0DDF}';
const ANUSVARA: char = '\u{0D82}';
const VISARGA: char = '\u{0D83}';

pub struct LegacyMap {
    entries: HashMap<Vec<char>, String>,
    /// Length of the longest ligature entry that starts a cluster.
    longest_cluster: usize,
}

/// Result of a conversion.
pub struct Converted {
    pub text: String,
    /// Sinhala characters that have no legacy code in the map.
    /// They are left as Unicode in `text`.
    pub missing: Vec<char>,
}

/// A letter that can start a syllable: independent vowels and consonants.
fn is_base(c: char) -> bool {
    ('\u{0D85}'..='\u{0D96}').contains(&c) || is_consonant(c)
}

fn is_consonant(c: char) -> bool {
    ('\u{0D9A}'..='\u{0DC6}').contains(&c)
}

fn is_sign(c: char) -> bool {
    matches!(c, '\u{0D82}' | '\u{0D83}' | '\u{0DCA}' | '\u{0DCF}'..='\u{0DDF}' | '\u{0DF2}' | '\u{0DF3}')
}

fn is_sinhala_block(c: char) -> bool {
    ('\u{0D80}'..='\u{0DFF}').contains(&c)
}

/// A ligature key starts and ends with a letter (ක + ් + ෂ). A key that ends in a
/// sign (බ + ් + ZWJ + ර + ි) is a whole-syllable override instead.
fn is_ligature_key(seq: &[char]) -> bool {
    seq.len() >= 3 && is_base(seq[0]) && is_base(seq[seq.len() - 1])
}

/// Joins the split forms of the kombuva vowels, so that ෙ + ා gives ො, and so on.
/// These pairs are canonically the same text, but some programs write them split.
fn join_vowel_signs(text: &str) -> Vec<char> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == KOMBUVA {
            let next = chars.get(i + 1).copied();
            let after = chars.get(i + 2).copied();
            let (joined, used) = match (next, after) {
                (Some(AELA), Some(HAL)) => (Some(KOMBUVA_AELA_HAL), 3),
                (Some(AELA), _) => (Some(KOMBUVA_AELA), 2),
                (Some(HAL), _) => (Some(KOMBUVA_HAL), 2),
                (Some(GAYANUKITTA), _) => (Some(KOMBUVA_GAYA), 2),
                (Some(KOMBUVA), _) => (Some(KOMBUVA_KOMBUVA), 2),
                _ => (None, 1),
            };
            if let Some(j) = joined {
                out.push(j);
                i += used;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

impl LegacyMap {
    /// The map that ships inside the program.
    pub fn builtin() -> Self {
        Self::parse(include_str!("legacy_map.txt")).expect("built-in legacy_map.txt is valid")
    }

    /// Loads a map from a text file (same format as legacy_map.txt).
    pub fn from_file(path: &str) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
        Self::parse(&text)
    }

    pub fn parse(src: &str) -> Result<Self, String> {
        let src = src.trim_start_matches('\u{feff}');
        let mut entries: HashMap<Vec<char>, String> = HashMap::new();
        let mut first_line: HashMap<Vec<char>, usize> = HashMap::new();
        for (n, raw) in src.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, rest) = line
                .split_once(char::is_whitespace)
                .ok_or_else(|| format!("line {}: expected '<hex> <code>'", n + 1))?;
            let code = rest.split_once(" //").map_or(rest, |(code, _)| code).trim();
            if code.is_empty() {
                return Err(format!("line {}: empty legacy code", n + 1));
            }
            let mut seq = Vec::new();
            for part in key.split('+') {
                let c = u32::from_str_radix(part, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or_else(|| format!("line {}: bad code point '{}'", n + 1, part))?;
                seq.push(c);
            }
            if let Some(first) = first_line.get(&seq) {
                return Err(format!("line {}: {key} is already defined on line {first}", n + 1));
            }
            first_line.insert(seq.clone(), n + 1);
            entries.insert(seq, code.to_string());
        }
        let longest_cluster =
            entries.keys().filter(|k| is_ligature_key(k)).map(Vec::len).max().unwrap_or(0);
        Ok(Self { entries, longest_cluster })
    }

    fn code(&self, seq: &[char]) -> Option<&str> {
        self.entries.get(seq).map(String::as_str)
    }

    /// Pairs of entries that give the same legacy code. One legacy code must
    /// mean one letter. The built-in map has none.
    pub fn conflicts(&self) -> Vec<(String, Vec<String>)> {
        let mut by_code: HashMap<&str, Vec<String>> = HashMap::new();
        for (key, code) in &self.entries {
            let shown: String = key.iter().map(|c| format!("{:04X}", *c as u32)).collect::<Vec<_>>().join("+");
            by_code.entry(code.as_str()).or_default().push(shown);
        }
        let mut found: Vec<(String, Vec<String>)> = by_code
            .into_iter()
            .filter(|(_, keys)| keys.len() > 1)
            .map(|(code, mut keys)| {
                keys.sort();
                (code.to_string(), keys)
            })
            .collect();
        found.sort();
        found
    }

    /// Appends the code for `ch` to `dest`. Without a code, the character is
    /// kept as it is and remembered in `missing`.
    fn push_code(&self, ch: char, dest: &mut String, missing: &mut Vec<char>) {
        match self.code(&[ch]) {
            Some(code) => dest.push_str(code),
            None => {
                dest.push(ch);
                if is_sinhala_block(ch) && !missing.contains(&ch) {
                    missing.push(ch);
                }
            }
        }
    }

    fn letter_code(&self, ch: char, missing: &mut Vec<char>) -> String {
        let mut s = String::new();
        self.push_code(ch, &mut s, missing);
        s
    }

    /// The letter with a hal kirima: its own glyph if the font has one (ඩ් = â),
    /// else the letter followed by the hal code.
    fn hal_form(&self, base_chars: &[char], base: &str, missing: &mut Vec<char>) -> String {
        if let [letter] = base_chars {
            if let Some(code) = self.code(&[*letter, HAL]) {
                return code.to_string();
            }
        }
        let mut s = base.to_string();
        self.push_code(HAL, &mut s, missing);
        s
    }

    /// Writes one vowel sign (or stray sign) for a letter whose code is `base`
    /// and whose conjunct codes are `conj`.
    fn compose(
        &self,
        base_chars: &[char],
        base: &str,
        conj: &str,
        sign: char,
        out: &mut String,
        missing: &mut Vec<char>,
    ) {
        let letter = |out: &mut String| {
            out.push_str(base);
            out.push_str(conj);
        };
        match sign {
            KOMBUVA => {
                self.push_code(KOMBUVA, out, missing);
                letter(out);
            }
            KOMBUVA_HAL => {
                self.push_code(KOMBUVA, out, missing);
                out.push_str(&self.hal_form(base_chars, base, missing));
                out.push_str(conj);
            }
            KOMBUVA_KOMBUVA => {
                self.push_code(KOMBUVA, out, missing);
                self.push_code(KOMBUVA, out, missing);
                letter(out);
            }
            KOMBUVA_AELA => {
                self.push_code(KOMBUVA, out, missing);
                letter(out);
                self.push_code(AELA, out, missing);
            }
            KOMBUVA_AELA_HAL => {
                self.push_code(KOMBUVA, out, missing);
                letter(out);
                self.push_code(AELA, out, missing);
                self.push_code(HAL, out, missing);
            }
            KOMBUVA_GAYA => {
                self.push_code(KOMBUVA, out, missing);
                letter(out);
                self.push_code(GAYANUKITTA, out, missing);
            }
            _ => {
                letter(out);
                self.push_code(sign, out, missing);
            }
        }
    }

    /// Converts one syllable that starts at `chars[start]`. Returns the index after it.
    fn syllable(&self, chars: &[char], start: usize, out: &mut String, missing: &mut Vec<char>) -> usize {
        let mut i = start;

        // 1. A ligature at the start of the cluster, longest first.
        let mut base_chars: Vec<char> = vec![chars[start]];
        let mut base_code: Option<String> = None;
        if is_consonant(chars[start]) {
            for len in (3..=self.longest_cluster).rev() {
                let Some(slice) = chars.get(start..start + len) else { continue };
                if !is_ligature_key(slice) {
                    continue;
                }
                if let Some(code) = self.code(slice) {
                    base_code = Some(code.to_string());
                    base_chars = slice.to_vec();
                    i = start + len;
                    break;
                }
            }
        }
        let base = match base_code {
            Some(code) => code,
            None => {
                i = start + 1;
                self.letter_code(chars[start], missing)
            }
        };

        // 2. Conjuncts: hal + joiner + letter (rakaransaya, yansaya).
        let mut cluster = base_chars.clone();
        let mut conj = String::new();
        while chars.get(i) == Some(&HAL) && chars.get(i + 1) == Some(&ZWJ) {
            let Some(&letter) = chars.get(i + 2) else { break };
            let key = [HAL, ZWJ, letter];
            let Some(code) = self.code(&key) else { break };
            conj.push_str(code);
            cluster.extend_from_slice(&key);
            i += 3;
        }

        // 3. Signs. A joiner in front of a sign (ක + ZWJ + ෘ) means nothing.
        let mut signs: Vec<char> = Vec::new();
        while let Some(&c) = chars.get(i) {
            if is_sign(c) {
                signs.push(c);
                i += 1;
            } else if (c == ZWJ || c == ZWNJ) && chars.get(i + 1).is_some_and(|&n| is_sign(n)) {
                i += 1;
            } else {
                break;
            }
        }
        let (main, extras): (Option<char>, &[char]) = match signs.first() {
            Some(&s) if s != ANUSVARA && s != VISARGA => (Some(s), &signs[1..]),
            _ => (None, &signs[..]),
        };

        match main {
            None => {
                out.push_str(&base);
                out.push_str(&conj);
            }
            Some(sign) => {
                let mut key = cluster.clone();
                key.push(sign);
                match self.code(&key) {
                    // The whole syllable has its own glyph (ඩ් ඩි ච් මි වි ...).
                    Some(whole) => out.push_str(whole),
                    None => self.compose(&base_chars, &base, &conj, sign, out, missing),
                }
            }
        }
        for &extra in extras {
            self.compose(&[], "", "", extra, out, missing);
        }
        i
    }

    pub fn convert(&self, text: &str) -> Converted {
        let chars = join_vowel_signs(text);
        let mut out = String::with_capacity(text.len());
        let mut missing = Vec::new();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if is_base(c) {
                i = self.syllable(&chars, i, &mut out, &mut missing);
            } else if c == ZWJ || c == ZWNJ {
                // Joiners mean nothing to a legacy font.
                i += 1;
            } else if is_sign(c) {
                // A sign with no letter in front of it.
                self.compose(&[], "", "", c, &mut out, &mut missing);
                i += 1;
            } else {
                // Space, digits, punctuation, Latin. Punctuation has entries of its own.
                self.push_code(c, &mut out, &mut missing);
                i += 1;
            }
        }

        Converted { text: out, missing }
    }

    /// A check sheet for the real font. Each row is a Unicode text and the legacy
    /// code made for it. After each group there is one line with only the codes.
    /// Paste that line into a document set to FM Abhaya and compare the glyphs
    /// with the rows above it.
    pub fn chart(&self) -> String {
        let letters = |range: &[std::ops::RangeInclusive<u32>]| -> Vec<String> {
            range
                .iter()
                .flat_map(|r| r.clone())
                .filter_map(char::from_u32)
                .filter(|c| is_base(*c) && c.is_alphabetic())
                .map(String::from)
                .collect()
        };
        let on_ka = |signs: &[u32]| -> Vec<String> {
            signs.iter().filter_map(|&cp| char::from_u32(cp)).map(|s| format!("ක{s}")).collect()
        };

        // Every letter + sign pair that has its own glyph, in code point order.
        let mut special: Vec<(Vec<char>, String)> = self
            .entries
            .iter()
            .filter(|(k, _)| k.len() == 2 && is_consonant(k[0]) && is_sign(k[1]))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        special.sort_by(|a, b| (a.0[1], a.0[0]).cmp(&(b.0[1], b.0[0])));
        let special: Vec<String> = special.into_iter().map(|(k, _)| k.into_iter().collect()).collect();

        let kombuva_on = ['ක', 'ඩ', 'ම', 'ද', 'ඛ', 'ථ', 'ඪ'];
        let kombuva_signs = ['\u{0DD9}', '\u{0DDA}', '\u{0DDB}', '\u{0DDC}', '\u{0DDD}', '\u{0DDE}'];
        let kombuva: Vec<String> = kombuva_on
            .iter()
            .flat_map(|c| kombuva_signs.iter().map(move |s| format!("{c}{s}")))
            .collect();

        let conj = |tail: &str| format!("{HAL}{ZWJ}{tail}");
        let ra = conj("\u{0DBB}");
        let ya = conj("\u{0DBA}");
        let mut conjuncts: Vec<String> = vec![
            format!("ක{ra}"),
            format!("ක{ya}"),
            format!("ද{ra}"),
            format!("ක{HAL}ෂ"),
            format!("ක{HAL}{ZWJ}ෂ"),
            format!("ර{ya}"),
            format!("ර{HAL}{ZWJ}ණ"),
            format!("බ{ra}ි"),
            format!("බ{ra}ී"),
        ];
        for sign in ['\u{0DD9}', '\u{0DDA}', '\u{0DDC}', '\u{0DDD}', '\u{0DDE}', '\u{0DDB}'] {
            conjuncts.push(format!("ක{ra}{sign}"));
            conjuncts.push(format!("ක{ya}{sign}"));
        }
        conjuncts.push(format!("ද{ra}\u{0DD9}"));
        conjuncts.push(format!("ද{ra}\u{0DDA}"));
        conjuncts.push(format!("ක{HAL}ෂ{ya}\u{0DDD}"));

        let mut punctuation: Vec<String> = self
            .entries
            .keys()
            .filter(|k| k.len() == 1 && !is_sinhala_block(k[0]))
            .map(|k| k[0].to_string())
            .collect();
        punctuation.sort();

        let groups: Vec<(&str, Vec<String>)> = vec![
            ("Checks from your list", ["ඩ්", "ඩි", "ච්", "චි", "මි", "මී", "වි"].map(String::from).to_vec()),
            ("Independent vowels", letters(&[0x0D85..=0x0D96])),
            ("Consonants", letters(&[0x0D9A..=0x0DC6])),
            (
                "Signs on ක",
                on_ka(&[
                    0x0DCA, 0x0DCF, 0x0DD0, 0x0DD1, 0x0DD2, 0x0DD3, 0x0DD4, 0x0DD6, 0x0DD8, 0x0DF2, 0x0DD9, 0x0DDA,
                    0x0DDB, 0x0DDC, 0x0DDD, 0x0DDE, 0x0DDF, 0x0D82, 0x0D83,
                ]),
            ),
            ("Special glyphs (letter + sign with its own code)", special),
            ("Kombuva on letters with special glyphs", kombuva),
            ("Conjuncts and ligatures", conjuncts),
            ("Punctuation", punctuation),
        ];

        let mut out = String::new();
        for (title, items) in &groups {
            out.push_str(&format!("== {title}\n"));
            let mut pasted: Vec<String> = Vec::new();
            for item in items {
                let done = self.convert(item);
                let shown = if done.missing.is_empty() { done.text.as_str() } else { "(missing)" };
                out.push_str(&format!("{item}  =  {shown}\n"));
                if done.missing.is_empty() {
                    pasted.push(done.text);
                }
            }
            out.push_str("Paste into FM Abhaya and compare:\n");
            out.push_str(&pasted.join(" "));
            out.push_str("\n\n");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::transliterate;

    fn legacy(singlish: &str) -> String {
        LegacyMap::builtin().convert(&transliterate(singlish)).text
    }

    fn conv(unicode: &str) -> String {
        LegacyMap::builtin().convert(unicode).text
    }

    fn check(table: &[(&str, &str)]) {
        let map = LegacyMap::builtin();
        for (unicode, expected) in table {
            let got = map.convert(unicode);
            assert_eq!(&got.text, expected, "{unicode} (missing: {:?})", got.missing);
        }
    }

    // ---------- the map itself ----------

    #[test]
    fn builtin_map_parses() {
        let _ = LegacyMap::builtin();
    }

    #[test]
    fn builtin_map_never_gives_one_code_to_two_letters() {
        // Two spellings of the same text may share a code. Nothing else may.
        let expected = vec![
            ("Ì".to_string(), vec!["0D9A+0DCA+0DC2".to_string(), "0D9A+0DCA+200D+0DC2".to_string()]), // ක්ෂ, with and without joiner
            ("—".to_string(), vec!["0022".to_string(), "201C".to_string()]), // " and “ (the table you sent gives " the same code as “)
        ];
        assert_eq!(LegacyMap::builtin().conflicts(), expected);
    }

    #[test]
    fn duplicate_lines_are_rejected() {
        let err = LegacyMap::parse("0D9A l\n0D9B L\n0D9A k").err().unwrap();
        assert!(err.contains("line 3") && err.contains("line 1"), "{err}");
    }

    #[test]
    fn conflicts_are_reported() {
        let map = LegacyMap::parse("0D9A l\n0D9B l\n0D9C g").unwrap();
        assert_eq!(map.conflicts(), vec![("l".to_string(), vec!["0D9A".to_string(), "0D9B".to_string()])]);
    }

    #[test]
    fn map_file_format() {
        // note after " //", legacy code that is a comment character, BOM, CRLF
        let src = "\u{feff}# comment\r\n0D9A l   // ka\r\n0D83 #\r\n0D9C .\r\n";
        let map = LegacyMap::parse(src).unwrap();
        assert_eq!(map.convert("\u{0D9A}\u{0D83}\u{0D9C}").text, "l#.");
        assert!(LegacyMap::parse("0D9A").is_err());
        assert!(LegacyMap::parse("ZZ l").is_err());
        assert!(LegacyMap::parse("0D9A   // only a note").is_err());
    }

    #[test]
    fn missing_codes_are_reported_not_lost() {
        let map = LegacyMap::parse("0D9A l").unwrap();
        let got = map.convert("\u{0D9B}\u{0D9A}");
        assert_eq!(got.text, "\u{0D9B}l");
        assert_eq!(got.missing, vec!['\u{0D9B}']);
    }

    #[test]
    fn your_own_line_beats_the_composed_result() {
        // No special glyph: composed from the pieces.
        let plain = LegacyMap::parse("0D9A l\n0DD2 s").unwrap();
        assert_eq!(plain.convert("කි").text, "ls");
        // A letter + sign line wins over the pieces.
        let own = LegacyMap::parse("0D9A l\n0DD2 s\n0D9A+0DD2 Z").unwrap();
        assert_eq!(own.convert("කි").text, "Z");
    }

    // ---------- the seven mappings you verified, and what the old converter gave ----------

    #[test]
    fn verified_special_glyphs() {
        check(&[
            ("ඩ්", "â"),
            ("ඩි", "ä"),
            ("ච්", "É"),
            ("චි", "Ñ"),
            ("මි", "ñ"),
            ("මී", "ó"),
            ("වි", "ú"),
        ]);
    }

    #[test]
    fn the_old_wrong_outputs_are_gone() {
        for (unicode, wrong) in
            [("ඩි", "vs"), ("ඩ්", "va"), ("ච්", "pa"), ("චි", "ps"), ("මි", "us"), ("මී", "uS")]
        {
            assert_ne!(conv(unicode), wrong, "{unicode}");
        }
    }

    // ---------- special glyphs, every one the reference has ----------

    #[test]
    fn hal_glyphs() {
        check(&[
            ("ඛ්", "Ä"), ("ච්", "É"), ("ඡ්", "þ"), ("ජ්", "Ê"), ("ඦ්", "`Ê"), ("ට්", "Ü"), ("ඩ්", "â"),
            ("ඬ්", "å"), ("ධ්", "è"), ("බ්", "í"), ("ම්", "ï"), ("ඹ්", "ò"), ("ර්", "¾"), ("ව්", "õ"),
            ("ඞ්", "Ù"),
        ]);
    }

    #[test]
    fn is_pilla_glyphs() {
        check(&[
            ("ඛි", "Å"), ("චි", "Ñ"), ("ඡි", "ý"), ("ජි", "ð"), ("ඦි", "`ð"), ("ටි", "á"), ("ඨි", "À"),
            ("ඩි", "ä"), ("ණි", "‚"), ("ඬි", "ç"), ("ථි", "Ó"), ("දි", "È"), ("ධි", "ê"), ("ඳි", "¢"),
            ("ඵි", "Ý"), ("බි", "ì"), ("මි", "ñ"), ("ඹි", "ô"), ("රි", "ß"), ("වි", "ú"),
        ]);
    }

    #[test]
    fn diga_is_pilla_glyphs() {
        check(&[
            ("ඛී", "Ç"), ("චී", "Ö"), ("ඡී", "Â"), ("ජී", "Ô"), ("ටී", "à"), ("ඨී", "Á"), ("ඩී", "ã"),
            ("ඪී", "Î"), ("ණී", "Œ"), ("ඬී", "é"), ("ථී", "Ò"), ("දී", "§"), ("ධී", "ë"), ("ඳී", "£"),
            ("ඵී", "Ú"), ("බී", "î"), ("මී", "ó"), ("ඹී", "ö"), ("රී", "Í"), ("වී", "ù"), ("ඞී", "ü"),
        ]);
    }

    #[test]
    fn papilla_glyphs() {
        check(&[
            ("කු", "l="), ("ගු", ".="), ("ඟු", "Õ="), ("තු", ";="), ("දු", "ÿ"), ("ඳු", "÷"), ("භු", "N="),
            ("රු", "re"), ("ලු", "¨"), ("ශු", "Y="), ("ළු", "¿"),
            ("කූ", "l+"), ("ගූ", ".+"), ("තූ", ";+"), ("දූ", "¥"), ("ඳූ", "ª"), ("භූ", "N+"), ("රූ", "rE"),
            ("ලූ", "Æ"), ("ශූ", "Y+"),
        ]);
    }

    #[test]
    fn letters_without_a_special_glyph_use_the_plain_sign() {
        check(&[("මු", "uq"), ("ඩු", "vq"), ("පි", "ms"), ("ඩෘ", "vD"), ("ඟූ", "ÕQ"), ("ඤු", "[q"), ("ඞි", "Xs")]);
    }

    #[test]
    fn ra_with_ae_pilla_has_its_own_glyphs() {
        check(&[("රැ", "/"), ("රෑ", "?"), ("කැ", "le"), ("කෑ", "lE")]);
    }

    // ---------- independent vowels and plain letters ----------

    #[test]
    fn independent_vowels() {
        check(&[
            ("අ", "w"), ("ආ", "wd"), ("ඇ", "we"), ("ඈ", "wE"), ("ඉ", "b"), ("ඊ", "B"), ("උ", "W"),
            ("ඌ", "W!"), ("ඍ", "R"), ("එ", "t"), ("ඒ", "ta"), ("ඓ", "ft"), ("ඔ", "T"), ("ඕ", "´"),
            ("ඖ", "T!"),
        ]);
    }

    #[test]
    fn u_and_e_are_not_swapped_any_more() {
        // The old map gave උ = t, which is එ, and ඌ = T, which is ඔ.
        assert_eq!(conv("උ"), "W");
        assert_eq!(conv("එ"), "t");
        assert_eq!(legacy("u"), "W");
        assert_eq!(legacy("e"), "t");
    }

    #[test]
    fn consonants() {
        check(&[
            ("ක", "l"), ("ඛ", "L"), ("ග", "."), ("ඝ", ">"), ("ඞ", "X"), ("ඟ", "Õ"), ("ච", "p"), ("ඡ", "P"),
            ("ජ", "c"), ("ඣ", "CO"), ("ඤ", "["), ("ඥ", "{"), ("ඦ", "`c"), ("ට", "g"), ("ඨ", "G"), ("ඩ", "v"),
            ("ඪ", "V"), ("ණ", "K"), ("ඬ", "~"), ("ත", ";"), ("ථ", ":"), ("ද", "o"), ("ධ", "O"), ("න", "k"),
            ("ඳ", "|"), ("ප", "m"), ("ඵ", "M"), ("බ", "n"), ("භ", "N"), ("ම", "u"), ("ඹ", "U"), ("ය", "h"),
            ("ර", "r"), ("ල", ","), ("ව", "j"), ("ශ", "Y"), ("ෂ", "I"), ("ස", "i"), ("හ", "y"), ("ළ", "<"),
            ("ෆ", "*"),
        ]);
    }

    #[test]
    fn plain_signs() {
        check(&[
            ("කා", "ld"), ("කැ", "le"), ("කෑ", "lE"), ("කි", "ls"), ("කී", "lS"), ("කෘ", "lD"), ("කෲ", "lDD"),
            ("කං", "lx"), ("කඃ", "l#"), ("ක්", "la"), ("කෟ", "l!"),
        ]);
    }

    // ---------- kombuva ----------

    #[test]
    fn kombuva_signs_move_in_front() {
        check(&[
            ("කෙ", "fl"), ("කේ", "fla"), ("කෛ", "ffl"), ("කො", "fld"), ("කෝ", "flda"), ("කෞ", "fl!"),
        ]);
    }

    #[test]
    fn kombuva_ee_uses_the_hal_glyph() {
        check(&[
            ("බේ", "fí"), ("මේ", "fï"), ("චේ", "fÉ"), ("ඩේ", "fâ"), ("ජේ", "fÊ"), ("ටේ", "fÜ"), ("ධේ", "fè"),
            ("වේ", "fõ"), ("ඹේ", "fò"), ("ඡේ", "fþ"), ("ඛේ", "fÄ"), ("ඬේ", "få"), ("රේ", "f¾"),
        ]);
    }

    #[test]
    fn kombuva_on_letters_without_special_glyphs() {
        check(&[("පේ", "fma"), ("දේ", "foa"), ("සේ", "fia"), ("නේ", "fka"), ("ලේ", "f,a"), ("ගේ", "f.a")]);
    }

    #[test]
    fn letters_the_reference_script_broke_now_work() {
        check(&[
            ("ඛෙ", "fL"),    // script gave fn (බ)
            ("ඪේ", "fVa"),   // script lost the letter: fa
            ("ෆේ", "f*a"),   // script lost the hal: f*
            ("ඬො", "f~d"),   // script added an extra a: f~da
            ("ථෙ", "f:"),    // script turned : into (
            ("ථේ", "f:a"),
            ("ථො", "f:d"),
            ("ථෝ", "f:da"),
            ("ඣෙ", "fCO"),   // script put the kombuva after: COf
            ("ඦෙ", "f`c"),   // script put the kombuva after: `cf
            ("ඳෙ", "f|"),    // script put an invisible character in
            ("ඟෙ", "fÕ"),    // script mixed Û and Õ
            ("ඟෝ", "fÕda"),
        ]);
    }

    #[test]
    fn ai_and_au_work_for_every_letter() {
        // The script handled ෛ for 12 letters and ෞ for 21. The rest stayed as Unicode.
        check(&[("ඛෛ", "ffL"), ("බෛ", "ffn"), ("ගෛ", "ff."), ("ඛෞ", "fL!"), ("ඹෞ", "fU!"), ("හෞ", "fy!")]);
    }

    #[test]
    fn split_vowel_signs_are_joined() {
        check(&[
            ("ක\u{0DD9}\u{0DCF}", "fld"),
            ("ක\u{0DD9}\u{0DCF}\u{0DCA}", "flda"),
            ("ක\u{0DD9}\u{0DDF}", "fl!"),
            ("ක\u{0DD9}\u{0DD9}", "ffl"),
            ("ක\u{0DD9}\u{0DCA}", "fla"),
            ("බ\u{0DD9}\u{0DCA}", "fí"),
        ]);
    }

    // ---------- conjuncts ----------

    #[test]
    fn rakaransaya_and_yansaya() {
        check(&[
            ("ක්\u{200D}ර", "l%"), ("ක්\u{200D}ය", "lH"), ("ක්\u{200D}රි", "l%s"), ("ක්\u{200D}රු", "l%q"), ("ධ්\u{200D}රි", "O%s"),
            ("ප්\u{200D}රශ්න", "m%Yak"),
        ]);
    }

    #[test]
    fn kombuva_goes_before_the_whole_conjunct() {
        check(&[
            ("ක්\u{200D}රෙ", "fl%"), ("ක්\u{200D}රො", "fl%d"), ("ක්\u{200D}රෝ", "fl%da"), ("ක්\u{200D}යෙ", "flH"), ("ක්\u{200D}යො", "flHd"),
            ("ක්\u{200D}යෝ", "flHda"), ("ප්\u{200D}රෞ", "fm%!"), ("ත්\u{200D}රෛ", "ff;%"),
        ]);
    }

    #[test]
    fn kombuva_ee_with_a_conjunct() {
        check(&[
            ("ක්\u{200D}රේ", "fla%"), ("බ්\u{200D}රේ", "fí%"), ("ඩ්\u{200D}රේ", "fâ%"), ("ධ්\u{200D}රේ", "fè%"), ("ග්\u{200D}රේ", "f.a%"),
            ("ශ්\u{200D}රේ", "fYa%"), // the script lost the %
            ("ක්\u{200D}යේ", "flaH"), // the script left ේ as Unicode
        ]);
    }

    #[test]
    fn dra_is_one_glyph_and_takes_every_sign() {
        check(&[
            ("ද්\u{200D}ර", "ø"), ("ද්\u{200D}රි", "øs"), ("ද්\u{200D}රී", "øS"), ("ද්\u{200D}රු", "øq"),
            ("ද්\u{200D}රෙ", "fø"), ("ද්\u{200D}රේ", "føa"), ("ද්\u{200D}රො", "fød"), ("ද්\u{200D}රෝ", "føda"), // the script left Unicode here
        ]);
    }

    #[test]
    fn ksha_ligature_with_and_without_joiner() {
        check(&[
            ("ක්\u{200D}ෂ", "Ì"), ("ක්ෂ", "Ì"), ("ක්\u{200D}ෂෙ", "fÌ"), ("ක්ෂෝ", "fÌda"), ("ක්\u{200D}ෂේ", "fÌa"),
            ("ක්\u{200D}ෂ්\u{200D}යෝ", "fÌHda"), ("ක්ෂි", "Ìs"), ("ලක්ෂ", ",Ì"),
        ]);
    }

    #[test]
    fn repaya_ligatures() {
        check(&[("ර්\u{200D}ය", "©"), ("ර්\u{200D}ණ", "“"), ("ආර්\u{200D}ය", "wd©"), ("ධර්ම", "O¾u"), ("කර්මය", "l¾uh")]);
    }

    #[test]
    fn bri_keeps_its_own_glyph_but_not_for_other_signs() {
        check(&[("බ්\u{200D}රි", "ì%"), ("බ්\u{200D}රී", "n%S"), ("බ්\u{200D}ර", "n%")]);
    }

    #[test]
    fn hal_in_the_middle_of_a_word() {
        check(&[("අම්මා", "wïud"), ("ඩ්ඩ", "âv"), ("ක්ත", "la;"), ("ක්\u{200D}ත", "la;")]);
    }

    #[test]
    fn stray_joiners_vanish() {
        assert_eq!(conv("\u{200D}\u{0D9A}"), "l");
        assert_eq!(conv("ක\u{200D}ෘ"), "lD");
        assert_eq!(conv("ක\u{200C}"), "l");
    }

    // ---------- punctuation and other characters ----------

    #[test]
    fn punctuation_has_its_own_codes() {
        check(&[
            (",", "\""), (".", "'"), ("'", "z"), ("(", "^"), (")", "&"), ("%", "]"), ("/", "$"), ("?", "@"),
            ("!", "^æ"), ("=", "}"), ("+", "¬"), (":", "("), (";", "¦"), ("~", "~&"), ("–", "-"), ("“", "—"),
            ("”", "˜"), ("\"", "—"),
        ]);
    }

    /// The table the user sent, every ASCII mark and its legacy code.
    #[test]
    fn every_ascii_mark_follows_the_users_table() {
        let table = [
            ("!", "^æ"), ("\"", "—"), ("#", "#"), ("$", "$"), ("%", "]"), ("&", "&"), ("'", "z"), ("(", "^"),
            (")", "&"), ("*", "*"), ("+", "¬"), (",", "\""), ("-", "-"), (".", "'"), ("/", "$"), (":", "("),
            (";", "¦"), ("<", "<"), ("=", "}"), (">", ">"), ("?", "@"), ("@", "@"), ("[", "["), ("\\", "\\"),
            ("]", "]"), ("^", "^"), ("_", "_"), ("`", "`"), ("{", "{"), ("|", "|"), ("}", "}"), ("~", "~&"),
        ];
        assert_eq!(table.len(), 32); // all the ASCII punctuation marks
        check(&table);
    }

    #[test]
    fn marks_in_a_sentence_are_converted_one_by_one() {
        assert_eq!(conv("ක?"), "l@");
        assert_eq!(conv("ක. ක: 5%"), "l' l( 5]");
        assert_eq!(conv("ක!"), "l^æ");
    }

    #[test]
    fn digits_spaces_and_latin_pass_through() {
        assert_eq!(conv("ක 12"), "l 12");
        assert_eq!(conv("ka-1"), "ka-1");
    }

    #[test]
    fn a_sign_with_no_letter_is_still_written() {
        check(&[("ා", "d"), ("ෙ", "f"), ("ේ", "fa"), ("ො", "fd")]);
    }

    // ---------- coverage ----------

    /// Letters and signs of the Sinhala block that Unicode has assigned.
    fn assigned() -> Vec<u32> {
        let mut v = vec![0x0D82, 0x0D83];
        v.extend(0x0D85..=0x0D96);
        v.extend(0x0D9A..=0x0DB1);
        v.extend(0x0DB3..=0x0DBB);
        v.push(0x0DBD);
        v.extend(0x0DC0..=0x0DC6);
        v.push(0x0DCA);
        v.extend(0x0DCF..=0x0DD4);
        v.push(0x0DD6);
        v.extend(0x0DD8..=0x0DDF);
        v.extend(0x0DE6..=0x0DEF);
        v.extend(0x0DF2..=0x0DF4);
        v
    }

    /// Characters with no known legacy code. They stay Unicode and are reported.
    const UNSUPPORTED: &[u32] = &[
        0x0D8E, // ඎ  no code in the reference
        0x0D8F, // ඏ  the reference gives the code of ක්ෂ
        0x0DF3, // ෳ  no code in the reference
        0x0DF4, // ෴  kunddaliya, no code in the reference
        0x0DE6, 0x0DE7, 0x0DE8, 0x0DE9, 0x0DEA, 0x0DEB, 0x0DEC, 0x0DED, 0x0DEE, 0x0DEF, // Sinhala lith digits
    ];

    #[test]
    fn every_assigned_sinhala_character_is_mapped_or_listed_as_unsupported() {
        let map = LegacyMap::builtin();
        for cp in assigned() {
            let c = char::from_u32(cp).unwrap();
            // Signs are tested on a letter, because they only make sense after one.
            let text = if is_sign(c) { format!("ක{c}") } else { c.to_string() };
            let got = map.convert(&text);
            if UNSUPPORTED.contains(&cp) {
                assert!(got.missing.contains(&c), "U+{cp:04X} is listed as unsupported but has a code");
            } else {
                assert!(got.missing.is_empty(), "U+{cp:04X} {c} has no legacy code");
                assert!(!got.text.chars().any(is_sinhala_block), "U+{cp:04X} {c} left Unicode in {}", got.text);
            }
        }
    }

    #[test]
    fn no_letter_sign_conjunct_combination_leaves_unicode_or_a_joiner() {
        let map = LegacyMap::builtin();
        let signs: Vec<char> = "\u{0DCA}\u{0DCF}\u{0DD0}\u{0DD1}\u{0DD2}\u{0DD3}\u{0DD4}\u{0DD6}\u{0DD8}\u{0DF2}\u{0DDF}\u{0DD9}\u{0DDA}\u{0DDB}\u{0DDC}\u{0DDD}\u{0DDE}\u{0D82}\u{0D83}"
            .chars()
            .collect();
        let conjuncts = ["", "\u{0DCA}\u{200D}\u{0DBB}", "\u{0DCA}\u{200D}\u{0DBA}"];
        let mut tested = 0;
        for cp in assigned().into_iter().filter(|c| (0x0D9A..=0x0DC6).contains(c)) {
            let letter = char::from_u32(cp).unwrap();
            for conj in conjuncts {
                for sign in std::iter::once(None).chain(signs.iter().map(|s| Some(*s))) {
                    let text = format!("{letter}{conj}{}", sign.map(String::from).unwrap_or_default());
                    let got = map.convert(&text);
                    tested += 1;
                    assert!(got.missing.is_empty(), "{text}: missing {:?}", got.missing);
                    assert!(
                        !got.text.chars().any(|c| is_sinhala_block(c) || c == ZWJ || c == ZWNJ || c == '\u{200B}'),
                        "{text}: {}",
                        got.text
                    );
                }
            }
        }
        assert!(tested > 2400, "tested only {tested}");
    }

    #[test]
    fn kombuva_always_comes_first_and_the_letter_is_never_lost() {
        let map = LegacyMap::builtin();
        for cp in (0x0D9A..=0x0DC6).filter(|c| assigned().contains(c)) {
            let letter = char::from_u32(cp).unwrap();
            let plain = map.convert(&letter.to_string()).text;
            for sign in ['\u{0DD9}', '\u{0DDA}', '\u{0DDB}', '\u{0DDC}', '\u{0DDD}', '\u{0DDE}'] {
                let got = map.convert(&format!("{letter}{sign}")).text;
                assert!(got.starts_with('f'), "{letter}{sign} -> {got}");
                let hal = map.convert(&format!("{letter}\u{0DCA}")).text;
                assert!(got.contains(&plain) || got.contains(&hal), "{letter}{sign} -> {got}");
            }
        }
    }

    #[test]
    fn random_text_never_panics_and_never_leaves_an_unreported_character() {
        // A small deterministic generator, so a failure can be repeated.
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut pool: Vec<char> = (0x0D80..=0x0DFF).filter_map(char::from_u32).collect();
        pool.extend(['\u{200D}', '\u{200C}', ' ', 'a', '1', '.', ',', '"', '\u{00F7}']);
        let map = LegacyMap::builtin();
        for _ in 0..30_000 {
            let len = (next() % 12) as usize;
            let text: String = (0..len).map(|_| pool[(next() % pool.len() as u64) as usize]).collect();
            let got = map.convert(&text);
            for c in got.text.chars().filter(|c| is_sinhala_block(*c)) {
                assert!(got.missing.contains(&c), "{text:?} left {c:?} without reporting it");
            }
        }
    }

    // ---------- the typing path ----------

    #[test]
    fn typing_path_ballaa_and_friends() {
        assert_eq!(legacy("ballaa"), "n,a,d");
        assert_eq!(legacy("ka"), "l");
        assert_eq!(legacy("k"), "la");
        assert_eq!(legacy("ku"), "l=");
        assert_eq!(legacy("kuu"), "l+");
        assert_eq!(legacy("kru"), "lD");
        assert_eq!(legacy("kax"), "lx");
        assert_eq!(legacy("aa"), "wd");
        assert_eq!(legacy("ke"), "fl");
        assert_eq!(legacy("kee"), "fla");
        assert_eq!(legacy("kai"), "ffl");
        assert_eq!(legacy("ko"), "fld");
        assert_eq!(legacy("koo"), "flda");
        assert_eq!(legacy("kau"), "fl!");
        assert_eq!(legacy("kya"), "lH");
        assert_eq!(legacy("kra"), "l%");
        assert_eq!(legacy("kro"), "fl%d");
        assert_eq!(legacy("kke"), "lafl");
    }

    #[test]
    fn typing_path_special_glyph_words() {
        assert_eq!(legacy("dd"), "ââ");
        assert_eq!(legacy("dda"), "âv");
        assert_eq!(legacy("di"), "ä");
        assert_eq!(legacy("chi"), "Ñ");
        assert_eq!(legacy("mi"), "ñ");
        assert_eq!(legacy("mii"), "ó");
        assert_eq!(legacy("wi"), "ú");
        assert_eq!(legacy("ammaa"), "wïud");
        assert_eq!(legacy("sixhala"), "isxy,");
        assert_eq!(legacy("koLaBa"), "fld<U");
        assert_eq!(legacy("prashna"), "m%Yak");
    }

    #[test]
    fn typing_path_while_a_word_is_half_typed() {
        // Each prefix of a word must convert without losing anything.
        let map = LegacyMap::builtin();
        for word in ["mama", "gedara", "ballaa", "kriyaa", "dharmaya", "sri lankaava", "wiidiyo"] {
            for end in 1..=word.len() {
                let got = map.convert(&transliterate(&word[..end]));
                assert!(got.missing.is_empty(), "{}", &word[..end]);
                assert!(!got.text.chars().any(is_sinhala_block), "{}", &word[..end]);
            }
        }
    }

    // ---------- the check sheet ----------

    #[test]
    fn chart_shows_the_special_glyphs_and_only_the_two_unknown_letters_as_missing() {
        let chart = LegacyMap::builtin().chart();
        assert!(chart.contains("ඩි  =  ä\n"));
        assert!(chart.contains("ම්  =  ï\n"));
        assert!(chart.contains("ක්\u{200D}ෂ  =  Ì\n"));
        assert!(chart.contains("Paste into FM Abhaya"));
        assert_eq!(chart.matches("(missing)").count(), 2, "{chart}"); // ඎ and ඏ
    }

    #[test]
    fn chart_marks_missing_rows_for_a_small_map() {
        let chart = LegacyMap::parse("0D9A l").unwrap().chart();
        assert!(chart.contains("(missing)"));
        assert!(chart.contains("ක  =  l\n"));
    }
}