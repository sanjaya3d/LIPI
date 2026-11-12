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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_safe_web_addresses() {
        for url in [GITHUB_URL, FACEBOOK_URL, YOUTUBE_URL] {
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
    fn the_name_is_what_was_asked_for() {
        assert_eq!(NAME, "LIPI");
        assert_eq!(NAME_SINHALA, "\u{0DBD}\u{0DD2}\u{0DB4}\u{0DD2}");
    }
}
