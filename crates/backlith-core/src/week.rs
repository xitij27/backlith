// SPDX-License-Identifier: GPL-3.0-or-later
//! The week the calendar shows: which days, what happens on each, how much
//! of each day is booked, and how busy that makes it.
//!
//! All times are the viewer's local wall-clock time. A day is always 1440
//! minutes, even on a daylight-saving change.

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Weekday};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One event, or one instance of a repeating event, in local time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Occurrence {
    pub calendar_id: String,
    pub title: String,
    pub location: Option<String>,
    /// Local wall-clock start.
    pub start: NaiveDateTime,
    /// Local wall-clock end, exclusive. For an all-day event, the next midnight.
    pub end: NaiveDateTime,
    pub all_day: bool,
    /// False for events marked free (`TRANSP:TRANSPARENT`): shown, not counted.
    pub counts_as_busy: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WeekMode {
    /// Monday to Sunday of the current week.
    ThisWeek,
    /// Today and the six days after it.
    Next7Days,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeekOptions {
    pub mode: WeekMode,
    pub show_weekend: bool,
    pub full_day_minutes: u32,
}

impl Default for WeekOptions {
    fn default() -> Self {
        Self { mode: WeekMode::ThisWeek, show_weekend: true, full_day_minutes: 480 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Level {
    Light,
    Steady,
    Busy,
    Packed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Gap {
    pub start_minute: u16,
    pub end_minute: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayEvent {
    pub calendar_id: String,
    pub title: String,
    pub location: Option<String>,
    /// Minutes since this day's midnight, 0..=1440.
    pub start_minute: u16,
    pub end_minute: u16,
    pub done: bool,
    pub counts_as_busy: bool,
    pub continues_from_previous_day: bool,
    pub continues_to_next_day: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AllDayEvent {
    pub calendar_id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayView {
    /// `YYYY-MM-DD`.
    pub date: String,
    /// `mon` to `sun`.
    pub weekday: String,
    pub is_today: bool,
    pub is_past: bool,
    pub booked_minutes: u32,
    pub level: Level,
    /// Sorted by start, then end.
    pub events: Vec<DayEvent>,
    pub all_day: Vec<AllDayEvent>,
    /// Gaps of an hour or more between the day's busy stretches.
    pub free_gaps: Vec<Gap>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayRef {
    pub date: String,
    pub booked_minutes: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeekView {
    pub days: Vec<DayView>,
    pub total_booked_minutes: u32,
    /// None when nothing is booked. Ties go to the earliest day.
    pub busiest: Option<DayRef>,
    pub lightest: Option<DayRef>,
    /// Minutes since today's midnight.
    pub now_minute: u16,
}

/// How busy a day is, from its booked time as a share of a full day:
/// Light under 37.5%, Steady under 75%, Busy under 100%, Packed from 100%.
pub fn level_for(booked_minutes: u32, full_day_minutes: u32) -> Level {
    // Integer form of the shares 37.5% (3/8) and 75% (3/4), to avoid rounding.
    let (b, f) = (u64::from(booked_minutes), u64::from(full_day_minutes));
    if b * 8 < f * 3 {
        Level::Light
    } else if b * 4 < f * 3 {
        Level::Steady
    } else if b < f {
        Level::Busy
    } else {
        Level::Packed
    }
}

/// The days shown, as `[start, end)`.
pub fn week_dates(now: NaiveDateTime, mode: WeekMode) -> (NaiveDate, NaiveDate) {
    let today = now.date();
    let start = match mode {
        WeekMode::ThisWeek => today - Duration::days(i64::from(today.weekday().num_days_from_monday())),
        WeekMode::Next7Days => today,
    };
    (start, start + Duration::days(7))
}

fn weekday_code(d: Weekday) -> &'static str {
    match d {
        Weekday::Mon => "mon",
        Weekday::Tue => "tue",
        Weekday::Wed => "wed",
        Weekday::Thu => "thu",
        Weekday::Fri => "fri",
        Weekday::Sat => "sat",
        Weekday::Sun => "sun",
    }
}

fn minute_of(day_start: NaiveDateTime, t: NaiveDateTime) -> u16 {
    // Callers clip `t` into [day_start, day_start + 1 day], so this is 0..=1440.
    (t - day_start).num_minutes().clamp(0, 1440) as u16
}

/// Sorted, merged busy intervals of one day, in minutes.
fn merge(mut spans: Vec<(u16, u16)>) -> Vec<(u16, u16)> {
    spans.sort_unstable();
    let mut merged: Vec<(u16, u16)> = Vec::with_capacity(spans.len());
    for (s, e) in spans {
        match merged.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => merged.push((s, e)),
        }
    }
    merged
}

fn build_day(date: NaiveDate, occurrences: &[Occurrence], now: NaiveDateTime, full_day_minutes: u32) -> DayView {
    let today = now.date();
    let day_start = date.and_hms_opt(0, 0, 0).expect("midnight exists");
    let day_end = day_start + Duration::days(1);
    let mut events = Vec::new();
    let mut all_day = Vec::new();

    for o in occurrences {
        if o.all_day {
            let last = if o.end > o.start { (o.end - Duration::seconds(1)).date() } else { o.start.date() };
            if o.start.date() <= date && date <= last {
                all_day.push(AllDayEvent { calendar_id: o.calendar_id.clone(), title: o.title.clone() });
            }
            continue;
        }
        let instant_here = o.start == o.end && o.start >= day_start && o.start < day_end;
        let (s, e) = (o.start.max(day_start), o.end.min(day_end));
        if s >= e && !instant_here {
            continue;
        }
        events.push(DayEvent {
            calendar_id: o.calendar_id.clone(),
            title: o.title.clone(),
            location: o.location.clone(),
            start_minute: minute_of(day_start, s),
            end_minute: minute_of(day_start, e.max(s)),
            done: date < today || o.end <= now,
            counts_as_busy: o.counts_as_busy,
            continues_from_previous_day: o.start < day_start,
            continues_to_next_day: o.end > day_end,
        });
    }
    events.sort_by_key(|e| (e.start_minute, e.end_minute));

    let busy = merge(
        events.iter().filter(|e| e.counts_as_busy && e.end_minute > e.start_minute).map(|e| (e.start_minute, e.end_minute)).collect(),
    );
    let booked_minutes: u32 = busy.iter().map(|(s, e)| u32::from(e - s)).sum();
    let free_gaps = busy
        .windows(2)
        .filter(|w| w[1].0 - w[0].1 >= 60)
        .map(|w| Gap { start_minute: w[0].1, end_minute: w[1].0 })
        .collect();

    DayView {
        date: date.format("%Y-%m-%d").to_string(),
        weekday: weekday_code(date.weekday()).to_string(),
        is_today: date == today,
        is_past: date < today,
        booked_minutes,
        level: level_for(booked_minutes, full_day_minutes),
        events,
        all_day,
        free_gaps,
    }
}

pub fn build_week(occurrences: &[Occurrence], now: NaiveDateTime, options: &WeekOptions) -> WeekView {
    let (start, end) = week_dates(now, options.mode);
    let days: Vec<DayView> = start
        .iter_days()
        .take_while(|d| *d < end)
        .filter(|d| options.show_weekend || !matches!(d.weekday(), Weekday::Sat | Weekday::Sun))
        .map(|d| build_day(d, occurrences, now, options.full_day_minutes))
        .collect();

    let total_booked_minutes = days.iter().map(|d| d.booked_minutes).sum();
    let as_ref = |d: &DayView| DayRef { date: d.date.clone(), booked_minutes: d.booked_minutes };
    let (busiest, lightest) = if total_booked_minutes == 0 {
        (None, None)
    } else {
        // `fold` keeps the first day on ties, so ties go to the earliest day.
        let most = days.iter().fold(None::<&DayView>, |best, d| match best {
            Some(b) if b.booked_minutes >= d.booked_minutes => Some(b),
            _ => Some(d),
        });
        let least = days.iter().fold(None::<&DayView>, |best, d| match best {
            Some(b) if b.booked_minutes <= d.booked_minutes => Some(b),
            _ => Some(d),
        });
        (most.map(as_ref), least.map(as_ref))
    };

    WeekView {
        days,
        total_booked_minutes,
        busiest,
        lightest,
        now_minute: (now.hour() * 60 + now.minute()) as u16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
    }
    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }
    fn now() -> NaiveDateTime {
        at("2026-11-25T13:40") // a Wednesday
    }
    fn ev(start: &str, end: &str, title: &str) -> Occurrence {
        Occurrence {
            calendar_id: "work".into(),
            title: title.into(),
            location: None,
            start: at(start),
            end: at(end),
            all_day: false,
            counts_as_busy: true,
        }
    }
    fn week(occs: &[Occurrence]) -> WeekView {
        build_week(occs, now(), &WeekOptions::default())
    }
    fn on<'a>(w: &'a WeekView, date: &str) -> &'a DayView {
        w.days.iter().find(|d| d.date == date).expect("day shown")
    }

    #[test]
    fn levels_follow_the_share_of_a_full_day() {
        assert_eq!(level_for(179, 480), Level::Light);
        assert_eq!(level_for(180, 480), Level::Steady);
        assert_eq!(level_for(359, 480), Level::Steady);
        assert_eq!(level_for(360, 480), Level::Busy);
        assert_eq!(level_for(479, 480), Level::Busy);
        assert_eq!(level_for(480, 480), Level::Packed);
    }

    #[test]
    fn this_week_runs_monday_to_sunday() {
        assert_eq!(week_dates(now(), WeekMode::ThisWeek), (day("2026-11-23"), day("2026-11-30")));
    }

    #[test]
    fn next_seven_days_starts_today() {
        assert_eq!(week_dates(now(), WeekMode::Next7Days), (day("2026-11-25"), day("2026-12-02")));
    }

    #[test]
    fn overlapping_events_count_once() {
        let w = week(&[ev("2026-11-26T10:00", "2026-11-26T11:00", "A"), ev("2026-11-26T10:30", "2026-11-26T12:00", "B")]);
        assert_eq!(on(&w, "2026-11-26").booked_minutes, 120);
    }

    #[test]
    fn all_day_and_free_events_do_not_count() {
        let mut holiday = ev("2026-11-26T00:00", "2026-11-27T00:00", "Holiday");
        holiday.all_day = true;
        let mut free = ev("2026-11-26T15:00", "2026-11-26T16:00", "Maybe");
        free.counts_as_busy = false;
        let w = week(&[holiday, free]);
        let thu = on(&w, "2026-11-26");
        assert_eq!(thu.all_day, vec![AllDayEvent { calendar_id: "work".into(), title: "Holiday".into() }]);
        assert_eq!(thu.events.len(), 1);
        assert!(!thu.events[0].counts_as_busy);
        assert_eq!(thu.booked_minutes, 0);
    }

    #[test]
    fn overnight_event_is_split() {
        let w = week(&[ev("2026-11-23T22:00", "2026-11-24T02:00", "Night shift")]);
        let mon = on(&w, "2026-11-23");
        let tue = on(&w, "2026-11-24");
        assert_eq!((mon.events[0].start_minute, mon.events[0].end_minute), (1320, 1440));
        assert!(mon.events[0].continues_to_next_day && !mon.events[0].continues_from_previous_day);
        assert_eq!((tue.events[0].start_minute, tue.events[0].end_minute), (0, 120));
        assert!(tue.events[0].continues_from_previous_day && !tue.events[0].continues_to_next_day);
        assert_eq!(mon.booked_minutes, 120);
        assert_eq!(tue.booked_minutes, 120);
    }

    #[test]
    fn past_and_done_flags() {
        let w = week(&[ev("2026-11-25T12:00", "2026-11-25T13:00", "Lunch"), ev("2026-11-25T13:30", "2026-11-25T15:00", "Sync")]);
        assert!(on(&w, "2026-11-23").is_past && on(&w, "2026-11-24").is_past);
        let wed = on(&w, "2026-11-25");
        assert!(wed.is_today && !wed.is_past);
        assert!(wed.events[0].done);
        assert!(!wed.events[1].done);
        assert_eq!(w.now_minute, 13 * 60 + 40);
    }

    #[test]
    fn hiding_the_weekend_drops_sat_and_sun() {
        let opts = WeekOptions { show_weekend: false, ..WeekOptions::default() };
        let w = build_week(&[], now(), &opts);
        assert_eq!(w.days.len(), 5);
        assert_eq!(w.days.last().unwrap().weekday, "fri");
    }

    #[test]
    fn free_gaps_of_an_hour_or_more() {
        let w = week(&[ev("2026-11-26T09:00", "2026-11-26T09:30", "Standup"), ev("2026-11-26T14:00", "2026-11-26T15:00", "Dentist")]);
        assert_eq!(on(&w, "2026-11-26").free_gaps, vec![Gap { start_minute: 570, end_minute: 840 }]);
        let w = week(&[ev("2026-11-26T09:00", "2026-11-26T09:30", "Standup"), ev("2026-11-26T10:00", "2026-11-26T11:00", "Lecture")]);
        assert!(on(&w, "2026-11-26").free_gaps.is_empty());
    }

    /// The sample week in docs/mockups/TimeGrid.dc.html.
    fn mockup_week() -> Vec<Occurrence> {
        let rows: &[(&str, &[(&str, &str, &str)])] = &[
            ("2026-11-23", &[("09:00", "09:30", "Standup"), ("10:00", "12:00", "Lecture · Machine Learning"), ("14:00", "15:00", "1:1 with supervisor"), ("18:00", "19:00", "Gym")]),
            ("2026-11-24", &[("09:00", "09:30", "Standup"), ("11:00", "12:30", "Design review"), ("13:00", "14:00", "Lunch with the team"), ("16:00", "17:30", "Lab session")]),
            ("2026-11-25", &[("09:00", "09:30", "Standup"), ("10:00", "11:30", "Lecture · Databases"), ("12:00", "13:00", "Lunch"), ("13:30", "15:00", "Project sync"), ("15:30", "17:00", "Deep work"), ("19:00", "21:00", "Hackathon prep")]),
            ("2026-11-26", &[("09:00", "09:30", "Standup"), ("14:00", "15:00", "Dentist")]),
            ("2026-11-27", &[("09:00", "09:30", "Standup"), ("10:00", "12:00", "Lecture · Machine Learning"), ("13:00", "17:00", "Hackathon")]),
            ("2026-11-28", &[("11:00", "13:00", "Football"), ("20:00", "22:00", "Dinner with friends")]),
            ("2026-11-29", &[("16:00", "17:00", "Call home")]),
        ];
        rows.iter()
            .flat_map(|(d, evs)| evs.iter().map(move |(s, e, t)| ev(&format!("{d}T{s}"), &format!("{d}T{e}"), t)))
            .collect()
    }

    #[test]
    fn mockup_week_sums() {
        let w = week(&mockup_week());
        assert_eq!(w.total_booked_minutes, 1800);
        assert_eq!(w.busiest, Some(DayRef { date: "2026-11-25".into(), booked_minutes: 480 }));
        assert_eq!(w.lightest, Some(DayRef { date: "2026-11-29".into(), booked_minutes: 60 }));
        assert_eq!(on(&w, "2026-11-25").level, Level::Packed);
    }

    #[test]
    fn an_empty_week_has_no_busiest_day() {
        let w = week(&[]);
        assert_eq!(w.days.len(), 7);
        assert_eq!((w.busiest, w.lightest), (None, None));
    }
}
