# Backlith core and planner: implementation plan (plan 1 of 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build everything in Backlith v1 that can be built and tested on Ubuntu: the calendar engine, the schedule format, the layout and icon-zone maths, calendar fetching, the wallpaper renderer's three calendar styles, and the planner app running against a pretend desktop.

**Architecture:** A Cargo workspace with two plain Rust crates (`backlith-core`: pure logic, no I/O; `backlith-sources`: fetching and secrets) and a Tauri 2 app (`app/`) whose Rust side talks to the desktop through a `Desktop` trait. On Linux that trait has a fake implementation. The interface is TypeScript and React. One set of React components draws both the planner's preview and, in plan 2, the wallpaper picture. Rust types shared with TypeScript are generated with `ts-rs`, so the two sides cannot drift apart.

**Tech Stack:** Rust stable, Tauri 2.12, chrono 0.4, chrono-tz 0.10, icalendar 0.17, rrule 0.14, serde/serde_json, ts-rs 12, reqwest 0.13 (rustls), keyring 4, httpmock 0.9, jsonschema 0.63, strsim 0.11, thiserror 2, anyhow 1, dirs 6, iana-time-zone 0.1, tokio 1, tempfile 3 (tests). Node 24, pnpm 10, TypeScript 5.9.3, React 19, Vite 8, Vitest 5, Testing Library (react 16, user-event 14, jest-dom 8), jsdom.

**Spec:** `docs/superpowers/specs/2026-11-22-backlith-v1-design.md`. Background: `docs/decisions.md`. Mockups: `docs/mockups/`.

**Plan 2 (later, on a Windows machine):** the Windows `Desktop` implementation (`IFolderView` icons, `IDesktopWallpaper`, `SHChangeNotifyRegister`, battery saver and lock state), the renderer's picture capture (spike 1 of the spec), start at sign-in, the per-user NSIS installer for x64 and ARM64, the Windows CI job, and the manual test checklist. Nothing in this plan may block that: no Unix-only APIs outside `app/src-tauri/src/desktop/fake.rs`.

## Before you start (needs the author)

These need `sudo` or a download, so the author runs or approves them once:

```bash
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

Check: `cargo --version` prints a version, and `pkg-config --modversion webkit2gtk-4.1` prints 2.x.

## Global Constraints

- Licence GPL-3.0-or-later. Every new source file starts with `SPDX-License-Identifier: GPL-3.0-or-later` as a comment. Every dependency's licence must be GPL-3.0-compatible (MIT, Apache-2.0, BSD, ISC, MPL-2.0, Unicode-3.0 are fine).
- Targets for later: Windows 10 22H2 and Windows 11, x64 and ARM64. `backlith-core` and `backlith-sources` must compile for `x86_64-pc-windows-msvc` unchanged.
- Busy levels, as a share of the full day: Light under 37.5%, Steady 37.5% to under 75%, Busy 75% to under 100%, Packed 100% or more. Full day defaults to 480 minutes.
- Booked time is the union of a day's timed events (overlaps count once). All-day events don't count as hours. `STATUS:CANCELLED` events are dropped. `TRANSP:TRANSPARENT` events show but don't count.
- Calendar links: `webcal://` is fetched as `https://`; fetched every 15 minutes with `ETag` / `If-Modified-Since`; a failed fetch keeps the last good copy. A link is stored only in the secret store, never in settings, logs, error messages or the picture.
- Redraw every 5 minutes, every 15 when battery saver is on, never while locked; at midnight; at once after any change.
- Schedule files: at most 1 MB; strict; errors in the form ``Event 3 (Databases lab): `end` 09:00 is before `start` 10:00``.
- Tray menu labels, exactly: Open planner, Redraw now, Pause, Put my icons back, Restore my old wallpaper, Quit.
- Interface copy is English. Dates and times shown to the user follow the system's regional format: format them with `Intl.DateTimeFormat` and the default locale. Tests pass an explicit `en-GB` locale.
- App files live in `dirs::data_dir()/Backlith/` (`%APPDATA%\Backlith\` on Windows, `~/.local/share/Backlith/` on Linux).
- Times inside the week model are local wall-clock times. A day is always 1440 minutes, even on a DST change day (Claude's call: off by an hour of "booked" on two days a year, in exchange for simple sums).

## Review Focus

1. **Outlook links with Windows time zone names** (`TZID:W. Europe Standard Time`). Expected: events land at the right local time, not shifted or dropped. Test in Task 2.
2. **A repeating event with one instance moved and one deleted** (`RECURRENCE-ID`, `EXDATE`). Expected: the moved one shows once, at its new time; the deleted one doesn't show. Test in Task 2.
3. **A week containing a DST change.** Expected: a weekly 10:00 event stays at 10:00 on both sides of the change. Test in Task 2.
4. **A calendar link that starts returning an HTML login page or an error.** Expected: the calendar keeps its last good events and shows "unreachable since HH:MM"; it is never emptied. Test in Task 6.
5. **Two desktop icons with the same name** (one on the user's Desktop, one on the Public Desktop). Expected: both are placed; neither overwrites the other. Test in Task 4.

---

## File map

```
Cargo.toml                         workspace: crates/*, app/src-tauri
rust-toolchain.toml                channel = "stable"
.cargo/config.toml                 TS_RS_EXPORT_DIR = app/src/generated
.github/workflows/ci.yml           Linux: cargo test, pnpm test, pnpm build
schemas/schedule-v1.json           JSON Schema 2020-12 for schedule files
schemas/schedule-v1.example.json   the spec's example
crates/backlith-core/
  data/windows-zones.csv           Windows zone name → IANA, from CLDR (with data/README.md)
  src/lib.rs
  src/week.rs                      week window, splitting by day, busy sums, levels, gaps
  src/ics.rs                       reading ICS text into occurrences
  src/zones_windows.rs             Windows time zone names → IANA
  src/schedule.rs                  schedule format v1: checking, expanding, LLM prompt
  src/layout.rs                    layout model, pixels ↔ fractions, snapping, overlaps
  src/icons.rs                     zone capacity, icon placement plan, first-run layout
  src/settings.rs                  settings file, undo record, render input
  src/redraw.rs                    when to redraw
  tests/fixtures/*.ics
crates/backlith-sources/
  src/lib.rs
  src/secrets.rs                   SecretStore trait, keyring and memory stores
  src/fetch.rs                     link normalising, fetching with cache
app/
  package.json, vite.config.ts, tsconfig.json, index.html
  src/generated/                   ts-rs output (committed)
  src/render/                      Wallpaper and the three calendar styles
  src/planner/                     planner window
  src/api.ts                       typed wrappers around Tauri commands
  src-tauri/
    Cargo.toml, tauri.conf.json, build.rs, icons/
    src/main.rs                    Tauri set-up, tray, windows
    src/core.rs                    AppCore: all command logic, testable without Tauri
    src/commands.rs                thin #[tauri::command] wrappers over AppCore
    src/desktop/mod.rs             Desktop trait
    src/desktop/fake.rs            FakeDesktop for Linux
    src/timers.rs                  15-minute fetch and redraw loop
```

---

### Task 1: Workspace and the week model

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `.cargo/config.toml`, `.github/workflows/ci.yml`, `crates/backlith-core/Cargo.toml`, `crates/backlith-core/src/lib.rs`, `crates/backlith-core/src/week.rs`
- Test: `crates/backlith-core/src/week.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Produces (all `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]`, `#[serde(rename_all = "camelCase")]`, `#[ts(export)]`):

```rust
pub struct Occurrence {
    pub calendar_id: String,
    pub title: String,
    pub location: Option<String>,
    pub start: NaiveDateTime,   // viewer's local wall-clock time
    pub end: NaiveDateTime,     // exclusive; all-day: next midnight
    pub all_day: bool,
    pub counts_as_busy: bool,   // false for TRANSP:TRANSPARENT
}
pub enum WeekMode { ThisWeek, Next7Days }          // serde: "thisWeek" | "next7Days"
pub struct WeekOptions { pub mode: WeekMode, pub show_weekend: bool, pub full_day_minutes: u32 }
impl Default for WeekOptions // ThisWeek, true, 480
pub enum Level { Light, Steady, Busy, Packed }     // serde: lower camel
pub struct Gap { pub start_minute: u16, pub end_minute: u16 }
pub struct DayEvent {
    pub calendar_id: String, pub title: String, pub location: Option<String>,
    pub start_minute: u16, pub end_minute: u16,    // 0..=1440 within this day
    pub done: bool, pub counts_as_busy: bool,
    pub continues_from_previous_day: bool, pub continues_to_next_day: bool,
}
pub struct AllDayEvent { pub calendar_id: String, pub title: String }
pub struct DayView {
    pub date: String,            // "2026-11-25"
    pub weekday: String,         // "mon".."sun"
    pub is_today: bool, pub is_past: bool,
    pub booked_minutes: u32, pub level: Level,
    pub events: Vec<DayEvent>,   // sorted by start, then end
    pub all_day: Vec<AllDayEvent>,
    pub free_gaps: Vec<Gap>,     // gaps ≥ 60 min between merged busy intervals
}
pub struct DayRef { pub date: String, pub booked_minutes: u32 }
pub struct WeekView {
    pub days: Vec<DayView>, pub total_booked_minutes: u32,
    pub busiest: Option<DayRef>, pub lightest: Option<DayRef>, // None when total is 0; ties → earliest day
    pub now_minute: u16,
}
pub fn level_for(booked_minutes: u32, full_day_minutes: u32) -> Level;
pub fn week_dates(now: NaiveDateTime, mode: WeekMode) -> (NaiveDate, NaiveDate); // [start, end)
pub fn build_week(occurrences: &[Occurrence], now: NaiveDateTime, options: &WeekOptions) -> WeekView;
```

- [ ] **Step 1: Create the workspace.** Root `Cargo.toml` with `members = ["crates/*"]` and `resolver = "3"`. Task 8 adds `"app/src-tauri"`. Add `rust-toolchain.toml` (`channel = "stable"`) and `.cargo/config.toml` with `[env] TS_RS_EXPORT_DIR = { value = "app/src/generated", relative = true }`. Add `crates/backlith-core` with dependencies chrono (serde), chrono-tz, serde, serde_json, thiserror and ts-rs (features `chrono-impl`, `serde-compat`). Add a CI workflow on `ubuntu-24.04` running `cargo test --workspace --locked`.

- [ ] **Step 2: Write the failing tests** in `week.rs`, with `now = 2026-11-25T13:40` (a Wednesday) unless stated:

```rust
#[test] fn levels_follow_the_share_of_a_full_day() {
    assert_eq!(level_for(179, 480), Level::Light);
    assert_eq!(level_for(180, 480), Level::Steady);
    assert_eq!(level_for(359, 480), Level::Steady);
    assert_eq!(level_for(360, 480), Level::Busy);
    assert_eq!(level_for(479, 480), Level::Busy);
    assert_eq!(level_for(480, 480), Level::Packed);
}
#[test] fn this_week_runs_monday_to_sunday()        // week_dates → (2026-11-23, 2026-11-30)
#[test] fn next_seven_days_starts_today()          // (2026-11-25, 2026-12-02)
#[test] fn overlapping_events_count_once()          // 10:00–11:00 + 10:30–12:00 → booked 120
#[test] fn all_day_and_free_events_do_not_count()   // all-day → in all_day, booked 0; transparent → in events, booked 0
#[test] fn overnight_event_is_split()               // Mon 22:00–Tue 02:00 → Mon 1320–1440 continues_to_next_day, Tue 0–120 continues_from_previous_day; booked 120 each
#[test] fn past_and_done_flags()                    // Mon/Tue is_past; Wed is_today; Wed event ending 13:00 done, ending 15:00 not
#[test] fn hiding_the_weekend_drops_sat_and_sun()   // 5 days, last is "fri"
#[test] fn free_gaps_of_an_hour_or_more()           // 09:00–09:30, 14:00–15:00 → one gap 570–840; 30-min gap → none
#[test] fn mockup_week_sums()                       // the events in docs/mockups/TimeGrid.dc.html → total 1800, busiest 2026-11-25 (480), lightest 2026-11-29 (60)
```

- [ ] **Step 3: Run the tests.** Run `cargo test -p backlith-core week`. Expected: FAIL, because the functions don't exist yet.

- [ ] **Step 4: Implement `week.rs`.**
  - For each shown day, clip every occurrence to [day 00:00, next day 00:00).
  - Merge the busy intervals (sort, then sweep) for `booked_minutes` and `free_gaps`.
  - `now_minute` = minutes since midnight of `now`.

- [ ] **Step 5: Run the tests.** Run `cargo test -p backlith-core`. Expected: PASS, and `app/src/generated/` now holds `WeekView.ts` and the other types.

- [ ] **Step 6: Commit** with `git add -A && git commit -m "Add the workspace and the week model"`.

---

### Task 2: Reading ICS calendars

**Files:**
- Create: `crates/backlith-core/src/ics.rs`, `src/zones_windows.rs`, `data/windows-zones.csv`, `data/README.md`, `tests/fixtures/*.ics`
- Test: `crates/backlith-core/tests/ics.rs`

**Interfaces:**
- Consumes: `Occurrence` (Task 1).
- Produces:

```rust
pub struct DateRange { pub from: NaiveDate, pub to: NaiveDate }   // local dates, [from, to)
#[derive(thiserror::Error)] pub enum IcsError { #[error("not a calendar")] NotCalendar, #[error("could not read the calendar: {0}")] Parse(String) }
pub fn read_ics(text: &str, calendar_id: &str, viewer_tz: chrono_tz::Tz, range: &DateRange) -> Result<Vec<Occurrence>, IcsError>;
pub fn windows_zone_to_iana(name: &str) -> Option<chrono_tz::Tz>;
```

- [ ] **Step 1: Build the zone table.** Make `data/windows-zones.csv` (`windows_name,iana`) from CLDR's `common/supplemental/windowsZones.xml`, using the `territory="001"` rows. `data/README.md` names the source and its licence (Unicode-3.0). `zones_windows.rs` loads the table with `include_str!` into a lazily built map.

- [ ] **Step 2: Write the fixtures and failing tests** in `tests/ics.rs` (viewer `Europe/Berlin` unless stated):

```rust
#[test] fn moved_and_deleted_instances()      // weekly_with_exdate_and_override.ics: Mon 10:00–11:00 Europe/Berlin from 2026-11-02, EXDATE 2026-11-16, RECURRENCE-ID 2026-11-23 moved to 14:00–15:00; range 2026-11-16..2026-11-30 → exactly one occurrence, 2026-11-23 14:00–15:00
#[test] fn outlook_windows_zone_names()       // TZID="W. Europe Standard Time" 2026-11-24T09:00 → 09:00 for Europe/Berlin, 08:00 for Europe/London
#[test] fn utc_and_floating_times()           // 2026-11-24T08:00:00Z → 09:00; floating 09:00 → 09:00 for any viewer
#[test] fn all_day_without_end()              // VALUE=DATE 2026-11-24, no DTEND → all_day, 2026-11-24T00:00 to 2026-11-25T00:00
#[test] fn duration_instead_of_end()          // DURATION:PT90M from 10:00 → ends 11:30
#[test] fn cancelled_dropped_transparent_kept()  // STATUS:CANCELLED → absent; TRANSP:TRANSPARENT → counts_as_busy == false
#[test] fn weekly_event_keeps_its_time_across_dst()  // Mon 10:00 Europe/Berlin from 2026-10-19, range 2026-10-19..2026-11-02 → 2026-10-19 10:00 and 2026-10-26 10:00
#[test] fn html_is_not_a_calendar()           // "<!doctype html><html>…" → Err(IcsError::NotCalendar)
```

- [ ] **Step 3: Run the tests.** Run `cargo test -p backlith-core --test ics`. Expected: FAIL.

- [ ] **Step 4: Implement `read_ics`.**
  - **Parsing:** use `icalendar`'s parser for components and properties. Text without `BEGIN:VCALENDAR` is `NotCalendar`.
  - **Repeats:** expand `RRULE` with `rrule`, using DTSTART in its own zone. Only expand over `range` widened by a day on each side, with a cap of 10,000 instances per event.
  - **Exceptions:** remove `EXDATE`s. Replace the instance whose original start matches a `RECURRENCE-ID`. Add `RDATE`s.
  - **Time zones:** an unknown `TZID` falls back to the viewer's zone. Convert everything to the viewer's wall-clock time.

- [ ] **Step 5: Run the tests.** Run `cargo test -p backlith-core`. Expected: PASS.

- [ ] **Step 6: Commit** with `git commit -am "Read ICS calendars"` (after `git add` of the new files).

---

### Task 3: The schedule format

**Files:**
- Create: `schemas/schedule-v1.json`, `schemas/schedule-v1.example.json` (the spec's example, verbatim), `crates/backlith-core/src/schedule.rs`
- Test: `crates/backlith-core/tests/schedule.rs` (with dev-dependency `jsonschema`)

**Interfaces:**
- Consumes: `Occurrence` (Task 1), `DateRange` (Task 2).
- Produces:

```rust
pub const MAX_SCHEDULE_BYTES: usize = 1_048_576;
pub struct ScheduleFile { pub name: String, pub timezone: Option<chrono_tz::Tz>, pub events: Vec<ScheduleEvent> }
pub enum When { Once { date: NaiveDate }, Weekly { days: Vec<Weekday>, from: NaiveDate, until: NaiveDate, every: u32, skip: Vec<(NaiveDate, NaiveDate)> } }
pub enum Times { AllDay, Span { start: NaiveTime, end: NaiveTime } }
pub struct ScheduleEvent { pub title: String, pub location: Option<String>, pub notes: Option<String>, pub when: When, pub times: Times }
pub struct ScheduleError { pub event: Option<usize>, pub title: Option<String>, pub message: String } // impl Display: "Event {n} ({title}): {message}" or "Event {n}: …" or just the message
pub fn check_schedule(bytes: &[u8]) -> Result<ScheduleFile, Vec<ScheduleError>>;
pub fn expand_schedule(file: &ScheduleFile, calendar_id: &str, viewer_tz: chrono_tz::Tz, range: &DateRange) -> Vec<Occurrence>;
pub fn llm_prompt() -> String;
```

- [ ] **Step 1: Write the failing tests.** Each error test asserts the exact `to_string()`:

```rust
#[test] fn the_spec_example_passes_both_checkers()   // check_schedule(example) is Ok; jsonschema validates it against schemas/schedule-v1.json
#[test] fn schema_and_checker_reject_the_same_things() // unknown field, both `date` and `days`: jsonschema invalid AND check_schedule Err
#[test] fn end_before_start()        // "Event 3 (Databases lab): `end` 09:00 is before `start` 10:00"
#[test] fn unknown_field_suggests()  // "Event 2 (Lab): unknown field `strat`; did you mean `start`?"
#[test] fn unknown_field_no_hint()   // "Event 1 (Gym): unknown field `colour`."
#[test] fn wrong_day_name()          // "Event 1 (Gym): `days` has \"monday\"; use mon, tue, wed, thu, fri, sat or sun."
#[test] fn date_and_days_together()  // "Event 1 (Gym): use either `date` or `days` with `from` and `until`, not both."
#[test] fn neither_date_nor_days()   // "Event 1 (Gym): needs either `date`, or `days` with `from` and `until`."
#[test] fn bad_date()                // "Event 1 (Gym): `from` \"19.10.2026\" is not a date like 2026-10-19."
#[test] fn bad_time()                // "Event 1 (Gym): `start` \"10am\" is not a time like 09:30."
#[test] fn until_before_from()       // "Event 1 (Gym): `until` 2026-10-01 is before `from` 2026-10-19."
#[test] fn all_day_with_times()      // "Event 1 (Gym): an `allDay` event has no `start` or `end`."
#[test] fn missing_times()           // "Event 1 (Gym): needs `start` and `end`, or `allDay`: true."
#[test] fn bad_skip()                // "Event 1 (Gym): `skip` \"Christmas\" is not a date or a range like 2026-12-21/2027-01-03."
#[test] fn bad_every()               // "Event 1 (Gym): `every` must be a whole number of weeks, 1 or more."
#[test] fn file_level_errors()       // "`backlith` must be 1.", "`name` is missing.", "`events` must list at least one event.", "`timezone` \"Mars/Base\" is not a time zone name like \"Europe/Berlin\"."
#[test] fn too_big()                 // 1_048_577 bytes → ["The file is larger than 1 MB."]
#[test] fn all_errors_at_once()      // a file with 3 problems returns 3 errors, in file order
#[test] fn expands_weekly_with_skip_and_every() // example, range 2026-12-14..2027-01-11: ML lecture on Mon 14, Thu 17 Dec, then none until Mon 4 Jan, Thu 7 Jan; `every: 2` from Mon 2026-10-19 → 19 Oct, 2 Nov, 16 Nov
#[test] fn file_zone_converts_to_viewer() // timezone Europe/Berlin, 10:00 → 09:00 for a Europe/London viewer
#[test] fn prompt_carries_schema_and_example() // llm_prompt() contains the schema file's text and "\"backlith\": 1"
```

- [ ] **Step 2: Run the tests.** Run `cargo test -p backlith-core --test schedule`. Expected: FAIL.

- [ ] **Step 3: Write `schemas/schedule-v1.json`.** JSON Schema 2020-12, `additionalProperties: false` at every level. Each event is a `oneOf` of the once shape and the weekly shape. The day enum is `mon` to `sun`. The patterns are `^\d{4}-\d{2}-\d{2}$`, `^([01]\d|2[0-3]):[0-5]\d$` and the skip range form.

- [ ] **Step 4: Implement `schedule.rs`.**
  - **Checking:** parse into `serde_json::Value` first, so every error is collected rather than stopping at the first. For unknown fields, suggest an allowed name within Levenshtein distance 2 (`strsim`).
  - **Expanding:** weeks are counted from the Monday of the week holding `from`, and an `every: n` repeat keeps weeks 0, n, 2n and so on.
  - **The prompt:** `llm_prompt()` is fixed text that explains the task (turn the timetable that follows into this format; output only JSON), then the schema and the example, both included with `include_str!`.

- [ ] **Step 5: Run the tests.** Run `cargo test -p backlith-core`. Expected: PASS.

- [ ] **Step 6: Commit** with `git commit -m "Add the schedule format, its schema and the LLM prompt"`.

---

### Task 4: Layout and icon zones

**Files:**
- Create: `crates/backlith-core/src/layout.rs`, `crates/backlith-core/src/icons.rs`
- Test: `#[cfg(test)]` modules in both

**Interfaces:**
- Consumes: `WeekOptions` (Task 1).
- Produces (same derives as Task 1):

```rust
pub struct RectF { pub x: f64, pub y: f64, pub w: f64, pub h: f64 }   // fractions of the work area
pub struct RectPx { pub x: i32, pub y: i32, pub w: i32, pub h: i32 }  // physical pixels
pub struct PointPx { pub x: i32, pub y: i32 }
pub struct IconGrid { pub cell_w: i32, pub cell_h: i32, pub scale: f64 }
pub enum CalendarStyle { TimeGrid, DayStacks, Skyline }
pub struct CalendarToolSettings { pub style: CalendarStyle, pub week: WeekOptions, pub calendar_ids: Vec<String>, pub show_titles: bool }
pub struct ZoneSettings { pub name: String, pub takes_new_icons: bool, pub icon_order: Vec<String> }
pub enum ItemKind { Calendar(CalendarToolSettings), IconZone(ZoneSettings) }   // serde tag = "kind"
pub struct LayoutItem { pub id: String, pub rect: RectF, pub item: ItemKind }
pub enum Base { Color { hex: String }, Image { path: String } }
pub struct Layout { pub version: u32, pub base: Base, pub items: Vec<LayoutItem> }
pub struct DesktopIcon { pub id: String, pub name: String, pub position: PointPx } // id = full parsing name, unique
pub struct ZoneOverflow { pub zone_id: String, pub icon_ids: Vec<String> }
pub struct IconPlan { pub moves: Vec<(String, PointPx)>, pub overflow: Vec<ZoneOverflow> }
pub const ZONE_LABEL_PX: i32 = 28;   // at 100% scaling; times grid.scale
#[derive(thiserror::Error)] pub enum LayoutError { TwoCalendars, Overlap(String, String), OutOfBounds(String), NewIconsZone }
pub fn to_px(r: &RectF, work_area: &RectPx) -> RectPx;
pub fn to_fraction(r: &RectPx, work_area: &RectPx) -> RectF;
pub fn snap_to_grid(r: &RectPx, work_area: &RectPx, grid: &IconGrid) -> RectPx;
pub fn find_overlaps(layout: &Layout) -> Vec<(String, String)>;
pub fn validate_layout(layout: &Layout) -> Result<(), LayoutError>;
pub fn zone_capacity(zone: &RectPx, grid: &IconGrid) -> usize;
pub fn plan_icons(layout: &Layout, icons: &[DesktopIcon], work_area: &RectPx, grid: &IconGrid) -> IconPlan;
pub fn first_run_layout(icons: &[DesktopIcon], work_area: &RectPx, grid: &IconGrid, current_wallpaper: Option<String>) -> Layout;
```

- [ ] **Step 1: Write the failing tests.** Use work area `{0,0,1920,1032}` and grid `{100,100,1.0}`:

```rust
#[test] fn fractions_round_trip_within_a_pixel()
#[test] fn snapping_aligns_to_cells()            // {130,40,290,310} → {100,0,300,300}
#[test] fn touching_edges_do_not_overlap()       // [0,0,0.5,1] and [0.5,0,0.5,1] → none; 1px more → one pair
#[test] fn layout_rules()                        // two calendars → TwoCalendars; zones but none takes_new_icons, or two that do → NewIconsZone; rect past 1.0 → OutOfBounds
#[test] fn capacity_leaves_room_for_the_label()  // {0,0,300,328} → 9
#[test] fn icons_fill_row_by_row_in_order()      // icon_order ["b","a"] → b at (0,28), a at (100,28)
#[test] fn unlisted_icons_go_to_the_new_icons_zone()  // appended after the listed ones, in current top-to-bottom then left-to-right order
#[test] fn overflow_is_reported_not_moved()      // 10 icons, capacity 9 → 9 placed, overflow [the 10th]
#[test] fn icons_already_in_place_are_not_moved()
#[test] fn same_name_different_id_both_placed()  // two icons named "New folder" with different ids → two distinct targets
#[test] fn first_run_fits_the_icons_on_the_left() // 12 icons → zone {0,0,200,1032}; calendar from x=300 to 1920, full height; base = Image(current) or Color "#0D1117"
```

- [ ] **Step 2: Run the tests.** Run `cargo test -p backlith-core layout icons`. Expected: FAIL.

- [ ] **Step 3: Implement both files.**
  - **Capacity:** `floor(w / cell_w) * floor((h - label) / cell_h)`, where label = `ZONE_LABEL_PX * scale`.
  - **First-run zone:** rows = `floor((h - label) / cell_h)`, and cols = `max(1, ceil(n / rows))`, capped at 40% of the work area's width.
  - **First-run calendar:** it starts one cell to the right of the zone.

- [ ] **Step 4: Run the tests.** Run `cargo test -p backlith-core`. Expected: PASS.

- [ ] **Step 5: Commit** with `git commit -m "Add the layout model and icon zone planning"`.

---

### Task 5: Settings, undo record, render input and redraw timing

**Files:**
- Create: `crates/backlith-core/src/settings.rs`, `crates/backlith-core/src/redraw.rs`
- Test: `#[cfg(test)]` modules in both

**Interfaces:**
- Consumes: `Layout`, `PointPx`, `RectPx` (Task 4); `WeekView` (Task 1).
- Produces:

```rust
pub enum SourceKind { Link, IcsFile { file: String }, Schedule { file: String } }  // a Link's URL lives in the secret store under "calendar-link:{id}"
pub struct CalendarSource { pub id: String, pub name: String, pub color: String, pub enabled: bool, pub kind: SourceKind }
pub struct Settings { pub version: u32, pub layout: Layout, pub calendars: Vec<CalendarSource>, pub start_at_sign_in: bool, pub paused: bool }
pub struct UndoRecord { pub icons: Vec<(String, PointPx)>, pub wallpaper: Option<String>, pub taken_at: NaiveDateTime }
pub struct RenderInput { pub layout: Layout, pub week: WeekView, pub calendars: Vec<CalendarSource>, pub screen: RectPx, pub work_area: RectPx, pub scale: f64 }
#[derive(thiserror::Error)] pub enum SettingsError { Io(String), Corrupt(String), NewerVersion(u32) }
pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, SettingsError>;  // missing file → Ok(None); version > 1 → NewerVersion
pub fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<(), SettingsError>;      // write "<name>.tmp", then rename
pub fn secret_key(source_id: &str) -> String;                                            // "calendar-link:{id}"

pub struct RedrawState { pub last_redraw: Option<NaiveDateTime>, pub dirty: bool, pub locked: bool, pub battery_saver: bool, pub paused: bool }
pub enum RedrawDecision { Now, At(NaiveDateTime), Hold }
pub fn decide_redraw(now: NaiveDateTime, state: &RedrawState) -> RedrawDecision;
```

- [ ] **Step 1: Write the failing tests:**

```rust
#[test] fn save_then_load_round_trips()
#[test] fn missing_file_is_none()
#[test] fn newer_version_is_refused()            // version 2 → NewerVersion(2)
#[test] fn a_failed_save_leaves_the_old_file()   // save into a read-only dir fails; the old file is unchanged
#[test] fn settings_never_hold_a_link()          // Settings with a Link source serialises without "http" or "webcal"
#[test] fn redraw_rules() {
    // locked or paused → Hold; dirty → Now; never drawn → Now
    // last 10:00, now 10:03 → At(10:05); battery saver → At(10:15)
    // last 23:58, now 00:01 next day → Now (midnight crossed)
}
```

- [ ] **Step 2: Run the tests.** Run `cargo test -p backlith-core settings redraw`. Expected: FAIL.

- [ ] **Step 3: Implement both files.** `save_json` must never leave a half-written file behind.

- [ ] **Step 4: Run the tests.** Run `cargo test -p backlith-core`. Expected: PASS, with the new TypeScript types generated.

- [ ] **Step 5: Commit** with `git commit -m "Add settings, the undo record and redraw timing"`.

---

### Task 6: Fetching calendar links, and secrets

**Files:**
- Create: `crates/backlith-sources/Cargo.toml`, `src/lib.rs`, `src/secrets.rs`, `src/fetch.rs`
- Test: `crates/backlith-sources/tests/fetch.rs` (httpmock), `#[cfg(test)]` in `secrets.rs`

**Interfaces:**
- Consumes: `read_ics`, `DateRange` (Task 2).
- Produces:

```rust
pub trait SecretStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, SecretError>;
    fn set(&self, key: &str, value: &str) -> Result<(), SecretError>;
    fn delete(&self, key: &str) -> Result<(), SecretError>;
}
pub struct KeyringStore;          // keyring service name "Backlith"
pub struct MemoryStore { /* Mutex<HashMap<String,String>> */ }
pub fn normalize_link(input: &str) -> Result<String, LinkError>;  // trim; webcal:// → https://; only http(s); LinkError::NotALink
pub struct CacheMeta { pub fetched_at: NaiveDateTime, pub etag: Option<String>, pub last_modified: Option<String> }
pub enum FetchOutcome { Updated, NotModified, Failed { reason: String } }   // reason never contains the URL
pub struct Fetcher { /* reqwest::Client, cache_dir: PathBuf */ }
impl Fetcher {
    pub fn new(cache_dir: PathBuf) -> Self;
    pub async fn fetch(&self, source_id: &str, url: &str, now: NaiveDateTime) -> FetchOutcome; // body → cache_dir/{id}.ics, meta → {id}.json
    pub fn cached_text(&self, source_id: &str) -> Option<String>;
    pub async fn check(&self, url: &str, viewer_tz: Tz, today: NaiveDate) -> Result<usize, String>; // events in the next 8 weeks
}
```

- [ ] **Step 1: Write the failing tests:**

```rust
#[test] fn webcal_becomes_https()               // "  webcal://x.example/a.ics " → "https://x.example/a.ics"; "ftp://…" and "hello" → NotALink
#[tokio::test] async fn first_fetch_caches_body_and_etag()     // 200 + ETag "v1" → Updated; cached_text is the body
#[tokio::test] async fn second_fetch_sends_if_none_match()     // mock expects If-None-Match: "v1", returns 304 → NotModified; cache unchanged
#[tokio::test] async fn server_error_keeps_last_good_copy()    // 500 → Failed; cached_text still the old body
#[tokio::test] async fn html_page_keeps_last_good_copy()       // 200 text/html login page → Failed { reason: "the link no longer returns a calendar" }; cache unchanged
#[tokio::test] async fn failure_reason_hides_the_link()        // unreachable host with a secret token in the path → reason contains neither the host nor the token
#[tokio::test] async fn check_counts_upcoming_events()
#[test] fn memory_store_round_trip()
```

- [ ] **Step 2: Run the tests.** Run `cargo test -p backlith-sources`. Expected: FAIL.

- [ ] **Step 3: Implement it.**
  - **Client:** reqwest with `rustls`, a 20 s timeout and a User-Agent of `Backlith/<version>`.
  - **Hiding the link:** errors are mapped through `reqwest::Error::without_url()` before being turned into text.
  - **Checking content:** a body only replaces the cache if `read_ics` accepts it.

- [ ] **Step 4: Run the tests.** Run `cargo test -p backlith-sources`. Expected: PASS.

- [ ] **Step 5: Commit** with `git commit -m "Fetch calendar links with a cache, and keep links in the secret store"`.

---

### Task 7: The interface project and the wallpaper renderer

**Files:**
- Create: `app/package.json`, `app/vite.config.ts`, `app/tsconfig.json`, `app/index.html`, `app/src/test-setup.ts`, `app/src/render/theme.ts`, `app/src/render/format.ts`, `app/src/render/Wallpaper.tsx`, `app/src/render/ZonePanel.tsx`, `app/src/render/calendar/{CalendarBlock,TimeGrid,DayStacks,Skyline,WeekHeader}.tsx`, `app/src/render/fit.ts`
- Modify: `.github/workflows/ci.yml` (add `pnpm install --frozen-lockfile`, `pnpm test`, `pnpm build` in `app/`)
- Test: `app/src/render/__tests__/*.test.tsx`

**Interfaces:**
- Consumes: the generated types in `app/src/generated/` (`RenderInput`, `WeekView`, `Layout`, `CalendarSource`…).
- Produces:

```ts
export function Wallpaper(props: { input: RenderInput; baseImageUrl?: string }): JSX.Element; // fixed-size root, screen px ÷ scale in CSS px
export function formatHours(minutes: number): string;         // 270 → "4h 30m", 480 → "8h", 0 → "0h"
export function formatTime(minuteOfDay: number, locale?: string): string;  // Intl.DateTimeFormat, hour + minute; formatTime(570, "en-GB") === "09:30", formatTime(570, "en-US") === "9:30 AM"
export function visibleHours(week: WeekView): [number, number]; // default [7, 22]; widened to whole hours around any earlier or later event
export function fitCount(availablePx: number, itemPx: number, total: number): { shown: number; more: number }; // reserves one row for "+N more" when needed
export function skylineFillPx(booked: number, fullDay: number, columnPx: number, scale: number): number; // max(88*scale, round(min(1, booked/(1.25*fullDay)) * 0.65 * columnPx))
```

- [ ] **Step 1: Set up the project.**
  - Run `pnpm create vite app --template react-ts`, then pin TypeScript 5.9.3.
  - Add `vitest`, `jsdom`, `@testing-library/react`, `@testing-library/user-event` and `@testing-library/jest-dom`.
  - Add the `test` script `vitest run` and the `build` script `tsc --noEmit && vite build`.
  - Fonts are Inter and Space Grotesk, bundled through `@fontsource/inter` and `@fontsource/space-grotesk`. The wallpaper must draw with no network.

- [ ] **Step 2: Write the failing tests.** Fixture: a `RenderInput` built from the mockup week, saved as `app/src/render/__tests__/mockup-week.json`.

```ts
test("formatHours", …)                       // the three values above
test("formatTime follows the locale", …)     // the two values above
test("time grid has a column per day and one now line", …)  // 7 columns; getAllByTestId("now-line").length === 1, inside today's column
test("visibleHours widens for an early event", …)           // event at 06:00 → [6, 22]
test("hidden titles show Busy only", …)                     // show_titles false → no event titles in the DOM; "Busy" per event
test("day stacks show free gaps", …)                         // "2h free" in Wednesday's card (17:00–19:00)
test("day stacks cut off with +N more", …)                   // fitCount(300, 64, 8) → { shown: 3, more: 5 }; the card shows "+5 more"
test("skyline fill follows booked time", …)                  // skylineFillPx(480, 480, 856, 1) === 445; skylineFillPx(60, 480, 856, 1) === 88
test("packed day is marked", …)                              // Wednesday's level label reads "Packed"
test("zones draw their labels at their rects", …)            // a zone named "Downloads" renders its label; style left/top from to_px maths
test("long titles end with an ellipsis", …)                  // event title element has text-overflow: ellipsis and white-space: nowrap
test("week header sums", …)                                  // "30h", "Wed · 8h", "Sun · 1h"
```

- [ ] **Step 3: Run the tests.** Run `cd app && pnpm test`. Expected: FAIL.

- [ ] **Step 4: Implement the renderer.**
  - **Source of truth:** the layout and the look come from `docs/mockups/` (`TimeGrid.dc.html`, `DayStacks.dc.html`, `Skyline.dc.html`). Colours and fonts go in `theme.ts`.
  - **Calendar colours:** each event's colour comes from its `CalendarSource.color`, used at 16–24% opacity for fills.
  - **Readability:** panels get a translucent dark backing (`rgba(13,17,23,0.72)`), so text reads over any base image.
  - **Keep it light:** no animation, and nothing loaded from the network.

- [ ] **Step 5: Run the tests and the build.** Run `pnpm test && pnpm build`. Expected: PASS, with the build succeeding.

- [ ] **Step 6: Commit** with `git commit -m "Add the wallpaper renderer and the three calendar styles"`.

---

### Task 8: The Tauri app on a pretend desktop

**Files:**
- Modify: root `Cargo.toml` (add `"app/src-tauri"` to `members`)
- Create: `app/src-tauri/{Cargo.toml,build.rs,tauri.conf.json}` (package name `backlith-app`), `app/src-tauri/icons/` (from `pnpm tauri icon` on a placeholder mark), `app/src-tauri/src/{main.rs,core.rs,commands.rs,timers.rs}`, `app/src-tauri/src/desktop/{mod.rs,fake.rs}`, `app/src/api.ts`
- Test: `#[cfg(test)]` in `core.rs`

**Interfaces:**
- Consumes: everything in `backlith-core` and `backlith-sources`.
- Produces:

```rust
pub trait Desktop: Send + Sync {
    fn screen(&self) -> anyhow::Result<RectPx>;
    fn work_area(&self) -> anyhow::Result<RectPx>;
    fn icon_grid(&self) -> anyhow::Result<IconGrid>;
    fn list_icons(&self) -> anyhow::Result<Vec<DesktopIcon>>;
    fn auto_arrange(&self) -> anyhow::Result<bool>;
    fn set_auto_arrange(&self, on: bool) -> anyhow::Result<()>;
    fn move_icons(&self, moves: &[(String, PointPx)]) -> anyhow::Result<()>;
    fn current_wallpaper(&self) -> anyhow::Result<Option<String>>;
    fn set_wallpaper(&self, png: &Path) -> anyhow::Result<()>;
    fn battery_saver(&self) -> bool;
    fn session_locked(&self) -> bool;
}
pub struct FakeDesktop; // screen 1920×1080, work area 1920×1032, grid 100×100 @1.0, 12 icons (two named "New folder"), auto-arrange on; state in memory

pub struct PlannerState { pub settings: Settings, pub icons: Vec<DesktopIcon>, pub screen: RectPx, pub work_area: RectPx, pub grid: IconGrid, pub week: WeekView, pub sources: Vec<SourceStatus>, pub auto_arrange: bool, pub first_run: bool }
pub struct SourceStatus { pub id: String, pub ok: bool, pub message: Option<String>, pub event_count: usize } // message e.g. "unreachable since 10:20"
pub struct ApplyResult { pub moved: usize, pub overflow: Vec<ZoneOverflow> }
pub enum UploadError { Exists { name: String }, Invalid { errors: Vec<String> }, TooBig }
pub struct AppCore { /* desktop, secrets, fetcher, data_dir, state */ }
impl AppCore {
    pub fn new(desktop: Arc<dyn Desktop>, secrets: Arc<dyn SecretStore>, data_dir: PathBuf) -> Self;
    pub fn planner_state(&self, now: NaiveDateTime) -> anyhow::Result<PlannerState>;
    pub fn save_layout(&self, layout: Layout) -> Result<(), String>;      // validate_layout first
    pub fn apply_layout(&self) -> Result<ApplyResult, String>;            // undo record once; auto-arrange off; move icons; mark dirty
    pub fn put_icons_back(&self) -> Result<(), String>;
    pub fn restore_wallpaper(&self) -> Result<(), String>;
    pub async fn add_link(&self, name: String, color: String, url: String) -> Result<SourceStatus, String>;
    pub fn upload(&self, file_name: String, bytes: Vec<u8>, replace: bool) -> Result<SourceStatus, UploadError>; // .json → schedule, .ics → calendar file
    pub fn update_source(&self, id: String, name: String, color: String, enabled: bool) -> Result<(), String>;
    pub fn remove_source(&self, id: String) -> Result<(), String>;        // also deletes its secret or file
    pub fn llm_prompt(&self) -> String;
    pub fn render_input(&self, now: NaiveDateTime) -> anyhow::Result<RenderInput>;
}
```

- Tauri commands in `commands.rs` share the method names (`planner_state`, `save_layout`, `apply_layout`, `put_icons_back`, `restore_wallpaper`, `add_link`, `upload`, `update_source`, `remove_source`, `llm_prompt`). `api.ts` wraps each with the generated types.

- [ ] **Step 1: Write the failing tests** in `core.rs`, using `FakeDesktop`, `MemoryStore` and a temp dir:

```rust
#[test] fn first_run_proposes_a_layout_that_fits_every_icon()   // first_run true; plan_icons(layout) has no overflow
#[test] fn first_run_keeps_a_copy_of_the_wallpaper()            // FakeDesktop's wallpaper file is copied into data_dir/base/; layout.base is Image of the copy
#[test] fn apply_takes_the_undo_record_only_once()               // apply twice after moving icons in between → record holds the original positions
#[test] fn apply_turns_off_auto_arrange()
#[test] fn put_icons_back_restores_positions()
#[tokio::test] async fn add_link_keeps_the_url_out_of_settings() // httpmock calendar; settings.json has no "http"; secret store has it under calendar-link:{id}
#[test] fn upload_reports_schedule_errors()                      // Invalid { errors } with the exact Task 3 strings
#[test] fn upload_same_name_needs_replace()                      // second upload of name "Uni" → Exists; with replace → Ok, still one source
#[test] fn remove_source_deletes_its_secret()
#[test] fn sources_report_since_when()                           // a failing link → message "unreachable since HH:MM", events kept from cache
```

- [ ] **Step 2: Run the tests.** Run `cargo test -p backlith-app`. Expected: FAIL.

- [ ] **Step 3: Implement `core.rs`, `desktop/` and `commands.rs`.**
  - **Locking:** `AppCore` keeps its state behind a `Mutex`, and settings are written with `save_json` after every change.
  - **Building the week:** read the enabled sources (cached ICS text, saved ICS files, schedule files), expand them over `week_dates` ± 1 day in the system's zone (`iana-time-zone`), then call `build_week`.
  - **First run:** copy the current wallpaper, if there is one, into `data_dir/base/`, and pass that copy's path to `first_run_layout`, so the base survives the user changing or deleting the original.

- [ ] **Step 4: Implement `main.rs` and `timers.rs`.**
  - **Windows of the app:** `tauri.conf.json` has `identifier: "dev.backlith.app"` and no windows at start.
  - **Tray:** the tray (feature `tray-icon`) has the six labels from Global Constraints.
  - **Planner window:** "Open planner" creates the `planner` window, which loads `index.html`. Closing it destroys the window rather than hiding it. On first run (no settings file) the planner opens by itself.
  - **Timers:** `timers.rs` runs one tokio task:
    - every 15 minutes it fetches the links;
    - every minute it asks `decide_redraw`;
    - on `Now` it builds the `RenderInput` and emits `render-input` to the planner if it's open. Plan 2 adds drawing the picture there.
  - **Redraw now and Pause:** "Redraw now" sets dirty, and "Pause" toggles `settings.paused`.

- [ ] **Step 5: Run the tests.** Run `cargo test --workspace`. Expected: PASS.

- [ ] **Step 6: Check by hand.** Run `cd app && pnpm tauri dev`. Expected: a tray icon with the six items, and "Open planner" opens a window that shows `planner_state` as JSON (the real planner comes in Task 9). Quit really quits.

- [ ] **Step 7: Commit** with `git commit -m "Add the Tauri app with a pretend desktop, tray and commands"`.

---

### Task 9: The planner canvas

**Files:**
- Create: `app/src/planner/{Planner.tsx,Canvas.tsx,ItemFrame.tsx,IconLayer.tsx,ApplyBar.tsx,geometry.ts,usePointerDrag.ts}`, `app/src/main.tsx` (mounts `Planner`)
- Test: `app/src/planner/__tests__/{geometry.test.ts,canvas.test.tsx,apply.test.tsx}`

**Interfaces:**
- Consumes: `api.ts` (Task 8), `Wallpaper` (Task 7), generated types.
- Produces:

```ts
export type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";
export function moveRect(r: RectPx, dx: number, dy: number, bounds: RectPx): RectPx;          // clamped inside bounds
export function resizeRect(r: RectPx, h: Handle, dx: number, dy: number, min: { w: number; h: number }, bounds: RectPx): RectPx;
export function snapRect(r: RectPx, workArea: RectPx, grid: IconGrid): RectPx;                 // same rule as core's snap_to_grid
export function collides(r: RectPx, others: RectPx[]): boolean;                               // touching edges don't collide
export function growToFit(zone: RectPx, iconCount: number, grid: IconGrid, bounds: RectPx, others: RectPx[]): RectPx | null; // grow down, then right; null if blocked
```

- [ ] **Step 1: Write the failing tests:**

```ts
test("snapRect matches the Rust rule", …)        // {130,40,290,310} → {100,0,300,300}
test("touching edges do not collide", …)
test("growToFit grows down first", …)            // 3×3 zone with 12 icons and room below → 3 cols × 4 rows
test("growToFit returns null when boxed in", …)
test("dragging a zone by pointer moves it on the grid", …) // pointerdown/move/up by +130,+40 → item rect +100,+0 in fractions
test("a drop onto another item snaps back", …)
test("dragging an icon into another zone updates icon_order", …) // inserted at the drop slot
test("an overfull zone shows its count and Make bigger", …)       // "3 icons don't fit" + button calls growToFit
test("first apply asks about auto arrange", …)   // text: Backlith will turn off “Auto arrange icons” so your icons stay in their zones. Continue / Cancel; Continue → save_layout then apply_layout
test("touch works like the mouse", …)            // pointerType "touch" drag gives the same result
```

- [ ] **Step 2: Run the tests.** Run `pnpm test`. Expected: FAIL.

- [ ] **Step 3: Implement the canvas.**
  - **Scale:** the canvas shows the screen scaled to fit the window. `Wallpaper` sits underneath as the live preview, with the icons drawn on top as labelled outlines.
  - **Input:** use pointer events with `setPointerCapture`, so mouse and touch behave the same. Handles are at least 44 CSS px for touch.
  - **Changes:** edits stay local until **Apply**. Apply is disabled while the layout is invalid.
  - **Dialogs:** they're in-page, never `window.confirm`.

- [ ] **Step 4: Run the tests and the build.** Run `pnpm test && pnpm build`. Expected: PASS.

- [ ] **Step 5: Check by hand.** Run `pnpm tauri dev`, then open the planner. You should be able to drag and resize the calendar and the zone, move pretend icons between zones, and press Apply. The pretend icons should end up in their zones.

- [ ] **Step 6: Commit** with `git commit -m "Add the planner canvas"`.

---

### Task 10: Calendar settings, sources and uploads

**Files:**
- Create: `app/src/planner/{SidePanel.tsx,CalendarSettings.tsx,SourcesPanel.tsx,AddLinkForm.tsx,UploadDrop.tsx,ReplaceDialog.tsx,clipboard.ts}`
- Modify: `README.md` (how to set up, run and test on Ubuntu), `docs/decisions.md` (what plan 1 built, what waits for plan 2)
- Test: `app/src/planner/__tests__/{settings.test.tsx,sources.test.tsx}`

**Interfaces:**
- Consumes: `api.ts` (Task 8), `CalendarToolSettings` (Task 4).
- Produces: nothing used by later tasks.

- [ ] **Step 1: Write the failing tests** (with `api.ts` mocked):

```ts
test("style choices", …)                // a select with exactly "Time grid", "Day stacks", "Load skyline"
test("week choices", …)                 // "This week (Mon–Sun)", "Next 7 days"; weekend toggle; full day 1–16 hours, default 8; "Show event titles" on by default
test("adding a link shows events found", …)  // add_link resolves { eventCount: 42 } → "42 events in the next 8 weeks"
test("a bad link shows why", …)              // add_link rejects "the link no longer returns a calendar" → shown under the field
test("upload errors list and copy", …)       // upload rejects Invalid → each error as a list item; "Copy errors" puts them, one per line, on the clipboard
test("same name asks to replace", …)         // Exists { name: "Uni" } → dialog "Replace “Uni”?"; Replace → upload(…, replace: true)
test("copy prompt", …)                       // "Copy prompt for an LLM" puts llm_prompt() on the clipboard and says "Copied"
test("sources can be switched off and recoloured", …) // calls update_source with enabled false / new colour
test("accepts .json and .ics only", …)       // file input accept=".json,.ics"; dropping a .pdf shows "Use a .json schedule or an .ics calendar file."
```

- [ ] **Step 2: Run the tests.** Run `pnpm test`. Expected: FAIL.

- [ ] **Step 3: Implement the panels.**
  - **Clipboard:** use `navigator.clipboard.writeText`, which works in WebView2 and WebKitGTK.
  - **Colours:** a palette of 8 calendar colours, chosen so neighbours differ in lightness and not only in hue.

- [ ] **Step 4: Run the tests and the build.** Run `pnpm test && pnpm build && cargo test --workspace`. Expected: all PASS.

- [ ] **Step 5: Check by hand.** In `pnpm tauri dev`:
  - add a public ICS link, for example a public-holidays calendar; its events should appear in the preview;
  - upload `schemas/schedule-v1.example.json`;
  - upload a broken copy and check the errors appear;
  - switch through all three styles.

- [ ] **Step 6: Update the docs.**
  - **`README.md`:** the set-up commands from "Before you start", plus `pnpm install`, `pnpm tauri dev`, `cargo test --workspace` and `pnpm test`.
  - **`docs/decisions.md`:** what plan 1 delivered, and that plan 2 waits for a Windows machine.

- [ ] **Step 7: Commit** with `git commit -m "Add calendar settings, sources and schedule uploads"`.

---

## Self-review notes

- **Spec coverage:** every v1 goal maps to a task or to plan 2.
  - Goal 1, the planner: Tasks 8–10.
  - Goal 2, icon zones: Tasks 4, 8 and 9. The real icon moves are plan 2.
  - Goal 3, the calendar: Tasks 1, 2, 7 and 10.
  - Goal 4, uploads: Tasks 3 and 10.
  - Goal 5, the picture: the render input is Tasks 5 and 8; capturing and setting it is plan 2.
  - Goal 6, the tray: Task 8. Start at sign-in is plan 2.
  - Performance targets are measured in plan 2.
- **Types:** names are shared through `ts-rs`, so a rename in Rust breaks the TypeScript build instead of drifting.
