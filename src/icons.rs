//! The small icons, described as plain shapes (lines, boxes, circles).
//! The window draws them at any screen size, so no image files are needed.
//! Sizes are for 96 DPI. No Windows calls here, so the shapes are tested on any system.

/// `Main` is the icon colour. `Back` is the colour of the surface under it,
/// used for marks that are cut out of a solid shape (the "i" in a disc).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Main,
    Back,
}

#[derive(Clone, Copy, Debug)]
pub enum Piece {
    /// A line through the points, with round ends. Repeat the first point to close a shape.
    Lines { points: &'static [(i32, i32)], width: i32, ink: Ink },
    /// The outline of a rounded box. The line is drawn inside the box.
    Frame { x: i32, y: i32, w: i32, h: i32, r: i32, width: i32, ink: Ink },
    /// A solid box.
    Block { x: i32, y: i32, w: i32, h: i32, ink: Ink },
    /// A solid box with round corners.
    Panel { x: i32, y: i32, w: i32, h: i32, r: i32, ink: Ink },
    /// A solid circle. `x` and `y` are the top left of the box around it, `d` is the diameter.
    Disc { x: i32, y: i32, d: i32, ink: Ink },
    /// A solid shape with straight sides, through the points (three or more).
    Poly { points: &'static [(i32, i32)], ink: Ink },
}

impl Piece {
    /// The area the piece covers: (left, top, right, bottom), including the line width.
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        match *self {
            Piece::Lines { points, width, .. } => {
                let half = (width + 1) / 2;
                let xs = points.iter().map(|p| p.0);
                let ys = points.iter().map(|p| p.1);
                (
                    xs.clone().min().unwrap_or(0) - half,
                    ys.clone().min().unwrap_or(0) - half,
                    xs.max().unwrap_or(0) + half,
                    ys.max().unwrap_or(0) + half,
                )
            }
            Piece::Frame { x, y, w, h, .. } | Piece::Block { x, y, w, h, .. } | Piece::Panel { x, y, w, h, .. } => {
                (x, y, x + w, y + h)
            }
            Piece::Disc { x, y, d, .. } => (x, y, x + d, y + d),
            Piece::Poly { points, .. } => {
                let xs = points.iter().map(|p| p.0);
                let ys = points.iter().map(|p| p.1);
                (
                    xs.clone().min().unwrap_or(0),
                    ys.clone().min().unwrap_or(0),
                    xs.max().unwrap_or(0),
                    ys.max().unwrap_or(0),
                )
            }
        }
    }
}

pub struct Icon {
    pub w: i32,
    pub h: i32,
    pub pieces: &'static [Piece],
}

use Ink::{Back, Main};

/// Window button: minimise.
pub const MINIMISE: Icon = Icon {
    w: 14,
    h: 14,
    pieces: &[Piece::Lines { points: &[(2, 7), (12, 7)], width: 2, ink: Main }],
};

/// Window button: close.
pub const CLOSE: Icon = Icon {
    w: 14,
    h: 14,
    pieces: &[
        Piece::Lines { points: &[(2, 2), (12, 12)], width: 2, ink: Main },
        Piece::Lines { points: &[(2, 12), (12, 2)], width: 2, ink: Main },
    ],
};

/// Keyboard, for the "Sinhala typing" card.
pub const KEYBOARD: Icon = Icon {
    w: 28,
    h: 20,
    pieces: &[
        Piece::Frame { x: 0, y: 0, w: 28, h: 20, r: 4, width: 2, ink: Main },
        Piece::Block { x: 5, y: 6, w: 2, h: 2, ink: Main },
        Piece::Block { x: 9, y: 6, w: 2, h: 2, ink: Main },
        Piece::Block { x: 13, y: 6, w: 2, h: 2, ink: Main },
        Piece::Block { x: 17, y: 6, w: 2, h: 2, ink: Main },
        Piece::Block { x: 21, y: 6, w: 2, h: 2, ink: Main },
        Piece::Block { x: 7, y: 10, w: 2, h: 2, ink: Main },
        Piece::Block { x: 11, y: 10, w: 2, h: 2, ink: Main },
        Piece::Block { x: 15, y: 10, w: 2, h: 2, ink: Main },
        Piece::Block { x: 19, y: 10, w: 2, h: 2, ink: Main },
        Piece::Block { x: 9, y: 14, w: 10, h: 2, ink: Main },
    ],
};

/// A solid keyboard with the keys cut out of it, for the "Keys" button.
pub const KEYBOARD_SOLID: Icon = Icon {
    w: 24,
    h: 16,
    pieces: &[
        Piece::Panel { x: 0, y: 0, w: 24, h: 16, r: 3, ink: Main },
        Piece::Block { x: 3, y: 3, w: 2, h: 2, ink: Back },
        Piece::Block { x: 7, y: 3, w: 2, h: 2, ink: Back },
        Piece::Block { x: 11, y: 3, w: 2, h: 2, ink: Back },
        Piece::Block { x: 15, y: 3, w: 2, h: 2, ink: Back },
        Piece::Block { x: 19, y: 3, w: 2, h: 2, ink: Back },
        Piece::Block { x: 5, y: 7, w: 2, h: 2, ink: Back },
        Piece::Block { x: 9, y: 7, w: 2, h: 2, ink: Back },
        Piece::Block { x: 13, y: 7, w: 2, h: 2, ink: Back },
        Piece::Block { x: 17, y: 7, w: 2, h: 2, ink: Back },
        Piece::Block { x: 7, y: 11, w: 10, h: 2, ink: Back },
    ],
};

/// A box with an arrow leaving it, for "Output".
pub const OUTPUT: Icon = Icon {
    w: 20,
    h: 20,
    pieces: &[
        Piece::Lines { points: &[(9, 3), (3, 3), (3, 17), (17, 17), (17, 11)], width: 2, ink: Main },
        Piece::Lines { points: &[(9, 11), (18, 2)], width: 2, ink: Main },
        Piece::Lines { points: &[(12, 2), (18, 2), (18, 8)], width: 2, ink: Main },
    ],
};

/// A disc with an "i" cut out of it.
pub const INFO: Icon = Icon {
    w: 24,
    h: 24,
    pieces: &[
        Piece::Disc { x: 0, y: 0, d: 24, ink: Main },
        Piece::Disc { x: 10, y: 5, d: 4, ink: Back },
        Piece::Lines { points: &[(12, 12), (12, 18)], width: 3, ink: Back },
    ],
};

/// A page with a folded corner, for "Show Logs".
pub const DOCUMENT: Icon = Icon {
    w: 18,
    h: 20,
    pieces: &[
        Piece::Lines { points: &[(3, 1), (11, 1), (16, 6), (16, 19), (3, 19), (3, 1)], width: 2, ink: Main },
        Piece::Lines { points: &[(11, 1), (11, 6), (16, 6)], width: 2, ink: Main },
        Piece::Lines { points: &[(6, 11), (13, 11)], width: 2, ink: Main },
        Piece::Lines { points: &[(6, 15), (13, 15)], width: 2, ink: Main },
    ],
};

/// A gear for the Settings button: a disc, eight round teeth around it, and a hole in the middle.
pub const GEAR: Icon = Icon {
    w: 20,
    h: 20,
    pieces: &[
        Piece::Disc { x: 3, y: 3, d: 14, ink: Main },
        // up, down, left, right
        Piece::Lines { points: &[(10, 2), (10, 5)], width: 3, ink: Main },
        Piece::Lines { points: &[(10, 15), (10, 18)], width: 3, ink: Main },
        Piece::Lines { points: &[(2, 10), (5, 10)], width: 3, ink: Main },
        Piece::Lines { points: &[(15, 10), (18, 10)], width: 3, ink: Main },
        // the four corners
        Piece::Lines { points: &[(4, 4), (6, 6)], width: 3, ink: Main },
        Piece::Lines { points: &[(14, 14), (16, 16)], width: 3, ink: Main },
        Piece::Lines { points: &[(16, 4), (14, 6)], width: 3, ink: Main },
        Piece::Lines { points: &[(4, 16), (6, 14)], width: 3, ink: Main },
        Piece::Disc { x: 7, y: 7, d: 6, ink: Back },
    ],
};

/// A small arrow head pointing left, for "Back".
pub const BACK: Icon = Icon {
    w: 10,
    h: 16,
    pieces: &[Piece::Lines { points: &[(8, 2), (2, 8), (8, 14)], width: 2, ink: Main }],
};

/// A cube, beside the developer name.
pub const CUBE: Icon = Icon {
    w: 20,
    h: 20,
    pieces: &[
        Piece::Lines { points: &[(10, 1), (18, 6), (18, 14), (10, 19), (2, 14), (2, 6), (10, 1)], width: 2, ink: Main },
        Piece::Lines { points: &[(2, 6), (10, 11), (18, 6)], width: 2, ink: Main },
        Piece::Lines { points: &[(10, 11), (10, 19)], width: 2, ink: Main },
    ],
};

/// Two links of a chain, for the shortcut row on the main page.
pub const LINK: Icon = Icon {
    w: 20,
    h: 12,
    pieces: &[
        Piece::Frame { x: 0, y: 1, w: 11, h: 10, r: 5, width: 2, ink: Main },
        Piece::Frame { x: 9, y: 1, w: 11, h: 10, r: 5, width: 2, ink: Main },
    ],
};

/// A globe: a circle, a narrow oval standing in it and a line across, for the Unicode button.
pub const GLOBE: Icon = Icon {
    w: 18,
    h: 18,
    pieces: &[
        Piece::Frame { x: 0, y: 0, w: 18, h: 18, r: 9, width: 2, ink: Main },
        Piece::Frame { x: 5, y: 0, w: 8, h: 18, r: 4, width: 2, ink: Main },
        Piece::Lines { points: &[(1, 9), (17, 9)], width: 2, ink: Main },
    ],
};

/// The letter A in a rounded box, for the FM Abhaya button (a font).
pub const LETTER_A: Icon = Icon {
    w: 18,
    h: 18,
    pieces: &[
        Piece::Frame { x: 0, y: 0, w: 18, h: 18, r: 4, width: 2, ink: Main },
        Piece::Lines { points: &[(5, 13), (9, 5), (13, 13)], width: 2, ink: Main },
        Piece::Lines { points: &[(6, 10), (12, 10)], width: 2, ink: Main },
    ],
};

/// A circle with a tick in it, beside the line that says what LIPI is sending.
pub const CHECK_CIRCLE: Icon = Icon {
    w: 18,
    h: 18,
    pieces: &[
        Piece::Frame { x: 0, y: 0, w: 18, h: 18, r: 9, width: 2, ink: Main },
        Piece::Lines { points: &[(5, 9), (8, 12), (13, 6)], width: 2, ink: Main },
    ],
};

/// A power mark: a ring that is open at the top, and a short stem going down into the gap.
/// For "Run on Windows Startup".
pub const POWER: Icon = Icon {
    w: 18,
    h: 18,
    pieces: &[
        Piece::Lines { points: &[(5, 5), (2, 9), (3, 14), (7, 17), (11, 17), (15, 13), (16, 9), (13, 5)], width: 2, ink: Main },
        Piece::Lines { points: &[(9, 1), (9, 9)], width: 2, ink: Main },
    ],
};

/// An arrow pointing down onto a flat line, for "Start Minimized to Tray".
pub const TRAY: Icon = Icon {
    w: 18,
    h: 18,
    pieces: &[
        Piece::Lines { points: &[(9, 2), (9, 12)], width: 2, ink: Main },
        Piece::Lines { points: &[(5, 8), (9, 12), (13, 8)], width: 2, ink: Main },
        Piece::Lines { points: &[(3, 16), (15, 16)], width: 2, ink: Main },
    ],
};

/// An arrow pointing down into an open tray, for "Check for Updates".
pub const DOWNLOAD: Icon = Icon {
    w: 18,
    h: 18,
    pieces: &[
        Piece::Lines { points: &[(9, 2), (9, 11)], width: 2, ink: Main },
        Piece::Lines { points: &[(5, 7), (9, 11), (13, 7)], width: 2, ink: Main },
        Piece::Lines { points: &[(2, 11), (2, 16), (16, 16), (16, 11)], width: 2, ink: Main },
    ],
};

/// The two angle brackets of code, beside the GitHub link in the Settings footer.
pub const CODE: Icon = Icon {
    w: 14,
    h: 10,
    pieces: &[
        Piece::Lines { points: &[(5, 1), (1, 5), (5, 9)], width: 2, ink: Main },
        Piece::Lines { points: &[(9, 1), (13, 5), (9, 9)], width: 2, ink: Main },
    ],
};

/// GitHub: a round head with two ears, and two gaps at the bottom for the legs.
pub const GITHUB: Icon = Icon {
    w: 28,
    h: 28,
    pieces: &[
        Piece::Disc { x: 1, y: 2, d: 26, ink: Main },
        Piece::Poly { points: &[(3, 11), (4, 0), (13, 4)], ink: Main },
        Piece::Poly { points: &[(25, 11), (24, 0), (15, 4)], ink: Main },
        Piece::Lines { points: &[(11, 27), (11, 22)], width: 2, ink: Back },
        Piece::Lines { points: &[(17, 27), (17, 22)], width: 2, ink: Back },
    ],
};

/// Facebook: a disc with an "f" cut out of it.
pub const FACEBOOK: Icon = Icon {
    w: 28,
    h: 28,
    pieces: &[
        Piece::Disc { x: 0, y: 0, d: 28, ink: Main },
        Piece::Lines { points: &[(21, 8), (19, 8), (17, 10), (17, 24)], width: 4, ink: Back },
        Piece::Lines { points: &[(13, 15), (22, 15)], width: 3, ink: Back },
    ],
};

/// YouTube: a rounded box with a play triangle cut out of it.
pub const YOUTUBE: Icon = Icon {
    w: 32,
    h: 22,
    pieces: &[
        Piece::Panel { x: 0, y: 0, w: 32, h: 22, r: 7, ink: Main },
        Piece::Poly { points: &[(13, 6), (13, 16), (22, 11)], ink: Back },
    ],
};

/// A briefcase for the portfolio link: a handle, a solid body, and a seam with a clasp.
pub const PORTFOLIO: Icon = Icon {
    w: 28,
    h: 24,
    pieces: &[
        Piece::Frame { x: 9, y: 0, w: 10, h: 8, r: 3, width: 2, ink: Main },
        Piece::Panel { x: 1, y: 5, w: 26, h: 19, r: 4, ink: Main },
        Piece::Lines { points: &[(3, 14), (25, 14)], width: 2, ink: Back },
        Piece::Panel { x: 12, y: 11, w: 4, h: 6, r: 1, ink: Main },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [(&str, &Icon); 22] = [
        ("gear", &GEAR),
        ("minimise", &MINIMISE),
        ("close", &CLOSE),
        ("keyboard", &KEYBOARD),
        ("keyboard solid", &KEYBOARD_SOLID),
        ("output", &OUTPUT),
        ("info", &INFO),
        ("document", &DOCUMENT),
        ("back", &BACK),
        ("cube", &CUBE),
        ("link", &LINK),
        ("globe", &GLOBE),
        ("letter a", &LETTER_A),
        ("check circle", &CHECK_CIRCLE),
        ("power", &POWER),
        ("tray", &TRAY),
        ("download", &DOWNLOAD),
        ("code", &CODE),
        ("github", &GITHUB),
        ("facebook", &FACEBOOK),
        ("youtube", &YOUTUBE),
        ("portfolio", &PORTFOLIO),
    ];

    #[test]
    fn every_piece_stays_inside_its_icon() {
        for (name, icon) in ALL {
            assert!(!icon.pieces.is_empty(), "{name}");
            for piece in icon.pieces {
                let (l, t, r, b) = piece.bounds();
                assert!(l >= 0 && t >= 0 && r <= icon.w && b <= icon.h, "{name}: {piece:?} sticks out of {}x{}", icon.w, icon.h);
            }
        }
    }

    #[test]
    fn lines_have_two_or_more_points_and_a_visible_width() {
        for (name, icon) in ALL {
            for piece in icon.pieces {
                if let Piece::Lines { points, width, .. } = piece {
                    assert!(points.len() >= 2, "{name}");
                    assert!(*width >= 2, "{name}: thin lines vanish on small icons");
                }
            }
        }
    }

    #[test]
    fn the_solid_keyboard_has_its_keys_cut_out_and_they_stay_on_the_panel() {
        let Piece::Panel { x, y, w, h, .. } = KEYBOARD_SOLID.pieces[0] else {
            panic!("the first piece must be the panel");
        };
        let keys: Vec<_> = KEYBOARD_SOLID.pieces[1..].iter().collect();
        assert_eq!(keys.len(), 10);
        for key in keys {
            assert!(matches!(key, Piece::Block { ink: Ink::Back, .. }), "{key:?}");
            let (l, t, r, b) = key.bounds();
            assert!(l > x + 1 && t > y + 1 && r < x + w - 1 && b < y + h - 1, "{key:?} is too close to the edge");
        }
    }

    #[test]
    fn back_points_left() {
        let Piece::Lines { points, .. } = BACK.pieces[0] else { panic!("expected a line") };
        let tip = points.iter().map(|p| p.0).min().unwrap();
        assert_eq!(points[1].0, tip); // the middle point is the leftmost one
    }

    #[test]
    fn solid_shapes_have_three_or_more_corners() {
        for (name, icon) in ALL {
            for piece in icon.pieces {
                if let Piece::Poly { points, .. } = piece {
                    assert!(points.len() >= 3, "{name}");
                }
            }
        }
    }

    #[test]
    fn the_play_triangle_is_cut_out_and_stays_inside_the_youtube_box() {
        let Piece::Panel { x, y, w, h, .. } = YOUTUBE.pieces[0] else {
            panic!("the first piece must be the box");
        };
        let Piece::Poly { ink, .. } = YOUTUBE.pieces[1] else { panic!("the second piece must be the triangle") };
        assert_eq!(ink, Ink::Back);
        let (l, t, r, b) = YOUTUBE.pieces[1].bounds();
        assert!(l > x + 4 && t > y + 2 && r < x + w - 4 && b < y + h - 2, "the triangle is too close to the edge");
    }

    #[test]
    fn the_f_is_cut_out_of_the_facebook_disc() {
        assert!(matches!(FACEBOOK.pieces[0], Piece::Disc { ink: Ink::Main, .. }));
        assert!(FACEBOOK.pieces[1..].iter().all(|p| matches!(p, Piece::Lines { ink: Ink::Back, .. })));
        assert_eq!(FACEBOOK.pieces.len(), 3); // the disc, the stem and the bar
    }

    #[test]
    fn the_social_icons_are_big_enough_to_read_and_small_enough_for_their_buttons() {
        // The footer buttons are 36 wide and 36 high at 96 DPI.
        for (name, icon) in [("github", &GITHUB), ("facebook", &FACEBOOK), ("youtube", &YOUTUBE), ("portfolio", &PORTFOLIO)] {
            assert!(icon.w >= 24 && icon.h >= 20, "{name} is too small to read");
            assert!(icon.w <= 32 && icon.h <= 32, "{name} does not fit its button");
        }
    }

    #[test]
    fn the_gear_has_eight_teeth_and_a_hole_that_stays_inside_the_disc() {
        let Piece::Disc { x, y, d, ink } = GEAR.pieces[0] else { panic!("the first piece must be the disc") };
        assert_eq!(ink, Ink::Main);
        let teeth = GEAR.pieces.iter().filter(|p| matches!(p, Piece::Lines { ink: Ink::Main, .. })).count();
        assert_eq!(teeth, 8);
        let Some(Piece::Disc { ink: Ink::Back, .. }) = GEAR.pieces.last() else { panic!("the hole must be last") };
        let (l, t, r, b) = GEAR.pieces.last().unwrap().bounds();
        assert!(l > x + 2 && t > y + 2 && r < x + d - 2 && b < y + d - 2, "the hole is too close to the edge");
        // the hole is in the middle of the icon
        assert_eq!((l + r, t + b), (GEAR.w, GEAR.h));
        // the teeth reach out past the disc, so it looks like a gear
        let left = GEAR.pieces.iter().map(|p| p.bounds().0).min().unwrap();
        let right = GEAR.pieces.iter().map(|p| p.bounds().2).max().unwrap();
        assert!(right - left >= d + 2, "no tooth sticks out");
        assert!(left >= 0 && right <= GEAR.w, "the teeth must stay inside the icon box");
    }

    #[test]
    fn the_info_mark_is_cut_out_of_the_disc() {
        let backs = INFO.pieces.iter().filter(|p| matches!(p, Piece::Lines { ink: Ink::Back, .. } | Piece::Disc { ink: Ink::Back, .. })).count();
        assert_eq!(backs, 2); // the dot and the stem
    }

    #[test]
    fn the_round_icons_are_really_round() {
        // A frame whose corner radius is half its width and height is a circle.
        for (name, icon) in [("globe", &GLOBE), ("check circle", &CHECK_CIRCLE)] {
            let Piece::Frame { w, h, r, .. } = icon.pieces[0] else { panic!("{name}: the first piece must be the circle") };
            assert_eq!((w, h, r * 2), (icon.w, icon.h, icon.w), "{name}");
        }
    }

    #[test]
    fn the_chain_links_overlap_a_little_and_fill_the_icon() {
        let Piece::Frame { x: x1, w: w1, .. } = LINK.pieces[0] else { panic!("first link") };
        let Piece::Frame { x: x2, w: w2, .. } = LINK.pieces[1] else { panic!("second link") };
        assert!(x2 < x1 + w1, "the links must overlap");
        assert_eq!((x1, x2 + w2), (0, LINK.w));
    }

    #[test]
    fn the_new_icons_fit_the_slots_they_are_drawn_in() {
        // Buttons on the main page are 58 high, rows are 38 high.
        for (name, icon, max_w) in [("globe", &GLOBE, 20), ("letter A", &LETTER_A, 20), ("check circle", &CHECK_CIRCLE, 20), ("link", &LINK, 24)] {
            assert!(icon.h <= 20, "{name} is too high");
            assert!(icon.w <= max_w, "{name} is too wide");
        }
    }

    #[test]
    fn the_power_ring_is_open_at_the_top_and_the_stem_stands_in_the_gap() {
        let Piece::Lines { points: ring, .. } = POWER.pieces[0] else { panic!("the first piece must be the ring") };
        let Piece::Lines { points: stem, .. } = POWER.pieces[1] else { panic!("the second piece must be the stem") };
        // no point of the ring is at the top middle, and the ring does not close
        assert!(ring.iter().all(|p| !(p.1 <= 3 && (p.0 - 9).abs() <= 3)), "the ring must have a gap at the top");
        assert_ne!(ring.first(), ring.last());
        // the stem is on the middle line, starts at the top and ends in the middle of the ring
        assert!(stem.iter().all(|p| p.0 == 9));
        assert_eq!(stem[0].1.min(stem[1].1), 1);
        assert!(stem[0].1.max(stem[1].1) >= 8);
    }

    #[test]
    fn the_arrows_point_down_and_the_download_tray_is_open_at_the_top() {
        for (name, icon) in [("tray", &TRAY), ("download", &DOWNLOAD)] {
            let Piece::Lines { points: stem, .. } = icon.pieces[0] else { panic!("{name}: the first piece must be the stem") };
            let Piece::Lines { points: head, .. } = icon.pieces[1] else { panic!("{name}: the second piece must be the head") };
            assert_eq!(stem[0].0, stem[1].0, "{name}: the stem is straight");
            assert!(stem[1].1 > stem[0].1, "{name}: the arrow points down");
            // the tip of the head is the lowest point of the head, in the middle, at the end of the stem
            assert_eq!(head[1], stem[1], "{name}");
            assert!(head[1].1 > head[0].1 && head[1].1 > head[2].1, "{name}");
        }
        let Piece::Lines { points: tray, .. } = DOWNLOAD.pieces[2] else { panic!("the tray") };
        assert!(tray[0].1 < tray[1].1 && tray[3].1 < tray[2].1, "the sides of the tray go up");
    }

    #[test]
    fn the_code_brackets_point_away_from_each_other() {
        let Piece::Lines { points: left, .. } = CODE.pieces[0] else { panic!("left bracket") };
        let Piece::Lines { points: right, .. } = CODE.pieces[1] else { panic!("right bracket") };
        assert!(left[1].0 < left[0].0, "the left one points left");
        assert!(right[1].0 > right[0].0, "the right one points right");
        assert!(left[0].0 < right[0].0);
    }

    #[test]
    fn the_briefcase_has_its_handle_on_top_and_the_seam_and_clasp_on_the_body() {
        let Piece::Frame { x: hx, y: hy, w: hw, .. } = PORTFOLIO.pieces[0] else { panic!("the first piece must be the handle") };
        let Piece::Panel { x, y, w, h, .. } = PORTFOLIO.pieces[1] else { panic!("the second piece must be the body") };
        let Piece::Lines { points: seam, ink: seam_ink, .. } = PORTFOLIO.pieces[2] else { panic!("the third piece must be the seam") };
        let Piece::Panel { ink: clasp_ink, .. } = PORTFOLIO.pieces[3] else { panic!("the last piece must be the clasp") };
        assert!(hy < y, "the handle starts above the body");
        assert!(hx > x && hx + hw < x + w, "the handle is in the middle of the body, not at its edges");
        assert_eq!(seam_ink, Ink::Back, "the seam is cut out of the body");
        assert_eq!(seam[0].1, seam[1].1, "the seam is a straight line across");
        assert!(seam[0].1 > y + 2 && seam[0].1 < y + h - 2, "the seam is inside the body");
        assert!(seam[0].0 > x + 1 && seam[1].0 < x + w - 1, "the seam stops short of the edges");
        assert_eq!(clasp_ink, Ink::Main, "the clasp is solid, over the seam");
        let (l, t, r, b) = PORTFOLIO.pieces[3].bounds();
        assert!(t < seam[0].1 && b > seam[0].1, "the clasp crosses the seam");
        assert_eq!(l + r, 2 * 14, "the clasp is in the middle");
    }
}
