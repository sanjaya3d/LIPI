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
pub const HEIGHT: i32 = 574;

/// The window has three pages. The Keys button opens the key hints, the gear in the
/// title bar opens Settings, and Back returns to the main page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Main,
    Keys,
    Settings,
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
    /// The "more details" button on the smart quotes card. It opens the GitHub page.
    TipButton,
    /// The three round icon buttons in the footer. The Settings page has a GitHub link too.
    GitHub,
    Facebook,
    YouTube,
    /// The fourth round button: the portfolio website.
    Portfolio,
    /// Key hint page and Settings page: the arrow in the title bar goes back to the main page.
    Back,
    /// Key hint page: one of the tabs, counted from 0.
    Tab(u8),
    /// The gear in the title bar. It is there on every page.
    Settings,
    /// Settings page: the "Run on startup" card.
    StartupCard,
    /// Settings page: the "Start hidden in tray" card.
    TrayCard,
    /// Settings page: "Check for updates".
    UpdateButton,
}

impl Hit {
    /// True where the mouse pointer should turn into a hand.
    pub fn clickable(self) -> bool {
        !matches!(self, Hit::None | Hit::Caption)
    }
}

/// One of the two switch cards on the Settings page. A tile with an icon, a name, a short note
/// and the switch on top, a thin line, and under it one detail row: a tick, a line of text and a status word.
#[derive(Clone, Copy, Debug)]
pub struct SettingCard {
    pub card: Rect,
    pub tile: Rect,
    pub title: Rect,
    pub note: Rect,
    /// The switch. It is the same size, and at the same distance from the edge, as the one on the main page.
    pub control: Rect,
    pub rule: Rect,
    pub detail_icon: Rect,
    pub detail_text: Rect,
    /// The right end of the detail row: a status word such as "Active".
    pub detail_state: Rect,
}

/// The last card on the Settings page: the same top part, with the version in a chip
/// instead of a switch, and the "Check for updates" button across the bottom.
#[derive(Clone, Copy, Debug)]
pub struct UpdateCard {
    pub card: Rect,
    pub tile: Rect,
    pub title: Rect,
    pub note: Rect,
    pub chip: Rect,
    pub rule: Rect,
    pub button: Rect,
}

pub struct Layout {
    pub dpi: i32,
    pub window: Rect,
    // title bar
    pub title_bar: Rect,
    pub title_rule: Rect,
    pub title_logo: Rect,
    pub title_text: Rect,
    /// The small "v1.2" badge after the name.
    pub title_badge: Rect,
    pub settings_button: Rect,
    pub minimise_button: Rect,
    pub close_button: Rect,
    // Sinhala Typing card: the tile, the name, a Sinhala line, ON or OFF and the switch,
    // then a row for the shortcut key
    pub typing_card: Rect,
    /// The top part of the card. A click here turns Sinhala typing on or off.
    pub typing_top: Rect,
    pub typing_icon: Rect,
    pub typing_label: Rect,
    pub typing_sub: Rect,
    pub typing_state: Rect,
    pub switch_track: Rect,
    pub shortcut_row: Rect,
    pub shortcut_icon: Rect,
    pub shortcut_label: Rect,
    // Target Encoding card: a heading row, two buttons, and what LIPI is sending
    pub encoding_card: Rect,
    pub encoding_icon: Rect,
    pub encoding_label: Rect,
    pub encoding_row: Rect,
    pub unicode_button: Rect,
    pub legacy_button: Rect,
    pub encoding_status_icon: Rect,
    pub encoding_status: Rect,
    // smart quotes strip, under the Target Encoding card
    pub tip_card: Rect,
    pub tip_text: Rect,
    pub tip_button: Rect,
    // logs, keys and footer
    pub logs_button: Rect,
    pub keys_button: Rect,
    pub footer_rule: Rect,
    pub dev_row: Rect,
    pub github_button: Rect,
    pub facebook_button: Rect,
    pub youtube_button: Rect,
    pub portfolio_button: Rect,
    // key hint page: the heading is `page_title` and `page_tag`, then the tabs, the grid and a note
    pub hint_grid: Rect,
    pub hint_note: Rect,
    // title bar of the Settings page and the key hint page: a back arrow, the logo, the name and the name of the page
    pub title_back: Rect,
    pub crumb_logo: Rect,
    pub crumb_text: Rect,
    pub crumb_page: Rect,
    // heading of the Settings page and the key hint page (a name with a tag on the right).
    // settings page: a line of text under it, three cards,
    // a bar that says changes are saved, and a footer
    pub page_title: Rect,
    pub page_tag: Rect,
    pub page_lead: Rect,
    pub startup: SettingCard,
    pub tray: SettingCard,
    pub update: UpdateCard,
    pub saved_bar: Rect,
    pub saved_dot: Rect,
    pub saved_text: Rect,
    pub saved_path: Rect,
    pub footer_dot: Rect,
    pub footer_text: Rect,
    pub footer_github: Rect,
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
            title_rule: r(1, 40, 399, 41),
            title_logo: r(14, 9, 36, 31),
            title_text: r(44, 0, 86, 40),
            title_badge: r(90, 10, 132, 30),
            settings_button: r(262, 0, 308, 40),
            minimise_button: r(308, 0, 354, 40),
            close_button: r(354, 0, 400, 40),
            // Sinhala Typing card: 120 tall. Tile and switch share the middle line (y 86).
            typing_card: r(24, 52, 376, 172),
            typing_top: r(24, 52, 376, 114),
            typing_icon: r(38, 66, 78, 106),
            typing_label: r(90, 64, 268, 86),
            typing_sub: r(90, 86, 268, 108),
            typing_state: r(270, 66, 306, 106),
            switch_track: r(310, 72, 362, 100),
            shortcut_row: r(38, 120, 362, 158),
            shortcut_icon: r(46, 129, 68, 149),
            shortcut_label: r(74, 120, 214, 158),
            // Target Encoding card: 168 tall, 10 under the typing card.
            encoding_card: r(24, 182, 376, 350),
            encoding_icon: r(38, 196, 58, 216),
            encoding_label: r(66, 192, 200, 220),
            encoding_row: r(38, 192, 362, 220),
            unicode_button: r(38, 232, 196, 290),
            legacy_button: r(204, 232, 362, 290),
            encoding_status_icon: r(38, 302, 56, 320),
            encoding_status: r(64, 300, 362, 338),
            // The smart quotes strip is 76 tall with 10 above it: room for three lines of Sinhala, 20 each.
            tip_card: r(24, 360, 376, 436),
            tip_text: r(38, 368, 262, 428),
            tip_button: r(268, 384, 362, 412),
            logs_button: r(24, 446, 192, 484),
            keys_button: r(208, 446, 376, 484),
            footer_rule: r(1, 494, 399, 495),
            dev_row: r(24, 502, 376, 522),
            github_button: r(116, 530, 152, 566),
            facebook_button: r(160, 530, 196, 566),
            youtube_button: r(204, 530, 240, 566),
            portfolio_button: r(248, 530, 284, 566),
            // The key hint page: the heading, four tabs (see `hint_tab`), then the grid and the note.
            // Cells are never taller than 44, so the grid has room for 8 rows.
            hint_grid: r(24, 126, 376, 526),
            hint_note: r(24, 530, 376, 568),
            // The Settings page has its own title bar: a back arrow, the logo, the name and the page name.
            title_back: r(6, 0, 46, 40),
            crumb_logo: r(50, 9, 72, 31),
            crumb_text: r(80, 0, 122, 40),
            crumb_page: r(122, 0, 258, 40),
            // Settings page. A heading, a line of text, then three cards 10 apart (106, 106 and 128 tall).
            page_title: r(24, 52, 260, 76),
            page_tag: r(260, 52, 376, 76),
            page_lead: r(24, 80, 376, 102),
            startup: Self::setting_card(dpi, 112),
            tray: Self::setting_card(dpi, 228),
            update: Self::update_card(dpi, 344),
            saved_bar: r(24, 484, 376, 520),
            saved_dot: r(38, 498, 46, 506),
            saved_text: r(54, 484, 236, 520),
            saved_path: r(238, 484, 364, 520),
            footer_dot: r(26, 543, 32, 549),
            footer_text: r(40, 534, 300, 558),
            footer_github: r(308, 534, 376, 558),
        }
    }

    /// A switch card on the Settings page whose top edge is at `top` (96 DPI). 106 tall.
    fn setting_card(dpi: i32, top: i32) -> SettingCard {
        let r = |l: i32, t: i32, rt: i32, b: i32| Rect {
            left: l * dpi / 96,
            top: t * dpi / 96,
            right: rt * dpi / 96,
            bottom: b * dpi / 96,
        };
        SettingCard {
            card: r(24, top, 376, top + 106),
            tile: r(38, top + 14, 78, top + 54),
            title: r(90, top + 12, 296, top + 34),
            note: r(90, top + 34, 296, top + 68),
            control: r(310, top + 20, 362, top + 48),
            rule: r(38, top + 72, 362, top + 73),
            detail_icon: r(38, top + 80, 56, top + 98),
            detail_text: r(64, top + 78, 240, top + 100),
            detail_state: r(244, top + 78, 362, top + 100),
        }
    }

    /// The update card whose top edge is at `top` (96 DPI). 128 tall.
    fn update_card(dpi: i32, top: i32) -> UpdateCard {
        let r = |l: i32, t: i32, rt: i32, b: i32| Rect {
            left: l * dpi / 96,
            top: t * dpi / 96,
            right: rt * dpi / 96,
            bottom: b * dpi / 96,
        };
        UpdateCard {
            card: r(24, top, 376, top + 128),
            tile: r(38, top + 14, 78, top + 54),
            title: r(90, top + 12, 292, top + 34),
            note: r(90, top + 34, 292, top + 68),
            chip: r(298, top + 23, 362, top + 45),
            rule: r(38, top + 72, 362, top + 73),
            button: r(38, top + 82, 362, top + 114),
        }
    }

    /// Tab `i` of the key hint page (0 to 3). Four tabs of 82 side by side, 8 apart, under the heading.
    pub fn hint_tab(&self, i: usize) -> Rect {
        let left = 24 + 90 * i as i32;
        Rect {
            left: self.scale(left),
            top: self.scale(84),
            right: self.scale(left + 82),
            bottom: self.scale(116),
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
        if self.settings_button.contains(x, y) {
            return Hit::Settings;
        }
        // The Settings page and the key hint page have a back arrow in the title bar.
        if view != View::Main && self.title_back.contains(x, y) {
            return Hit::Back;
        }
        if self.title_bar.contains(x, y) {
            return Hit::Caption;
        }
        match view {
            View::Main => {
                if self.typing_top.contains(x, y) {
                    Hit::TypingCard
                } else if self.unicode_button.contains(x, y) {
                    Hit::UnicodeButton
                } else if self.legacy_button.contains(x, y) {
                    Hit::LegacyButton
                } else if self.tip_button.contains(x, y) {
                    Hit::TipButton
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
                } else if self.portfolio_button.contains(x, y) {
                    Hit::Portfolio
                } else {
                    Hit::None
                }
            }
            View::Keys => {
                for i in 0..HINT_TABS {
                    if self.hint_tab(i).contains(x, y) {
                        return Hit::Tab(i as u8);
                    }
                }
                Hit::None
            }
            View::Settings => {
                if self.startup.card.contains(x, y) {
                    Hit::StartupCard
                } else if self.tray.card.contains(x, y) {
                    Hit::TrayCard
                } else if self.update.button.contains(x, y) {
                    Hit::UpdateButton
                } else if self.footer_github.contains(x, y) {
                    Hit::GitHub
                } else {
                    Hit::None
                }
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
                (l.settings_button, Hit::Settings),
                (l.title_text, Hit::Caption),
                (l.title_logo, Hit::Caption),
                (l.title_badge, Hit::Caption),
                (l.typing_top, Hit::TypingCard),
                (l.typing_icon, Hit::TypingCard),
                (l.switch_track, Hit::TypingCard),
                (l.shortcut_row, Hit::None), // only the top of the card turns typing on and off
                (l.encoding_card, Hit::None),
                (l.encoding_row, Hit::None),
                (l.encoding_status, Hit::None),
                (l.unicode_button, Hit::UnicodeButton),
                (l.legacy_button, Hit::LegacyButton),
                (l.tip_button, Hit::TipButton),
                (l.tip_card, Hit::None),
                (l.tip_text, Hit::None),
                (l.logs_button, Hit::LogsButton),
                (l.keys_button, Hit::KeysButton),
                (l.github_button, Hit::GitHub),
                (l.facebook_button, Hit::Facebook),
                (l.youtube_button, Hit::YouTube),
                (l.portfolio_button, Hit::Portfolio),
                (l.dev_row, Hit::None),
            ];
            for (rect, expected) in cases {
                let (x, y) = centre(&rect);
                assert_eq!(l.hit(View::Main, x, y), expected, "{rect:?} at dpi {dpi}");
            }
            assert_eq!(l.hit(View::Main, -5, -5), Hit::None);
        }
    }

    #[test]
    fn hit_testing_finds_each_control_on_the_settings_page() {
        for dpi in [96, 120, 144, 192] {
            let l = Layout::new(dpi);
            let cases = [
                (l.minimise_button, Hit::Minimise),
                (l.close_button, Hit::Close),
                (l.settings_button, Hit::Settings),
                (l.title_back, Hit::Back),
                (l.crumb_logo, Hit::Caption),
                (l.crumb_text, Hit::Caption),
                (l.crumb_page, Hit::Caption),
                (l.page_title, Hit::None),
                (l.page_tag, Hit::None),
                (l.page_lead, Hit::None),
                (l.startup.card, Hit::StartupCard),
                (l.startup.control, Hit::StartupCard),
                (l.startup.note, Hit::StartupCard),
                (l.startup.detail_text, Hit::StartupCard),
                (l.tray.card, Hit::TrayCard),
                (l.tray.control, Hit::TrayCard),
                (l.tray.note, Hit::TrayCard),
                (l.tray.detail_state, Hit::TrayCard),
                (l.update.title, Hit::None), // only the button opens the page
                (l.update.chip, Hit::None),
                (l.update.button, Hit::UpdateButton),
                (l.saved_bar, Hit::None),
                (l.footer_text, Hit::None),
                (l.footer_github, Hit::GitHub),
            ];
            for (rect, expected) in cases {
                let (x, y) = centre(&rect);
                assert_eq!(l.hit(View::Settings, x, y), expected, "{rect:?} at dpi {dpi}");
            }
            // the gaps between the parts click nothing
            for y in [106, 223, 339] {
                assert_eq!(l.hit(View::Settings, l.scale(200), l.scale(y)), Hit::None, "y {y} at dpi {dpi}");
            }
        }
    }

    #[test]
    fn the_gear_works_on_every_page_and_stays_clear_of_the_other_title_bar_parts() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let (x, y) = centre(&l.settings_button);
            for view in [View::Main, View::Keys, View::Settings] {
                assert_eq!(l.hit(view, x, y), Hit::Settings, "{view:?} at dpi {dpi}");
            }
            assert!(!l.settings_button.overlaps(&l.title_text), "dpi {dpi}");
            assert!(!l.settings_button.overlaps(&l.title_logo), "dpi {dpi}");
            assert_eq!(l.settings_button.right, l.minimise_button.left, "dpi {dpi}");
            assert_eq!(l.settings_button.width(), l.minimise_button.width(), "dpi {dpi}: same size as its neighbours");
            assert!(l.settings_button.width() >= l.scale(32) && l.settings_button.height() >= l.scale(32), "dpi {dpi}");
        }
    }

    #[test]
    fn hit_testing_finds_each_control_on_the_key_hint_page() {
        for dpi in [96, 120, 144, 192] {
            let l = Layout::new(dpi);
            let mut cases = vec![
                (l.minimise_button, Hit::Minimise),
                (l.close_button, Hit::Close),
                (l.title_back, Hit::Back),
                (l.crumb_logo, Hit::Caption),
                (l.crumb_text, Hit::Caption),
                (l.crumb_page, Hit::Caption),
                (l.settings_button, Hit::Settings),
                (l.page_title, Hit::None),
                (l.page_tag, Hit::None),
                (l.hint_grid, Hit::None),
                (l.hint_note, Hit::None),
                (l.logs_button, Hit::None), // belongs to the other page
                (l.keys_button, Hit::None),
                (l.tip_button, Hit::None),
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
    fn the_back_arrow_works_on_the_settings_and_key_pages_only() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let (x, y) = centre(&l.title_back);
            assert_eq!(l.hit(View::Settings, x, y), Hit::Back, "dpi {dpi}");
            assert_eq!(l.hit(View::Keys, x, y), Hit::Back, "dpi {dpi}");
            // on the main page that spot is the logo, which only drags the window
            assert_eq!(l.hit(View::Main, x, y), Hit::Caption, "dpi {dpi}");
        }
    }

    #[test]
    fn the_key_page_heading_and_tabs_sit_in_order_above_the_grid() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            // same heading place as the Settings page
            assert!(l.title_rule.bottom <= l.page_title.top, "dpi {dpi}");
            assert!(l.page_title.bottom < l.hint_tab(0).top, "dpi {dpi}");
            for i in 0..HINT_TABS {
                let tab = l.hint_tab(i);
                assert!(tab.bottom < l.hint_grid.top, "dpi {dpi}");
                assert!(tab.height() >= l.scale(30), "dpi {dpi}: a tab must be easy to click");
            }
            assert!(l.hint_grid.bottom < l.hint_note.top, "dpi {dpi}");
        }
    }

    #[test]
    fn the_pointer_is_a_hand_only_over_things_that_click() {
        assert!(Hit::Close.clickable());
        assert!(Hit::GitHub.clickable());
        assert!(Hit::Facebook.clickable());
        assert!(Hit::YouTube.clickable());
        assert!(Hit::Portfolio.clickable());
        assert!(Hit::TypingCard.clickable());
        assert!(Hit::KeysButton.clickable());
        assert!(Hit::TipButton.clickable());
        assert!(Hit::Back.clickable());
        assert!(Hit::Tab(2).clickable());
        assert!(Hit::Settings.clickable());
        assert!(Hit::StartupCard.clickable());
        assert!(Hit::TrayCard.clickable());
        assert!(Hit::UpdateButton.clickable());
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
                    l.typing_card,
                    l.encoding_card,
                    l.tip_card,
                    l.logs_button,
                    l.keys_button,
                    l.footer_rule,
                    l.dev_row,
                    l.github_button,
                    l.facebook_button,
                    l.youtube_button,
                    l.portfolio_button,
                ],
                dpi,
            );
            // the three cards and the two buttons are one column, with a gap between neighbours
            assert!(l.typing_card.bottom < l.encoding_card.top, "dpi {dpi}");
            assert!(l.encoding_card.bottom < l.tip_card.top, "dpi {dpi}");
            assert!(l.tip_card.bottom < l.logs_button.top, "dpi {dpi}");
            assert!(l.logs_button.bottom < l.footer_rule.top, "dpi {dpi}");
            assert!(l.footer_rule.bottom <= l.dev_row.top, "dpi {dpi}");
        }
    }

    #[test]
    fn blocks_of_the_key_hint_page_do_not_overlap_and_fit_inside_the_window() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let mut blocks = vec![l.title_bar, l.title_rule, l.page_title, l.page_tag, l.hint_grid, l.hint_note];
            blocks.extend((0..HINT_TABS).map(|i| l.hint_tab(i)));
            check_blocks(&blocks, dpi);
            // Leave room for the 2 pixel window outline at the bottom.
            assert!(l.hint_note.bottom <= l.client_size().1 - l.scale(2), "dpi {dpi}");
        }
    }

    #[test]
    fn blocks_of_the_settings_page_do_not_overlap_and_fit_inside_the_window() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            check_blocks(
                &[
                    l.title_bar,
                    l.title_rule,
                    l.page_title,
                    l.page_tag,
                    l.page_lead,
                    l.startup.card,
                    l.tray.card,
                    l.update.card,
                    l.saved_bar,
                    l.footer_text,
                    l.footer_github,
                ],
                dpi,
            );
            // leave room for the 2 pixel window outline at the bottom, and a clear gap under the last card
            assert!(l.footer_github.bottom <= l.client_size().1 - l.scale(2), "dpi {dpi}");
            assert!(l.update.card.bottom < l.saved_bar.top, "dpi {dpi}");
            assert!(l.saved_bar.bottom < l.footer_text.top, "dpi {dpi}");
            assert!(l.page_lead.bottom < l.startup.card.top, "dpi {dpi}");
        }
    }

    #[test]
    fn the_settings_cards_are_as_wide_as_the_other_cards_with_equal_gaps_between() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            for card in [l.startup.card, l.tray.card, l.update.card, l.saved_bar] {
                assert_eq!((card.left, card.right), (l.typing_card.left, l.typing_card.right), "dpi {dpi}");
            }
            let gap1 = l.tray.card.top - l.startup.card.bottom;
            let gap2 = l.update.card.top - l.tray.card.bottom;
            assert_eq!(gap1, gap2, "dpi {dpi}");
            assert!(gap1 >= l.scale(8), "dpi {dpi}");
            assert_eq!(l.startup.card.height(), l.tray.card.height(), "dpi {dpi}");
        }
    }

    #[test]
    fn a_switch_card_holds_its_tile_texts_switch_and_detail_row() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let inside = |inner: &Rect, outer: &Rect| {
                inner.left >= outer.left && inner.right <= outer.right && inner.top >= outer.top && inner.bottom <= outer.bottom
            };
            for c in [l.startup, l.tray] {
                for part in [c.tile, c.title, c.note, c.control, c.rule, c.detail_icon, c.detail_text, c.detail_state] {
                    assert!(inside(&part, &c.card), "{part:?} sticks out of {:?} at dpi {dpi}", c.card);
                }
                // top part: the title above the note, and neither runs under the switch
                assert!(c.tile.right < c.title.left, "dpi {dpi}");
                assert!(c.title.bottom <= c.note.top, "dpi {dpi}");
                assert!(c.title.right < c.control.left && c.note.right < c.control.left, "dpi {dpi}");
                // the note has room for two lines
                assert!(c.note.height() >= l.scale(32), "dpi {dpi}");
                // the tile and the switch are on the same middle line
                let (_, tile_y) = centre(&c.tile);
                let (_, switch_y) = centre(&c.control);
                assert!((tile_y - switch_y).abs() <= 1, "dpi {dpi}");
                // the switch is the very same size, and in the very same column, as on the main page
                assert_eq!((c.control.width(), c.control.height()), (l.switch_track.width(), l.switch_track.height()), "dpi {dpi}");
                assert_eq!((c.control.left, c.control.right), (l.switch_track.left, l.switch_track.right), "dpi {dpi}");
                // the same space left of the tile as right of the switch
                assert_eq!(c.tile.left - c.card.left, c.card.right - c.control.right, "dpi {dpi}");
                // the line is under the note, the detail row under the line
                assert!(c.note.bottom <= c.rule.top, "dpi {dpi}");
                assert!(c.rule.bottom <= c.detail_text.top && c.rule.bottom <= c.detail_icon.top, "dpi {dpi}");
                assert_eq!((c.rule.left, c.rule.right), (c.tile.left, c.control.right), "dpi {dpi}");
                // the detail row: the tick, then the text, then the status word at the right end
                assert!(c.detail_icon.right < c.detail_text.left, "dpi {dpi}");
                assert!(c.detail_text.right <= c.detail_state.left, "dpi {dpi}");
                assert_eq!(c.detail_state.right, c.control.right, "dpi {dpi}");
                let (_, icon_y) = centre(&c.detail_icon);
                let (_, text_y) = centre(&c.detail_text);
                let (_, state_y) = centre(&c.detail_state);
                assert!((icon_y - text_y).abs() <= 1 && (state_y - text_y).abs() <= 1, "dpi {dpi}");
                // some air under the detail row
                assert!(c.card.bottom - c.detail_state.bottom >= l.scale(4), "dpi {dpi}");
            }
        }
    }

    #[test]
    fn the_update_card_has_room_for_its_parts_and_a_big_enough_button() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let u = l.update;
            for part in [u.tile, u.title, u.note, u.chip, u.rule, u.button] {
                assert!(
                    part.left >= u.card.left && part.right <= u.card.right && part.top >= u.card.top && part.bottom <= u.card.bottom,
                    "{part:?} sticks out of {:?} at dpi {dpi}",
                    u.card
                );
            }
            // the top part is laid out like the switch cards, with the chip where the switch is
            assert_eq!(u.tile, Layout::setting_card(dpi, 344).tile, "dpi {dpi}");
            assert_eq!(u.chip.right, l.startup.control.right, "dpi {dpi}");
            assert!(u.title.right < u.chip.left && u.note.right < u.chip.left, "dpi {dpi}");
            let (_, chip_y) = centre(&u.chip);
            let (_, tile_y) = centre(&u.tile);
            assert!((chip_y - tile_y).abs() <= 1, "dpi {dpi}");
            // the chip is wide enough for "v1.2.0" in a mono font
            assert!(u.chip.width() >= l.scale(56) && u.chip.height() >= l.scale(20), "dpi {dpi}");
            // title above note, note above the line, the line above the button
            assert!(u.title.bottom <= u.note.top && u.note.bottom <= u.rule.top && u.rule.bottom <= u.button.top, "dpi {dpi}");
            // wide and tall enough to click and to hold "Check for updates"
            assert!(u.button.width() >= l.scale(200) && u.button.height() >= l.scale(32), "dpi {dpi}");
            // the same space left and right of the button, and the same under it as at the sides
            assert_eq!(u.button.left - u.card.left, u.card.right - u.button.right, "dpi {dpi}");
            assert_eq!(u.card.bottom - u.button.bottom, u.button.left - u.card.left, "dpi {dpi}");
        }
    }

    #[test]
    fn the_settings_title_bar_has_room_for_the_arrow_the_name_and_the_page_name() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let inside = |inner: &Rect, outer: &Rect| {
                inner.left >= outer.left && inner.right <= outer.right && inner.top >= outer.top && inner.bottom <= outer.bottom
            };
            for part in [l.title_back, l.crumb_logo, l.crumb_text, l.crumb_page] {
                assert!(inside(&part, &l.title_bar), "{part:?} at dpi {dpi}");
            }
            // left to right, none on top of another, and clear of the gear
            assert!(l.title_back.right <= l.crumb_logo.left, "dpi {dpi}");
            assert!(l.crumb_logo.right < l.crumb_text.left, "dpi {dpi}");
            assert!(l.crumb_text.right <= l.crumb_page.left, "dpi {dpi}");
            assert!(l.crumb_page.right < l.settings_button.left, "dpi {dpi}");
            // the arrow is a big enough target
            assert!(l.title_back.width() >= l.scale(32) && l.title_back.height() >= l.scale(32), "dpi {dpi}");
            // the name is as wide as on the other pages
            assert_eq!(l.crumb_text.width(), l.title_text.width(), "dpi {dpi}");
            assert_eq!((l.crumb_logo.width(), l.crumb_logo.height()), (l.title_logo.width(), l.title_logo.height()), "dpi {dpi}");
        }
    }

    #[test]
    fn the_settings_footer_and_saved_bar_hold_their_parts() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let inside = |inner: &Rect, outer: &Rect| {
                inner.left >= outer.left && inner.right <= outer.right && inner.top >= outer.top && inner.bottom <= outer.bottom
            };
            for part in [l.saved_dot, l.saved_text, l.saved_path] {
                assert!(inside(&part, &l.saved_bar), "{part:?} at dpi {dpi}");
            }
            assert!(l.saved_dot.right < l.saved_text.left, "dpi {dpi}");
            assert!(l.saved_text.right <= l.saved_path.left, "dpi {dpi}");
            // the text on the left and the path on the right have room for about 28 and 16 characters
            assert!(l.saved_text.width() >= l.scale(170) && l.saved_path.width() >= l.scale(110), "dpi {dpi}");
            let (_, dot_y) = centre(&l.saved_dot);
            let (_, bar_y) = centre(&l.saved_bar);
            assert!((dot_y - bar_y).abs() <= 1, "dpi {dpi}");
            // the footer: a dot, a line of text and the GitHub link, in that order, on one line
            assert!(l.footer_dot.right < l.footer_text.left, "dpi {dpi}");
            assert!(l.footer_text.right <= l.footer_github.left, "dpi {dpi}");
            let (_, dot_y) = centre(&l.footer_dot);
            let (_, text_y) = centre(&l.footer_text);
            assert!((dot_y - text_y).abs() <= 1, "dpi {dpi}");
            assert_eq!(l.footer_github.right, l.typing_card.right, "dpi {dpi}");
            assert!(l.footer_github.height() >= l.scale(22) && l.footer_github.width() >= l.scale(60), "dpi {dpi}");
        }
    }

    #[test]
    fn the_smart_quotes_strip_sits_under_the_encoding_card_and_holds_its_text_and_button() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            // same width and edges as the cards
            assert_eq!((l.tip_card.left, l.tip_card.right), (l.typing_card.left, l.typing_card.right), "dpi {dpi}");
            assert_eq!((l.tip_card.left, l.tip_card.right), (l.encoding_card.left, l.encoding_card.right), "dpi {dpi}");
            // text on the left, button on the right, none on top of the other
            assert!(l.tip_text.right < l.tip_button.left, "dpi {dpi}");
            // the button is on the middle line of the strip
            let (_, button_y) = centre(&l.tip_button);
            let (_, card_y) = centre(&l.tip_card);
            assert!((button_y - card_y).abs() <= 1, "dpi {dpi}");
            // the same space above and below the text
            assert_eq!(l.tip_text.top - l.tip_card.top, l.tip_card.bottom - l.tip_text.bottom, "dpi {dpi}");
            // big enough to click and to read
            assert!(l.tip_button.height() >= l.scale(26), "dpi {dpi}");
            assert!(l.tip_button.width() >= l.scale(90), "dpi {dpi}");
            assert!(l.tip_text.height() >= l.scale(60), "dpi {dpi}: room for three lines of 20");
            assert!(l.tip_text.width() >= l.scale(200), "dpi {dpi}: the Sinhala line is about 200 wide");
        }
    }

    #[test]
    fn the_typing_card_holds_the_tile_the_texts_the_switch_and_the_shortcut_row() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let inside = |inner: &Rect, outer: &Rect| {
                inner.left >= outer.left && inner.right <= outer.right && inner.top >= outer.top && inner.bottom <= outer.bottom
            };
            for part in [l.typing_top, l.typing_icon, l.typing_label, l.typing_sub, l.typing_state, l.switch_track, l.shortcut_row] {
                assert!(inside(&part, &l.typing_card), "{part:?} sticks out of the card at dpi {dpi}");
            }
            for part in [l.shortcut_icon, l.shortcut_label] {
                assert!(inside(&part, &l.shortcut_row), "{part:?} sticks out of the row at dpi {dpi}");
            }
            // the click area is above the row, so a click on the row does nothing
            assert!(l.typing_top.bottom <= l.shortcut_row.top, "dpi {dpi}");
            // the name is above its Sinhala line, and neither runs under the ON or OFF text or the switch
            assert!(l.typing_label.bottom <= l.typing_sub.top, "dpi {dpi}");
            assert!(l.typing_label.right <= l.typing_state.left && l.typing_sub.right <= l.typing_state.left, "dpi {dpi}");
            assert!(l.typing_state.right < l.switch_track.left, "dpi {dpi}");
            assert!(l.typing_icon.right < l.typing_label.left, "dpi {dpi}");
            // the tile and the switch sit on the same middle line, and the switch keeps its size
            let (_, tile_y) = centre(&l.typing_icon);
            let (_, switch_y) = centre(&l.switch_track);
            assert!((tile_y - switch_y).abs() <= 1, "dpi {dpi}");
            assert_eq!((l.switch_track.width(), l.switch_track.height()), (l.scale(52), l.scale(28)), "dpi {dpi}");
            // the same space left of the tile as right of the switch
            assert_eq!(l.typing_icon.left - l.typing_card.left, l.typing_card.right - l.switch_track.right, "dpi {dpi}");
            // the row has room for its chip: the label stops well before the right side
            assert!(l.shortcut_label.right <= l.shortcut_row.right - l.scale(120), "dpi {dpi}");
        }
    }

    #[test]
    fn the_encoding_card_holds_a_heading_two_equal_buttons_and_the_status_line() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let inside = |inner: &Rect, outer: &Rect| {
                inner.left >= outer.left && inner.right <= outer.right && inner.top >= outer.top && inner.bottom <= outer.bottom
            };
            for part in [
                l.encoding_icon,
                l.encoding_label,
                l.encoding_row,
                l.unicode_button,
                l.legacy_button,
                l.encoding_status_icon,
                l.encoding_status,
            ] {
                assert!(inside(&part, &l.encoding_card), "{part:?} sticks out of the card at dpi {dpi}");
            }
            // two buttons of the same size side by side, a small gap between them, edges on the card padding
            assert_eq!((l.unicode_button.width(), l.unicode_button.height()), (l.legacy_button.width(), l.legacy_button.height()), "dpi {dpi}");
            assert_eq!(l.unicode_button.top, l.legacy_button.top, "dpi {dpi}");
            assert!(l.unicode_button.right < l.legacy_button.left, "dpi {dpi}");
            assert!(l.legacy_button.left - l.unicode_button.right <= l.scale(12), "dpi {dpi}");
            assert_eq!(l.unicode_button.left, l.encoding_row.left, "dpi {dpi}");
            assert_eq!(l.legacy_button.right, l.encoding_row.right, "dpi {dpi}");
            // tall enough for the name and the line under it
            assert!(l.unicode_button.height() >= l.scale(54), "dpi {dpi}");
            // heading, then buttons, then the status line, each below the last
            assert!(l.encoding_row.bottom < l.unicode_button.top, "dpi {dpi}");
            assert!(l.unicode_button.bottom < l.encoding_status.top, "dpi {dpi}");
            // the heading's name stops before the key chip that sits at the right end of the row
            assert!(l.encoding_label.right <= l.encoding_row.right - l.scale(150), "dpi {dpi}");
            // two lines of text fit in the status area, and the tick sits beside the first line
            assert!(l.encoding_status.height() >= l.scale(36), "dpi {dpi}");
            assert!(l.encoding_status_icon.right < l.encoding_status.left, "dpi {dpi}");
            assert!(l.encoding_status_icon.top >= l.encoding_status.top - l.scale(2), "dpi {dpi}");
        }
    }

    #[test]
    fn the_title_bar_has_room_for_the_name_the_badge_and_the_three_buttons() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            assert!(l.title_logo.right < l.title_text.left, "dpi {dpi}");
            assert!(l.title_text.right <= l.title_badge.left, "dpi {dpi}");
            assert!(l.title_badge.right < l.settings_button.left, "dpi {dpi}");
            assert!(l.title_badge.width() >= l.scale(40) && l.title_badge.height() >= l.scale(18), "dpi {dpi}");
            // the badge is on the middle line of the bar
            let (_, y) = centre(&l.title_badge);
            let (_, bar_y) = centre(&l.title_bar);
            assert!((y - bar_y).abs() <= 1, "dpi {dpi}");
        }
    }

    #[test]
    fn the_bottom_of_the_window_still_has_room_for_the_outline() {
        let l = Layout::new(96);
        assert_eq!(HEIGHT, 574);
        assert!(l.youtube_button.bottom <= l.window.bottom - l.scale(2));
        assert!(l.portfolio_button.bottom <= l.window.bottom - l.scale(2));
        assert!(l.hint_note.bottom <= l.window.bottom - l.scale(2));
        assert!(l.footer_text.bottom <= l.window.bottom - l.scale(2));
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
    fn the_four_footer_buttons_are_equal_evenly_spaced_and_centred_under_the_developer_row() {
        for dpi in [96, 144, 192] {
            let l = Layout::new(dpi);
            let buttons = [l.github_button, l.facebook_button, l.youtube_button, l.portfolio_button];
            for b in buttons {
                assert_eq!((b.width(), b.height()), (buttons[0].width(), buttons[0].height()), "dpi {dpi}");
                assert_eq!(b.top, buttons[0].top, "dpi {dpi}");
                assert!(b.top > l.dev_row.bottom, "dpi {dpi}: the buttons must sit under the developer row");
                assert!(b.width() >= l.scale(32), "dpi {dpi}: too small to click");
            }
            let gaps: Vec<i32> = buttons.windows(2).map(|pair| pair[1].left - pair[0].right).collect();
            for gap in &gaps {
                assert!(*gap > 0, "dpi {dpi}: the buttons touch");
                assert!((gap - gaps[0]).abs() <= 1, "dpi {dpi}: gaps {gaps:?}");
            }
            // The group is in the middle of the window.
            let left_space = buttons[0].left;
            let right_space = l.window.right - buttons[3].right;
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
            for part in [l.title_logo, l.title_text, l.title_badge, l.settings_button, l.minimise_button, l.close_button] {
                assert!(inside(&part, &l.title_bar), "{part:?} at dpi {dpi}");
            }
            for part in [l.tip_text, l.tip_button] {
                assert!(inside(&part, &l.tip_card), "{part:?} at dpi {dpi}");
            }
            for i in 0..HINT_TABS {
                assert!(inside(&l.hint_tab(i), &l.window), "tab {i} at dpi {dpi}");
            }
            assert!(inside(&l.title_bar, &l.window));
            for button in [l.github_button, l.facebook_button, l.youtube_button, l.portfolio_button] {
                assert!(inside(&button, &l.window), "{button:?} at dpi {dpi}");
            }
        }
    }

    #[test]
    fn small_parts_are_centred_on_their_card() {
        let l = Layout::new(96);
        // the shortcut icon sits on the middle line of its row, and the tick on the first line of the status
        let (_, y) = centre(&l.shortcut_icon);
        let (_, row_y) = centre(&l.shortcut_row);
        assert!((y - row_y).abs() <= 1, "{:?} is off the middle of {:?}", l.shortcut_icon, l.shortcut_row);
        let (_, y) = centre(&l.encoding_icon);
        let (_, row_y) = centre(&l.encoding_row);
        assert!((y - row_y).abs() <= 1);
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
