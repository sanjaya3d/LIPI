// LIPI (ලිපි): type Singlish in any app and get Sinhala.
//
//   cargo run                     open the window (Windows)
//   cargo run -- --legacy         start with legacy (FM Abhaya) output chosen
//   cargo run -- --console        no window, log lines in this console
//   cargo run -- --test           terminal tester: type Singlish, see both outputs
//   cargo run -- --chart          print the legacy check sheet
//   cargo run -- --keys           print the key hint pages (what to type for each letter)
//   cargo run -- --convert        read Sinhala Unicode from the keyboard or a pipe, write FM Abhaya text
//                                 (lipi --convert < unicode.txt > legacy.txt)
//   cargo run -- --map my.txt     use your own legacy map file (works with all of the above)
//
// Settings file (made on first run, in a folder Explorer hides):
//   %APPDATA%\LIPI\settings.txt   toggle keys, starting state, start in tray.
//                     An old settings.txt next to the program is moved there the first time.
//
// File next to the program (optional):
//   legacy_map.txt    your own legacy map. Used instead of the built-in one if it is there.
//                     (--map beats it.) Copy src/legacy_map.txt next to the program to start.
//
// A release build (cargo build --release) has no console window behind the window.
// That also means --test, --chart and --console print nothing in a release build.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[allow(dead_code)]
mod about;
#[allow(dead_code)]
mod engine;
#[cfg(windows)]
mod gui;
#[allow(dead_code)]
mod hints;
#[allow(dead_code)]
mod icons;
#[allow(dead_code)]
mod keymap;
#[allow(dead_code)]
mod layout;
#[allow(dead_code)]
mod legacy;
#[allow(dead_code)]
mod session;
#[allow(dead_code)]
mod settings;
#[allow(dead_code)]
mod state;
#[allow(dead_code)]
mod theme;
#[cfg(windows)]
mod win;

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

const MAP_FILE_NAME: &str = "legacy_map.txt";

/// The folder the program is in. Falls back to the current folder.
fn program_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Picks the legacy map: --map file, then legacy_map.txt next to the program, then the
/// built-in one. The notes say which one was used, or why a file was skipped.
fn choose_map(args: &[String], dir: &Path) -> (legacy::LegacyMap, Vec<String>) {
    if let Some(i) = args.iter().position(|a| a == "--map") {
        let Some(path) = args.get(i + 1) else {
            eprintln!("--map needs a file name");
            exit(1)
        };
        return match legacy::LegacyMap::from_file(path) {
            Ok(map) => (map, vec![format!("Legacy map: {path}")]),
            Err(e) => {
                eprintln!("{e}");
                exit(1)
            }
        };
    }

    let file = dir.join(MAP_FILE_NAME);
    if file.is_file() {
        let shown = file.to_string_lossy().into_owned();
        return match legacy::LegacyMap::from_file(&shown) {
            Ok(map) => (map, vec![format!("Legacy map: {shown}")]),
            Err(e) => (
                legacy::LegacyMap::builtin(),
                vec![format!("Could not use {shown}: {e}"), "Legacy map: built-in".to_string()],
            ),
        };
    }
    (legacy::LegacyMap::builtin(), vec!["Legacy map: built-in".to_string()])
}

fn terminal_tester(map: &legacy::LegacyMap) {
    println!("Type Singlish and press Enter. Quit with Ctrl+C.");
    for line in io::stdin().lock().lines() {
        let line = line.unwrap_or_default();
        let unicode = engine::transliterate(&line);
        let legacy = map.convert(&unicode);
        println!("Unicode: {unicode}");
        println!("Legacy : {}", legacy.text);
        if !legacy.missing.is_empty() {
            let list: Vec<String> = legacy
                .missing
                .iter()
                .map(|c| format!("{c} (U+{:04X})", *c as u32))
                .collect();
            println!("No legacy code for: {}", list.join(", "));
        }
        let _ = io::stdout().flush();
    }
}

/// Reads Sinhala Unicode on stdin and writes the legacy text, one line for each line.
/// Characters with no legacy code are listed on stderr.
fn convert_stdin(map: &legacy::LegacyMap) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap_or_default();
        let converted = map.convert(&line);
        let _ = writeln!(out, "{}", converted.text);
        if !converted.missing.is_empty() {
            let list: Vec<String> =
                converted.missing.iter().map(|c| format!("{c} (U+{:04X})", *c as u32)).collect();
            eprintln!("No legacy code for: {}", list.join(", "));
        }
    }
}

/// Reads settings.txt (from the LIPI folder in AppData) and makes it the settings for the
/// whole program. Sets the starting state of the typing switch.
/// Returns true if legacy output should start.
fn apply_settings(args: &[String], program: &Path) -> bool {
    let dir = settings::folder(program);
    let mut notes = settings::migrate(program, &dir);
    let (chosen, loaded) = settings::load(&dir);
    notes.extend(loaded);
    notes.push(format!("Settings file: {}", dir.join(settings::FILE_NAME).display()));
    for note in notes {
        state::log(note);
    }
    state::set_enabled(chosen.start_typing);
    let legacy_output = args.iter().any(|a| a == "--legacy") || chosen.start_legacy;
    settings::install(chosen);
    legacy_output
}

#[cfg(windows)]
fn start_console(map: legacy::LegacyMap, legacy_output: bool) {
    let chosen = settings::get();
    state::set_legacy(legacy_output);
    println!("{} is running. It works in every app, including this window.", about::NAME);
    println!("  Tap {:<11} Sinhala typing on or off", settings::key_name(chosen.on_off_key));
    println!("  Tap {:<11} switch between Unicode and legacy output", settings::key_name(chosen.output_key));
    println!("  Quit             press Ctrl+C in this window");
    println!();
    let _hook = win::start(map);
    loop {
        for line in state::take_logs() {
            println!("{line}");
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[cfg(not(windows))]
fn start_console(_map: legacy::LegacyMap, _legacy_output: bool) {
    eprintln!("The keyboard hook works on Windows only. Use --test or --chart on this system.");
}

#[cfg(windows)]
fn start_window(map: legacy::LegacyMap, legacy_output: bool) {
    gui::run(map, legacy_output);
}

#[cfg(not(windows))]
fn start_window(_map: legacy::LegacyMap, _legacy_output: bool) {
    eprintln!("The window and keyboard hook work on Windows only. Use --test or --chart on this system.");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = program_dir();
    let (map, map_notes) = choose_map(&args, &dir);

    if args.iter().any(|a| a == "--chart") {
        print!("{}", map.chart());
    } else if args.iter().any(|a| a == "--keys") {
        print!("{}", hints::sheet());
    } else if args.iter().any(|a| a == "--convert") {
        convert_stdin(&map);
    } else if args.iter().any(|a| a == "--test") {
        for note in &map_notes {
            eprintln!("{note}");
        }
        terminal_tester(&map);
    } else {
        // The window and console modes use settings.txt. The two above do not need it.
        let legacy_output = apply_settings(&args, &dir);
        for note in map_notes {
            state::log(note);
        }
        if args.iter().any(|a| a == "--console") {
            start_console(map, legacy_output);
        } else {
            start_window(map, legacy_output);
        }
    }
}
