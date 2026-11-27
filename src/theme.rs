//! The colours of the window, measured from the reference picture: a near-black
//! background with one lime accent. Nothing else is used in the UI.
//! Windows wants colours as 0x00BBGGRR, so `rgb` puts the bytes in that order.

pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

/// #0C0E10  the window background, and the dark text on lime shapes and on the switch knob
pub const BG: u32 = rgb(0x0C, 0x0E, 0x10);
/// #BEF92C  the lime: the chosen button, icons, links, key names, the window outline
pub const ACCENT: u32 = rgb(0xBE, 0xF9, 0x2C);
/// #FFFFFF  main text
pub const TEXT: u32 = rgb(0xFF, 0xFF, 0xFF);
/// #E1E5EC  the lighter grey used for hints and small labels
pub const MUTED: u32 = rgb(0xE1, 0xE5, 0xEC);
/// #9AA3AB  quieter grey for the second line of a card and for small notes
pub const DIM: u32 = rgb(0x9A, 0xA3, 0xAB);
/// #4B535A  grey edges (key cells) and the switch track when off
pub const EDGE: u32 = rgb(0x4B, 0x53, 0x5A);
/// #14171A  the fill of a card
pub const CARD: u32 = rgb(0x14, 0x17, 0x1A);
/// #0F1215  a well cut into a card (the shortcut row, the button that is not chosen)
pub const WELL: u32 = rgb(0x0F, 0x12, 0x15);
/// #1E2226  icon tiles and key chips, a step lighter than the card
pub const CHIP: u32 = rgb(0x1E, 0x22, 0x26);
/// #2B3035  the thin line around cards, wells and chips
pub const LINE: u32 = rgb(0x2B, 0x30, 0x35);
/// #465A1F  lime at about 30 percent on a card: the border of the smart quotes strip
pub const ACCENT_DIM: u32 = rgb(0x46, 0x5A, 0x1F);
/// #1E2613  lime at about 10 percent on the background: the fill of the version badge
pub const ACCENT_TINT: u32 = rgb(0x1E, 0x26, 0x13);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_are_the_reference_ones_in_windows_byte_order() {
        assert_eq!(BG, 0x0010_0E0C); // #0C0E10
        assert_eq!(ACCENT, 0x002C_F9BE); // #BEF92C
        assert_eq!(TEXT, 0x00FF_FFFF); // #FFFFFF
        assert_eq!(MUTED, 0x00EC_E5E1); // #E1E5EC
        assert_eq!(DIM, 0x00AB_A39A); // #9AA3AB
        assert_eq!(EDGE, 0x005A_534B); // #4B535A
        assert_eq!(CARD, 0x001A_1714); // #14171A
        assert_eq!(WELL, 0x0015_120F); // #0F1215
        assert_eq!(CHIP, 0x0026_221E); // #1E2226
        assert_eq!(LINE, 0x0035_302B); // #2B3035
        assert_eq!(ACCENT_DIM, 0x001F_5A46); // #465A1F
        assert_eq!(ACCENT_TINT, 0x0013_261E); // #1E2613
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
        for surface in [BG, CARD, WELL, CHIP] {
            assert!(contrast(TEXT, surface) >= 7.0, "white on {surface:06X}");
            assert!(contrast(MUTED, surface) >= 7.0, "light grey on {surface:06X}");
            assert!(contrast(ACCENT, surface) >= 7.0, "lime on {surface:06X}"); // lime text and icons
            assert!(contrast(DIM, surface) >= 4.5, "quiet grey on {surface:06X}"); // the usual minimum for small text
        }
        assert!(contrast(BG, ACCENT) >= 7.0); // dark text on a lime button
        assert!(contrast(ACCENT, ACCENT_TINT) >= 7.0); // the version badge
    }

    #[test]
    fn grey_edges_are_visible_but_quieter_than_the_lime() {
        assert!(contrast(EDGE, BG) >= 1.8);
        assert!(contrast(EDGE, BG) < contrast(ACCENT, BG));
        // the thin line is only a hint on a card, never as loud as the lime
        assert!(contrast(LINE, CARD) >= 1.1);
        assert!(contrast(LINE, CARD) < contrast(EDGE, CARD));
        assert!(contrast(ACCENT_DIM, CARD) < contrast(ACCENT, CARD));
    }

    #[test]
    fn cards_are_lighter_than_the_window_and_wells_are_darker_than_cards() {
        assert!(contrast(CARD, BG) > 1.0);
        assert!(contrast(CHIP, BG) > contrast(CARD, BG));
        assert!(contrast(WELL, BG) < contrast(CARD, BG));
    }
}
