# LIPI v1.0.2

Type Singlish in any app and get Sinhala, live, as you type.

This update makes FM Abhaya (legacy) typing much easier, moves the settings file out of sight, and adds your contact links to the window.

## What's new

- **Punctuation in legacy mode.** In FM Abhaya mode, keys such as `?` `.` `:` `%` `!` `(` `)` now type the correct FM Abhaya codes. You can write full sentences without switching back to Unicode. This is made for the US keyboard layout.
- **Settings file moved.** `settings.txt` now lives in `%APPDATA%\LIPI`, so no file is left next to `lipi.exe`. To open the folder, press Win + R, type `%APPDATA%\LIPI` and press Enter. If you already have a `settings.txt` next to the exe, LIPI moves it there the first time you run this version, so your keys and choices stay the same.
- **New footer buttons.** Three round icon buttons under the developer name: GitHub, Facebook and YouTube. Click one to open that page in your browser.

## Fixed

- In legacy mode, `!` typed an extra `^` before `æ`. It now types only `æ`.

## Download

Download `lipi.exe` below and run it. You do not need to install anything. `lipi-windows.zip` has the same exe in a zip file.

## Updating

Close LIPI, replace the old `lipi.exe` with the new one, and run it. Your settings carry over.

## Notes

- Windows only.
- The exe is not code-signed, so Windows may show a blue "Windows protected your PC" window. Click **More info**, then **Run anyway**. The whole source code is in this repository.
- In an app that runs as administrator, run LIPI as administrator too.
- Turn Sinhala typing off (tap Right Ctrl) before you type a password.
- License: GPL-3.0-or-later.
