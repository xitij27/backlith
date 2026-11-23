# Research and decisions

Started on 22 November 2026. Add to this file whenever something is decided, found out or said by the author, and commit it with the work.

## The idea

Make the desktop background show the calendar: this week's events from all of the author's calendars, and how busy each day will be. The author wants it as the background, not as widgets: "As background, I won't need to worry too much about the placement." The app owns the layout. The author was open to widgets if they turned out better; they did not.

The scope grew the same day. In the author's words: "not just a calendar tool, but really something which improves the productiveness of your wallpaper in the background. Like, it could also hold other tools, not just calendar." So the project is a background that holds tools, and the calendar is the first of them. The data file and the page should be built so more tools can be added.

## What the author said

- It is mainly for Windows. The author's own machine runs Ubuntu.
- A local app running in the background, the way Deskflow does, is fine.
- A browser tab that has to stay open is not: people close tabs.
- Start with Windows, and with Lively as the host.
- "Calendar wallpaper" as a name "sounds so basic"; the author asked for something more exciting.
- Weekscape was not exciting enough either: "Come up with something modern, something futuristic." The author picked Holodesk from a list of suggestions.
- Holodesk had to go: holodesk.com is a whole company. The author wants a name nobody has used before, and will choose from Claude's recommendations.
- Chose Backlith.
- Asked for it to be pushed as a public GitHub repository under the MIT licence.
- After hearing how MIT and GPL differ, switched to GPL the same day: "we can keep ours as GPL license too".

## Name

**Backlith** (folder `backlith`), chosen by the author on 22 November 2026: back + lith, as in monolith. The monolith behind your windows. Holodesk was chosen before it and dropped the same day, because holodesk.com is a company.

How it got there, all on 22 November 2026:

1. Claude named the folder `calendar-wallpaper`. The author: it "sounds so basic".
2. Claude renamed it Weekscape. The author: not exciting enough; "something modern, something futuristic".
3. Claude offered Precog, T-Minus, Holodesk, Strata, Kairo, Underglow, Sidereal and Timecast, with Precog as its own pick. The author chose Holodesk.
4. The author found that holodesk.com is a company, and asked for names nobody has used before. Since the project now holds more than a calendar, the name should be about the background, not about time.
5. Claude offered Backlith, Voidpane, Nyxpane, Synapane and Aetherpane, with Backlith as its pick. The author said "I like backlit more". Asked whether that meant the plain word Backlit (backlit.com is registered, and more than 100 GitHub repositories use it) or Backlith, the author chose Backlith.

Others using Holodesk (searched 22 November 2026; trademarks not checked):

- HoloDesk by Nanjing DSeeLab, on Apple's App Store: a utility for holographic 3D display screens.
- HoloDesk by Plansysteme, on Microsoft's marketplace: a 3D collaboration service. This matters if Holodesk is ever listed in the Microsoft Store.
- HoloDesk, a Microsoft Research demo from about 2011–12 (a Kinect and a see-through display).
- No GitHub repository named holodesk was found.
- holodesk.com, a company (found by the author).

New candidates, checked by Claude on 22 November 2026. "Clean" means: the exact word got no web search results, no GitHub repository has that exact name, and the .com is not registered (checked with the .com registry's RDAP service at rdap.verisign.com). Trademarks were not checked, and anyone can register a domain at any time.

- **Backlith** (clean): back + lith, as in monolith. The monolith behind your windows. Claude's pick, and the author's choice.
- **Voidpane** (clean): puts the empty desktop, the void, to work. GitHub has only Voidpanel repositories.
- **Nyxpane** (clean): Nyx, the Greek goddess of night. GitHub has only nyxpanel repositories.
- **Synapane** (clean): synapse + pane. Tools wired together on one surface.
- **Aetherpane** (.com free, no web results): one small GitHub repository uses the name, lihytaihe-lang/aetherpane, a design skill for AI coding agents with 3 stars (April 2026).

Checked and dropped:

- Halopane: .com free, but KoksalBerkay/Halopane is a spatial browser for XR glasses, made on 17 November 2026. Too close in spirit.
- Clean but about time, which fits less now: Zeitlume, Tesserday. Also with a free .com: Zeitglass, Zeitglow, Zeitgrid, Zeitpane, Weeklume, Tessaweek, Glimweek, Dayglaze, Weekglaze, Chronopane (Chronopane has 4 GitHub repositories), Subpane (too close to the common word subpanel), Kyropane, Xyrapane.
- .com already registered: Sublume, Underlume, Kronode, Horaline, Chronoveil, Lumaweek, Infrapane, Infradesk, Infralume, Desklume, Glowpane, Backlume, Underpane, Lumadeck, Nexpane, Wallsense, Deskglow, Hyperpane, Neopane, Corewall, Basedeck, Paneon, Deskeon, Opalith, Lumivane, Plexdesk, Deskvane, Prismdesk, Novapane, Holvane, Pannox, Oriva, Lyrex, Nexwall, Lumiwall, Wallion, Undrift, Weekpane, Lumapane, Holopane, Kairolume, Tempolume, Chronolume, Stratalume, Hyperweek, Timeglaze, Foreglow, Orrerium, Vantlume, Aftergrid, Undergrid, Deskaura.

## Repository and licence

- Public at https://github.com/xitij27/backlith since 22 November 2026, at the author's request. `main` is the only branch.
- Licence: GPL-3.0-or-later, copyright Kshitij Parashar (the name written as in the author's `jarvis` project). Chosen by the author on 22 November 2026.
- It was MIT for the first few hours. The commits up to `0a3a91b` were published under MIT and stay MIT for anyone who copied them then; they hold only documents, no code.
- Why GPL: changed versions that anyone gives out must stay open source, and Lively Wallpaper is GPL-3.0 too, so a future tray app can reuse Lively's code, for example its trick for placing a window behind the desktop icons.
- "Or later" is the wording the GPL itself recommends. It lets the project move to a future GPL version without asking every contributor.
- What GPL rules out: nobody can make a closed-source version, including the author, once code from other contributors or from Lively is inside. The author can still sell copies, as long as the source comes with them.
- New source files start with `SPDX-License-Identifier: GPL-3.0-or-later`.

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

Decisions 2 to 5 below were replaced on 22 November 2026 by the v1 design in `docs/superpowers/specs/2026-11-22-backlith-v1-design.md`. Backlith is now one Tauri app that draws the wallpaper as a picture. Lively, Task Scheduler and `data.js` are no longer part of the plan. They are kept here for the reasoning.

1. **Background, not widgets.** One full-screen layout: nothing to place, one theme, one refresh. A static image cannot be clicked, and being hidden under windows is the same for widgets anyway.
2. **Windows first, with Lively as the host.** Lively is a tray app that starts with Windows. It draws the page with its own engine, in a window behind the desktop icons, so there is no tab or window anyone can close by accident. It pauses the wallpaper while a window is maximised or full screen. It can pass clicks on the bare desktop to the page, and a `LivelyProperties.json` file gives a settings panel.
3. **No server.** A script runs every 10 minutes from Task Scheduler, fetches the calendars, writes `data.js` beside the page and exits. The page loads `data.js` with a `<script>` tag, reloads itself every 10 minutes, and moves the "now" line with its own clock in between.
   - Why not let the page fetch the calendars: Google and Outlook ICS servers do not allow requests from other sites, so they fail in the page. And the secret links must stay out of the page.
   - Why a `<script>` tag and not `fetch("data.json")`: a page opened from a local file usually cannot fetch the files beside it; a script tag still works, however Lively opens the file.
   - Lively may copy imported wallpapers into its own library folder. Check which folder it actually loads from, and have the script write there.
   - What this gives up: no refresh on click; and Google or Microsoft login cannot run inside a page opened from a file, so the script does the login once and keeps the token.
4. **PNG fallback.** On each run the script also draws the page to a PNG and sets it as the normal Windows wallpaper. If Lively quits or crashes, the desktop still shows a recent week. Use PNG, because Windows recompresses JPEG wallpapers. Windows Spotlight and slideshow must be off, or they replace the image.
5. **Own tray app later, maybe.** A small Tauri app (same WebView2 engine) could replace Lively and the scheduled task with one install. The trick for placing a window behind the desktop icons is not documented by Microsoft, and Windows updates can break it. Lively already deals with that, so it comes first.

## v1 design talk (22 November 2026; design approved the same day)

### What the author said

- The first feature is a native desktop app for planning the background: which tools it holds and where they sit. The calendar is the first tool.
- Tools must not be covered by desktop icons.
- The author has very few desktop icons, but Backlith must work for people with many: they should be able to plan and organise their icons so the calendar stays usable.
- The calendar should be elaborate, with each day as a column.
- All three calendar mockups should be offered to users as styles to choose from.
- Asked about performance: "I would want something which works efficiently in the background without taking too many resources." After hearing that a live web wallpaper keeps about 100–150 MB in use, chose option 1 in picture mode: one Tauri app, a small tray part, the wallpaper redrawn as an image.
- For testing on Windows, the author has a Windows tablet ("a tablet with windows should be accessible now"). Its Windows version and whether it is x64 or ARM-based are not known yet.
- How Claude reaches the tablet (found 22 November 2026):
  - Claude in Chrome: the browser extension, in Edge on the tablet, is how the author's earlier testing "happened on the tablet" with no setup. The portfolio project's notes say the same: the only browser connected to it was an Edge on Windows with a touchscreen. It drives the browser only, so it can test the planner's interface by touch, but it cannot install Backlith, move icons or set the wallpaper. On 22 November it listed no connected browser; Edge has to be open on the tablet with the extension connected.
  - The tablet and the Ubuntu laptop are on the same private network, and the tablet answers there, but it had no remote shell, Remote Desktop or PowerShell remoting open. For the Windows parts, Claude proposed turning on Windows' OpenSSH server with key-only login, limited to that network, and made a key for it on the laptop at `~/.ssh/backlith_tablet`. Not set up yet.
  - The author then chose to build first and test "on a proper Windows machine later", so the tablet set-up is on hold.
  - Commands run over SSH on Windows do not reach the signed-in desktop. Anything that changes the desktop (icons, wallpaper) has to be started inside the user's session, for example through a scheduled task that runs only when the user is logged on.
- The calendar settings and calendar links for v1 "look fine".
- People should be able to upload their own schedules in a set format, added to the calendar. The format is Claude's to decide. The case the author gave: take a university timetable, have an LLM convert it into Backlith's format, upload it, and see it in the calendar.

### Found out

- Nothing drawn as the wallpaper can sit above the desktop icons. That holds for Lively and every other wallpaper app, so tools and icons have to be kept apart.
- Windows lets an app read and set desktop icon positions, through the desktop's shell view (`IFolderView::GetItemPosition` and `IFolderView::SelectAndPositionItems`). Setting them needs "Auto arrange icons" off. A new file saved to the desktop goes into Windows' next free slot.

### Claude's proposals, which the author accepted ("that could work")

- The planner shows the screen at small scale, with the real icons where they sit.
- Each tool is a block you place and resize.
- Icon zones: you draw one or more areas for icons (for example "Shortcuts" and "Downloads"), and Backlith moves the icons into them. Tools and zones cannot overlap. A zone too small for its icons says how many don't fit and offers to grow.
- Backlith paints a soft labelled panel behind each zone on the wallpaper, so icons look boxed. Prior art: Stardock Fences, which is paid and closed source.
- Backlith stays in the tray after the planner closes, and moves new desktop icons into a zone.
- v1 plans the main screen only.

### Calendar sources (Claude's calls, the author: "looks fine")

- v1 reads calendar links (ICS addresses) from Google, Outlook.com, iCloud and others. Nobody signs in.
- "Sign in with Google" and "Sign in with Microsoft" come after v1. Google reviews any public app that reads calendars, and until it passes only 100 people can use it, behind a warning screen. Work Microsoft accounts often need IT approval.
- In the planner, the calendar block's settings: style, which calendars and their colours, show the weekend, length of a full day.

### Uploaded schedules (Claude's proposal; the author: "cool")

- Each uploaded file becomes its own calendar, with its own colour and on/off box.
- Backlith accepts its own JSON format and `.ics` files.
- Why JSON: LLMs write it reliably, a published JSON Schema lets Backlith check a file and give exact errors, and it reads easily. LLM-written `.ics` often breaks (line folding, repeat rules, time zones); CSV cannot hold repeats with exceptions cleanly; YAML breaks on indentation.
- The planner has a "Copy prompt for an LLM" button: the format, an example and the rules, ready to paste into any LLM with the timetable. When an upload fails, a "Copy errors" button gives the LLM what to fix.

### Calendar styles

Mockups are in `docs/mockups/` (files for claude.ai's Design canvas; they need its runtime to render, but the layout and the sums are readable). The author's own copy of the canvas is https://claude.ai/artifact/T2v88V5RrQdZb6Ct94aXRh, which only the author can open.

- **Time grid** (`TimeGrid.dc.html`): hours down the side, events as blocks at their real times, a "now" line in today's column, a fullness bar in each day's header.
- **Day stacks** (`DayStacks.dc.html`): one card per day, hours booked shown large, events listed in order, gaps of an hour or more shown as free time.
- **Load skyline** (`Skyline.dc.html`): each column fills from the bottom by how busy the day is, so the week reads like a skyline; events listed above the fill.
- Shared by all three: today highlighted, past events faded, a colour per calendar, a week summary (hours booked, busiest and lightest day), and a switch to hide the weekend. 8 hours counts as a full day: Light under 3 h, Steady 3 to 6 h, Busy 6 to 8 h, Packed 8 h or more.
- The mockups' look (dark ground, Inter with Space Grotesk for numbers and headings, orange for work, blue for personal, red for packed) was Claude's call for the mockups, not decided.

## Plan 1 built (22–23 November 2026)

The author chose to build natively in one session ("can you do native development?") and to test on a proper Windows machine later. Plan 1, `docs/superpowers/plans/2026-11-22-backlith-core-and-planner.md`, was built on the branch `core-and-planner` and merged into `main` on 23 November 2026, at the author's choice ("merge to main locally"). The branch is deleted. Nothing of it is pushed yet: pushing is the author's to say.

What exists:
- **`backlith-core`:** the week model and busy levels, reading ICS (repeats, moved and deleted instances, Windows zone names from Outlook, DST), the schedule format with its JSON Schema and LLM prompt, layouts and icon zones, settings, the undo record and redraw timing.
- **`backlith-sources`:** fetching calendar links with ETag and If-Modified-Since, keeping the last good copy on failure, and secrets in the system store.
- **The Tauri app:** `AppCore`, the tray with its six items, the planner window opened on first run, a 15-minute fetch and a 1-minute redraw check, and a pretend desktop for Linux.
- **The interface:** the wallpaper renderer with the three styles, and the planner (canvas with drag, resize, snapping, icon zones, Make bigger, Apply with the Auto arrange question, calendar settings, zones, calendar links, uploads, Copy prompt and Copy errors).
- **Tests:** about 130 in Rust and 46 in the interface. CI runs both on Linux.

What the author said while it was built:
- "I don't understand why you're trying to run the application over here on Ubuntu. This is just for development." So the app is not launched on Ubuntu to check it; tests and builds are the checks until Windows. It had been run once, for about two minutes, and Claude could not see its window.

What waits for plan 2 (Windows): the real `Desktop` (icons through `IFolderView`, `IDesktopWallpaper`, watching the desktop folders, battery saver and lock state), drawing the picture (spike 1 of the design), start at sign-in, the installers for x64 and ARM64, a CSP for the app, and checking reqwest's TLS (aws-lc-rs) builds on Windows ARM64.

A slip on the way: Rust's `target/` folder was not git-ignored at first, so the branch's early commits carried about 10,000 build files. They were taken out of the branch's history before anything was pushed, and `target/` is now ignored.

## Plan 1 review and the calls made (23 November 2026)

A fresh reviewer read the whole branch and answered "ready to merge with fixes": one critical finding, ten important and fourteen minor. Every critical and important one was fixed in one pass, each with a test that failed first, and one minor finding was raised to important. After the fixes: Rust 139 tests and interface 53 tests pass, and the interface builds.

Fixed:

- Critical 1, the link leaking as a Referer on redirects
- Important 1, unsaved layout edits lost on any source change
- Important 2, all-day titles shown with titles off
- Important 3, overlapping events drawn on top of each other
- Important 4, a BOM makes a calendar unreadable
- Important 5, unknown TZIDs ignoring the file's VTIMEZONE and tzone://Microsoft names
- Important 6, the pause after restoring the wallpaper not saved
- Important 7, the undo record could hold Backlith's own picture
- Important 9, no undo buttons in the planner
- Important 10, keeping Auto arrange left no way to save
- Minor 9 as re-graded

Left for later (minor):

- remove_source saves before deleting the secret, so a failed delete orphans the secret and skips forget().
- a source removed while its link is being fetched can get its cache file and failure entry back.
- "unreachable since" is in memory only and resets on restart.
- "unreachable since HH:MM" is always 24-hour, and rangeLabel builds "23 – 29 November" by hand instead of Intl formatRange.
- the viewer's time zone is read once at start-up.
- the 20 MB cap is checked after a chunked body is fully buffered.
- add_link downloads the calendar twice (check, then fetch).
- a failed save in upload leaves the new source in memory; replacing an .ics with a .json leaves the old file.
- decide_redraw stalls when the clock goes backwards (DST fall-back or a manual change).
- the planner preview shows no base image, there is no way to change the base, and it doesn't listen to render-input.
- sync commands parse calendars on the main thread, and uploads travel as JSON number arrays.
- overlap errors name ids, removeZone mutates in place, all-lowercase ICS gives no events, SourcesPanel folds three planned files, 8 clippy warnings, index.html lacks an SPDX line.
- saved types have no serde defaults, so adding a field later breaks older files.

Every call Claude made on the author's behalf while building plan 1, with what it costs if wrong. Each line names the task it was made in; "Final" means the review.

- Pre-flight: Ruling: Task 8's PlannerState, SourceStatus, ApplyResult and UploadError also derive TS and export to app/src/generated — the frontend needs them typed — cost if wrong: none, extra generated files.
- Task 3: Ruling: the "did you mean" hint uses OSA distance (a swap of two letters is one edit) within max(1, length/3), not Levenshtein ≤ 2 — Levenshtein 2 suggested `from` for `room` — cost if wrong: a few typos of short names get no hint.
- Task 3: Ruling: added messages the plan did not list but strict checking needs: "`every` and `skip` only go with `days`.", "`days` needs `from` and `until`, and they need `days`.", "`days` must list at least one day, like [\"mon\", \"thu\"].", "`skip` … ends before it starts.", "`skip` must be a list, …", "`end` X is the same as `start` X", "`allDay` must be true or false.", "`location` must be text.", "is not an object.", "The file is not plain text (UTF-8).", "The file must be one JSON object, starting with {." — cost if wrong: wording only.
- Task 3: Ruling: serde_json gets `preserve_order` workspace-wide so errors come out in file order — cost if wrong: none.
- Task 4: Ruling: CalendarToolSettings.calendar_ids empty means "every enabled calendar" — the first-run layout must show calendars added later without a second step — cost if wrong: a user who wants none shown must switch sources off.
- Task 4: Ruling: an icon listed in two zones' icon_order goes to the first zone; zones fill only up to capacity and snapping clamps a rect inside the work area — the plan named neither — cost if wrong: small.
- Task 5: Ruling: UndoRecord also carries `version`, so load_json's newer-version check covers it too — cost if wrong: one extra field.
- Task 5: Ruling: a scheduled redraw is never later than the next midnight (decide_redraw returns At(min(last+interval, midnight))) — the spec redraws at midnight — cost if wrong: none.
- Task 6: Ruling: failure reasons are fixed phrases chosen by the kind of error ("could not reach the calendar server", "the calendar server did not answer in time", "the calendar server answered N", "the link no longer returns a calendar", "the calendar is larger than 20 MB"), never reqwest's own text, which can contain the URL even after without_url() — cost if wrong: less detail for debugging.
- Task 6: Ruling: calendars over 20 MB are refused, and source ids are made file-safe before naming cache files — the plan named neither — cost if wrong: a huge calendar would need the cap raised.
- Task 6: Ruling: reqwest's default TLS (rustls with aws-lc-rs and the platform verifier) is kept. Note for plan 2: aws-lc-sys may need CMake/NASM or clang to build on Windows, especially ARM64; switch to the ring provider there if it does — cost if wrong: a build fix in plan 2.
- Task 7: Ruling: formatTime uses two-digit hours when the locale's clock is 24-hour (hourCycle h23/h24) — en-GB with a numeric hour gives "9:30", but the plan pins "09:30" — cost if wrong: none.
- Task 7: Ruling: the mockup fixture comes from a Rust example (crates/backlith-core/examples/mockup_render_input.rs) so the TypeScript tests run on the core's real output — cost if wrong: none.
- Task 7: Ruling: events marked free are drawn with a lighter, dashed fill in every style, and an empty day in day stacks reads "Nothing planned" — the plan named neither — cost if wrong: looks only.
- Task 8: Ruling: "Pause" is a check item (ticked while paused), keeping its exact label — cost if wrong: none.
- Task 8: Ruling: "Restore my old wallpaper" also pauses Backlith, so the next redraw does not paint over the wallpaper just restored — cost if wrong: the user has to untick Pause to resume.
- Task 8: Ruling: the app's CSP is left at Tauri's default (null) for now; plan 2 should set one once the renderer's asset loading is known — cost if wrong: weaker protection against injected scripts in the planner until then.
- Task 8: Ruling: add_link checks the link (counting events) and fetches it into the cache before saving the source, and rolls back the secret if anything fails — cost if wrong: none.
- Task 8: Ruling: the author said running the app on Ubuntu is pointless ("This is just for development"), so the plan's remaining "check by hand with pnpm tauri dev" steps (Tasks 9 and 10) are skipped; tests and builds are the checks until a Windows machine is available — cost if wrong: interface wiring errors that only show in the running app are found later, on Windows.
- Task 9: Ruling: the planner previews where icons will go with a TypeScript copy of plan_icons (placeIcons), so dragging needs no round trip; Rust stays the authority on Apply — cost if wrong: a preview that differs from the result if the two drift (both follow the same tests' numbers).
- Task 9: Ruling: dropping an icon into a zone writes every zone's icon order out in full, not only the target's — needed so the dropped icon leaves its old zone — cost if wrong: none.
- Task 9: Ruling: growToFit also refuses a size that runs past the work area, and "Make bigger" says "No room to grow this zone." when blocked — cost if wrong: none.
- Task 9: Ruling: the plan's by-hand check in `pnpm tauri dev` was skipped at the author's word (see Task 8) — cost if wrong: see Task 8.
- Task 10: Ruling: added a `preview_week` command so the planner previews the week with unsaved calendar settings (weekend, week, full day) — the plan had no way to preview them without saving — cost if wrong: none.
- Task 10: Ruling: the side panel can add, rename and remove icon zones and choose which takes new icons — the design says the user draws one or more zones, which the plan's tasks never offered — cost if wrong: none.
- Task 10: Ruling: calendar sources (colour, on/off, add, remove, upload) take effect at once; only layout changes wait for Apply — cost if wrong: a user may expect Apply to cover them.
- Task 10: Ruling: removing a calendar asks once in the row (Remove / Keep), not in a dialog — cost if wrong: none.
- History: Ruling: Rust's target/ and app/src-tauri/gen/ had been committed from Task 1 on (10,069 files). Both are now git-ignored, and the branch was rewritten with filter-branch to drop them; source content is unchanged (checked with git diff against backup/core-and-planner-before-cleanup). New commits: Task 1 7c8db05, 2 2e3fc0d, 3 5b7afa5, 4 f17e11e, 5 2ef19b4, 6 cd9bc0b, 7 a3d3862, 8 69fa9f7, 9 54b97d5, 10 7700734. The SHAs in the lines above are the old ones. — cost if wrong: none; nothing was pushed.
- Final: Ruling: Important 8, CI without Tauri's libraries — fixed in ci.yml with the README's apt line; it can only be checked by a CI run, which needs a push — cost if wrong: a red first CI run.
- Final: Ruling: Windows Desktop, picture capture, installer, start at sign-in, folder watch, battery saver and lock state — plan 2 — cost if wrong: none now.
- Final: Ruling: overflow icons stay where Windows put them, even over a tool — the spec says so; the planner warns and offers Make bigger — cost if wrong: an icon may cover part of the calendar until the zone is grown.
- Final: Ruling: a valid but empty calendar replaces the cache — it can't be told from a calendar that really is empty — cost if wrong: a server returning an empty calendar by mistake empties it until fixed.
- Final: Ruling: RANGE=THISANDFUTURE overrides and RDATE;VALUE=PERIOD are not handled — rare, not named — cost if wrong: such series show unchanged instances.
- Final: Ruling: two copies of Backlith at once — start-up is plan 2 — cost if wrong: duplicated fetches until then.
- Final: Ruling: keyboard-only editing of the canvas — the spec asks for mouse and touch — cost if wrong: the canvas is not usable by keyboard alone.
- Final: Ruling: busy hours on a DST day use a 1440-minute day — the plan's explicit trade — cost if wrong: an hour off twice a year.
- Final: Ruling: the CSP stays null until plan 2 — no innerHTML paths exist — cost if wrong: less protection if a script injection path appears.
- Final: Ruling: RAM and CPU targets are measured on Windows in plan 2 — cost if wrong: none now.
- Final: Ruling: the bundled fonts are OFL — bundling is aggregation, compatible with the GPL — cost if wrong: none.
- Final: Ruling: "booked this week" also shows in Next-7-days mode — wording only — cost if wrong: slightly odd wording.
- Final: Ruling: Credential Manager's size limit for very long links — needs Windows to test, plan 2 — cost if wrong: a very long link may not save on Windows.
- Final: Ruling: put_icons_back doesn't turn Auto arrange back on — the spec promises positions only — cost if wrong: the user turns it back on by hand.

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
