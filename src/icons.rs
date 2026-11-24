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

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [(&str, &Icon); 13] = [
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
        ("github", &GITHUB),
        ("facebook", &FACEBOOK),
        ("youtube", &YOUTUBE),
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
        for (name, icon) in [("github", &GITHUB), ("facebook", &FACEBOOK), ("youtube", &YOUTUBE)] {
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
}
