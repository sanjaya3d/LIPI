//! The name and the contact links shown in the window.
//! Plain text only, no Windows calls, so it is tested on any system.

pub const NAME: &str = "LIPI";
pub const NAME_SINHALA: &str = "ලිපි";
pub const TAGLINE: &str = "Type Singlish in any app and get Sinhala";
pub const DEVELOPER: &str = "SANJAYA3D";

/// The three round icon buttons under the developer name.
/// Clicking one opens its address in the browser.
pub const GITHUB_URL: &str = "https://github.com/sanjaya3d/LIPI";
pub const FACEBOOK_URL: &str = "https://facebook.com/sanjaya3d";
pub const YOUTUBE_URL: &str = "https://www.youtube.com/@SANJAYA3D/";

/// The warning card between the output buttons and the key hints.
/// In legacy mode, Adobe and Microsoft apps turn the full stop into a curly quote.
/// The Sinhala says: "In Adobe and Microsoft software, turn off the smart quotes feature."
/// The button says "More details" and opens the README section that explains how and why.
/// The joiner (U+200D) inside the word "kriya" is written as an escape, so it cannot get lost.
pub const TIP_LINE1: &str = "Adobe සහ Microsoft මෘදුකාංග වලදී";
pub const TIP_LINE2: &str = "\"smart quotes\" විශේෂාංගය ක්\u{200D}රියා විරහිත කරන්න";
pub const TIP_BUTTON: &str = "වැඩි විස්තර";
pub const TIP_URL: &str = "https://github.com/sanjaya3d/LIPI#smart-quotes-adobe-and-microsoft-apps";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_safe_web_addresses() {
        for url in [GITHUB_URL, FACEBOOK_URL, YOUTUBE_URL, TIP_URL] {
            assert!(url.starts_with("https://"), "{url}");
            assert!(!url.contains(' ') && url.is_ascii(), "{url}");
        }
    }

    #[test]
    fn each_link_goes_to_the_right_place() {
        assert!(GITHUB_URL.starts_with("https://github.com/sanjaya3d/"));
        assert!(FACEBOOK_URL.starts_with("https://facebook.com/sanjaya3d"));
        assert!(YOUTUBE_URL.starts_with("https://www.youtube.com/@SANJAYA3D"));
    }

    #[test]
    fn the_tip_goes_to_the_readme_section_on_the_project_page() {
        assert!(TIP_URL.starts_with(GITHUB_URL));
        assert!(TIP_URL.ends_with("#smart-quotes-adobe-and-microsoft-apps"));
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
}
