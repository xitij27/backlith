# Research and decisions

Started on 22 November 2026. Add to this file whenever something is decided, found out or said by the author, and commit it with the work.

## The idea

Make the desktop background show the calendar: this week's events from all of the author's calendars, and how busy each day will be. The author wants it as the background, not as widgets: "As background, I won't need to worry too much about the placement." The app owns the layout. The author was open to widgets if they turned out better; they did not.

## What the author said

- It is mainly for Windows. The author's own machine runs Ubuntu.
- A local app running in the background, the way Deskflow does, is fine.
- A browser tab that has to stay open is not: people close tabs.
- Start with Windows, and with Lively as the host.
- "Calendar wallpaper" as a name "sounds so basic"; the author asked for something more exciting.

## Name

**Weekscape** (folder `weekscape`), chosen by Claude on 22 November 2026 after the author turned down `calendar-wallpaper`: the week, laid out as a landscape on the desktop. It also hints at a possible look, with busy days as peaks.

## What already exists (searched 22 November 2026)

- **Conky + gcalcli** (Linux): draws an agenda and a 3-week grid on the desktop, refreshed every few minutes. Google Calendar only, terminal-style text, mostly X11.
- **Lively Wallpaper** (Windows, open source, GPL-3.0): shows any web page as the wallpaper through WebView2. Maintained; version 2.2.x came out in 2025–26. The community has clock and weather wallpapers but no good calendar one.
- **DeskCal** (github.com/avataru/DeskCal): a web page used as wallpaper, with a month view and Google Calendar events marked. The closest match found. Licensed CC BY-NC-SA. Whether it is still maintained was not checked.
- **Rainmeter** (Windows): skins pinned to the desktop. The widget style this project avoids.
- **WallCal** (macOS, paid, closed source): calendar on the wallpaper. Shows the idea has demand.
- **Wayland shells** (Quickshell, eww, AGS): can draw on the background layer, but not on GNOME.
- **How busy the week is**: only paid tools (Reclaim, Clockwise, Google Calendar Time Insights). The open-source side has booking tools and free-busy-cal, an abandoned PHP script.

Conclusion: no maintained open-source tool merges all calendars onto the wallpaper with a view of the week's load. The gap is real.

## Decisions

1. **Background, not widgets.** One full-screen layout: nothing to place, one theme, one refresh. A static image cannot be clicked, and being hidden under windows is the same for widgets anyway.
2. **Windows first, with Lively as the host.** Lively is a tray app that starts with Windows. It draws the page with its own engine, in a window behind the desktop icons, so there is no tab or window anyone can close by accident. It pauses the wallpaper while a window is maximised or full screen. It can pass clicks on the bare desktop to the page, and a `LivelyProperties.json` file gives a settings panel.
3. **No server.** A script runs every 10 minutes from Task Scheduler, fetches the calendars, writes `data.js` beside the page and exits. The page loads `data.js` with a `<script>` tag, reloads itself every 10 minutes, and moves the "now" line with its own clock in between.
   - Why not let the page fetch the calendars: Google and Outlook ICS servers do not allow requests from other sites, so they fail in the page. And the secret links must stay out of the page.
   - Why a `<script>` tag and not `fetch("data.json")`: a page opened from a local file usually cannot fetch the files beside it; a script tag still works, however Lively opens the file.
   - Lively may copy imported wallpapers into its own library folder. Check which folder it actually loads from, and have the script write there.
   - What this gives up: no refresh on click; and Google or Microsoft login cannot run inside a page opened from a file, so the script does the login once and keeps the token.
4. **PNG fallback.** On each run the script also draws the page to a PNG and sets it as the normal Windows wallpaper. If Lively quits or crashes, the desktop still shows a recent week. Use PNG, because Windows recompresses JPEG wallpapers. Windows Spotlight and slideshow must be off, or they replace the image.
5. **Own tray app later, maybe.** A small Tauri app (same WebView2 engine) could replace Lively and the scheduled task with one install. The trick for placing a window behind the desktop icons is not documented by Microsoft, and Windows updates can break it. Lively already deals with that, so it comes first.

## Calendar data

- **First:** ICS links, with no login. Google's "Secret address in iCal format", Outlook's "Publish calendar", iCloud's public calendar links.
- **Later:** Google Calendar API, Microsoft Graph, CalDAV (iCloud, Fastmail, Nextcloud). Each can return busy and free times directly: Google `freeBusy`, Graph `getSchedule`, CalDAV free-busy-query (RFC 4791).
- Repeating events must be expanded. Plain ICS parsers often do not do this; for example Python needs `recurring-ical-events` on top of `icalendar`, and in JavaScript `ical.js` can expand them.
- A work Microsoft 365 account may forbid publishing an ICS link. Graph then needs an app registration, and possibly an administrator's consent.
- A secret ICS link gives anyone who has it full read access to that calendar. Keep it like a password, in a git-ignored config file.

## Ideas for "how busy"

- Booked hours compared with working hours, for each day.
- The longest free block in each day.
- How many meetings run back to back.

## Windows details to remember

- The scheduled task must run as the signed-in user ("Run only when user is logged on"). A task that runs whether or not the user is logged on runs outside the desktop session and cannot set the wallpaper.
- A console program started by Task Scheduler flashes a console window each run. Use a windowless launcher, such as `pythonw.exe` for Python, or build the script as a GUI-subsystem executable.
- Setting the wallpaper: `IDesktopWallpaper::SetWallpaper` (one image per monitor) or `SystemParametersInfo(SPI_SETDESKWALLPAPER)`.
- Drawing the PNG: a headless browser screenshot of the same page. Microsoft Edge is on every Windows 10 and 11 machine and can do this headless, so nothing extra has to be installed. Playwright is the alternative.
- Desktop icons sit on top of the wallpaper. Keep a column clear in the layout.

## Ubuntu notes, for later

- The author's machine (22 November 2026): Ubuntu 24.04.5 LTS, GNOME 46, X11 session, one 1920×1080 screen, dark mode.
- Ubuntu 25.10 dropped the GNOME session on X11, and 26.04 LTS is Wayland only. Avoid X11-only tricks (Conky as a desktop window, `xwinwrap`). GNOME on Wayland has no background layer for apps, so use the image route.
- Set both `org.gnome.desktop.background picture-uri` and `picture-uri-dark`: in dark mode GNOME reads the dark one.
- GNOME may not reload an image when its path is unchanged. Switch between two filenames.
- Use a systemd user timer, not cron: cron jobs cannot reach the session bus, so `gsettings` fails there.
- Ubuntu's Chromium is a snap, and snaps cannot read hidden folders such as `~/.cache`. Use Playwright's own Chromium.
- A live version on GNOME would be a GNOME Shell extension drawing into the background. The extension API changes with each GNOME release.

## Open questions

- Which language for the script. TypeScript on Node is a candidate, since the author works in it and the page is JavaScript anyway; Python, PowerShell and C# are the others.
- Which calendars to read: Google, Outlook (personal or work), iCloud, others.
- Working hours, and what counts as busy.
- How the wallpaper looks.

## Sources

- Lively: https://github.com/rocksdanister/lively and https://rocksdanister.com/lively/changelog
- DeskCal: https://github.com/avataru/DeskCal
- gcalcli and Conky: https://raw.githubusercontent.com/insanum/gcalcli/master/docs/README.md
- Google Calendar on the Linux desktop: https://www.linuxuprising.com/2018/11/how-to-embed-google-calendar-on-your.html
- WallCal: https://apps.apple.com/app/wallcal/id1581367286
- free-busy-cal: https://packagist.org/packages/mporcheron/free-busy-cal
- Free/busy in CalDAV: https://devguide.calconnect.org/Scheduling/FreeBusyAvailability
