// SPDX-License-Identifier: GPL-3.0-or-later
use backlith_core::{check_schedule, expand_schedule, llm_prompt, DateRange, MAX_SCHEDULE_BYTES};
use chrono::{NaiveDate, NaiveDateTime};
use serde_json::{json, Value};

const SCHEMA: &str = include_str!("../../../schemas/schedule-v1.json");
const EXAMPLE: &str = include_str!("../../../schemas/schedule-v1.example.json");

fn errors(v: Value) -> Vec<String> {
    match check_schedule(v.to_string().as_bytes()) {
        Ok(_) => panic!("expected errors for {v}"),
        Err(e) => e.iter().map(ToString::to_string).collect(),
    }
}
/// A file with one event, built from the given fields.
fn one(event: Value) -> Value {
    json!({ "backlith": 1, "name": "Test", "events": [event] })
}
fn gym(extra: Value) -> Value {
    let mut e = json!({ "title": "Gym", "days": ["mon"], "from": "2026-10-19", "until": "2026-12-18", "start": "18:00", "end": "19:00" });
    for (k, v) in extra.as_object().unwrap() {
        if v.is_null() {
            e.as_object_mut().unwrap().remove(k);
        } else {
            e[k] = v.clone();
        }
    }
    one(e)
}
fn schema_accepts(v: &Value) -> bool {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    jsonschema::validator_for(&schema).unwrap().is_valid(v)
}
fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}
fn at(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
}

#[test]
fn the_spec_example_passes_both_checkers() {
    assert!(check_schedule(EXAMPLE.as_bytes()).is_ok(), "{:?}", check_schedule(EXAMPLE.as_bytes()).err());
    assert!(schema_accepts(&serde_json::from_str(EXAMPLE).unwrap()));
}

#[test]
fn schema_and_checker_reject_the_same_things() {
    for bad in [gym(json!({ "strat": "18:00" })), gym(json!({ "date": "2026-10-19" })), gym(json!({ "allDay": true }))] {
        assert!(!schema_accepts(&bad), "schema let through {bad}");
        assert!(check_schedule(bad.to_string().as_bytes()).is_err(), "checker let through {bad}");
    }
}

#[test]
fn end_before_start() {
    let file = json!({ "backlith": 1, "name": "Uni", "events": [
        { "title": "Lecture", "date": "2026-11-24", "start": "10:00", "end": "11:00" },
        { "title": "Seminar", "date": "2026-11-24", "start": "12:00", "end": "13:00" },
        { "title": "Databases lab", "date": "2026-11-24", "start": "10:00", "end": "09:00" }
    ]});
    assert_eq!(errors(file), vec!["Event 3 (Databases lab): `end` 09:00 is before `start` 10:00"]);
}

#[test]
fn unknown_field_suggests() {
    let file = json!({ "backlith": 1, "name": "Uni", "events": [
        { "title": "Lecture", "date": "2026-11-24", "start": "10:00", "end": "11:00" },
        { "title": "Lab", "date": "2026-11-24", "strat": "10:00", "start": "10:00", "end": "11:00" }
    ]});
    assert_eq!(errors(file), vec!["Event 2 (Lab): unknown field `strat`; did you mean `start`?"]);
}

#[test]
fn unknown_field_no_hint() {
    assert_eq!(errors(gym(json!({ "colour": "red" }))), vec!["Event 1 (Gym): unknown field `colour`."]);
}

#[test]
fn wrong_day_name() {
    assert_eq!(
        errors(gym(json!({ "days": ["monday"] }))),
        vec!["Event 1 (Gym): `days` has \"monday\"; use mon, tue, wed, thu, fri, sat or sun."]
    );
}

#[test]
fn date_and_days_together() {
    assert_eq!(
        errors(gym(json!({ "date": "2026-10-19" }))),
        vec!["Event 1 (Gym): use either `date` or `days` with `from` and `until`, not both."]
    );
}

#[test]
fn neither_date_nor_days() {
    assert_eq!(
        errors(gym(json!({ "days": null, "from": null, "until": null }))),
        vec!["Event 1 (Gym): needs either `date`, or `days` with `from` and `until`."]
    );
}

#[test]
fn bad_date() {
    assert_eq!(errors(gym(json!({ "from": "19.10.2026" }))), vec!["Event 1 (Gym): `from` \"19.10.2026\" is not a date like 2026-10-19."]);
}

#[test]
fn bad_time() {
    assert_eq!(errors(gym(json!({ "start": "10am" }))), vec!["Event 1 (Gym): `start` \"10am\" is not a time like 09:30."]);
}

#[test]
fn until_before_from() {
    assert_eq!(errors(gym(json!({ "until": "2026-10-01" }))), vec!["Event 1 (Gym): `until` 2026-10-01 is before `from` 2026-10-19."]);
}

#[test]
fn all_day_with_times() {
    assert_eq!(errors(gym(json!({ "allDay": true }))), vec!["Event 1 (Gym): an `allDay` event has no `start` or `end`."]);
}

#[test]
fn missing_times() {
    assert_eq!(errors(gym(json!({ "start": null, "end": null }))), vec!["Event 1 (Gym): needs `start` and `end`, or `allDay`: true."]);
}

#[test]
fn bad_skip() {
    assert_eq!(
        errors(gym(json!({ "skip": ["Christmas"] }))),
        vec!["Event 1 (Gym): `skip` \"Christmas\" is not a date or a range like 2026-12-21/2027-01-03."]
    );
}

#[test]
fn bad_every() {
    assert_eq!(errors(gym(json!({ "every": 0 }))), vec!["Event 1 (Gym): `every` must be a whole number of weeks, 1 or more."]);
    assert_eq!(errors(gym(json!({ "every": 1.5 }))), vec!["Event 1 (Gym): `every` must be a whole number of weeks, 1 or more."]);
}

#[test]
fn missing_title() {
    assert_eq!(errors(one(json!({ "date": "2026-11-24", "start": "10:00", "end": "11:00" }))), vec!["Event 1: `title` is missing."]);
}

#[test]
fn file_level_errors() {
    assert_eq!(errors(json!({ "backlith": 2, "name": "X", "events": [{ "title": "A", "date": "2026-11-24", "allDay": true }] })), vec!["`backlith` must be 1."]);
    assert_eq!(errors(json!({ "backlith": 1, "events": [{ "title": "A", "date": "2026-11-24", "allDay": true }] })), vec!["`name` is missing."]);
    assert_eq!(errors(json!({ "backlith": 1, "name": "X", "events": [] })), vec!["`events` must list at least one event."]);
    assert_eq!(
        errors(json!({ "backlith": 1, "name": "X", "timezone": "Mars/Base", "events": [{ "title": "A", "date": "2026-11-24", "allDay": true }] })),
        vec!["`timezone` \"Mars/Base\" is not a time zone name like \"Europe/Berlin\"."]
    );
}

#[test]
fn not_json() {
    let e = check_schedule(b"{ backlith: 1 }").unwrap_err();
    assert_eq!(e.len(), 1);
    assert!(e[0].to_string().starts_with("This is not valid JSON: "), "{}", e[0]);
}

#[test]
fn too_big() {
    let big = vec![b' '; MAX_SCHEDULE_BYTES + 1];
    let e: Vec<String> = check_schedule(&big).unwrap_err().iter().map(ToString::to_string).collect();
    assert_eq!(e, vec!["The file is larger than 1 MB."]);
}

#[test]
fn all_errors_at_once() {
    let file = json!({ "backlith": 1, "name": "Uni", "events": [
        { "title": "A", "date": "24.11.2026", "start": "10:00", "end": "11:00" },
        { "title": "B", "date": "2026-11-24", "start": "10:00", "end": "11:00", "room": "1" },
        { "title": "C", "date": "2026-11-24", "start": "11:00", "end": "10:00" }
    ]});
    assert_eq!(
        errors(file),
        vec![
            "Event 1 (A): `date` \"24.11.2026\" is not a date like 2026-10-19.",
            "Event 2 (B): unknown field `room`.",
            "Event 3 (C): `end` 10:00 is before `start` 11:00",
        ]
    );
}

#[test]
fn expands_weekly_with_skip_and_every() {
    let file = check_schedule(EXAMPLE.as_bytes()).unwrap();
    let r = DateRange { from: d("2026-12-14"), to: d("2027-01-11") };
    let starts: Vec<_> = expand_schedule(&file, "uni", chrono_tz::Europe::Berlin, &r).into_iter().map(|o| o.start).collect();
    assert_eq!(starts, vec![at("2026-12-14T10:00"), at("2026-12-17T10:00"), at("2027-01-04T10:00"), at("2027-01-07T10:00")]);

    let fortnightly = gym(json!({ "every": 2, "until": "2026-11-30" }));
    let file = check_schedule(fortnightly.to_string().as_bytes()).unwrap();
    let r = DateRange { from: d("2026-10-19"), to: d("2026-11-23") };
    let starts: Vec<_> = expand_schedule(&file, "gym", chrono_tz::Europe::Berlin, &r).into_iter().map(|o| o.start).collect();
    assert_eq!(starts, vec![at("2026-10-19T18:00"), at("2026-11-02T18:00"), at("2026-11-16T18:00")]);
}

#[test]
fn file_zone_converts_to_viewer() {
    let file = check_schedule(EXAMPLE.as_bytes()).unwrap();
    let r = DateRange { from: d("2027-02-18"), to: d("2027-02-19") };
    let v = expand_schedule(&file, "uni", chrono_tz::Europe::London, &r);
    assert_eq!(v.len(), 1);
    assert_eq!((v[0].start, v[0].end), (at("2027-02-18T08:00"), at("2027-02-18T10:00")));
    assert_eq!(v[0].title, "Exam: Databases");
}

#[test]
fn all_day_events_stay_on_their_date() {
    let file = check_schedule(one(json!({ "title": "Holiday", "date": "2026-12-24", "allDay": true })).to_string().as_bytes()).unwrap();
    let r = DateRange { from: d("2026-12-21"), to: d("2026-12-28") };
    let v = expand_schedule(&file, "h", chrono_tz::Pacific::Auckland, &r);
    assert!(v[0].all_day);
    assert_eq!((v[0].start, v[0].end), (at("2026-12-24T00:00"), at("2026-12-25T00:00")));
}

#[test]
fn prompt_carries_schema_and_example() {
    let p = llm_prompt();
    assert!(p.contains(SCHEMA.trim()));
    assert!(p.contains("\"backlith\": 1"));
}
