//! The key hint page: what to type to get each Sinhala letter.
//!
//! Every cell is built from the tables in `engine.rs`, so the page cannot disagree with what
//! typing really does. The tests type each cell's keys into the engine and compare the result.
//! No Windows calls here, so all of it is tested on any system.

use crate::engine::{self, KeyGroups};
use crate::layout::HINT_TABS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Vowels,
    Consonants,
    Signs,
    Examples,
}

/// The tabs, left to right.
pub const TABS: [Tab; HINT_TABS] = [Tab::Vowels, Tab::Consonants, Tab::Signs, Tab::Examples];

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Vowels => "Vowels",
            Tab::Consonants => "Consonants",
            Tab::Signs => "Signs",
            Tab::Examples => "Examples",
        }
    }

    /// Position in `TABS`, counted from 0.
    pub fn index(self) -> u8 {
        TABS.iter().position(|t| *t == self).unwrap_or(0) as u8
    }

    /// The tab at position `i`. A number that is too big gives the first tab.
    pub fn from_index(i: u8) -> Tab {
        TABS.get(i as usize).copied().unwrap_or(Tab::Vowels)
    }
}

/// One box on the page: the Sinhala result and the Singlish keys that give it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub glyph: String,
    pub keys: String,
}

/// What the page shows: a grid of cells and a line of notes under it.
#[derive(Clone, Debug)]
pub struct Page {
    pub columns: usize,
    pub cells: Vec<Cell>,
    pub note: String,
}

impl Page {
    pub fn rows(&self) -> usize {
        self.cells.len().div_ceil(self.columns.max(1))
    }
}

/// Words that show how letters join up. The Sinhala is made by the engine.
/// The first seven show the marks and joined letters of the reference sheet.
const EXAMPLES: [&str; 14] = [
    "k", "kax", "kazn", "kaH", "kaX", "kya", "kra", "mama", "ammaa", "gedhara", "nuwara", "koLaBa", "sixhala",
    "prashna",
];

fn cell(glyph: impl Into<String>, keys: impl Into<String>) -> Cell {
    Cell { glyph: glyph.into(), keys: keys.into() }
}

/// "AA = ඈ, ou = ඖ": the other keys that give the same thing as the main key.
/// `show` turns the Sinhala character into what is printed.
fn also(groups: &KeyGroups, show: impl Fn(char) -> String) -> String {
    let mut items = Vec::new();
    for (glyph, keys) in groups {
        for key in keys.iter().skip(1) {
            items.push(format!("{key} = {}", show(*glyph)));
        }
    }
    items.join(", ")
}

pub fn page(tab: Tab) -> Page {
    match tab {
        Tab::Vowels => {
            let groups = engine::vowel_keys();
            Page {
                columns: 4,
                cells: groups.iter().map(|(glyph, keys)| cell(*glyph, keys[0])).collect(),
                note: format!(
                    "Also: {}. Straight after a consonant, use the Signs tab.",
                    also(&groups, |c| c.to_string())
                ),
            }
        }
        Tab::Consonants => {
            let groups = engine::consonant_keys();
            // ඞ has no key of its own in the reference sheet, only "kaX", so it is on the
            // Examples page and not here. The last row has one box less.
            Page {
                columns: 5,
                cells: groups.iter().map(|(glyph, keys)| cell(*glyph, format!("{}a", keys[0]))).collect(),
                note: format!(
                    "Also: {}. A letter on its own gets a hal: k = {}",
                    also(&groups, |c| c.to_string()),
                    engine::transliterate("k")
                ),
            }
        }
        Tab::Signs => {
            let groups = engine::sign_keys();
            let mut cells = vec![cell("ක", "ka")];
            cells.extend(groups.iter().map(|(sign, keys)| cell(format!("ක{sign}"), format!("k{}", keys[0]))));
            Page {
                columns: 4,
                cells,
                note: format!("Type them after a consonant. Also: {}", also(&groups, |c| format!("ක{c}"))),
            }
        }
        Tab::Examples => Page {
            columns: 2,
            cells: EXAMPLES.iter().map(|keys| cell(engine::transliterate(keys), *keys)).collect(),
            note: format!(
                "Capitals matter: ta = {}, Ta = {}. x, zn, H and X are typed after a letter.",
                engine::transliterate("ta"),
                engine::transliterate("Ta")
            ),
        },
    }
}

/// The whole page set as plain text, for `lipi --keys` in a terminal.
pub fn sheet() -> String {
    let mut out = String::new();
    for tab in TABS {
        let page = page(tab);
        out.push_str(&format!("== {} ==\n", tab.label()));
        for row in page.cells.chunks(page.columns) {
            let line: Vec<String> = row.iter().map(|c| format!("{} {:<9}", c.glyph, c.keys)).collect();
            out.push_str(line.join("  ").trim_end());
            out.push('\n');
        }
        out.push_str(&format!("{}\n\n", page.note));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Layout;

    #[test]
    fn typing_the_keys_of_every_cell_gives_its_glyph() {
        for tab in TABS {
            for c in page(tab).cells {
                assert_eq!(engine::transliterate(&c.keys), c.glyph, "{:?}: {}", tab, c.keys);
            }
        }
    }

    #[test]
    fn each_page_has_the_expected_shape_and_leaves_no_empty_row() {
        // (tab, columns, rows, boxes). Only the consonant page has a gap, in its last row.
        let shapes = [
            (Tab::Vowels, 4, 4, 16),
            (Tab::Consonants, 5, 8, 39),
            (Tab::Signs, 4, 4, 16),
            (Tab::Examples, 2, 7, 14),
        ];
        for (tab, columns, rows, boxes) in shapes {
            let p = page(tab);
            assert_eq!((p.columns, p.rows(), p.cells.len()), (columns, rows, boxes), "{tab:?}");
            assert!(p.cells.len() <= columns * rows, "{tab:?} runs over its grid");
            assert!(p.cells.len() > columns * (rows - 1), "{tab:?} has an empty last row");
        }
    }

    #[test]
    fn no_letter_appears_twice_on_the_letter_pages() {
        for tab in [Tab::Vowels, Tab::Consonants, Tab::Signs] {
            let mut seen = std::collections::HashSet::new();
            for c in page(tab).cells {
                assert!(seen.insert(c.glyph.clone()), "{:?} shows {} twice", tab, c.glyph);
            }
        }
    }

    #[test]
    fn every_key_of_the_engine_is_on_the_page_either_as_main_key_or_in_the_note() {
        let vowels = page(Tab::Vowels);
        for (_, keys) in engine::vowel_keys() {
            assert!(vowels.cells.iter().any(|c| c.keys == keys[0]));
            for alias in &keys[1..] {
                assert!(vowels.note.contains(&format!("{alias} = ")), "vowel alias {alias}");
            }
        }
        let consonants = page(Tab::Consonants);
        for (_, keys) in engine::consonant_keys() {
            assert!(consonants.cells.iter().any(|c| c.keys == format!("{}a", keys[0])));
            for alias in &keys[1..] {
                assert!(consonants.note.contains(&format!("{alias} = ")), "consonant alias {alias}");
            }
        }
        let signs = page(Tab::Signs);
        for (_, keys) in engine::sign_keys() {
            assert!(signs.cells.iter().any(|c| c.keys == format!("k{}", keys[0])));
            for alias in &keys[1..] {
                assert!(signs.note.contains(&format!("{alias} = ")), "sign alias {alias}");
            }
        }
    }

    #[test]
    fn nga_is_shown_only_the_way_the_sheet_types_it() {
        // The sheet has no key for ඞ on its own. It gives "kaX" only.
        assert!(page(Tab::Consonants).cells.iter().all(|c| c.glyph != "ඞ" && c.keys != "X"));
        assert!(page(Tab::Examples).cells.contains(&cell("කඞ", "kaX")));
    }

    #[test]
    fn the_signs_page_has_no_key_the_sheet_does_not_give() {
        let p = page(Tab::Signs);
        for key in ["kR", "kRu", "kou"] {
            assert!(p.cells.iter().all(|c| c.keys != key), "{key} is not a vowel sign");
        }
        for key in ["R", "Ru", "ou"] {
            assert!(!p.note.contains(&format!("{key} = ")), "{key} is not a vowel sign");
        }
        assert!(p.cells.iter().any(|c| c.keys == "kru" && c.glyph == "කෘ"));
        assert!(p.cells.iter().any(|c| c.keys == "kruu" && c.glyph == "කෲ"));
    }

    #[test]
    fn the_vowel_page_still_lists_r_ru_and_ou_as_vowels() {
        let p = page(Tab::Vowels);
        assert!(p.cells.contains(&cell("ඍ", "R")));
        assert!(p.cells.contains(&cell("ඎ", "Ru")));
        assert!(p.note.contains("ou = ඖ"));
    }

    #[test]
    fn examples_show_the_joining_rules() {
        let p = page(Tab::Examples);
        let find = |keys: &str| p.cells.iter().find(|c| c.keys == keys).unwrap().glyph.clone();
        assert_eq!(find("k"), "ක්");
        assert_eq!(find("kax"), "කං");
        assert_eq!(find("kazn"), "කං");
        assert_eq!(find("kaH"), "කඃ");
        assert_eq!(find("kaX"), "කඞ");
        assert_eq!(find("kya"), "ක්\u{200D}ය");
        assert_eq!(find("kra"), "ක්\u{200D}ර");
        assert_eq!(find("sixhala"), "සිංහල");
        assert_eq!(find("koLaBa"), "කොළඹ");
        assert_eq!(find("prashna"), "ප්\u{200D}රශ්න");
        assert_eq!(find("gedhara"), "ගෙදර");
    }

    #[test]
    fn labels_and_keys_are_short_enough_for_their_boxes() {
        for tab in TABS {
            assert!(tab.label().chars().count() <= 10, "{tab:?}");
            let p = page(tab);
            assert!(p.note.chars().count() <= 100, "{tab:?} note is too long for two lines: {}", p.note);
            for c in &p.cells {
                assert!(!c.keys.is_empty() && !c.keys.contains(' '), "{tab:?}: {:?}", c.keys);
                assert!(c.keys.chars().count() <= 9, "{tab:?}: {}", c.keys);
            }
            if tab != Tab::Examples {
                for c in &p.cells {
                    assert!(c.glyph.chars().count() <= 2, "{tab:?}: {}", c.glyph);
                    assert!(c.keys.chars().count() <= 5, "{tab:?}: {}", c.keys);
                }
            }
        }
    }

    #[test]
    fn tab_numbers_round_trip() {
        for (i, tab) in TABS.iter().enumerate() {
            assert_eq!(tab.index() as usize, i);
            assert_eq!(Tab::from_index(i as u8), *tab);
        }
        assert_eq!(Tab::from_index(200), Tab::Vowels);
    }

    #[test]
    fn every_page_fits_its_grid_on_every_screen_size() {
        for dpi in [96, 120, 144, 192] {
            let l = Layout::new(dpi);
            for tab in TABS {
                let p = page(tab);
                let (columns, rows) = (p.columns, p.rows());
                let cells: Vec<_> = (0..p.cells.len()).map(|i| l.hint_cell(columns, rows, i)).collect();
                let last = cells.last().unwrap();
                assert!(last.bottom <= l.hint_grid.bottom, "{tab:?} at dpi {dpi}");
                assert!(last.right <= l.hint_grid.right, "{tab:?} at dpi {dpi}");
                for (i, a) in cells.iter().enumerate() {
                    assert!(a.height() >= l.scale(30), "{tab:?} cell {i} too low at dpi {dpi}");
                    assert!(a.width() >= l.scale(60), "{tab:?} cell {i} too narrow at dpi {dpi}");
                    for b in &cells[i + 1..] {
                        assert!(!a.overlaps(b), "{tab:?} at dpi {dpi}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_text_sheet_lists_every_tab() {
        let s = sheet();
        for tab in TABS {
            assert!(s.contains(&format!("== {} ==", tab.label())));
        }
        assert!(s.contains("ක chha") || s.contains("ඡ chha"));
    }
}