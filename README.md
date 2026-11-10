<p align="center">
  <img src="assets/logo.png" width="128" alt="LIPI logo">
</p>

# LIPI (ලිපි)

Type Singlish in any app and get Sinhala, live, as you type.

LIPI is a small Windows program written in Rust. You type Sinhala words in English letters, for example `ammaa`, and LIPI writes අම්මා in the app you are using: Photoshop, Premiere Pro, Blender, Word, a browser and so on.

## Features

- Works in any app, because it watches the keyboard for the whole system.
- Two output types: Unicode, and FM Abhaya (legacy) for apps that cannot show Unicode Sinhala properly.
- The word is redrawn after every key, and Backspace works one English letter at a time.
- One small exe. No installer, no extra libraries (Rust standard library only). About 1.8 MB of RAM on the author's PC.
- A key hint page inside the window (the Keys button) and `lipi --keys` for the terminal.
- You can use your own FM Abhaya map file.

## Quick start

1. Run `lipi.exe`. The window opens.
2. Tap **Right Ctrl** to switch Sinhala typing on or off.
3. Tap **Right Shift** to switch between Unicode and legacy (FM Abhaya) output.
4. Type in any app. For example `ballaa` gives බල්ලා.

You can change both keys, and what the program starts with, in `settings.txt`. The program creates this file next to the exe the first time it runs.

## How to type

- Keys are case-sensitive: `ta` gives ට and `Ta` gives ඨ, `na` gives න and `Na` gives ණ.
- A consonant on its own gets a hal: `k` gives ක්, and `ka` gives ක.
- Two consonants in a row join up. `y` and `r` after another consonant make yansaya and rakaransaya: `kya` gives ක්‍ය and `kra` gives ක්‍ර.
- `x` or `zn` gives ං, `H` gives ඃ and `X` gives ඞ. Type them after a Sinhala letter: `kax` gives කං, `kaH` gives කඃ and `kaX` gives කඞ.
- A capital letter that no key uses is read as lowercase, so `Kamal` gives කමල්.
- Anything LIPI does not know, such as digits and punctuation, is passed through as typed.

### Key chart

This is the output of `lipi --keys`.

```
== Vowels ==
අ a          ආ aa         ඇ A          ඈ Aa
ඉ i          ඊ ii         උ u          ඌ uu
ඍ R          ඎ Ru         එ e          ඒ ee
ඓ ai         ඔ o          ඕ oo         ඖ au
Also: AA = ඈ, ou = ඖ. Straight after a consonant, use the Signs tab.

== Consonants ==
ක ka         ඛ kha        ග ga         ඝ gha        ඟ zga
ච cha        ඡ chha       ජ ja         ඦ zja        ඤ zka
ඥ zha        ට ta         ඨ Ta         ඩ da         ඪ Da
ණ Na         ඬ zda        ත tha        ථ thha       ද dha
ධ dhha       න na         ඳ zdha       ප pa         ඵ pha
බ ba         භ bha        ම ma         ඹ Ba         ය ya
ර ra         ල la         ළ La         ව wa         ශ sha
ෂ Sha        ස sa         හ ha         ෆ fa
Also: q = ද, zq = ඳ, v = ව, S = ෂ. A letter on its own gets a hal: k = ක්

== Signs ==
ක ka         කා kaa        කැ kA         කෑ kAa
කි ki         කී kii        කු ku         කූ kuu
කෘ kru        කෲ kruu       කෙ ke         කේ kee
කෛ kai        කො ko         කෝ koo        කෞ kau
Type them after a consonant. Also: AA = කෑ

== Examples ==
ක් k          කං kax
කං kazn       කඃ kaH
කඞ kaX        ක්‍ය kya
ක්‍ර kra        මම mama
අම්මා ammaa      ගෙදර gedhara
නුවර nuwara     කොළඹ koLaBa
සිංහල sixhala    ප්‍රශ්න prashna
Capitals matter: ta = ට, Ta = ඨ. x, zn, H and X are typed after a letter.
```

## Output types

**Unicode** is normal Sinhala text. It works in modern apps and websites.

**Legacy (FM Abhaya)** is for apps that cannot shape Unicode Sinhala. LIPI turns the Sinhala into FM Abhaya character codes using a conversion table. The text looks right when the font is FM Abhaya, so that font must be installed.

To use your own table, copy `src/legacy_map.txt` next to the exe, edit it, and restart the program. You can also start with `lipi --map my.txt`.

To convert a text file from Unicode to FM Abhaya:

```
lipi --convert < unicode.txt > legacy.txt
```

## Settings

`settings.txt` is created next to the exe on the first run:

```
# Tap this key alone to switch Sinhala typing on or off.
on_off_key = RightCtrl

# Tap this key alone to switch between Unicode and legacy output.
output_key = RightShift

# What the program starts with.
start_typing = on
start_output = unicode

# yes: start hidden in the tray. no: open the window.
start_in_tray = no
```

Comments must be on their own lines, starting with `#`.

Choices for the two keys: `RightCtrl`, `LeftCtrl`, `RightShift`, `LeftShift`, `RightAlt`, `LeftAlt`, `ScrollLock`, `Pause`. They must be different keys. Change a value, save, and restart the program.

## Command line

| Command | What it does |
| --- | --- |
| `lipi` | Opens the window |
| `lipi --legacy` | Starts with legacy output |
| `lipi --console` | No window, log lines in the console |
| `lipi --test` | Terminal tester: type Singlish, see both outputs |
| `lipi --keys` | Prints the key chart |
| `lipi --chart` | Prints the legacy check sheet |
| `lipi --convert` | Converts Unicode text from the keyboard or a pipe to FM Abhaya |
| `lipi --map my.txt` | Uses your own legacy map (works with all of the above) |

A release build has no console window, so `--test`, `--chart`, `--keys` and `--console` print nothing in it. Run them with `cargo run -- --keys` instead.

## How it works

1. A Windows keyboard hook sees each key press before the app does.
2. LIPI keeps the English letters of the word you are typing. The word ends at a space, punctuation, Enter, an arrow key, a mouse click or a change of window. Shortcuts with Ctrl, Alt or the Windows key are left alone.
3. After every key, the whole word is converted again with the rules above.
4. The app never receives your English letter. LIPI sends it the smallest change instead: a few backspaces, then the new Sinhala characters.

## Privacy

LIPI has no network code and sends nothing anywhere. The Show Logs window keeps the last 500 lines in memory only, and nothing is saved to a file. The only file the program writes is `settings.txt`.

## Limits

- Windows only.
- In an app that runs as administrator, LIPI must also run as administrator, or Windows will block it.
- Turn Sinhala typing off (tap Right Ctrl) before you type a password, or the letters will be converted too.
- The exe is not code-signed, so some antivirus programs may warn about it. The source code is here, so you can read it and build it yourself.
- FM Abhaya output needs the FM Abhaya font in the app you paste into.

## Build

Install Rust with the MSVC toolchain, then run this in the project folder:

```
cargo build --release
```

The program is `target\release\lipi.exe`. The exe icon comes from `lipi.ico` and is added at build time. To run the tests:

```
cargo test
```

## Project layout

| File | What it does |
| --- | --- |
| `src/engine.rs` | Singlish to Sinhala Unicode, and the word composer |
| `src/legacy.rs`, `src/legacy_map.txt` | Sinhala Unicode to FM Abhaya |
| `src/session.rs` | The word being typed, for either output type |
| `src/keymap.rs` | What each key press means |
| `src/win.rs` | The Windows keyboard hook |
| `src/settings.rs`, `src/state.rs` | Settings and shared state |
| `src/gui.rs`, `src/layout.rs`, `src/icons.rs`, `src/theme.rs` | The window, drawn by hand with no libraries |
| `src/hints.rs` | The key hint page |
| `src/about.rs` | Name and contact details |
| `build.rs` | Adds the icon to the exe |

## Author

Sanjaya3D

- Facebook: [facebook.com/sanjaya3d](https://facebook.com/sanjaya3d)
- GitHub: [github.com/sanjaya3d](https://github.com/sanjaya3d)
