// SPDX-License-Identifier: GPL-3.0-or-later
//! Backlith's own schedule format, version 1: a timetable written by hand
//! or by an LLM, checked strictly so every mistake is named.
//!
//! The rules match `schemas/schedule-v1.json`. The checker reads the file as
//! plain JSON first, so it can report every problem at once, in file order.

use std::fmt;

use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Weekday};
use chrono_tz::Tz;
use serde_json::{Map, Value};

use crate::ics::{resolve, DateRange};
use crate::week::Occurrence;

/// Uploaded schedules may be at most 1 MB.
pub const MAX_SCHEDULE_BYTES: usize = 1_048_576;

const SCHEMA: &str = include_str!("../../../schemas/schedule-v1.json");
const EXAMPLE: &str = include_str!("../../../schemas/schedule-v1.example.json");

const FILE_FIELDS: &[&str] = &["backlith", "name", "timezone", "events"];
const EVENT_FIELDS: &[&str] =
    &["title", "location", "notes", "date", "days", "from", "until", "every", "skip", "start", "end", "allDay"];
const DAY_NAMES: &[(&str, Weekday)] = &[
    ("mon", Weekday::Mon),
    ("tue", Weekday::Tue),
    ("wed", Weekday::Wed),
    ("thu", Weekday::Thu),
    ("fri", Weekday::Fri),
    ("sat", Weekday::Sat),
    ("sun", Weekday::Sun),
];

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleFile {
    pub name: String,
    pub timezone: Option<Tz>,
    pub events: Vec<ScheduleEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum When {
    Once { date: NaiveDate },
    Weekly { days: Vec<Weekday>, from: NaiveDate, until: NaiveDate, every: u32, skip: Vec<(NaiveDate, NaiveDate)> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Times {
    AllDay,
    Span { start: NaiveTime, end: NaiveTime },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleEvent {
    pub title: String,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub when: When,
    pub times: Times,
}

/// One problem in a schedule file. `event` counts from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleError {
    pub event: Option<usize>,
    pub title: Option<String>,
    pub message: String,
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.event, &self.title) {
            (Some(n), Some(t)) => write!(f, "Event {n} ({t}): {}", self.message),
            (Some(n), None) => write!(f, "Event {n}: {}", self.message),
            _ => f.write_str(&self.message),
        }
    }
}

fn file_error(message: impl Into<String>) -> ScheduleError {
    ScheduleError { event: None, title: None, message: message.into() }
}

/// "unknown field `strat`; did you mean `start`?" or "unknown field `colour`."
fn unknown_field(name: &str, allowed: &[&str]) -> String {
    // A swap of two letters counts as one edit (`strat` → `start`). Short
    // names get less slack, so `room` is not taken for `from`.
    let slack = (name.chars().count() / 3).max(1);
    let near = allowed
        .iter()
        .map(|a| (strsim::osa_distance(&name.to_ascii_lowercase(), &a.to_ascii_lowercase()), *a))
        .filter(|(dist, _)| *dist <= slack)
        .min_by_key(|(dist, _)| *dist);
    match near {
        Some((_, a)) => format!("unknown field `{name}`; did you mean `{a}`?"),
        None => format!("unknown field `{name}`."),
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    (s.len() == 10).then(|| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()).flatten()
}

fn parse_time(s: &str) -> Option<NaiveTime> {
    (s.len() == 5).then(|| NaiveTime::parse_from_str(s, "%H:%M").ok()).flatten()
}

fn show(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{s}\""),
        other => other.to_string(),
    }
}

/// Collects one event's problems, each prefixed with its number and title.
struct EventCheck<'a> {
    number: usize,
    title: Option<String>,
    errors: &'a mut Vec<ScheduleError>,
}

impl EventCheck<'_> {
    fn fail(&mut self, message: impl Into<String>) {
        self.errors.push(ScheduleError { event: Some(self.number), title: self.title.clone(), message: message.into() });
    }

    fn date(&mut self, obj: &Map<String, Value>, key: &str) -> Option<NaiveDate> {
        let v = obj.get(key)?;
        match v.as_str().and_then(parse_date) {
            Some(d) => Some(d),
            None => {
                self.fail(format!("`{key}` {} is not a date like 2026-10-19.", show(v)));
                None
            }
        }
    }

    fn time(&mut self, obj: &Map<String, Value>, key: &str) -> Option<NaiveTime> {
        let v = obj.get(key)?;
        match v.as_str().and_then(parse_time) {
            Some(t) => Some(t),
            None => {
                self.fail(format!("`{key}` {} is not a time like 09:30.", show(v)));
                None
            }
        }
    }

    fn text(&mut self, obj: &Map<String, Value>, key: &str) -> Option<String> {
        match obj.get(key)? {
            Value::String(s) => Some(s.clone()),
            _ => {
                self.fail(format!("`{key}` must be text."));
                None
            }
        }
    }
}

fn check_event(number: usize, value: &Value, errors: &mut Vec<ScheduleError>) -> Option<ScheduleEvent> {
    let Some(obj) = value.as_object() else {
        errors.push(ScheduleError { event: Some(number), title: None, message: "is not an object.".into() });
        return None;
    };
    let title = obj.get("title").and_then(Value::as_str).map(str::trim).filter(|t| !t.is_empty()).map(String::from);
    let before = errors.len();
    let mut c = EventCheck { number, title: title.clone(), errors };
    if title.is_none() {
        c.fail("`title` is missing.");
    }
    for key in obj.keys() {
        if !EVENT_FIELDS.contains(&key.as_str()) {
            c.fail(unknown_field(key, EVENT_FIELDS));
        }
    }
    let location = c.text(obj, "location");
    let notes = c.text(obj, "notes");

    // When it happens.
    let has_date = obj.contains_key("date");
    let has_weekly = ["days", "from", "until"].iter().any(|k| obj.contains_key(*k));
    let when = match (has_date, has_weekly) {
        (true, true) => {
            c.fail("use either `date` or `days` with `from` and `until`, not both.");
            None
        }
        (false, false) => {
            c.fail("needs either `date`, or `days` with `from` and `until`.");
            None
        }
        (true, false) => {
            if obj.contains_key("every") || obj.contains_key("skip") {
                c.fail("`every` and `skip` only go with `days`.");
            }
            c.date(obj, "date").map(|date| When::Once { date })
        }
        (false, true) => check_weekly(obj, &mut c),
    };

    // At what times.
    let all_day = match obj.get("allDay") {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => {
            c.fail("`allDay` must be true or false.");
            false
        }
    };
    let has_times = obj.contains_key("start") || obj.contains_key("end");
    let times = if all_day {
        if has_times {
            c.fail("an `allDay` event has no `start` or `end`.");
            None
        } else {
            Some(Times::AllDay)
        }
    } else if !(obj.contains_key("start") && obj.contains_key("end")) {
        c.fail("needs `start` and `end`, or `allDay`: true.");
        None
    } else {
        match (c.time(obj, "start"), c.time(obj, "end")) {
            (Some(start), Some(end)) if end > start => Some(Times::Span { start, end }),
            (Some(start), Some(end)) if end == start => {
                c.fail(format!("`end` {} is the same as `start` {}", end.format("%H:%M"), start.format("%H:%M")));
                None
            }
            (Some(start), Some(end)) => {
                c.fail(format!("`end` {} is before `start` {}", end.format("%H:%M"), start.format("%H:%M")));
                None
            }
            _ => None,
        }
    };

    if errors.len() > before {
        return None;
    }
    Some(ScheduleEvent { title: title?, location, notes, when: when?, times: times? })
}

fn check_weekly(obj: &Map<String, Value>, c: &mut EventCheck<'_>) -> Option<When> {
    if !(obj.contains_key("days") && obj.contains_key("from") && obj.contains_key("until")) {
        c.fail("`days` needs `from` and `until`, and they need `days`.");
        return None;
    }
    let mut days = Vec::new();
    match obj.get("days") {
        Some(Value::Array(list)) if !list.is_empty() => {
            for v in list {
                match v.as_str().and_then(|s| DAY_NAMES.iter().find(|(n, _)| *n == s)) {
                    Some((_, wd)) if !days.contains(wd) => days.push(*wd),
                    Some(_) => {}
                    None => c.fail(format!("`days` has {}; use mon, tue, wed, thu, fri, sat or sun.", show(v))),
                }
            }
        }
        _ => c.fail("`days` must list at least one day, like [\"mon\", \"thu\"]."),
    }
    let from = c.date(obj, "from");
    let until = c.date(obj, "until");
    if let (Some(f), Some(u)) = (from, until) {
        if u < f {
            c.fail(format!("`until` {u} is before `from` {f}."));
        }
    }
    let every = match obj.get("every") {
        None => Some(1),
        Some(v) => match v.as_u64() {
            Some(n) if (1..=u64::from(u32::MAX)).contains(&n) => Some(n as u32),
            _ => {
                c.fail("`every` must be a whole number of weeks, 1 or more.");
                None
            }
        },
    };
    let mut skip = Vec::new();
    match obj.get("skip") {
        None => {}
        Some(Value::Array(list)) => {
            for v in list {
                let parsed = v.as_str().and_then(|s| match s.split_once('/') {
                    Some((a, b)) => parse_date(a).zip(parse_date(b)),
                    None => parse_date(s).map(|d| (d, d)),
                });
                match parsed {
                    Some((a, b)) if b >= a => skip.push((a, b)),
                    Some(_) => c.fail(format!("`skip` {} ends before it starts.", show(v))),
                    None => c.fail(format!("`skip` {} is not a date or a range like 2026-12-21/2027-01-03.", show(v))),
                }
            }
        }
        Some(_) => c.fail("`skip` must be a list, like [\"2026-12-24\", \"2026-12-21/2027-01-03\"]."),
    }
    Some(When::Weekly { days, from: from?, until: until?, every: every?, skip })
}

pub fn check_schedule(bytes: &[u8]) -> Result<ScheduleFile, Vec<ScheduleError>> {
    if bytes.len() > MAX_SCHEDULE_BYTES {
        return Err(vec![file_error("The file is larger than 1 MB.")]);
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Err(vec![file_error("The file is not plain text (UTF-8).")]);
    };
    let root: Value = match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(v) => v,
        Err(e) => return Err(vec![file_error(format!("This is not valid JSON: {e}"))]),
    };
    let Some(obj) = root.as_object() else {
        return Err(vec![file_error("The file must be one JSON object, starting with {.")]);
    };

    let mut errors = Vec::new();
    for key in obj.keys() {
        if !FILE_FIELDS.contains(&key.as_str()) {
            errors.push(file_error(unknown_field(key, FILE_FIELDS)));
        }
    }
    if obj.get("backlith").and_then(Value::as_u64) != Some(1) {
        errors.push(file_error("`backlith` must be 1."));
    }
    let name = obj.get("name").and_then(Value::as_str).map(str::trim).filter(|n| !n.is_empty()).map(String::from);
    if name.is_none() {
        errors.push(file_error("`name` is missing."));
    }
    let timezone = match obj.get("timezone") {
        None => None,
        Some(v) => match v.as_str().and_then(|s| s.parse::<Tz>().ok()) {
            Some(tz) => Some(tz),
            None => {
                errors.push(file_error(format!("`timezone` {} is not a time zone name like \"Europe/Berlin\".", show(v))));
                None
            }
        },
    };
    let mut events = Vec::new();
    match obj.get("events") {
        Some(Value::Array(list)) if !list.is_empty() => {
            for (i, v) in list.iter().enumerate() {
                if let Some(e) = check_event(i + 1, v, &mut errors) {
                    events.push(e);
                }
            }
        }
        _ => errors.push(file_error("`events` must list at least one event.")),
    }

    if errors.is_empty() {
        Ok(ScheduleFile { name: name.expect("checked"), timezone, events })
    } else {
        Err(errors)
    }
}

/// The days one event falls on within `[from, to)`, before times are added.
fn event_dates(when: &When, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
    match when {
        When::Once { date } => vec![*date],
        When::Weekly { days, from: first, until, every, skip } => {
            let anchor = *first - Duration::days(i64::from(first.weekday().num_days_from_monday()));
            let start = (*first).max(from);
            let end = (*until).min(to);
            start
                .iter_days()
                .take_while(|d| *d <= end)
                .filter(|d| days.contains(&d.weekday()))
                .filter(|d| ((*d - anchor).num_days() / 7) % i64::from(*every) == 0)
                .filter(|d| !skip.iter().any(|(a, b)| a <= d && d <= b))
                .collect()
        }
    }
}

pub fn expand_schedule(file: &ScheduleFile, calendar_id: &str, viewer_tz: Tz, range: &DateRange) -> Vec<Occurrence> {
    let zone = file.timezone.unwrap_or(viewer_tz);
    let window_start = range.from.and_time(NaiveTime::MIN);
    let window_end = range.to.and_time(NaiveTime::MIN);
    // A day either side, for events moved across midnight by a zone change.
    let (from, to) = (range.from - Duration::days(1), range.to + Duration::days(1));
    let mut out = Vec::new();
    for ev in &file.events {
        for date in event_dates(&ev.when, from, to) {
            let (start, end, all_day) = match ev.times {
                Times::AllDay => (date.and_time(NaiveTime::MIN), (date + Duration::days(1)).and_time(NaiveTime::MIN), true),
                Times::Span { start, end } => (
                    resolve(zone, date.and_time(start)).with_timezone(&viewer_tz).naive_local(),
                    resolve(zone, date.and_time(end)).with_timezone(&viewer_tz).naive_local(),
                    false,
                ),
            };
            if end > window_start && start < window_end {
                out.push(Occurrence {
                    calendar_id: calendar_id.to_string(),
                    title: ev.title.clone(),
                    location: ev.location.clone(),
                    start,
                    end,
                    all_day,
                    counts_as_busy: true,
                });
            }
        }
    }
    out.sort_by(|a, b| (a.start, a.end).cmp(&(b.start, b.end)));
    out
}

/// A prompt to paste into any LLM, followed by a timetable.
pub fn llm_prompt() -> String {
    format!(
        "Please turn the timetable that follows this message into a Backlith schedule file.\n\
         \n\
         Write it as one JSON object in the format below. Reply with the JSON only: no explanation and no code fences.\n\
         \n\
         Rules:\n\
         - \"backlith\" is always 1.\n\
         - \"name\" names the calendar, for example \"University timetable, winter 2026/27\".\n\
         - \"timezone\" is the IANA time zone the timetable's times are in, for example \"Europe/Berlin\". Leave it out if you don't know it.\n\
         - A repeating event has \"days\" (mon, tue, wed, thu, fri, sat, sun) and \"from\" and \"until\", the first and last dates it can happen, both included. Add \"every\": 2 for every other week.\n\
         - A one-off event has \"date\" instead.\n\
         - \"start\" and \"end\" are 24-hour times like 09:30, and \"end\" is after \"start\". An event that runs past midnight is written as two events.\n\
         - For an all-day event, write \"allDay\": true and leave out \"start\" and \"end\".\n\
         - \"skip\" lists dates, or ranges like 2026-12-21/2027-01-03, when a repeating event does not happen, such as holidays.\n\
         - Dates are written YYYY-MM-DD. Use only the fields the schema allows; \"location\" and \"notes\" are optional.\n\
         \n\
         The JSON Schema:\n\
         {}\n\
         \n\
         An example:\n\
         {}\n",
        SCHEMA.trim(),
        EXAMPLE.trim()
    )
}
