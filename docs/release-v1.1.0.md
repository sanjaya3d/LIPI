# LIPI v1.1.0

Type Singlish in any app and get Sinhala, live, as you type.

This update adds a tray menu, a Settings page with run on startup, and a button to check for updates. LIPI also never opens two windows now.

## What's new

- **Tray menu.** Right-click the LIPI icon near the clock to get **Open LIPI**, **Sinhala typing** (ticked when it is on) and **Exit**. A left click still opens the window.
- **One window only.** If LIPI is already running and you start it again, the first window comes to the front. If it cannot be found, a message tells you to look in Hidden icons (the ^ arrow near the clock).
- **Settings page.** Click the gear icon in the title bar. It has two switches:
  - **Run on startup** starts LIPI when Windows starts. It is a normal startup entry for your user only, and you can also see it in Task Manager under Startup apps.
  - **Start in tray** hides the window when LIPI starts.
- **Check for updates.** The Settings page shows the version number and a button that opens the newest release page on GitHub. LIPI itself still never connects to the internet.
- **Low memory.** LIPI gives unused memory back to Windows, so it stays small while it sits in the tray.
- **Smart quotes card.** The **වැඩි විස්තර** button now opens the Smart quotes section of the website.
- **Website and README.** A new example word, a section called "Is the download safe?", and a note that LIPI is built using Rust.

## Download

Download `lipi.exe` below and run it. You do not need to install anything. `lipi-windows.zip` has the same exe in a zip file.

## Updating

Close LIPI (right-click the tray icon, then **Exit**), replace the old `lipi.exe` with the new one, and run it. Your settings carry over.

## Notes

- Windows only.
- The exe is not code-signed, so Windows may show a blue "Windows protected your PC" window. Click **More info**, then **Run anyway**. The whole source code is in this repository.
- In an app that runs as administrator, run LIPI as administrator too.
- Turn Sinhala typing off (tap Right Ctrl) before you type a password.
- License: GPL-3.0-or-later.
