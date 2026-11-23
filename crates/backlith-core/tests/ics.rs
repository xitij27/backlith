// SPDX-License-Identifier: GPL-3.0-or-later
use backlith_core::{read_ics, windows_zone_to_iana, DateRange, IcsError, Occurrence};
use chrono::{NaiveDate, NaiveDateTime};
use chrono_tz::Tz;

const BERLIN: Tz = chrono_tz::Europe::Berlin;
const LONDON: Tz = chrono_tz::Europe::London;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}
fn at(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
}
fn range(from: &str, to: &str) -> DateRange {
    DateRange { from: d(from), to: d(to) }
}
fn read(text: &str, tz: Tz, r: DateRange) -> Vec<Occurrence> {
    let mut v = read_ics(text, "cal", tz, &r).expect("reads");
    v.sort_by_key(|o| o.start);
    v
}
fn titled<'a>(v: &'a [Occurrence], title: &str) -> &'a Occurrence {
    v.iter().find(|o| o.title == title).unwrap_or_else(|| panic!("no {title} in {v:#?}"))
}

#[test]
fn moved_and_deleted_instances() {
    let v = read(include_str!("fixtures/weekly_with_exdate_and_override.ics"), BERLIN, range("2026-11-16", "2026-11-30"));
    assert_eq!(v.len(), 1, "{v:#?}");
    assert_eq!(v[0].title, "Team meeting (moved)");
    assert_eq!((v[0].start, v[0].end), (at("2026-11-23T14:00"), at("2026-11-23T15:00")));
}

#[test]
fn outlook_windows_zone_names() {
    assert_eq!(windows_zone_to_iana("W. Europe Standard Time"), Some(BERLIN));
    let text = include_str!("fixtures/outlook_windows_tz.ics");
    let berlin = read(text, BERLIN, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&berlin, "Review").start, at("2026-11-24T09:00"));
    assert_eq!(titled(&berlin, "Planning").start, at("2026-11-24T11:00"), "quoted TZID");
    let london = read(text, LONDON, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&london, "Review").start, at("2026-11-24T08:00"));
}

#[test]
fn utc_and_floating_times() {
    let text = include_str!("fixtures/times.ics");
    let v = read(text, BERLIN, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&v, "UTC").start, at("2026-11-24T09:00"));
    assert_eq!(titled(&v, "Lecture, Room 2").start, at("2026-11-24T09:00"));
    let v = read(text, LONDON, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&v, "Lecture, Room 2").start, at("2026-11-24T09:00"), "floating stays put");
}

#[test]
fn text_is_unescaped() {
    let v = read(include_str!("fixtures/times.ics"), BERLIN, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&v, "Lecture, Room 2").location.as_deref(), Some("Building S1; Hall A"));
}

#[test]
fn all_day_without_end() {
    let v = read(include_str!("fixtures/times.ics"), BERLIN, range("2026-11-23", "2026-11-30"));
    let h = titled(&v, "Holiday");
    assert!(h.all_day);
    assert_eq!((h.start, h.end), (at("2026-11-24T00:00"), at("2026-11-25T00:00")));
}

#[test]
fn duration_instead_of_end() {
    let v = read(include_str!("fixtures/times.ics"), BERLIN, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&v, "Workshop").end, at("2026-11-25T11:30"));
}

#[test]
fn cancelled_dropped_transparent_kept() {
    let v = read(include_str!("fixtures/times.ics"), BERLIN, range("2026-11-23", "2026-11-30"));
    assert!(v.iter().all(|o| o.title != "Cancelled"));
    assert!(!titled(&v, "Maybe").counts_as_busy);
    assert!(titled(&v, "Workshop").counts_as_busy);
}

#[test]
fn weekly_event_keeps_its_time_across_dst() {
    // Summer time ends in Europe on Sunday 25 October 2026.
    let v = read(include_str!("fixtures/dst.ics"), BERLIN, range("2026-10-19", "2026-11-02"));
    let starts: Vec<_> = v.iter().map(|o| o.start).collect();
    assert_eq!(starts, vec![at("2026-10-19T10:00"), at("2026-10-26T10:00")]);
}

#[test]
fn only_the_range_is_returned() {
    let v = read(include_str!("fixtures/dst.ics"), BERLIN, range("2026-11-02", "2026-11-03"));
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].start, at("2026-11-02T10:00"));
}

#[test]
fn html_is_not_a_calendar() {
    let page = "<!doctype html><html><head><title>Sign in</title></head><body>Sign in</body></html>";
    assert_eq!(read_ics(page, "cal", BERLIN, &range("2026-11-23", "2026-11-30")), Err(IcsError::NotCalendar));
}

#[test]
fn a_byte_order_mark_is_ignored() {
    let text = format!("\u{feff}{}", include_str!("fixtures/times.ics"));
    let v = read(&text, BERLIN, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&v, "UTC").start, at("2026-11-24T09:00"));
}

#[test]
fn customized_zone_uses_the_files_own_vtimezone() {
    let v = read(include_str!("fixtures/customized_tz.ics"), BERLIN, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&v, "Call with New York").start, at("2026-11-24T15:00"));
}

#[test]
fn microsoft_tzone_names() {
    let v = read(include_str!("fixtures/tzone_utc.ics"), BERLIN, range("2026-11-23", "2026-11-30"));
    assert_eq!(titled(&v, "Exchange UTC").start, at("2026-11-24T10:00"));
}
