# Backlith v1 design

Date: 22 November 2026. Status: draft, waiting for the author's approval.

Background and research: `docs/decisions.md`. Calendar mockups: `docs/mockups/`.

## Intent

Backlith turns the Windows desktop background into a working surface. It holds tools, laid out by the app as one background rather than as floating widgets, so nothing has to be placed by hand. The first tool is a week calendar that merges all of a person's calendars and shows how busy each day is going to be.

Why: the author wants to see the week, and how loaded it is, at a glance, every time the desktop shows. In the author's words, it should be "something which improves the productiveness of your wallpaper in the background". It is meant for other people too, including people with many desktop icons, who must be able to organise their icons so the calendar stays readable.

## Goals for v1

1. **A planner app.** A native Windows app showing the main screen at small scale, with the real desktop icons where they sit. The user places and resizes tool blocks, draws icon zones, and applies the plan.
2. **Icon zones.** Areas the user draws for icons. Backlith moves icons into them and keeps them there, so icons never cover a tool.
3. **The calendar tool**, with three styles to choose from: time grid, day stacks and load skyline (the mockups). It reads calendar links (ICS addresses) and uploaded schedule files.
4. **Uploaded schedules** in a Backlith JSON format that an LLM can produce from something like a university timetable, plus uploaded `.ics` files.
5. **The wallpaper as a picture.** Backlith draws the layout to a PNG and sets it as the normal Windows wallpaper, again every 5 minutes and whenever something changes.
6. **A small tray app** that starts at sign-in, keeps the wallpaper current and keeps icons in their zones.

## Non-goals for v1

- Signing in with Google or Microsoft. Comes after v1 (see Constraints).
- Live mode, meaning a clickable or animated wallpaper drawn by a window behind the icons. Comes later, as an option.
- More than one monitor. v1 plans and draws the main screen only. Other screens keep their wallpaper.
- Tools other than the calendar and icon zones. The layout must make room for them later (see Layout model).
- Editing calendar events. Backlith only reads them.
- Ubuntu or macOS builds, the Microsoft Store, and automatic updates.

## Anti-goals (failure even if it works)

- Needing a browser tab, or a second app such as Lively, to work.
- A noticeable background cost. The tray part must not hold a web engine open between redraws.
- Losing or scrambling someone's icons. Backlith only moves icons; it never deletes, renames or moves files. Every move can be undone.
- Leaking calendar links. A secret ICS address is only ever sent to its own calendar server. It is never written to logs, layout files or the wallpaper.
- Tools hidden under icons.

## Constraints

- **Platform:** Windows 10 22H2 and Windows 11 (Claude's call). Both have the WebView2 runtime or can get it from the installer.
- **Licence:** GPL-3.0-or-later. Every dependency must be compatible. Code ported from Lively Wallpaper (GPL-3.0) is allowed.
- **Where it's built:** the author develops on Ubuntu 24.04. Windows builds run on GitHub Actions' Windows runners. Windows-only behaviour can only be tested on a real Windows PC, by the author.
- **Icons are always on top:** whatever is drawn as the wallpaper sits under the desktop icons. Tools stay readable only by keeping icons out of their way.
- **Moving icons:** Windows lets an app read and set desktop icon positions through the desktop's shell view: `IShellWindows::FindWindowSW` with `SWC_DESKTOP`, then `IFolderView` / `IFolderView2` (`GetItemPosition`, `SelectAndPositionItems`, `SetCurrentFolderFlags`). Setting positions only sticks with "Auto arrange icons" off. A new file saved to the desktop goes into Windows' next free slot.
- **Wallpaper files:** use PNG, because Windows recompresses JPEG wallpapers. Windows slideshow and Windows Spotlight replace the wallpaper, so they must be off.
- **Signing in:** Google reviews any public app that reads calendars, and until it passes only 100 people can use it, behind a warning screen. Work Microsoft 365 accounts often need IT approval. Both are why v1 uses calendar links only.

## Approaches considered

1. **One Tauri app that draws the wallpaper itself.** Chosen. Rust for the Windows work, TypeScript for the interface, Windows' built-in WebView2 engine to draw. The author asked about performance, so it runs in **picture mode**: no web engine stays running, and the wallpaper is redrawn as an image.
2. **A Tauri planner, with Lively showing the wallpaper.** Rejected. Every user would have to install and set up a second app, and Lively's own process stays running.
3. **A C#/.NET app (WinUI 3).** Rejected. It can only be built on Windows, so the author's Ubuntu machine would be out for most of the work, and C# is new to the author.
- **Electron** was dropped early: 150 MB or more, and awkward access to Windows desktop features.
- **Live mode** (a web page in a window behind the icons) was set aside for v1. It keeps 100–150 MB of RAM in use and relies on an undocumented Windows trick, which is the most fragile part of such apps.

## The design

### Pieces

- **Core (Rust, one process).** Lives in the tray. It holds the settings, fetches and parses calendars, works out the week, moves icons, asks the renderer for a new picture and sets it as the wallpaper. Idle, it should use about 20 MB of RAM or less and no CPU.
- **Planner window (TypeScript in WebView2).** Opened from the tray, and on first run. Its web engine is closed again when the window closes.
- **Renderer.** Turns a layout plus the week's data into a PNG at the screen's physical resolution, without showing any window, then exits. The same page code draws the planner's live preview, so what the user plans is exactly what the wallpaper shows. How it captures the picture is left to the builder, after the first spike (see Risks): a WebView2 capture in a hidden window, or Microsoft Edge in headless mode, which is on every Windows 10 and 11 machine.
- **Interface framework:** TypeScript with React (Claude's call; the author works in React).

### Layout model

- **Areas:** a layout is a list of items on the main screen. Each item has a kind, a rectangle and its own settings. v1 has two kinds, `calendar` (one per layout) and `iconZone` (any number).
- **Units:** rectangles are stored as fractions of the screen's work area (the screen minus the taskbar), so a layout survives a change of resolution or scaling.
- **Base:** what sits behind the tools, either a solid colour or an image. On first run the base is the user's current wallpaper, copied into Backlith's folder, so nothing they liked is lost (Claude's call).
- **New tools later:** each tool kind registers its default size, its settings and how it draws. Adding a tool kind must not change the layout format or the calendar.

### The planner

- **What it shows:** the screen at small scale, with the base, the rendered tools and the real icons. Icons can be drawn as labelled outlines; real icon images are the builder's choice.
- **First run:** the planner proposes a layout from what it finds. That's one zone on the left sized for the icons there now, and the calendar filling the rest (Claude's call).
- **Editing:**
  - Drag and resize tools and zones, snapping to Windows' icon grid. They can't overlap.
  - Drag icons between zones. A zone too small for its icons says how many don't fit and offers to grow.
  - A side panel shows the selected item's settings.
- **Apply:** before moving anything for the first time, Backlith saves where every icon was. Then it turns "Auto arrange icons" off (the planner says so first), moves the icons, draws the wallpaper and sets it.
- **Undo:** "Put my icons back" restores the saved positions. "Restore my old wallpaper" brings back the wallpaper from before Backlith. Both are in the planner and the tray.

### Icon zones

- **Look:** each zone has a name. Backlith paints a soft labelled panel behind it on the wallpaper, so the icons look boxed.
- **Order:** icons sit in a zone in the order the user gave, filling Windows' icon grid row by row.
- **New icons:** one zone is marked as the place for new icons (by default, the first one). Backlith watches the desktop folders for changes (`SHChangeNotifyRegister`), not by polling. When a file is added, it moves the new icon into that zone after a short delay.
- **Overflow:** icons that don't fit anywhere stay where Windows put them. The tray shows a notice ("3 icons don't fit in Downloads").
- **After restarts:** at start-up, and after Explorer restarts, Backlith puts icons back into their zones from its own record.
- **Both desktop folders:** the desktop shows items from the user's own Desktop folder and the shared Public Desktop. Watch both.
- **Other monitors:** icons there are left alone in v1.
- **Auto arrange:** if the user won't let Backlith turn off "Auto arrange icons", zones can't hold icons. The planner explains this and still lets them place tools.

### The calendar tool

- **Styles:** time grid, day stacks or load skyline, as in `docs/mockups/`, switchable at any time.
- **Settings:**
  - the style
  - which calendars to show, with a name and colour each
  - show the weekend
  - the length of a full day (default 8 h)
  - which week to show: this week, Monday to Sunday, by default, or the next 7 days (Claude's call)
  - show event titles: on by default. Turning it off shows only busy blocks, for people who share their screen (Claude's call).
- **How busy:** booked time is the union of a day's timed events, so overlapping events count once. All-day events don't count as hours and show as a label at the top of the day. Cancelled events, and events marked "free" (`TRANSP:TRANSPARENT`), are left out. Each day gets a level:

  | Level | Booked time, as a share of a full day |
  |---|---|
  | Light | under 37.5% |
  | Steady | 37.5% to under 75% |
  | Busy | 75% to under 100% |
  | Packed | 100% or more |

  With an 8 h day that's 3 h, 6 h and 8 h, as in the mockups.
- **Week summary:** hours booked, the busiest day and the lightest day.
- **Times:**
  - Everything shows in Windows' local time zone.
  - Events that run past midnight are split across the days they touch.
  - Today is highlighted and past events are faded.
- **Look:** the mockups' dark style is Claude's starting point, not a decision. Panels sit on a translucent dark backing, so text stays readable over any base image.

### Calendar sources

- **Calendar links (ICS addresses):** the user pastes a link and gives it a name and colour. A `webcal://` link is fetched as `https://`.
  - Backlith fetches it once to check it, and shows how many events it found.
  - The link is stored in Windows Credential Manager, never in the settings file.
  - Each link is fetched every 15 minutes, using `ETag` / `If-Modified-Since` when the server supports them.
  - If a fetch fails, the last good copy stays in use. The planner and the tray say since when ("Work calendar unreachable since 10:20").
  - Repeating events are expanded, including their exceptions, in Rust.
- **Uploaded `.ics` files:** treated like a link that never changes. The file is copied into Backlith's folder.
- **Uploaded Backlith schedules:** see the next section. Each uploaded file becomes its own calendar with its own colour and on/off switch. Uploading a file with the same `name` again asks whether to replace the old one.

### The schedule format (version 1)

- **Who it's for:** written by an LLM or by hand. Published in the repo as a JSON Schema at `schemas/schedule-v1.json`.
- **Top level:**
  - `backlith`: required, always `1`.
  - `name`: required.
  - `timezone`: an IANA zone name such as `Europe/Berlin`. Optional; defaults to Windows' zone.
  - `events`: required, at least one.
- **Each event:**
  - `title`: required. `location` and `notes` are optional.
  - Either a one-off `date` (`YYYY-MM-DD`), or a weekly repeat: `days` (from `mon` to `sun`), `from` and `until` (dates, both included), plus an optional `every`, the number of weeks between repeats (default 1).
  - `start` and `end` (`HH:MM`, 24-hour clock), with `end` after `start`, or `allDay: true`.
  - `skip`: optional dates or date ranges (`YYYY-MM-DD` or `YYYY-MM-DD/YYYY-MM-DD`).
  - An event that runs past midnight must be written as two events in v1.
- **Strict checking:** unknown fields are errors, with a "did you mean" hint, so an LLM's typos are caught.
  - Every error names the event by its number and title, for example "Event 3 (Databases lab): `end` 09:00 is before `start` 10:00".
  - A file may be at most 1 MB.
- **Planner buttons:** "Copy prompt for an LLM" copies a ready prompt: the rules, the schema and a worked example like the one below. "Copy errors" copies the problems found, for the LLM to fix.

```json
{
  "backlith": 1,
  "name": "University timetable, winter 2026/27",
  "timezone": "Europe/Berlin",
  "events": [
    {
      "title": "Machine Learning (lecture)",
      "location": "Room A1",
      "days": ["mon", "thu"],
      "start": "10:00",
      "end": "11:40",
      "from": "2026-10-19",
      "until": "2027-02-12",
      "skip": ["2026-12-21/2027-01-03"]
    },
    {
      "title": "Exam: Databases",
      "date": "2027-02-18",
      "start": "09:00",
      "end": "11:00"
    }
  ]
}
```

### When the wallpaper is redrawn

- every 5 minutes, so the "now" line and the fading of past events keep up
- right after any change in the planner, an upload, or new calendar data
- at midnight
- after a change of resolution or scaling
- after the PC wakes from sleep

If the new picture is identical to the last one, Backlith doesn't set it again. While the session is locked it doesn't redraw at all.

To set the wallpaper, Backlith writes `wallpaper-a.png` and `wallpaper-b.png` in turn and calls `IDesktopWallpaper::SetWallpaper` for the main screen. If the wallpaper is found replaced (by slideshow, Spotlight or the user), Backlith pauses and says so in the tray instead of fighting it.

### Tray, start-up and install

- **Tray menu:** Open planner, Redraw now, Pause, Put my icons back, Restore my old wallpaper, Quit.
- **Start-up:** Backlith starts at sign-in through the current user's `Run` registry key, with a setting to turn that off.
- **Quitting:** the last picture simply stays as the wallpaper.
- **Install:** a per-user installer that needs no administrator rights (Tauri's NSIS bundle), carrying the WebView2 bootstrapper for machines without it. It's published on GitHub Releases, unsigned in v1, so Windows SmartScreen will warn on first install (Claude's call; signing costs money).
- **Uninstall:** offers to restore the old wallpaper and put the icons back.
- **Files:** settings, uploaded files, the calendar cache and the wallpaper pictures live in `%APPDATA%\Backlith\`.
- **Language:** the interface is in English only in v1 (Claude's call). Dates and times follow Windows' regional format.

### Performance targets (Claude's call)

- Tray process idle: at most about 20 MB of RAM, about 0% CPU, no web engine running.
- One redraw: finished within 3 seconds, peaking at most about 250 MB including the web engine, which then exits.
- The planner, while open: like any small app. Its engine closes with the window.

The real figures get measured on the author's Windows PC.

### Building and testing

- **Split:** everything that isn't Windows-specific goes in its own Rust crate with unit tests that run on Linux. That's ICS parsing and repeat expansion, the busy sums, the week maths, schedule checking and layout maths.
- **Windows parts:** icons, setting the wallpaper and the desktop change watch sit behind an interface. A fake version on Linux lets the planner run on Ubuntu with pretend icons.
- **CI:** GitHub Actions runs the tests on Linux and builds the Windows installer on a Windows runner.
- **Manual testing:** Windows behaviour is checked by the author against a written checklist kept in the repo.

## Risks, and the first spikes

1. **Drawing the picture without showing a window:** compare a hidden WebView2 capture with headless Edge, for speed, memory and reliability. This decides the renderer.
2. **Moving icons on current Windows 11 builds:** check that `IFolderView` reads and sets positions, turns off auto-arrange and survives an Explorer restart.
3. **Setting the wallpaper every 5 minutes:** check for a visible fade or flicker. If it's distracting, redraw only when something visible changed.
4. **Coordinates and scaling:** match icon positions, the work area and the picture's pixels at 100%, 125% and 150% scaling.

## Left to the builder

The repo layout, crate and library choices, styling details within the mockups' direction, what a style shows when a day has more events than fit (for example "+3 more"), the smallest size a calendar block may have, how icons are drawn in the planner, the renderer's capture method (after spike 1), the exact wording of the LLM prompt and the error messages, and the test set-up.
