//! Where everything sits in the main window.
//! Sizes are written for 96 DPI and scaled to the real screen DPI.
//! No Windows calls in here, so it is tested on any system.

/// Same memory layout as the Windows RECT, so it can be passed to Windows directly.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left < other.right
            && other.left < self.right
            && self.top < other.bottom
            && other.top < self.bottom
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// Size of the window at 96 DPI. The window has no Windows title bar, so this is all of it.
pub const WIDTH: i32 = 400;
pub const HEIGHT: i32 = 488;

/// The window has two pages. The Keys button opens the second one, Back returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Main,
    Keys,
}

/// How many tabs the key hint page has.
pub const HINT_TABS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    None,
    /// The title bar. Dragging here moves the window.
    Caption,
    Minimise,
    Close,
    TypingCard,
    UnicodeButton,
    LegacyButton,
    LogsButton,
    KeysButton,
    /// The three round icon buttons in the footer.
    GitHub,
    Facebook,
    YouTube,
    /// Key hint page: back to the main page.
    Back,
    /// Key hint page: one of the tabs, counted from 0.
    Tab(u8),
}

impl Hit {
    /// True where the mouse pointer should turn into a hand.
    pub fn clickable(self) -> bool {
        !matches!(self, Hit::None | Hit::Caption)
    }
}

pub struct Layout {
    pub dpi: i32,
    pub window: Rect,
    // title bar
    pub title_bar: Rect,
    pub title_rule: Rect,
    pub title_logo: Rect,
    pub title_text: Rect,
    pub minimise_button: Rect,
    pub close_button: Rect,
    // header
    pub logo_tile: Rect,
    pub logo_divider: Rect,
    pub name: Rect,
    pub tagline: Rect,
    pub rule: Rect,
    // Sinhala typing card
    pub typing_card: Rect,
    pub typing_icon: Rect,
    pub typing_label: Rect,
    pub typing_state: Rect,
    pub switch_track: Rect,
    // output
    pub output_icon: Rect,
    pub output_label: Rect,
    pub unicode_button: Rect,
    pub legacy_button: Rect,
    // key hints
    pub info_panel: Rect,
    pub info_icon: Rect,
    pub info_line1: Rect,
    pub info_line2: Rect,
    // logs, keys and footer
    pub logs_button: Rect,
    pub keys_button: Rect,
    pub dev_row: Rect,
    pub github_button: Rect,
    pub facebook_button: Rect,
    pub youtube_button: Rect,
    // key hint page
    pub back_button: Rect,
    pub hint_title: Rect,
    pub hint_grid: Rect,
    pub hint_note: Rect,
}

impl Layout {
    pub fn new(dpi: i32) -> Self {
        let dpi = dpi.max(96);
        let r = |l: i32, t: i32, rt: i32, b: i32| Rect {
            left: l * dpi / 96,
            top: t * dpi / 96,
            right: rt * dpi / 96,
            bottom: b * dpi / 96,
        };
        Self {
            dpi,
            window: r(0, 0, WIDTH, HEIGHT),
            title_bar: r(0, 0, 400, 40),
            title_rule: r(1, 40, 399, 42),
            title_logo: r(14, 9, 36, 31),
            title_text: r(44, 0, 240, 40),
            minimise_button: r(308, 0, 354, 40),
            close_button: r(354, 0, 400, 40),
            logo_tile: r(24, 56, 76, 108),
            logo_divider: r(88, 58, 90, 106),
            name: r(102, 50, 384, 92),
            tagline: r(102, 92, 384, 114),
            rule: r(24, 124, 376, 126),
            typing_card: r(24, 136, 376, 192),
            typing_icon: r(42, 154, 70, 174),
            typing_label: r(86, 136, 232, 192),
            typing_state: r(232, 136, 296, 192),
            switch_track: r(308, 150, 360, 178),
            output_icon: r(26, 206, 46, 226),
            output_label: r(54, 202, 220, 230),
            unicode_button: r(24, 236, 192, 272),
            legacy_button: r(208, 236, 376, 272),
            info_panel: r(24, 286, 376, 340),
            info_icon: r(40, 301, 64, 325),
            info_line1: r(80, 290, 372, 313),
            info_line2: r(80, 313, 372, 336),
            logs_button: r(24, 354, 192, 392),
            keys_button: r(208, 354, 376, 392),
            dev_row: r(24, 406, 376, 430),
            github_button: r(138, 436, 174, 472),
            facebook_button: r(182, 436, 218, 472),
            youtube_button: r(226, 436, 262, 472),
            back_button: r(24, 52, 112, 88),
            hint_title: r(126, 52, 376, 88),
            hint_grid: r(24, 142, 376, 440),
            hint_note: r(24, 444, 376, 482),
        }
    }

    /// Tab `i` of the key hint page (0 to 3). Four tabs of 82 side by side, 8 apart.
    pub fn hint_tab(&self, i: usize) -> Rect {
        let left = 24 + 90 * i as i32;
        Rect {
            left: self.scale(left),
            top: self.scale(100),
            right: self.scale(left + 82),
            bottom: self.scale(132),
        }
    }

    /// Cell number `index` (counted along the rows) of a grid with `columns` and `rows`,
    /// laid out inside `hint_grid`. Cells never get taller than 44 at 96 DPI.
    pub fn hint_cell(&self, columns: usize, rows: usize, index: usize) -> Rect {
        let grid = self.hint_grid;
        let gap = self.scale(5);
        let columns = columns.max(1) as i32;
        let rows = rows.max(1) as i32;
        let cell_w = (grid.width() - gap * (columns - 1)) / columns;
        let cell_h = ((grid.height() - gap * (rows - 1)) / rows).min(self.scale(44));
        let (col, row) = (index as i32 % columns, index as i32 / columns);
        let left = grid.left + col * (cell_w + gap);
        let top = grid.top + row * (cell_h + gap);
        Rect { left, top, right: left + cell_w, bottom: top + cell_h }
    }

    /// Where the note goes: straight under the last row of a grid with `columns` and `rows`,
    /// but never lower than `hint_note`.
    pub fn hint_note_at(&self, columns: usize, rows: usize) -> Rect {
        let rows = rows.max(1);
        let last_row = self.hint_cell(columns, rows, (rows - 1) * columns.max(1));
        let top = (last_row.bottom + self.scale(8)).min(self.hint_note.top);
        Rect { left: self.hint_note.left, top, right: self.hint_note.right, bottom: top + self.hint_note.height() }
    }

    /// Scales a size written for 96 DPI.
    pub fn scale(&self, v: i32) -> i32 {
        v * self.dpi / 96
    }

    /// Size of the window on this screen.
    pub fn client_size(&self) -> (i32, i32) {
        (self.scale(WIDTH), self.scale(HEIGHT))
    }

    /// What is under the point (x, y), in window pixels, on the given page.
    pub fn hit(&self, view: View, x: i32, y: i32) -> Hit {
        // The title bar is the same on both pages.
        if self.minimise_button.contains(x, y) {
            return Hit::Minimise;
        }
        if self.close_button.contains(x, y) {
            return Hit::Close;
        }
        if self.title_bar.contains(x, y) {
            return Hit::Caption;
        }
        match view {
            View::Main => {
                if self.typing_card.contains(x, y) {
                    Hit::TypingCard
                } else if self.unicode_button.contains(x, y) {
                    Hit::UnicodeButton
                } else if self.legacy_button.contains(x, y) {
                    Hit::LegacyButton
                } else if self.logs_button.contains(x, y) {
                    Hit::LogsButton
                } else if self.keys_button.contains(x, y) {
                    Hit::KeysButton
                } else if self.github_button.contains(x, y) {
                    Hit::GitHub
                } else if self.facebook_button.contains(x, y) {
                    Hit::Facebook
                } else if self.youtube_button.contains(x, y) {
                    Hit::YouTube
                } else {
                    Hit::None
                }
            }
            View::Keys => {
                if self.back_button.contains(x, y) {
                    return Hit::Back;
                }
                for i in 0..HINT_TABS {
                    if self.hint_tab(i).contains(x, y) {
                        return Hit::Tab(i as u8);
                    }
                }
                Hit::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn centre(r: &Rect) -> (i32, i32) {
        ((r.left + r.right) / 2, (r.top + r.bottom) / 2)
    }

    #[test]
    fn hit_testing_finds_each_control_on_the_main_page() {
        for dpi in [96, 120, 144, 192] {
            let l = Layout::new(dpi);
            let cases = [
                (l.minimise_button, Hit::Minimise),
                (l.close_button, Hit::Close),
                (l.title_text, Hit::Caption),
                (l.title_logo, Hit::Caption),
                (l.typing_card, Hit::TypingCard),
                (l.switch_track, Hit::TypingCard),
                (l.unicode_button, Hit::UnicodeButton),
                (l.legacy_button, Hit::LegacyButton),
                (l.logs_button, Hit::LogsButton),
                (l.keys_button, Hit::KeysButton),
                (l.github_button, Hit::GitHub),
                (l.facebook_button, Hit::Facebook),
                (l.youtube_button, Hit::YouTube),
                (l.info_panel, Hit::None),
                (l.logo_tile, Hit::None),
                (l.dev_row, Hit::None),
                (l.back_button, Hit::None), // belongs to the other page
            ];
            for (rect, expected) in cases {
                let (x, y) = centre(&rect);
                assert_eq!(l.hit(View::Main, x, y), expected, "{rect:?} at dpi {dpi}");
            }
            assert_eq!(l.hit(View::Main, -5, -5), Hit::None);
        }
    }

    #[test]
    fn hit_testing_finds_each_control_on_the_key_hint_page() {
        for dpi in [96, 120, 144, 192] {
            let l = Layout::new(dpi);
            let mut cases = vec![
                (l.minimise_button, Hit::Minimise),
                (l.close_button, Hit::Close),
                (l.title_text, Hit::Caption),
                (l.back_button, Hit::Back),
                (l.hint_title, Hit::None),
                (l.hint_grid, Hit::None),
                (l.hint_note, Hit::None),
                (l.logs_button, Hit::None), // belongs to the other page
                (l.keys_button, Hit::None),
                (l.typing_card, Hit::None),
            ];
            for i in 0..HINT_TABS {
                cases.push((l.hint_tab(i), Hit::Tab(i as u8)));
            }
            for (rect, expected) in cases {
                let (x, y) = centre(&rect);
                assert_eq!(l.hit(View::Keys, x, y), expected, "{rect:?} at dpi {dpi}");
            }
        }
    }

    #[test]
    fn the_pointer_is_a_hand_only_over_things_that_click() {
        assert!(Hit::Close.clickable());
        assert!(Hit::GitHub.clickable());
        assert!(Hit::Facebook.clickable());
        assert!(Hit::YouTube.clickable());
        assert!(Hit::TypingCard.clickable());
        assert!(Hit::KeysButton.clickable());
        assert!(Hit::Back.clickable());
        assert!(Hit::Tab(2).clickable());
        assert!(!Hit::Caption.clickable());
        assert!(!Hit::None.clickable());
    }

    fn check_blocks(blocks: &[Rect], dpi: i32) {
        let l = Layout::new(dpi);
        let (w, h) = l.client_size();
        for (i, a) in blocks.iter().enumerate() {
            assert!(a.left >= 0 && a.top >= 0 && a.right <= w && a.bottom <= h, "{a:?} at dpi {dpi}");
            for b in &blocks[i + 1..] {
                assert!(!a.overlaps(b), "{a:?} overlaps {b:?} at dpi {dpi}");
            }
        }
    }

    #[test]
    fn blocks_of_the_main_page_do_not_overlap_and_fit_inside_the_window() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            check_blocks(
                &[
                    l.title_bar,
                    l.title_rule,
                    l.logo_tile,
                    l.logo_divider,
                    l.name,
                    l.tagline,
                    l.rule,
                    l.typing_card,
                    l.output_icon,
                    l.output_label,
                    l.unicode_button,
                    l.legacy_button,
                    l.info_panel,
                    l.logs_button,
                    l.keys_button,
                    l.dev_row,
                    l.github_button,
                    l.facebook_button,
                    l.youtube_button,
                ],
                dpi,
            );
        }
    }

    #[test]
    fn blocks_of_the_key_hint_page_do_not_overlap_and_fit_inside_the_window() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let mut blocks = vec![l.title_bar, l.title_rule, l.back_button, l.hint_title, l.hint_grid, l.hint_note];
            blocks.extend((0..HINT_TABS).map(|i| l.hint_tab(i)));
            check_blocks(&blocks, dpi);
            // Leave room for the 2 pixel window outline at the bottom.
            assert!(l.hint_note.bottom <= l.client_size().1 - l.scale(2), "dpi {dpi}");
        }
    }

    #[test]
    fn the_two_bottom_buttons_are_equal_and_line_up_with_the_cards() {
        let l = Layout::new(96);
        assert_eq!(l.logs_button.width(), l.keys_button.width());
        assert_eq!((l.logs_button.top, l.logs_button.bottom), (l.keys_button.top, l.keys_button.bottom));
        assert_eq!(l.logs_button.left, l.typing_card.left);
        assert_eq!(l.keys_button.right, l.typing_card.right);
        assert!(l.logs_button.right < l.keys_button.left);
    }

    #[test]
    fn the_three_footer_buttons_are_equal_evenly_spaced_and_centred_under_the_developer_row() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let buttons = [l.github_button, l.facebook_button, l.youtube_button];
            for b in buttons {
                assert_eq!((b.width(), b.height()), (buttons[0].width(), buttons[0].height()), "dpi {dpi}");
                assert_eq!(b.top, buttons[0].top, "dpi {dpi}");
                assert!(b.top > l.dev_row.bottom, "dpi {dpi}: the buttons must sit under the developer row");
                assert!(b.width() >= l.scale(32), "dpi {dpi}: too small to click");
            }
            let gap1 = buttons[1].left - buttons[0].right;
            let gap2 = buttons[2].left - buttons[1].right;
            assert!((gap1 - gap2).abs() <= 1, "dpi {dpi}: gaps {gap1} and {gap2}");
            assert!(gap1 > 0, "dpi {dpi}: the buttons touch");
            // The group is in the middle of the window.
            let left_space = buttons[0].left;
            let right_space = l.window.right - buttons[2].right;
            assert!((left_space - right_space).abs() <= 2, "dpi {dpi}: {left_space} against {right_space}");
            // Room left under them for the window outline.
            assert!(buttons[0].bottom <= l.window.bottom - l.scale(2), "dpi {dpi}");
        }
    }

    #[test]
    fn the_four_tabs_span_the_grid_exactly() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            assert_eq!(l.hint_tab(0).left, l.hint_grid.left, "dpi {dpi}");
            assert_eq!(l.hint_tab(HINT_TABS - 1).right, l.hint_grid.right, "dpi {dpi}");
            for i in 1..HINT_TABS {
                assert!(l.hint_tab(i - 1).right < l.hint_tab(i).left, "dpi {dpi}");
            }
        }
    }

    #[test]
    fn grid_cells_stay_inside_the_grid_and_never_touch() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            for (columns, rows) in [(2, 7), (4, 4), (5, 8), (1, 1)] {
                let cells: Vec<Rect> = (0..columns * rows).map(|i| l.hint_cell(columns, rows, i)).collect();
                for (i, a) in cells.iter().enumerate() {
                    assert!(
                        a.left >= l.hint_grid.left
                            && a.right <= l.hint_grid.right
                            && a.top >= l.hint_grid.top
                            && a.bottom <= l.hint_grid.bottom,
                        "cell {i} of {columns}x{rows} sticks out at dpi {dpi}: {a:?}"
                    );
                    assert!(a.height() >= l.scale(30), "cell {i} of {columns}x{rows} is too low at dpi {dpi}");
                    assert!(a.height() <= l.scale(44), "dpi {dpi}");
                    for b in &cells[i + 1..] {
                        assert!(!a.overlaps(b), "{a:?} overlaps {b:?} in {columns}x{rows} at dpi {dpi}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_note_sits_under_the_grid_and_inside_the_window() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let (_, window_height) = l.client_size();
            for (columns, rows) in [(2, 7), (4, 4), (5, 8)] {
                let note = l.hint_note_at(columns, rows);
                let last = l.hint_cell(columns, rows, columns * rows - 1);
                assert!(note.top >= last.bottom, "{columns}x{rows} at dpi {dpi}: note touches the grid");
                assert!(note.top <= l.hint_note.top, "{columns}x{rows} at dpi {dpi}");
                assert!(note.bottom <= window_height - l.scale(2), "{columns}x{rows} at dpi {dpi}");
                assert_eq!(note.height(), l.hint_note.height());
            }
        }
    }

    #[test]
    fn inner_parts_stay_inside_their_parents() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let inside = |inner: &Rect, outer: &Rect| {
                inner.left >= outer.left && inner.right <= outer.right && inner.top >= outer.top && inner.bottom <= outer.bottom
            };
            for part in [l.title_logo, l.title_text, l.minimise_button, l.close_button] {
                assert!(inside(&part, &l.title_bar), "{part:?} at dpi {dpi}");
            }
            for part in [l.typing_icon, l.typing_label, l.typing_state, l.switch_track] {
                assert!(inside(&part, &l.typing_card), "{part:?} at dpi {dpi}");
            }
            for part in [l.info_icon, l.info_line1, l.info_line2] {
                assert!(inside(&part, &l.info_panel), "{part:?} at dpi {dpi}");
            }
            for i in 0..HINT_TABS {
                assert!(inside(&l.hint_tab(i), &l.window), "tab {i} at dpi {dpi}");
            }
            assert!(inside(&l.title_bar, &l.window));
            for button in [l.github_button, l.facebook_button, l.youtube_button] {
                assert!(inside(&button, &l.window), "{button:?} at dpi {dpi}");
            }
        }
    }

    #[test]
    fn small_parts_are_centred_on_their_card() {
        let l = Layout::new(96);
        // the icon, the label and the switch all sit on the middle line of their card
        for (part, card) in [
            (l.typing_icon, l.typing_card),
            (l.switch_track, l.typing_card),
            (l.info_icon, l.info_panel),
        ] {
            let (_, y) = centre(&part);
            let (_, cy) = centre(&card);
            assert!((y - cy).abs() <= 1, "{part:?} is off the middle of {card:?}");
        }
    }

    #[test]
    fn sizes_scale_with_dpi() {
        let a = Layout::new(96);
        let b = Layout::new(192);
        assert_eq!(b.typing_card.right, a.typing_card.right * 2);
        assert_eq!(b.client_size(), (WIDTH * 2, HEIGHT * 2));
        assert_eq!(Layout::new(50).dpi, 96); // never smaller than 96
    }

    #[test]
    fn edges_of_a_rect() {
        let r = Rect { left: 10, top: 10, right: 20, bottom: 20 };
        assert!(r.contains(10, 10));
        assert!(r.contains(19, 19));
        assert!(!r.contains(20, 20));
        assert!(!r.contains(9, 15));
        assert_eq!((r.width(), r.height()), (10, 10));
    }
}
