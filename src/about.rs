//! The name and the contact links shown in the window.
//! Plain text only, no Windows calls, so it is tested on any system.

pub const NAME: &str = "LIPI";
pub const NAME_SINHALA: &str = "ලිපි";
pub const TAGLINE: &str = "Type Singlish in any app and get Sinhala";
pub const DEVELOPER: &str = "SANJAYA3D";

/// The round icon buttons under the developer name.
/// Clicking one opens its address in the browser.
pub const GITHUB_URL: &str = "https://github.com/sanjaya3d/LIPI";
pub const FACEBOOK_URL: &str = "https://facebook.com/sanjaya3d";
pub const YOUTUBE_URL: &str = "https://www.youtube.com/@SANJAYA3D/";
/// The fourth round button: the developer's portfolio website.
pub const PORTFOLIO_URL: &str = "https://sanjaya3d.github.io/portfolio/";

/// The version of this program. It comes from Cargo.toml, so it can never be out of date.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The "Check for updates" button on the Settings page opens this page in the browser.
/// GitHub sends it on to the newest release. The program itself never goes online.
pub const RELEASES_URL: &str = "https://github.com/sanjaya3d/LIPI/releases/latest";

/// The warning card between the output buttons and the key hints.
/// In legacy mode, Adobe and Microsoft apps turn the full stop into a curly quote.
/// The Sinhala says: "In Adobe and Microsoft software, turn off the smart quotes feature."
/// The button says "More details" and opens the Smart quotes section of the website, which explains how and why.
/// The joiner (U+200D) inside the word "kriya" is written as an escape, so it cannot get lost.
pub const TIP_LINE1: &str = "Adobe සහ Microsoft මෘදුකාංග වලදී";
pub const TIP_LINE2: &str = "\"smart quotes\" විශේෂාංගය ක්\u{200D}රියා විරහිත කරන්න";
pub const TIP_BUTTON: &str = "වැඩි විස්තර";
pub const TIP_URL: &str = "https://sanjaya3d.github.io/LIPI/#smart-quotes";

/// The phrase in the smart quotes text that is shown in lime. Its space is a no-break space,
/// so the phrase is never split over two lines.
pub const TIP_PHRASE: &str = "\"smart\u{00A0}quotes\"";

/// The whole smart quotes text as one string, so that it can be wrapped to the width of the card.
pub fn tip_full() -> String {
    format!("{TIP_LINE1} {TIP_LINE2}").replace("\"smart quotes\"", TIP_PHRASE)
}

/// Breaks `text` into lines, only at spaces, so that no line is wider than `max` pixels.
/// `measure` gives the width of a piece of text. A single word that is wider than `max`
/// stays on a line of its own. Each line is a byte range (start, end) of `text`.
pub fn wrap_words(text: &str, max: i32, mut measure: impl FnMut(&str) -> i32) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut line: Option<(usize, usize)> = None;
    let mut pos = 0;
    for word in text.split(' ') {
        let (start, end) = (pos, pos + word.len());
        pos = end + 1;
        if word.is_empty() {
            continue;
        }
        line = match line {
            None => Some((start, end)),
            Some((first, last)) => {
                if measure(&text[first..end]) <= max {
                    Some((first, end))
                } else {
                    lines.push((first, last));
                    Some((start, end))
                }
            }
        };
    }
    lines.extend(line);
    lines
}

/// Cuts one line (a byte range) around the highlighted phrase (a byte range) into pieces
/// (start, end, highlighted). Empty pieces are left out.
pub fn pieces(line: (usize, usize), phrase: (usize, usize)) -> Vec<(usize, usize, bool)> {
    let (start, end) = line;
    let (p_start, p_end) = (phrase.0.clamp(start, end), phrase.1.clamp(start, end));
    let mut out = Vec::new();
    if p_start >= p_end {
        out.push((start, end, false)); // the phrase is not on this line
    } else {
        out.push((start, p_start, false));
        out.push((p_start, p_end, true));
        out.push((p_end, end, false));
    }
    out.retain(|piece| piece.0 < piece.1);
    out
}

/// The version as the title bar badge shows it: "1.2.0" becomes "v1.2".
pub fn short_version() -> String {
    let mut parts = VERSION.split('.');
    match (parts.next(), parts.next()) {
        (Some(major), Some(minor)) => format!("v{major}.{minor}"),
        _ => format!("v{VERSION}"),
    }
}

/// The Sinhala line under "Sinhala Typing": typing is on, or typing is off.
/// The joiner (U+200D) inside "kriya" is written as an escape, so it cannot get lost.
pub const TYPING_ON_SI: &str = "සිංහල ටයිප් කිරීම සක්\u{200D}රියයි";
pub const TYPING_OFF_SI: &str = "සිංහල ටයිප් කිරීම අක්\u{200D}රියයි";

/// The two output buttons: the name, and what it is for.
pub const UNICODE_NAME: &str = "Unicode";
pub const UNICODE_FOR: &str = "Web, Chrome & Docs";
pub const LEGACY_NAME: &str = "FM Abhaya";
pub const LEGACY_FOR: &str = "Photoshop, Premiere Pro";

/// The line under the output buttons: what LIPI is sending right now.
pub fn encoding_status(enabled: bool, legacy: bool) -> &'static str {
    if !enabled {
        "Sinhala typing is off. Your keys go to the app as they are."
    } else if legacy {
        "Currently sending FM Abhaya letters. Use an FM Abhaya font in the app."
    } else {
        "Currently sending standard Sinhala Unicode text to the app that has focus."
    }
}

// ---- The Settings page. Plain English, short enough for the cards in `layout.rs`.

/// The page name in the title bar, after the program name.
pub const PAGE_NAME: &str = "Settings";
/// The page name in the title bar of the key hint page, and its heading and tag.
pub const KEYS_PAGE_NAME: &str = "Key hints";
pub const KEYS_HEADING: &str = "KEY HINTS";
pub const KEYS_TAG: &str = "SINGLISH TO SINHALA";
pub const SETTINGS_HEADING: &str = "GENERAL & STARTUP";
pub const SETTINGS_TAG: &str = "ENGINE CONFIG";
pub const SETTINGS_LEAD: &str = "Manage how LIPI starts and runs in the background.";

pub const STARTUP_TITLE: &str = "Run on Windows Startup";
pub const STARTUP_NOTE: &str = "Launch LIPI automatically when you sign in to Windows.";
pub const STARTUP_DETAIL: &str = "Windows registry, this user only";

pub const TRAY_TITLE: &str = "Start Minimized to Tray";
pub const TRAY_NOTE: &str = "Quiet background mode, no window when LIPI starts.";
pub const TRAY_DETAIL: &str = "Tray icon in the notification area";

pub const UPDATE_TITLE: &str = "Check for Updates";
pub const UPDATE_NOTE: &str = "Opens the newest release on GitHub in your browser.";
pub const UPDATE_BUTTON: &str = "Check for updates";

/// The bar under the cards. The path is the folder that holds settings.txt.
pub const SAVED_NOTE: &str = "Changes saved automatically";
pub const SAVED_PATH: &str = "%APPDATA%\\LIPI";

/// The link at the right end of the Settings footer.
pub const GITHUB_LABEL: &str = "GitHub";

/// The status word at the end of the "Run on startup" detail row.
pub fn startup_status(on: bool) -> &'static str {
    if on { "Active" } else { "Inactive" }
}

/// The status word at the end of the "Start in tray" detail row.
pub fn tray_status(on: bool) -> &'static str {
    if on { "Silent mode" } else { "Window shown" }
}

/// The version in the chip on the update card: "v1.2.0".
pub fn version_chip() -> String {
    format!("v{VERSION}")
}

/// The first part of the Settings footer: "Rust Engine v1.2.0".
pub fn engine_line() -> String {
    format!("Rust Engine v{VERSION}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_safe_web_addresses() {
        for url in [GITHUB_URL, FACEBOOK_URL, YOUTUBE_URL, PORTFOLIO_URL, TIP_URL, RELEASES_URL] {
            assert!(url.starts_with("https://"), "{url}");
            assert!(!url.contains(' ') && url.is_ascii(), "{url}");
        }
    }

    #[test]
    fn each_link_goes_to_the_right_place() {
        assert!(GITHUB_URL.starts_with("https://github.com/sanjaya3d/"));
        assert!(FACEBOOK_URL.starts_with("https://facebook.com/sanjaya3d"));
        assert!(YOUTUBE_URL.starts_with("https://www.youtube.com/@SANJAYA3D"));
        assert_eq!(PORTFOLIO_URL, "https://sanjaya3d.github.io/portfolio/");
    }

    #[test]
    fn the_update_button_opens_the_latest_release_page_of_this_project() {
        assert_eq!(RELEASES_URL, format!("{GITHUB_URL}/releases/latest"));
    }

    #[test]
    fn the_version_is_three_numbers_like_1_1_0() {
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "{VERSION}");
        assert!(parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())), "{VERSION}");
    }

    #[test]
    fn the_tip_goes_to_the_smart_quotes_section_of_the_website() {
        assert_eq!(TIP_URL, "https://sanjaya3d.github.io/LIPI/#smart-quotes");
    }

    #[test]
    fn the_tip_text_is_the_wording_that_was_asked_for() {
        assert!(TIP_LINE1.starts_with("Adobe ") && TIP_LINE1.contains("Microsoft"));
        assert!(TIP_LINE2.contains("\"smart quotes\""));
        // the word "kriya" needs its joiner (U+200D) after the hal kirima, or the letters do not join
        assert!(TIP_LINE2.contains("\u{0D9A}\u{0DCA}\u{200D}\u{0DBB}\u{0DD2}\u{0DBA}\u{0DCF}"));
        assert_eq!(TIP_BUTTON, "\u{0DC0}\u{0DD0}\u{0DA9}\u{0DD2} \u{0DC0}\u{0DD2}\u{0DC3}\u{0DCA}\u{0DAD}\u{0DBB}");
    }

    #[test]
    fn the_name_is_what_was_asked_for() {
        assert_eq!(NAME, "LIPI");
        assert_eq!(NAME_SINHALA, "\u{0DBD}\u{0DD2}\u{0DB4}\u{0DD2}");
    }

    #[test]
    fn the_badge_shows_major_and_minor_only() {
        let badge = short_version();
        assert!(badge.starts_with('v'), "{badge}");
        let parts: Vec<&str> = badge[1..].split('.').collect();
        assert_eq!(parts.len(), 2, "{badge}");
        assert!(VERSION.starts_with(&badge[1..]), "{badge} against {VERSION}");
    }

    #[test]
    fn the_typing_lines_say_on_and_off_and_keep_their_joiner() {
        // "kriya" needs its joiner (U+200D) after the hal kirima, or the letters do not join
        let kriya = "\u{0D9A}\u{0DCA}\u{200D}\u{0DBB}\u{0DD2}\u{0DBA}";
        assert!(TYPING_ON_SI.contains(kriya) && TYPING_OFF_SI.contains(kriya));
        assert!(TYPING_ON_SI.ends_with("\u{0DC3}\u{0D9A}\u{0DCA}\u{200D}\u{0DBB}\u{0DD2}\u{0DBA}\u{0DBA}\u{0DD2}"));
        assert!(TYPING_OFF_SI.ends_with("\u{0D85}\u{0D9A}\u{0DCA}\u{200D}\u{0DBB}\u{0DD2}\u{0DBA}\u{0DBA}\u{0DD2}"));
        assert_ne!(TYPING_ON_SI, TYPING_OFF_SI);
    }

    #[test]
    fn the_status_line_follows_the_switch_and_the_output() {
        let unicode = encoding_status(true, false);
        let legacy = encoding_status(true, true);
        let off = encoding_status(false, false);
        assert!(unicode.contains("Unicode") && !unicode.contains("FM Abhaya"));
        assert!(legacy.contains("FM Abhaya") && !legacy.contains("Unicode"));
        assert!(off.contains("off"));
        // typing off wins over the output choice
        assert_eq!(encoding_status(false, true), off);
        // two lines at most in the card (about 46 letters a line)
        for line in [unicode, legacy, off] {
            assert!(line.len() <= 92, "{line}");
        }
    }

    #[test]
    fn the_output_buttons_have_short_names_and_labels() {
        assert_eq!((UNICODE_NAME, LEGACY_NAME), ("Unicode", "FM Abhaya"));
        for label in [UNICODE_FOR, LEGACY_FOR] {
            assert!(label.len() <= 26, "{label} would not fit under the name");
        }
    }

    #[test]
    fn the_settings_texts_are_short_plain_ascii_that_fit_their_cards() {
        // limits come from the card sizes in layout.rs (about 6.5 pixels a character at 96 DPI)
        for (name, text, max) in [
            ("heading", SETTINGS_HEADING, 24),
            ("tag", SETTINGS_TAG, 20),
            ("keys page name", KEYS_PAGE_NAME, 16),
            ("keys heading", KEYS_HEADING, 24),
            ("keys tag", KEYS_TAG, 24),
            ("lead", SETTINGS_LEAD, 56),
            ("startup title", STARTUP_TITLE, 28),
            ("startup note", STARTUP_NOTE, 58),
            ("startup detail", STARTUP_DETAIL, 36),
            ("tray title", TRAY_TITLE, 28),
            ("tray note", TRAY_NOTE, 58),
            ("tray detail", TRAY_DETAIL, 36),
            ("update title", UPDATE_TITLE, 26),
            ("update note", UPDATE_NOTE, 58),
            ("update button", UPDATE_BUTTON, 24),
            ("saved note", SAVED_NOTE, 30),
            ("saved path", SAVED_PATH, 18),
        ] {
            assert!(text.is_ascii(), "{name}: plain ASCII only");
            assert!(text.chars().count() <= max, "{name} is too long: {} characters", text.chars().count());
            assert!(!text.is_empty() && text.trim() == text, "{name}");
        }
    }

    #[test]
    fn the_status_words_are_short_and_different() {
        for on in [true, false] {
            assert!(startup_status(on).len() <= 14 && tray_status(on).len() <= 14);
        }
        assert_ne!(startup_status(true), startup_status(false));
        assert_ne!(tray_status(true), tray_status(false));
        assert_eq!(startup_status(true), "Active");
    }

    #[test]
    fn the_saved_path_names_the_folder_that_holds_the_settings_file() {
        assert_eq!(SAVED_PATH, format!("%APPDATA%\\{}", crate::settings::FOLDER_NAME));
    }

    #[test]
    fn the_version_texts_carry_the_cargo_version() {
        assert_eq!(version_chip(), format!("v{VERSION}"));
        assert_eq!(engine_line(), format!("Rust Engine v{VERSION}"));
        assert!(version_chip().len() <= 9, "the chip is 64 pixels wide");
    }

    // A stand-in for the real font: every character is 6 pixels wide.
    fn six(s: &str) -> i32 {
        s.chars().count() as i32 * 6
    }

    fn texts<'a>(text: &'a str, lines: &[(usize, usize)]) -> Vec<&'a str> {
        lines.iter().map(|&(a, b)| &text[a..b]).collect()
    }

    #[test]
    fn wrapping_breaks_only_at_spaces_and_keeps_every_word_in_order() {
        let text = "aa bb cc dd ee";
        let lines = wrap_words(text, 30, six); // 5 characters a line
        assert_eq!(texts(text, &lines), ["aa bb", "cc dd", "ee"]);
        let wide = wrap_words(text, 1000, six);
        assert_eq!(texts(text, &wide), [text]);
        // however narrow, the words stay whole and in order
        let narrow = wrap_words(text, 1, six);
        assert_eq!(texts(text, &narrow), ["aa", "bb", "cc", "dd", "ee"]);
    }

    #[test]
    fn wrapping_handles_empty_text_extra_spaces_and_a_word_wider_than_the_line() {
        assert!(wrap_words("", 100, six).is_empty());
        assert!(wrap_words("   ", 100, six).is_empty());
        let text = "  ab   cd  ";
        let lines = wrap_words(text, 100, six);
        assert_eq!(lines.len(), 1);
        assert_eq!(texts(text, &lines), ["ab   cd"]);
        let long = "abcdefghij kl";
        assert_eq!(texts(long, &wrap_words(long, 30, six)), ["abcdefghij", "kl"]);
    }

    #[test]
    fn no_wrapped_line_is_wider_than_the_limit_unless_it_is_one_word() {
        let text = tip_full();
        for max in [90, 150, 224, 330] {
            let lines = wrap_words(&text, max, six);
            for &(a, b) in &lines {
                let line = &text[a..b];
                assert!(six(line) <= max || !line.contains(' '), "{line:?} at {max}");
            }
            // the lines, put together again, are the same words
            let joined: Vec<&str> = lines.iter().flat_map(|&(a, b)| text[a..b].split(' ')).collect();
            assert_eq!(joined, text.split(' ').filter(|w| !w.is_empty()).collect::<Vec<_>>(), "max {max}");
        }
    }

    #[test]
    fn the_smart_quotes_text_fits_three_lines_in_the_card_and_the_phrase_is_not_split() {
        let text = tip_full();
        let lines = wrap_words(&text, 224, six);
        assert!(lines.len() <= 3, "{} lines", lines.len());
        // the phrase is in the text once, and it sits wholly on one line
        let start = text.find(TIP_PHRASE).expect("the phrase must be in the text");
        let phrase = (start, start + TIP_PHRASE.len());
        assert!(lines.iter().any(|&(a, b)| a <= phrase.0 && phrase.1 <= b), "the phrase was split over two lines");
        // and the words are the same ones as in the two lines that were asked for
        assert!(text.starts_with(TIP_LINE1) && text.ends_with("\u{0D9A}\u{0DBB}\u{0DB1}\u{0DCA}\u{0DB1}"));
    }

    #[test]
    fn the_phrase_is_cut_out_of_a_line_into_plain_lime_and_plain_pieces() {
        // line 0..20, phrase 5..12: plain, lime, plain
        assert_eq!(pieces((0, 20), (5, 12)), [(0, 5, false), (5, 12, true), (12, 20, false)]);
        // the phrase starts the line, or ends it
        assert_eq!(pieces((5, 20), (5, 12)), [(5, 12, true), (12, 20, false)]);
        assert_eq!(pieces((0, 12), (5, 12)), [(0, 5, false), (5, 12, true)]);
        // the phrase is not on this line, or only the end of it is
        assert_eq!(pieces((0, 4), (5, 12)), [(0, 4, false)]);
        assert_eq!(pieces((12, 30), (5, 12)), [(12, 30, false)]);
        assert_eq!(pieces((8, 20), (5, 12)), [(8, 12, true), (12, 20, false)]);
        // no phrase at all
        assert_eq!(pieces((0, 9), (0, 0)), [(0, 9, false)]);
    }
}
