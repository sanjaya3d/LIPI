//! The colours of the window, measured from the reference picture: a near-black
//! background with one lime accent. Nothing else is used in the UI.
//! Windows wants colours as 0x00BBGGRR, so `rgb` puts the bytes in that order.

pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

/// #111315  window background, the knob of the switch, text on lime shapes
pub const BG: u32 = rgb(0x11, 0x13, 0x15);
/// #BEF92C  the lime: outlines, rules, icons, the chosen button, links, key names
pub const ACCENT: u32 = rgb(0xBE, 0xF9, 0x2C);
/// #FFFFFF  main text
pub const TEXT: u32 = rgb(0xFF, 0xFF, 0xFF);
/// #E1E5EC  the lighter grey used for hints and small labels
pub const MUTED: u32 = rgb(0xE1, 0xE5, 0xEC);
/// #4B535A  grey edges (hint panel, key cells) and the switch track when off
pub const EDGE: u32 = rgb(0x4B, 0x53, 0x5A);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_are_the_reference_ones_in_windows_byte_order() {
        assert_eq!(BG, 0x0015_1311); // #111315
        assert_eq!(ACCENT, 0x002C_F9BE); // #BEF92C
        assert_eq!(TEXT, 0x00FF_FFFF); // #FFFFFF
        assert_eq!(MUTED, 0x00EC_E5E1); // #E1E5EC
        assert_eq!(EDGE, 0x005A_534B); // #4B535A
    }

    /// WCAG contrast ratio between two colours. 4.5 is the usual minimum for small text.
    fn contrast(a: u32, b: u32) -> f64 {
        fn channel(v: u32) -> f64 {
            let c = v as f64 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        fn luminance(c: u32) -> f64 {
            0.2126 * channel(c & 0xFF) + 0.7152 * channel((c >> 8) & 0xFF) + 0.0722 * channel((c >> 16) & 0xFF)
        }
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn every_text_colour_is_readable_on_the_surface_it_sits_on() {
        assert!(contrast(TEXT, BG) >= 7.0);
        assert!(contrast(MUTED, BG) >= 7.0);
        assert!(contrast(ACCENT, BG) >= 7.0); // lime text and icons on the background
        assert!(contrast(BG, ACCENT) >= 7.0); // dark text on a lime button
    }

    #[test]
    fn grey_edges_are_visible_but_quieter_than_the_lime() {
        assert!(contrast(EDGE, BG) >= 1.8);
        assert!(contrast(EDGE, BG) < contrast(ACCENT, BG));
    }
}