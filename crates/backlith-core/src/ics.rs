// SPDX-License-Identifier: GPL-3.0-or-later
//! Reading iCalendar (ICS) text, from calendar links and uploaded `.ics`
//! files, into occurrences in the viewer's local time.
//!
//! Repeating events are expanded with the `rrule` crate. Exceptions are
//! handled here: `EXDATE` removes an instance, `RDATE` adds one, and an
//! event carrying a `RECURRENCE-ID` replaces the instance it names.

use std::collections::HashMap;

use chrono::{DateTime, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use icalendar::parser::{read_calendar, unfold, Component, Property};

use crate::week::Occurrence;
use crate::zones_windows::windows_zone_to_iana;

/// Local dates, `[from, to)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IcsError {
    #[error("not a calendar")]
    NotCalendar,
    #[error("could not read the calendar: {0}")]
    Parse(String),
}

/// The most instances one repeating event may produce.
const MAX_INSTANCES: u16 = 10_000;

/// A `DTSTART`-like value, in the frame the calendar wrote it in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Stamp {
    Date(NaiveDate),
    Utc(NaiveDateTime),
    Zoned(NaiveDateTime, Tz),
    Floating(NaiveDateTime),
}

/// The instant a zone's wall-clock time means. A time that a DST change
/// skips moves forward an hour; a time it repeats takes the first meaning.
fn resolve(tz: Tz, n: NaiveDateTime) -> DateTime<Tz> {
    match tz.from_local_datetime(&n) {
        LocalResult::Single(t) => t,
        LocalResult::Ambiguous(first, _) => first,
        LocalResult::None => tz
            .from_local_datetime(&(n + Duration::hours(1)))
            .earliest()
            .unwrap_or_else(|| tz.from_utc_datetime(&n)),
    }
}

impl Stamp {
    fn is_date(&self) -> bool {
        matches!(self, Stamp::Date(_))
    }

    /// Wall-clock time in the viewer's zone. Dates and floating times stay as written.
    fn to_viewer(self, viewer: Tz) -> NaiveDateTime {
        match self {
            Stamp::Date(d) => d.and_time(NaiveTime::MIN),
            Stamp::Floating(n) => n,
            Stamp::Utc(n) => Utc.from_utc_datetime(&n).with_timezone(&viewer).naive_local(),
            Stamp::Zoned(n, tz) => resolve(tz, n).with_timezone(&viewer).naive_local(),
        }
    }

    /// What `EXDATE` and `RECURRENCE-ID` are matched on: the instant, or the
    /// written time for dates and floating times.
    fn key(self) -> NaiveDateTime {
        match self {
            Stamp::Date(d) => d.and_time(NaiveTime::MIN),
            Stamp::Floating(n) | Stamp::Utc(n) => n,
            Stamp::Zoned(n, tz) => resolve(tz, n).naive_utc(),
        }
    }

    /// The written date, in the event's own frame.
    fn own_date(self) -> NaiveDate {
        match self {
            Stamp::Date(d) => d,
            Stamp::Floating(n) | Stamp::Utc(n) | Stamp::Zoned(n, _) => n.date(),
        }
    }

    /// This stamp moved on by `length`, in the viewer's wall-clock time.
    fn end_after(self, length: Duration, viewer: Tz) -> NaiveDateTime {
        match self {
            Stamp::Date(d) => d.and_time(NaiveTime::MIN) + length,
            Stamp::Floating(n) => n + length,
            Stamp::Utc(n) => (Utc.from_utc_datetime(&n) + length).with_timezone(&viewer).naive_local(),
            Stamp::Zoned(n, tz) => (resolve(tz, n) + length).with_timezone(&viewer).naive_local(),
        }
    }
}

/// One `VEVENT`, read but not yet expanded.
struct RawEvent {
    uid: String,
    title: String,
    location: Option<String>,
    cancelled: bool,
    transparent: bool,
    start: Stamp,
    end: Option<Stamp>,
    duration: Option<Duration>,
    rrule: Option<String>,
    exdates: Vec<Stamp>,
    rdates: Vec<Stamp>,
    recurrence_id: Option<Stamp>,
}

fn param(p: &Property<'_>, key: &str) -> Option<String> {
    p.params
        .iter()
        .find(|x| x.key.as_str().eq_ignore_ascii_case(key))
        .and_then(|x| x.val.as_ref())
        .map(|v| v.as_str().trim().trim_matches('"').to_string())
}

/// A `TZID` as an IANA zone: an IANA name, a Windows name, or a path that
/// ends in an IANA name (`/mozilla.org/20050126_1/Europe/Berlin`).
fn zone_named(name: &str, viewer: Tz) -> Tz {
    if let Ok(tz) = name.parse::<Tz>() {
        return tz;
    }
    if let Some(tz) = windows_zone_to_iana(name) {
        return tz;
    }
    name.match_indices('/')
        .find_map(|(i, _)| name[i + 1..].parse::<Tz>().ok())
        .unwrap_or(viewer)
}

fn parse_stamp(value: &str, tzid: Option<&str>, date_only: bool, viewer: Tz) -> Option<Stamp> {
    let v = value.trim();
    if date_only || (v.len() == 8 && v.bytes().all(|b| b.is_ascii_digit())) {
        return NaiveDate::parse_from_str(v, "%Y%m%d").ok().map(Stamp::Date);
    }
    if let Some(u) = v.strip_suffix(['Z', 'z']) {
        return NaiveDateTime::parse_from_str(u, "%Y%m%dT%H%M%S").ok().map(Stamp::Utc);
    }
    let n = NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").ok()?;
    Some(match tzid {
        Some(t) => Stamp::Zoned(n, zone_named(t, viewer)),
        None => Stamp::Floating(n),
    })
}

fn stamps(p: &Property<'_>, viewer: Tz) -> Vec<Stamp> {
    let tzid = param(p, "TZID");
    let date_only = param(p, "VALUE").is_some_and(|v| v.eq_ignore_ascii_case("DATE"));
    p.val
        .as_str()
        .split(',')
        .filter_map(|v| parse_stamp(v, tzid.as_deref(), date_only, viewer))
        .collect()
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// An RFC 5545 duration such as `PT90M`, `P1D`, `P1W` or `-PT15M`.
fn parse_duration(text: &str) -> Option<Duration> {
    let t = text.trim();
    let (negative, t) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let t = t.strip_prefix('P')?;
    let mut total = Duration::zero();
    let mut number = String::new();
    let mut in_time = false;
    for c in t.chars() {
        match c {
            '0'..='9' => number.push(c),
            'T' => in_time = true,
            unit => {
                let n: i64 = number.parse().ok()?;
                number.clear();
                total += match (unit, in_time) {
                    ('W', false) => Duration::weeks(n),
                    ('D', false) => Duration::days(n),
                    ('H', true) => Duration::hours(n),
                    ('M', true) => Duration::minutes(n),
                    ('S', true) => Duration::seconds(n),
                    _ => return None,
                };
            }
        }
    }
    if !number.is_empty() {
        return None;
    }
    Some(if negative { -total } else { total })
}

fn read_event(c: &Component<'_>, index: usize, viewer: Tz) -> Option<RawEvent> {
    let mut ev = RawEvent {
        uid: format!("backlith-no-uid-{index}"),
        title: String::new(),
        location: None,
        cancelled: false,
        transparent: false,
        start: Stamp::Floating(NaiveDateTime::MIN),
        end: None,
        duration: None,
        rrule: None,
        exdates: Vec::new(),
        rdates: Vec::new(),
        recurrence_id: None,
    };
    let mut has_start = false;
    for p in &c.properties {
        let value = p.val.as_str();
        match p.name.as_str().to_ascii_uppercase().as_str() {
            "UID" => ev.uid = value.trim().to_string(),
            "SUMMARY" => ev.title = unescape(value).trim().to_string(),
            "LOCATION" => ev.location = Some(unescape(value).trim().to_string()).filter(|l| !l.is_empty()),
            "STATUS" => ev.cancelled = value.trim().eq_ignore_ascii_case("CANCELLED"),
            "TRANSP" => ev.transparent = value.trim().eq_ignore_ascii_case("TRANSPARENT"),
            "DTSTART" => {
                if let Some(s) = stamps(p, viewer).into_iter().next() {
                    ev.start = s;
                    has_start = true;
                }
            }
            "DTEND" => ev.end = stamps(p, viewer).into_iter().next(),
            "DURATION" => ev.duration = parse_duration(value),
            "RRULE" => ev.rrule = Some(value.trim().to_string()),
            "EXDATE" => ev.exdates.extend(stamps(p, viewer)),
            "RDATE" => ev.rdates.extend(stamps(p, viewer)),
            "RECURRENCE-ID" => ev.recurrence_id = stamps(p, viewer).into_iter().next(),
            _ => {}
        }
    }
    if ev.title.is_empty() {
        ev.title = "(No title)".into();
    }
    has_start.then_some(ev)
}

/// How long each instance lasts. Never negative.
fn length_of(ev: &RawEvent) -> Duration {
    let span = match ev.end {
        Some(end) => match (ev.start, end) {
            (Stamp::Date(a), Stamp::Date(b)) => Duration::days((b - a).num_days()),
            (a, b) => b.key() - a.key(),
        },
        None => match ev.duration {
            Some(d) => d,
            None if ev.start.is_date() => Duration::days(1),
            None => Duration::zero(),
        },
    };
    span.max(Duration::zero())
}

/// RRULE's `UNTIL` must be in UTC when `DTSTART` has a zone, yet many feeds
/// write it as a local time or a bare date. Rewrite it in UTC.
fn normalize_until(rule: &str, start: Stamp) -> String {
    rule.split(';')
        .map(|part| {
            let Some((key, value)) = part.split_once('=') else {
                return part.to_string();
            };
            if !key.eq_ignore_ascii_case("UNTIL") {
                return part.to_string();
            }
            let local = match parse_stamp(value, None, false, chrono_tz::UTC) {
                Some(Stamp::Utc(n)) => return format!("UNTIL={}Z", n.format("%Y%m%dT%H%M%S")),
                Some(Stamp::Date(d)) => d.and_hms_opt(23, 59, 59).expect("a valid time"),
                Some(Stamp::Floating(n) | Stamp::Zoned(n, _)) => n,
                None => return String::new(),
            };
            let utc = match start {
                Stamp::Zoned(_, tz) => resolve(tz, local).naive_utc(),
                _ => local,
            };
            format!("UNTIL={}Z", utc.format("%Y%m%dT%H%M%S"))
        })
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(";")
}

/// The starts of a repeating event between `from` and `to` (viewer-ish
/// local bounds, already widened by the caller), in the event's own frame.
fn expand_rule(rule: &str, start: Stamp, from: NaiveDateTime, to: NaiveDateTime) -> Option<Vec<Stamp>> {
    let rule = normalize_until(rule, start);
    let (header, frame) = match start {
        Stamp::Zoned(n, tz) => (format!("DTSTART;TZID={}:{}", tz.name(), n.format("%Y%m%dT%H%M%S")), rrule::Tz::Tz(tz)),
        Stamp::Utc(n) => (format!("DTSTART:{}Z", n.format("%Y%m%dT%H%M%S")), rrule::Tz::UTC),
        // Dates and floating times have no zone: expand them as if in UTC
        // and read the wall-clock values back unchanged.
        Stamp::Floating(n) => (format!("DTSTART;TZID=UTC:{}", n.format("%Y%m%dT%H%M%S")), rrule::Tz::UTC),
        Stamp::Date(d) => (format!("DTSTART;TZID=UTC:{}", d.format("%Y%m%dT000000")), rrule::Tz::UTC),
    };
    let set: rrule::RRuleSet = format!("{header}\nRRULE:{rule}").parse().ok()?;
    let bound = |n: NaiveDateTime| match frame {
        rrule::Tz::Tz(tz) => resolve(tz, n).with_timezone(&frame),
        _ => Utc.from_utc_datetime(&n).with_timezone(&frame),
    };
    let found = set.after(bound(from)).before(bound(to)).all(MAX_INSTANCES);
    Some(
        found
            .dates
            .into_iter()
            .map(|dt| match start {
                Stamp::Zoned(_, tz) => Stamp::Zoned(dt.naive_local(), tz),
                Stamp::Utc(_) => Stamp::Utc(dt.naive_utc()),
                Stamp::Floating(_) => Stamp::Floating(dt.naive_utc()),
                Stamp::Date(_) => Stamp::Date(dt.naive_utc().date()),
            })
            .collect(),
    )
}

/// Whether an `EXDATE` removes this instance. A date-only `EXDATE` removes
/// every instance written on that date.
fn excluded(instance: Stamp, exdates: &[Stamp]) -> bool {
    exdates.iter().any(|x| match x {
        Stamp::Date(d) => instance.own_date() == *d,
        _ => x.key() == instance.key(),
    })
}

/// Every occurrence that overlaps `range`, in `viewer_tz` wall-clock time.
pub fn read_ics(text: &str, calendar_id: &str, viewer_tz: Tz, range: &DateRange) -> Result<Vec<Occurrence>, IcsError> {
    if !text.to_ascii_uppercase().contains("BEGIN:VCALENDAR") {
        return Err(IcsError::NotCalendar);
    }
    let unfolded = unfold(text);
    let calendar =
        read_calendar(&unfolded).map_err(|e| IcsError::Parse(e.lines().next().unwrap_or_default().to_string()))?;

    let events: Vec<RawEvent> = calendar
        .components
        .iter()
        .filter(|c| c.name.as_str().eq_ignore_ascii_case("VEVENT"))
        .enumerate()
        .filter_map(|(i, c)| read_event(c, i, viewer_tz))
        .collect();

    // The instances that overrides replace, per UID.
    let mut overridden: HashMap<&str, Vec<NaiveDateTime>> = HashMap::new();
    for ev in &events {
        if let Some(rid) = ev.recurrence_id {
            overridden.entry(ev.uid.as_str()).or_default().push(rid.key());
        }
    }

    let window_start = range.from.and_time(NaiveTime::MIN);
    let window_end = range.to.and_time(NaiveTime::MIN);
    let mut out = Vec::new();

    for ev in &events {
        if ev.cancelled {
            continue;
        }
        let length = length_of(ev);
        let mut starts = match (&ev.rrule, ev.recurrence_id) {
            // An override is one instance, at its own time.
            (_, Some(_)) => vec![ev.start],
            (Some(rule), None) => {
                let from = window_start - Duration::days(1) - length;
                let to = window_end + Duration::days(1);
                expand_rule(rule, ev.start, from, to).unwrap_or_else(|| vec![ev.start])
            }
            (None, None) => vec![ev.start],
        };
        if ev.recurrence_id.is_none() {
            starts.extend(ev.rdates.iter().copied());
        }
        let replaced = if ev.recurrence_id.is_none() { overridden.get(ev.uid.as_str()) } else { None };

        for start in starts {
            if excluded(start, &ev.exdates) || replaced.is_some_and(|r| r.contains(&start.key())) {
                continue;
            }
            let begin = start.to_viewer(viewer_tz);
            let end = start.end_after(length, viewer_tz);
            let overlaps = if end > begin {
                end > window_start && begin < window_end
            } else {
                begin >= window_start && begin < window_end
            };
            if overlaps {
                out.push(Occurrence {
                    calendar_id: calendar_id.to_string(),
                    title: ev.title.clone(),
                    location: ev.location.clone(),
                    start: begin,
                    end,
                    all_day: start.is_date(),
                    counts_as_busy: !ev.transparent,
                });
            }
        }
    }
    out.sort_by(|a, b| (a.start, a.end).cmp(&(b.start, b.end)));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(parse_duration("PT90M"), Some(Duration::minutes(90)));
        assert_eq!(parse_duration("P1DT2H"), Some(Duration::hours(26)));
        assert_eq!(parse_duration("P2W"), Some(Duration::weeks(2)));
        assert_eq!(parse_duration("-PT15M"), Some(Duration::minutes(-15)));
        assert_eq!(parse_duration("90 minutes"), None);
    }

    #[test]
    fn until_is_moved_to_utc() {
        let start = Stamp::Zoned(
            NaiveDate::from_ymd_opt(2026, 11, 2).unwrap().and_hms_opt(10, 0, 0).unwrap(),
            chrono_tz::Europe::Berlin,
        );
        assert_eq!(normalize_until("FREQ=WEEKLY;UNTIL=20261130T100000", start), "FREQ=WEEKLY;UNTIL=20261130T090000Z");
        assert_eq!(normalize_until("FREQ=DAILY;UNTIL=20261130", start), "FREQ=DAILY;UNTIL=20261130T225959Z");
        assert_eq!(normalize_until("FREQ=DAILY;UNTIL=20261130T090000Z", start), "FREQ=DAILY;UNTIL=20261130T090000Z");
    }

    #[test]
    fn zone_paths_and_unknown_names() {
        assert_eq!(zone_named("/mozilla.org/20050126_1/Europe/Berlin", chrono_tz::UTC), chrono_tz::Europe::Berlin);
        assert_eq!(zone_named("Nowhere Standard Time", chrono_tz::Europe::London), chrono_tz::Europe::London);
    }
}
