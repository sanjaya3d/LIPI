//! Settings read from settings.txt, which sits next to the program.
//! Reading the text (`parse`) is separate from reading the file (`load`),
//! so the rules can be tested on any system.

use std::path::Path;
use std::sync::OnceLock;

use crate::keymap::*;

pub const FILE_NAME: &str = "settings.txt";

/// Written next to the program the first time it runs.
pub const TEMPLATE: &str = "\
# LIPI settings.
# Lines starting with # are comments. Change a value, save, and restart the program.

# Tap this key alone to switch Sinhala typing on or off.
# Choices: RightCtrl, LeftCtrl, RightShift, LeftShift, RightAlt, LeftAlt, ScrollLock, Pause
on_off_key = RightCtrl

# Tap this key alone to switch between Unicode and legacy output.
# Same choices. It must be a different key.
output_key = RightShift

# What the program starts with.
# start_typing: on or off.   start_output: unicode or legacy.
start_typing = on
start_output = unicode

# yes: start hidden in the tray. no: open the window.
start_in_tray = no
";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub on_off_key: u32,
    pub output_key: u32,
    pub start_typing: bool,
    pub start_legacy: bool,
    pub start_in_tray: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            on_off_key: DEFAULT_ON_OFF_VK,
            output_key: DEFAULT_OUTPUT_VK,
            start_typing: true,
            start_legacy: false,
            start_in_tray: false,
        }
    }
}

/// Keys that are safe to use for a tap: pressed alone they do nothing in most apps.
/// (setting name, name shown to the user, virtual key)
const KEYS: &[(&str, &str, u32)] = &[
    ("rightctrl", "Right Ctrl", VK_RCONTROL),
    ("leftctrl", "Left Ctrl", VK_LCONTROL),
    ("rightshift", "Right Shift", VK_RSHIFT),
    ("leftshift", "Left Shift", VK_LSHIFT),
    ("rightalt", "Right Alt", VK_RMENU),
    ("leftalt", "Left Alt", VK_LMENU),
    ("scrolllock", "Scroll Lock", VK_SCROLL),
    ("pause", "Pause", VK_PAUSE),
];

/// "RightCtrl", "right ctrl" and "right-ctrl" all mean the same key.
pub fn key_from_name(name: &str) -> Option<u32> {
    let wanted: String = name
        .chars()
        .filter(|c| !matches!(c, ' ' | '_' | '-'))
        .map(|c| c.to_ascii_lowercase())
        .collect();
    KEYS.iter().find(|k| k.0 == wanted).map(|k| k.2)
}

/// Name for showing in the window, for example "Right Ctrl".
pub fn key_name(vk: u32) -> &'static str {
    KEYS.iter().find(|k| k.2 == vk).map_or("?", |k| k.1)
}

fn flag(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "on" | "yes" | "true" | "1" => Some(true),
        "off" | "no" | "false" | "0" => Some(false),
        _ => None,
    }
}

/// Reads the text of a settings file. Never fails: a bad line gives a warning
/// and that setting keeps its default.
pub fn parse(text: &str) -> (Settings, Vec<String>) {
    let mut s = Settings::default();
    let mut warnings = Vec::new();

    for (n, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            warnings.push(format!("{FILE_NAME} line {}: expected 'name = value'", n + 1));
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        let bad = |warnings: &mut Vec<String>| {
            warnings.push(format!("{FILE_NAME} line {}: '{value}' is not a valid value for {key}", n + 1));
        };

        match key.as_str() {
            "on_off_key" => match key_from_name(value) {
                Some(vk) => s.on_off_key = vk,
                None => bad(&mut warnings),
            },
            "output_key" => match key_from_name(value) {
                Some(vk) => s.output_key = vk,
                None => bad(&mut warnings),
            },
            "start_typing" => match flag(value) {
                Some(on) => s.start_typing = on,
                None => bad(&mut warnings),
            },
            "start_output" => match value.to_ascii_lowercase().as_str() {
                "unicode" => s.start_legacy = false,
                "legacy" => s.start_legacy = true,
                _ => bad(&mut warnings),
            },
            "start_in_tray" => match flag(value) {
                Some(on) => s.start_in_tray = on,
                None => bad(&mut warnings),
            },
            _ => warnings.push(format!("{FILE_NAME} line {}: unknown setting '{key}'", n + 1)),
        }
    }

    if s.on_off_key == s.output_key {
        warnings.push(format!(
            "{FILE_NAME}: on_off_key and output_key are the same key. Using the default keys for both."
        ));
        s.on_off_key = DEFAULT_ON_OFF_VK;
        s.output_key = DEFAULT_OUTPUT_VK;
    }
    (s, warnings)
}

/// Reads settings.txt from `dir`. If it is not there, writes the template
/// (when the folder allows it) and uses the defaults.
pub fn load(dir: &Path) -> (Settings, Vec<String>) {
    let path = dir.join(FILE_NAME);
    match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut notes = Vec::new();
            if std::fs::write(&path, TEMPLATE.replace('\n', "\r\n")).is_ok() {
                notes.push(format!("Created {} with the default settings.", path.display()));
            }
            (Settings::default(), notes)
        }
        Err(e) => (Settings::default(), vec![format!("Could not read {}: {e}", path.display())]),
    }
}

static CURRENT: OnceLock<Settings> = OnceLock::new();

/// Makes these the settings for the whole program. Call once, at the start.
pub fn install(settings: Settings) {
    let _ = CURRENT.set(settings);
}

/// The settings in use. Before `install`, the defaults.
pub fn get() -> &'static Settings {
    CURRENT.get_or_init(Settings::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_template_means_the_defaults() {
        let (s, warnings) = parse(TEMPLATE);
        assert_eq!(s, Settings::default());
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn crlf_and_bom_are_fine() {
        let (s, warnings) = parse(&format!("\u{feff}{}", TEMPLATE.replace('\n', "\r\n")));
        assert_eq!(s, Settings::default());
        assert!(warnings.is_empty());
    }

    #[test]
    fn every_choice_is_understood() {
        let text = "on_off_key = Left Shift\noutput_key = scroll-lock\nstart_typing = OFF\nstart_output = Legacy\nstart_in_tray = Yes\n";
        let (s, warnings) = parse(text);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s.on_off_key, VK_LSHIFT);
        assert_eq!(s.output_key, VK_SCROLL);
        assert!(!s.start_typing);
        assert!(s.start_legacy);
        assert!(s.start_in_tray);
    }

    #[test]
    fn bad_lines_warn_and_keep_defaults() {
        let (s, warnings) = parse("on_off_key = F12\nstart_output = fancy\nnonsense\ncolour = red\n");
        assert_eq!(s, Settings::default());
        assert_eq!(warnings.len(), 4, "{warnings:?}");
        assert!(warnings[0].contains("line 1"));
        assert!(warnings[1].contains("start_output"));
        assert!(warnings[2].contains("line 3"));
        assert!(warnings[3].contains("unknown setting 'colour'"));
    }

    #[test]
    fn the_same_key_twice_falls_back() {
        let (s, warnings) = parse("on_off_key = Pause\noutput_key = Pause\n");
        assert_eq!(s.on_off_key, DEFAULT_ON_OFF_VK);
        assert_eq!(s.output_key, DEFAULT_OUTPUT_VK);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn names_round_trip() {
        for (_, shown, vk) in KEYS {
            assert_eq!(key_from_name(shown), Some(*vk), "{shown}");
            assert_eq!(key_name(*vk), *shown);
        }
        assert_eq!(key_name(0x41), "?");
        assert_eq!(key_from_name("F12"), None);
    }

    #[test]
    fn a_missing_file_gives_defaults_and_writes_the_template() {
        let dir = std::env::temp_dir().join(format!("singlish-settings-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (s, notes) = load(&dir);
        assert_eq!(s, Settings::default());
        assert_eq!(notes.len(), 1);
        let written = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap();
        assert_eq!(parse(&written).0, Settings::default());

        // a second run reads the file instead
        std::fs::write(dir.join(FILE_NAME), "start_output = legacy\n").unwrap();
        let (s, notes) = load(&dir);
        assert!(s.start_legacy);
        assert!(notes.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
