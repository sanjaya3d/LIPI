//! The name and the contact details shown in the window.
//! Plain text only, no Windows calls, so it is tested on any system.

pub const NAME: &str = "LIPI";
pub const NAME_SINHALA: &str = "ලිපි";
pub const TAGLINE: &str = "Type Singlish in any app and get Sinhala";
pub const DEVELOPER: &str = "SANJAYA3D";

/// What the window shows, and the address the click opens.
pub const FACEBOOK_LABEL: &str = "Contact me on: ";
pub const FACEBOOK_SHOWN: &str = "facebook.com/sanjaya3d";
pub const FACEBOOK_URL: &str = "https://facebook.com/sanjaya3d";

pub const GITHUB_LABEL: &str = "GitHub: ";
pub const GITHUB_SHOWN: &str = "github.com/sanjaya3d/LIPI";
pub const GITHUB_URL: &str = "https://github.com/sanjaya3d/LIPI";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_safe_web_addresses() {
        for (shown, url) in [(FACEBOOK_SHOWN, FACEBOOK_URL), (GITHUB_SHOWN, GITHUB_URL)] {
            assert!(url.starts_with("https://"), "{url}");
            assert!(url.ends_with(shown), "{url} should end with {shown}");
            assert!(!url.contains(' ') && url.is_ascii(), "{url}");
        }
    }

    #[test]
    fn the_name_is_what_was_asked_for() {
        assert_eq!(NAME, "LIPI");
        assert_eq!(NAME_SINHALA, "\u{0DBD}\u{0DD2}\u{0DB4}\u{0DD2}");
    }
}
