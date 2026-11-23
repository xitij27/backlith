// SPDX-License-Identifier: GPL-3.0-or-later
//! Reading iCalendar (ICS) text, from calendar links and uploaded `.ics`
//! files, into occurrences in the viewer's local time.
//!
//! Repeating events are expanded with the `rrule` crate. Exceptions are
//! handled here: `EXDATE` removes an instance, `RDATE` adds one, and an
//! event carrying a `RECURRENCE-ID` replaces the instance it names.

use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Utc, Weekday};
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
pub(crate) fn resolve(tz: Tz, n: NaiveDateTime) -> DateTime<Tz> {
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

/// The zones a calendar file can name: the viewer's, and those its own
/// `VTIMEZONE` blocks define under names no table knows.
struct Zones {
    viewer: Tz,
    custom: HashMap<String, Tz>,
}

/// A zone name every table knows: IANA, Windows, or Microsoft's
/// `tzone://Microsoft/…` form.
fn known_zone(name: &str) -> Option<Tz> {
    if let Ok(tz) = name.parse::<Tz>() {
        return Some(tz);
    }
    if let Some(tz) = windows_zone_to_iana(name) {
        return Some(tz);
    }
    let rest = name.strip_prefix("tzone://Microsoft/")?;
    if rest.eq_ignore_ascii_case("utc") {
        return Some(chrono_tz::UTC);
    }
    windows_zone_to_iana(rest).or_else(|| rest.parse().ok())
}

/// A `TZID` as an IANA zone: a known name, one the file's own `VTIMEZONE`
/// defines, or a path ending in an IANA name
/// (`/mozilla.org/20050126_1/Europe/Berlin`). Otherwise the viewer's zone.
fn zone_named(name: &str, zones: &Zones) -> Tz {
    known_zone(name)
        .or_else(|| zones.custom.get(name).copied())
        .or_else(|| name.match_indices('/').find_map(|(i, _)| name[i + 1..].parse::<Tz>().ok()))
        .unwrap_or(zones.viewer)
}

/// `+0100`, `-0500`, `+053000` as seconds east of UTC.
fn parse_offset(text: &str) -> Option<i32> {
    let t = text.trim();
    let (sign, digits) = match t.as_bytes().first()? {
        b'+' => (1, &t[1..]),
        b'-' => (-1, &t[1..]),
        _ => return None,
    };
    if !(digits.len() == 4 || digits.len() == 6) || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n = |r: std::ops::Range<usize>| digits[r].parse::<i32>().ok();
    let secs = n(0..2)? * 3600 + n(2..4)? * 60 + if digits.len() == 6 { n(4..6)? } else { 0 };
    Some(sign * secs)
}

/// One `STANDARD` or `DAYLIGHT` part of a `VTIMEZONE`.
struct Observance {
    daylight: bool,
    offset: i32,
    /// From its yearly RRULE: the month, and which weekday of it (`2SU`, `-1SU`).
    rule: Option<(u32, i32, Weekday)>,
}

fn weekday_code(code: &str) -> Option<Weekday> {
    Some(match code {
        "MO" => Weekday::Mon,
        "TU" => Weekday::Tue,
        "WE" => Weekday::Wed,
        "TH" => Weekday::Thu,
        "FR" => Weekday::Fri,
        "SA" => Weekday::Sat,
        "SU" => Weekday::Sun,
        _ => return None,
    })
}

fn yearly_rule(rrule: &str) -> Option<(u32, i32, Weekday)> {
    let mut month = None;
    let mut byday = None;
    for part in rrule.split(';') {
        let (k, v) = part.split_once('=')?;
        match k.to_ascii_uppercase().as_str() {
            "BYMONTH" => month = v.parse::<u32>().ok(),
            "BYDAY" => {
                let v = v.trim().to_ascii_uppercase();
                let (num, code) = v.split_at(v.len().saturating_sub(2));
                let n = if num.is_empty() { 1 } else { num.parse::<i32>().ok()? };
                byday = Some((n, weekday_code(code)?));
            }
            _ => {}
        }
    }
    let (n, wd) = byday?;
    Some((month?, n, wd))
}

/// The `n`th `wd` of a month; negative `n` counts from the end.
fn nth_weekday(year: i32, month: u32, n: i32, wd: Weekday) -> Option<NaiveDate> {
    if n > 0 {
        return NaiveDate::from_weekday_of_month_opt(year, month, wd, u8::try_from(n).ok()?);
    }
    let next = if month == 12 { NaiveDate::from_ymd_opt(year + 1, 1, 1)? } else { NaiveDate::from_ymd_opt(year, month + 1, 1)? };
    let mut d = next.pred_opt()?;
    while d.weekday() != wd {
        d = d.pred_opt()?;
    }
    Some(d - Duration::weeks(i64::from(-n - 1)))
}

/// The UTC offset on the 15th of each month of `year`, as the observances define it.
fn monthly_offsets(obs: &[Observance], year: i32) -> Option<[i32; 12]> {
    let standard = obs.iter().find(|o| !o.daylight);
    let daylight = obs.iter().find(|o| o.daylight);
    let mut out = [0; 12];
    for (i, slot) in out.iter_mut().enumerate() {
        let mid = NaiveDate::from_ymd_opt(year, i as u32 + 1, 15)?;
        *slot = match (standard, daylight) {
            (Some(s), Some(d)) => match (s.rule, d.rule) {
                (Some((sm, sn, sw)), Some((dm, dn, dw))) => {
                    let ends = nth_weekday(year, sm, sn, sw)?;
                    let starts = nth_weekday(year, dm, dn, dw)?;
                    let summer = if starts < ends { starts <= mid && mid < ends } else { !(ends <= mid && mid < starts) };
                    if summer { d.offset } else { s.offset }
                }
                _ => s.offset,
            },
            (Some(o), None) | (None, Some(o)) => o.offset,
            (None, None) => return None,
        };
    }
    Some(out)
}

/// Zones tried first when matching a `VTIMEZONE` by its offsets, so a
/// common zone wins over an obscure one with the same clock.
const PREFERRED_ZONES: [&str; 20] = [
    "America/New_York", "America/Chicago", "America/Denver", "America/Phoenix", "America/Los_Angeles",
    "America/Anchorage", "Pacific/Honolulu", "America/Sao_Paulo", "Europe/London", "Europe/Berlin",
    "Europe/Paris", "Europe/Athens", "Europe/Moscow", "Asia/Dubai", "Asia/Kolkata",
    "Asia/Shanghai", "Asia/Tokyo", "Australia/Sydney", "Pacific/Auckland", "UTC",
];

/// An IANA zone whose clock matches a `VTIMEZONE` block on the 15th of
/// every month of a recent year. Outlook names zones like this
/// "Customized Time Zone".
fn match_vtimezone(c: &Component<'_>) -> Option<Tz> {
    let obs: Vec<Observance> = c
        .components
        .iter()
        .filter_map(|o| {
            let daylight = match o.name.as_str().to_ascii_uppercase().as_str() {
                "DAYLIGHT" => true,
                "STANDARD" => false,
                _ => return None,
            };
            let get = |key: &str| o.properties.iter().find(|p| p.name.as_str().eq_ignore_ascii_case(key)).map(|p| p.val.as_str().to_string());
            Some(Observance { daylight, offset: parse_offset(&get("TZOFFSETTO")?)?, rule: get("RRULE").as_deref().and_then(yearly_rule) })
        })
        .collect();
    let year = 2026;
    let wanted = monthly_offsets(&obs, year)?;
    let table = crate::zones_windows::all_iana();
    let fixed = (-14..=12).map(|h: i32| if h == 0 { "Etc/GMT".to_string() } else { format!("Etc/GMT{}{}", if h > 0 { "+" } else { "-" }, h.abs()) });
    PREFERRED_ZONES
        .iter()
        .map(|z| z.to_string())
        .chain(table.into_iter().map(String::from))
        .chain(fixed)
        .filter_map(|name| name.parse::<Tz>().ok())
        .find(|tz| {
            (1..=12).all(|m| {
                let noon = NaiveDate::from_ymd_opt(year, m, 15).and_then(|d| d.and_hms_opt(12, 0, 0));
                noon.is_some_and(|n| resolve(*tz, n).offset().fix().local_minus_utc() == wanted[m as usize - 1])
            })
        })
}

fn parse_stamp(value: &str, tzid: Option<&str>, date_only: bool, zones: &Zones) -> Option<Stamp> {
    let v = value.trim();
    if date_only || (v.len() == 8 && v.bytes().all(|b| b.is_ascii_digit())) {
        return NaiveDate::parse_from_str(v, "%Y%m%d").ok().map(Stamp::Date);
    }
    if let Some(u) = v.strip_suffix(['Z', 'z']) {
        return NaiveDateTime::parse_from_str(u, "%Y%m%dT%H%M%S").ok().map(Stamp::Utc);
    }
    let n = NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").ok()?;
    Some(match tzid {
        Some(t) => Stamp::Zoned(n, zone_named(t, zones)),
        None => Stamp::Floating(n),
    })
}

fn stamps(p: &Property<'_>, zones: &Zones) -> Vec<Stamp> {
    let tzid = param(p, "TZID");
    let date_only = param(p, "VALUE").is_some_and(|v| v.eq_ignore_ascii_case("DATE"));
    p.val
        .as_str()
        .split(',')
        .filter_map(|v| {
            parse_stamp(v, tzid.as_deref(), date_only, zones).or_else(|| {
                // An unquoted TZID holding a colon (`TZID=tzone://Microsoft/Utc:2026…`)
                // is cut at the colon by the parser: put the name back together.
                let tz = tzid.as_deref()?;
                let (rest_of_name, time) = v.rsplit_once(':')?;
                parse_stamp(time, Some(&format!("{tz}:{rest_of_name}")), date_only, zones)
            })
        })
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

fn read_event(c: &Component<'_>, index: usize, zones: &Zones) -> Option<RawEvent> {
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
                if let Some(s) = stamps(p, zones).into_iter().next() {
                    ev.start = s;
                    has_start = true;
                }
            }
            "DTEND" => ev.end = stamps(p, zones).into_iter().next(),
            "DURATION" => ev.duration = parse_duration(value),
            "RRULE" => ev.rrule = Some(value.trim().to_string()),
            "EXDATE" => ev.exdates.extend(stamps(p, zones)),
            "RDATE" => ev.rdates.extend(stamps(p, zones)),
            "RECURRENCE-ID" => ev.recurrence_id = stamps(p, zones).into_iter().next(),
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
            let utc_only = Zones { viewer: chrono_tz::UTC, custom: HashMap::new() };
            let local = match parse_stamp(value, None, false, &utc_only) {
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
    let text = text.trim_start_matches('\u{feff}');
    if !text.to_ascii_uppercase().contains("BEGIN:VCALENDAR") {
        return Err(IcsError::NotCalendar);
    }
    let unfolded = unfold(text);
    let calendar =
        read_calendar(&unfolded).map_err(|e| IcsError::Parse(e.lines().next().unwrap_or_default().to_string()))?;

    // Zones the file defines itself, under names no table knows.
    let mut zones = Zones { viewer: viewer_tz, custom: HashMap::new() };
    for c in calendar.components.iter().filter(|c| c.name.as_str().eq_ignore_ascii_case("VTIMEZONE")) {
        let Some(id) = c.properties.iter().find(|p| p.name.as_str().eq_ignore_ascii_case("TZID")) else { continue };
        let id = id.val.as_str().trim().trim_matches('"').to_string();
        if known_zone(&id).is_none() {
            if let Some(tz) = match_vtimezone(c) {
                zones.custom.insert(id, tz);
            }
        }
    }

    let events: Vec<RawEvent> = calendar
        .components
        .iter()
        .filter(|c| c.name.as_str().eq_ignore_ascii_case("VEVENT"))
        .enumerate()
        .filter_map(|(i, c)| read_event(c, i, &zones))
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
        let utc = Zones { viewer: chrono_tz::UTC, custom: HashMap::new() };
        let london = Zones { viewer: chrono_tz::Europe::London, custom: HashMap::new() };
        assert_eq!(zone_named("/mozilla.org/20050126_1/Europe/Berlin", &utc), chrono_tz::Europe::Berlin);
        assert_eq!(zone_named("Nowhere Standard Time", &london), chrono_tz::Europe::London);
        assert_eq!(zone_named("tzone://Microsoft/Utc", &london), chrono_tz::UTC);
    }

    #[test]
    fn offsets_and_weekday_rules() {
        assert_eq!(parse_offset("-0500"), Some(-5 * 3600));
        assert_eq!(parse_offset("+0530"), Some(5 * 3600 + 30 * 60));
        assert_eq!(parse_offset("0100"), None);
        assert_eq!(yearly_rule("FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10"), Some((10, -1, Weekday::Sun)));
        assert_eq!(nth_weekday(2026, 3, 2, Weekday::Sun), NaiveDate::from_ymd_opt(2026, 3, 8));
        assert_eq!(nth_weekday(2026, 10, -1, Weekday::Sun), NaiveDate::from_ymd_opt(2026, 10, 25));
    }
}
