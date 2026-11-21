<p align="center">
  <img src="assets/banner.png" width="560" alt="LIPI (ලිපි)">
</p>

<p align="center">
  <b>Type Singlish in any app and get Sinhala, live, as you type.</b>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/LICENSE-GPLv3%2B-BEF92C?style=for-the-badge&labelColor=2b3035" alt="License: GPLv3+"></a>
  <a href="https://github.com/sanjaya3d/LIPI/releases/latest"><img src="https://img.shields.io/github/v/release/sanjaya3d/LIPI?style=for-the-badge&labelColor=2b3035&color=BEF92C&label=RELEASE" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/PLATFORM-WINDOWS-BEF92C?style=for-the-badge&labelColor=2b3035" alt="Platform: Windows">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/BUILT%20WITH-RUST-BEF92C?style=for-the-badge&labelColor=2b3035&logo=rust&logoColor=white" alt="Built with Rust">
  <img src="https://img.shields.io/badge/DEPENDENCIES-NONE-BEF92C?style=for-the-badge&labelColor=2b3035" alt="Dependencies: none">
  <a href="https://github.com/sanjaya3d/LIPI/stargazers"><img src="https://img.shields.io/github/stars/sanjaya3d/LIPI?style=for-the-badge&labelColor=2b3035&color=BEF92C&label=STARS&logo=github&logoColor=white" alt="GitHub stars"></a>
</p>

<p align="center">
  <a href="https://github.com/sanjaya3d/LIPI/releases/latest">
    <img src="assets/download-button.svg" width="420" alt="Download the latest LIPI release for Windows">
  </a>
</p>

<p align="center">
  <a href="#features">Features</a> &nbsp;·&nbsp;
  <a href="#quick-start">Quick start</a> &nbsp;·&nbsp;
  <a href="#how-to-type">How to type</a> &nbsp;·&nbsp;
  <a href="#settings">Settings</a> &nbsp;·&nbsp;
  <a href="#faq">FAQ</a> &nbsp;·&nbsp;
  <a href="#build-from-source">Build</a>
</p>

Type `ammaa` and LIPI writes අම්මා in Photoshop, Premiere Pro, Blender, Word, your browser and any other Windows app. The output can be Unicode or FM Abhaya.

> **ලිපි** යනු ඉංග්‍රීසි අකුරින් (Singlish) ටයිප් කරන දේ ඒ මොහොතේම සිංහල අකුරු බවට හරවන කුඩා Windows වැඩසටහනකි. උදාහරණයක් ලෙස `ammaa` ටයිප් කළ විට අම්මා ලැබේ. Photoshop, Premiere Pro, Blender වැනි ඕනෑම app එකක මෙය වැඩ කරයි. Unicode සහ FM Abhaya (legacy) යන දෙවර්ගයෙන්ම සිංහල ලිවිය හැක.

<p align="center">
  <img src="assets/hero.png" width="100%" alt="Four example words, ammaa, ballaa, sixhala and koLaBa, turning into Sinhala, next to the LIPI window">
</p>

<p align="center"><sub>Illustration (generated for this README, not a screenshot): four example words and the LIPI window.</sub></p>

## Features

<table>
  <tr>
    <td width="33%" valign="top"><b>Works in any app</b><br>LIPI watches the keyboard for the whole system, so it works wherever you can type.</td>
    <td width="33%" valign="top"><b>Unicode or FM Abhaya</b><br>Switch with one key tap. In legacy mode the punctuation keys type the right codes too (US keyboard).</td>
    <td width="33%" valign="top"><b>Live typing</b><br>The word is redrawn after every key, and Backspace works one English letter at a time.</td>
  </tr>
  <tr>
    <td valign="top"><b>Tiny and private</b><br>One small exe, no installer, no libraries. About 2 MB of RAM. No network code at all.</td>
    <td valign="top"><b>Lives in the tray</b><br>Left click the tray icon to open the window. Right click to switch Sinhala typing on or off.</td>
    <td valign="top"><b>Key hints built in</b><br>The <b>Keys</b> button shows what to type for each letter. <code>lipi --keys</code> prints the same chart.</td>
  </tr>
</table>

You can also use your own FM Abhaya map file. See [Output types](#output-types).

## Quick start

1. Download `lipi.exe` from the [latest release](https://github.com/sanjaya3d/LIPI/releases/latest) and run it. The window opens. If Windows shows a blue warning, see the [FAQ](#faq).
2. Tap **Right Ctrl** to switch Sinhala typing on or off.
3. Tap **Right Shift** to switch between Unicode and legacy (FM Abhaya) output.
4. Type in any app. For example `ballaa` gives බල්ලා.

You can change both keys, and what the program starts with, in [`settings.txt`](#settings).

## How to type

- Keys are case-sensitive: `ta` gives ට and `Ta` gives ඨ, `na` gives න and `Na` gives ණ.
- A consonant on its own gets a hal: `k` gives ක්, and `ka` gives ක.
- Two consonants in a row join up. `y` and `r` after another consonant make yansaya and rakaransaya: `kya` gives ක්‍ය and `kra` gives ක්‍ර.
- `x` or `zn` gives ං, `H` gives ඃ and `X` gives ඞ. Type them after a Sinhala letter: `kax` gives කං, `kaH` gives කඃ and `kaX` gives කඞ.
- A capital letter that no key uses is read as lowercase, so `Kamal` gives කමල්.
- Anything LIPI does not know, such as digits and punctuation, is passed through as typed.

<details>
<summary><b>Key chart</b> (the output of <code>lipi --keys</code>)</summary>

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

</details>

## Output types

**Unicode** is normal Sinhala text. It works in modern apps and websites.

**Legacy (FM Abhaya)** is for apps that cannot shape Unicode Sinhala. LIPI turns the Sinhala into FM Abhaya character codes using a conversion table. The text looks right when the font is FM Abhaya, so that font must be installed.

To use your own table, copy `src/legacy_map.txt` next to the exe, edit it, and restart the program. You can also start with `lipi --map my.txt`.

To convert a text file from Unicode to FM Abhaya:

```
lipi --convert < unicode.txt > legacy.txt
```


## Smart quotes (Adobe and Microsoft apps)

Legacy (FM Abhaya) මාදිලියේදී, සමහර මෘදුකාංගවල තිත (`.`) වෙනුවට අමුතු වක්‍ර සලකුණක් දිස්විය හැකියි. LIPI මඟින් නිවැරදි අක්ෂරය ඇතුළත් කළත්, මෘදුකාංගය විසින් එය පසුව ස්වයංක්‍රීයව වෙනස් කරයි. එක් සැකසුමක් අක්‍රිය කිරීමෙන් මෙම ගැටලුව විසඳාගත හැකියි.

> Adobe සහ Microsoft මෘදුකාංගවලදී "Smart Quotes" විශේෂාංගය ක්‍රියා විරහිත කරන්න.

**මෙය සිදුවන්නේ ඇයි?** FM Abhaya අකුරු රටාවේදී තිත (`.`) සඳහා භාවිත වන්නේ සාමාන්‍ය සෘජු apostrophe (`'`) සලකුණට අදාළ අක්ෂර කේතයයි. ඒ නිසා LIPI මඟින් `.` යතුර එබූ විට එම අක්ෂරය ඇතුළත් වේ. Photoshop, Premiere Pro, Word, PowerPoint සහ තවත් සමහර මෘදුකාංගවල ඇති "Smart Quotes" විශේෂාංගය මඟින් ඔබ ටයිප් කරන සෘජු apostrophe (`'`) සලකුණ වක්‍ර apostrophe (`‘` හෝ `’`) සලකුණක් බවට ස්වයංක්‍රීයව වෙනස් කරයි. FM Abhaya අකුරු රටාවේදී එම වක්‍ර apostrophe කේතයට වෙනත් හැඩයක් ලබා දී ඇති නිසා, තිත වෙනුවට වක්‍ර සලකුණක් දිස්වේ. Notepad සහ බොහෝ සාමාන්‍ය පෙළ ඇතුළත් කරන ස්ථානවල මෙම ගැටලුව ඇති නොවේ. Unicode ප්‍රතිදානයට මෙය බලපාන්නේද නැත.

**මෙය විසඳාගන්නේ කෙසේද?** ඔබ ටයිප් කරන මෘදුකාංගයේ Smart Quotes විශේෂාංගය අක්‍රිය කරන්න:

| මෘදුකාංගය | සැකසුම වෙනස් කළ යුතු ස්ථානය |
| --- | --- |
| Photoshop | Edit → Preferences → Type වෙත ගොස් **Use Smart Quotes** යන්නෙහි සලකුණ ඉවත් කරන්න. |
| Premiere Pro | Preferences → Graphics වෙත ගොස් Smart Quotes සම්බන්ධ සැකසුමක් සොයා බලන්න. එහි ස්ථානය සහ නම අනුවාදය අනුව වෙනස් විය හැකි අතර, සමහර අනුවාදවල මෙම සැකසුම නොතිබිය හැකියි. |
| Word සහ PowerPoint | File → Options → Proofing → AutoCorrect Options → **AutoFormat As You Type** වෙත ගොස් **"Straight quotes" with "smart quotes"** යන්නෙහි සලකුණ ඉවත් කරන්න. Word හි **AutoFormat** ටැබය තුළද එම සැකසුම අක්‍රිය කරන්න. |
| වෙනත් මෘදුකාංග | Text හෝ Type සැකසුම් තුළ "Smart Quotes" හෝ "Typographer's Quotes" යන සැකසුම් සොයා බලන්න. |

සැකසුම වෙනස් කළ පසු තිත නැවත ටයිප් කරන්න. දැනටමත් ලේඛනයේ ඇති වැරදි සලකුණු ස්වයංක්‍රීයව නිවැරදි නොවේ. එම නිසා වැරදි වක්‍ර සලකුණ මකා දමා තිත නැවත ඇතුළත් කරන්න.


The LIPI window has a card about this. Its **වැඩි විස්තර** button opens this section.

## Settings

LIPI keeps its settings in your AppData folder, which Explorer hides by default, so no settings file sits next to the exe. To open the folder, press **Win + R**, type `%APPDATA%\LIPI` and press Enter. The file `settings.txt` is created there the first time LIPI runs.

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

If you used an older version that kept `settings.txt` next to the exe, LIPI moves that file to the new folder the first time it runs.

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

## FAQ

<details>
<summary><b>Windows shows "Windows protected your PC". Is LIPI safe?</b></summary>

The exe is not code-signed, so Windows SmartScreen warns about every new program like this. Click **More info**, then **Run anyway**. LIPI has no network code, and the whole source is in this repository, so you can read it or build the exe yourself.
</details>

<details>
<summary><b>My antivirus warns about LIPI.</b></summary>

LIPI uses a Windows keyboard hook to see your key presses, the same way every typing tool does, and some antivirus programs flag that. LIPI never sends anything anywhere. If you do not trust the download, build it yourself (see [Build from source](#build-from-source)).
</details>

<details>
<summary><b>I type, but I only get English letters.</b></summary>

Check that Sinhala typing is on (the switch in the window says ON). Tap your on or off key once to change it. Also, if the app you are typing in runs as administrator, run LIPI as administrator too, or Windows will not let LIPI see the keys.
</details>

<details>
<summary><b>The text looks like strange symbols.</b></summary>

That is normal for legacy (FM Abhaya) output when the font is not FM Abhaya. Install the FM Abhaya font and choose it in the app, or switch LIPI to Unicode output.
</details>

<details>
<summary><b>In legacy mode the full stop shows as a curl in Photoshop, Premiere Pro or Word.</b></summary>

The app is changing the straight apostrophe, which FM Abhaya uses for the full stop, into a curly "smart quote". Turn smart quotes off in that app. The steps are in [Smart quotes](#smart-quotes-adobe-and-microsoft-apps).
</details>

<details>
<summary><b>Can I change the keys that switch typing and output?</b></summary>

Yes. Edit `on_off_key` and `output_key` in `settings.txt`. See [Settings](#settings).
</details>

<details>
<summary><b>How do I start LIPI when Windows starts?</b></summary>

Press **Win + R**, type `shell:startup` and press Enter. Put a shortcut to `lipi.exe` in that folder. To start hidden in the tray, set `start_in_tray = yes` in `settings.txt`.
</details>

<details>
<summary><b>How do I remove LIPI?</b></summary>

Close LIPI with the X button in its window, then delete `lipi.exe` and the folder `%APPDATA%\LIPI`. Nothing else is installed.
</details>

<details>
<summary><b>Does it work on Mac or Linux?</b></summary>

No. LIPI is Windows only, because it uses the Windows keyboard hook.
</details>

## How it works

1. A Windows keyboard hook sees each key press before the app does.
2. LIPI keeps the English letters of the word you are typing. The word ends at a space, punctuation, Enter, an arrow key, a mouse click or a change of window. Shortcuts with Ctrl, Alt or the Windows key are left alone.
3. After every key, the whole word is converted again with the rules above.
4. The app never receives your English letter. LIPI sends it the smallest change instead: a few backspaces, then the new Sinhala characters.

## Privacy

LIPI has no network code and sends nothing anywhere. The Show Logs window keeps its lines in memory only, and nothing is saved to a file. The only file the program writes is `settings.txt`.

## Limits

- Windows only.
- In an app that runs as administrator, LIPI must also run as administrator, or Windows will block it.
- Turn Sinhala typing off (tap Right Ctrl) before you type a password, or the letters will be converted too.
- The exe is not code-signed, so some antivirus programs may warn about it.
- FM Abhaya output needs the FM Abhaya font in the app you paste into.
- The legacy punctuation table is made for the US keyboard layout.
- In legacy mode, apps with "smart quotes" turned on (Photoshop, Premiere Pro, Word) can change the full stop into a curl. See [Smart quotes](#smart-quotes-adobe-and-microsoft-apps).

## Build from source

Install Rust with the MSVC toolchain, then run this in the project folder:

```
cargo build --release
```

The program is `target\release\lipi.exe`. The exe icon comes from `lipi.ico` and is added at build time. To run the tests:

```
cargo test
```

<details>
<summary><b>Project layout</b></summary>

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
| `src/about.rs` | Name and contact links |
| `build.rs` | Adds the icon to the exe |

</details>

## Contributing

Found a wrong letter or a bad FM Abhaya glyph? [Open an issue](https://github.com/sanjaya3d/LIPI/issues) with the word you typed and what you expected. Pull requests are welcome. Please run `cargo test` before you send one.

## License

LIPI is free software under the [GNU General Public License v3.0 or later](LICENSE).

<p align="center">
  If LIPI saves you time, please <a href="https://github.com/sanjaya3d/LIPI">star the repo</a> and share it with a friend.
</p>

## Author

Made by **SANJAYA3D**

<p>
  <a href="https://github.com/sanjaya3d"><img src="https://img.shields.io/badge/GITHUB-SANJAYA3D-BEF92C?style=for-the-badge&labelColor=2b3035&logo=github&logoColor=white" alt="GitHub: sanjaya3d"></a>
  <a href="https://facebook.com/sanjaya3d"><img src="https://img.shields.io/badge/FACEBOOK-SANJAYA3D-BEF92C?style=for-the-badge&labelColor=2b3035&logo=facebook&logoColor=white" alt="Facebook: sanjaya3d"></a>
  <a href="https://www.youtube.com/@SANJAYA3D/"><img src="https://img.shields.io/badge/YOUTUBE-%40SANJAYA3D-BEF92C?style=for-the-badge&labelColor=2b3035&logo=youtube&logoColor=white" alt="YouTube: SANJAYA3D"></a>
</p>
