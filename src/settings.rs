//! Settings read from settings.txt. The file is kept in the user's AppData folder
//! (C:\Users\<you>\AppData\Roaming\LIPI), which Explorer hides by default, so nothing
//! extra sits next to the program. Reading the text (`parse`) is separate from
//! reading the file (`load`), so the rules can be tested on any system.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::keymap::*;

pub const FILE_NAME: &str = "settings.txt";

/// The folder made inside AppData.
pub const FOLDER_NAME: &str = "LIPI";

/// Written the first time the program runs.
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

/// Where settings.txt lives: the LIPI folder inside AppData\Roaming.
/// If AppData cannot be found, it is the program's own folder, as in older versions.
pub fn folder(program_dir: &Path) -> PathBuf {
    folder_from(std::env::var_os("APPDATA"), program_dir)
}

fn folder_from(appdata: Option<OsString>, program_dir: &Path) -> PathBuf {
    match appdata {
        Some(base) if !base.is_empty() => PathBuf::from(base).join(FOLDER_NAME),
        _ => program_dir.to_path_buf(),
    }
}

/// Older versions kept settings.txt next to the program. The first time, this moves that
/// file into `dir`, so the keys the user chose are kept. It does nothing when `dir` already
/// has a settings file, when there is no old file, or when both folders are the same.
/// The old file is removed only after the copy is checked to be exactly the same.
pub fn migrate(old_dir: &Path, dir: &Path) -> Vec<String> {
    if old_dir == dir {
        return Vec::new();
    }
    let old_path = old_dir.join(FILE_NAME);
    let new_path = dir.join(FILE_NAME);
    if new_path.exists() {
        return Vec::new();
    }
    let Ok(bytes) = std::fs::read(&old_path) else {
        return Vec::new();
    };
    let copied = std::fs::create_dir_all(dir).and_then(|_| std::fs::write(&new_path, &bytes));
    if let Err(e) = copied {
        return vec![format!("Could not move {} to {}: {e}", old_path.display(), new_path.display())];
    }
    let same = std::fs::read(&new_path).map_or(false, |now| now == bytes);
    if same && std::fs::remove_file(&old_path).is_ok() {
        vec![format!("Moved {} to {}", old_path.display(), new_path.display())]
    } else {
        vec![format!(
            "Copied {} to {}. The old file can be deleted.",
            old_path.display(),
            new_path.display()
        )]
    }
}

/// Reads settings.txt from `dir`. If it is not there, makes the folder and writes the
/// template (when that is allowed) and uses the defaults.
pub fn load(dir: &Path) -> (Settings, Vec<String>) {
    let path = dir.join(FILE_NAME);
    match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut notes = Vec::new();
            let written = std::fs::create_dir_all(dir)
                .and_then(|_| std::fs::write(&path, TEMPLATE.replace('\n', "\r\n")));
            if written.is_ok() {
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

    /// A fresh folder name for each test, so tests running side by side never share files.
    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lipi-settings-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn the_settings_folder_is_lipi_inside_appdata() {
        let program = Path::new("C:\\Tools\\LIPI");
        let got = folder_from(Some(OsString::from("C:\\Users\\me\\AppData\\Roaming")), program);
        assert_eq!(got, PathBuf::from("C:\\Users\\me\\AppData\\Roaming").join("LIPI"));
        assert!(got.ends_with(FOLDER_NAME));
    }

    #[test]
    fn without_appdata_the_program_folder_is_used() {
        let program = Path::new("C:\\Tools\\LIPI");
        assert_eq!(folder_from(None, program), program);
        assert_eq!(folder_from(Some(OsString::new()), program), program);
    }

    #[test]
    fn a_missing_folder_is_made_and_the_template_written() {
        let dir = temp("fresh").join("deeper");
        assert!(!dir.exists());
        let (s, notes) = load(&dir);
        assert_eq!(s, Settings::default());
        assert_eq!(notes.len(), 1);
        assert_eq!(parse(&std::fs::read_to_string(dir.join(FILE_NAME)).unwrap()).0, Settings::default());
        let _ = std::fs::remove_dir_all(dir.parent().unwrap());
    }

    #[test]
    fn the_old_file_next_to_the_program_is_moved_and_its_keys_are_kept() {
        let base = temp("move");
        let (old, new) = (base.join("program"), base.join("appdata").join("LIPI"));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join(FILE_NAME), "on_off_key = Pause\r\nstart_output = legacy\r\n").unwrap();

        let notes = migrate(&old, &new);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].starts_with("Moved "), "{notes:?}");
        assert!(!old.join(FILE_NAME).exists(), "the old file should be gone");

        let (s, warnings) = load(&new);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s.on_off_key, VK_PAUSE);
        assert!(s.start_legacy);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn nothing_is_moved_when_the_new_file_already_exists() {
        let base = temp("keep");
        let (old, new) = (base.join("program"), base.join("LIPI"));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(old.join(FILE_NAME), "start_typing = off\n").unwrap();
        std::fs::write(new.join(FILE_NAME), "start_typing = on\n").unwrap();

        assert!(migrate(&old, &new).is_empty());
        assert!(old.join(FILE_NAME).exists(), "the old file is left alone");
        assert_eq!(std::fs::read_to_string(new.join(FILE_NAME)).unwrap(), "start_typing = on\n");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn nothing_happens_without_an_old_file_or_in_the_same_folder() {
        let base = temp("none");
        let (old, new) = (base.join("program"), base.join("LIPI"));
        std::fs::create_dir_all(&old).unwrap();
        assert!(migrate(&old, &new).is_empty());
        assert!(!new.exists(), "no folder is made when there is nothing to move");

        std::fs::write(old.join(FILE_NAME), "start_typing = off\n").unwrap();
        assert!(migrate(&old, &old).is_empty());
        assert!(old.join(FILE_NAME).exists());
        let _ = std::fs::remove_dir_all(&base);
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
