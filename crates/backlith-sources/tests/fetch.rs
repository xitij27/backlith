// SPDX-License-Identifier: GPL-3.0-or-later
use backlith_sources::{FetchOutcome, Fetcher};
use chrono::{NaiveDate, NaiveDateTime};
use httpmock::prelude::*;

const ICS_V1: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:a\r\nDTSTART:20261124T090000Z\r\nDTEND:20261124T100000Z\r\nSUMMARY:One\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
const WEEKLY: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:w\r\nDTSTART;TZID=Europe/Berlin:20261102T100000\r\nDTEND;TZID=Europe/Berlin:20261102T110000\r\nRRULE:FREQ=WEEKLY\r\nSUMMARY:Weekly\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

fn now() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2026-11-25T13:40", "%Y-%m-%dT%H:%M").unwrap()
}

#[tokio::test]
async fn first_fetch_caches_body_and_etag() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cal.ics");
            then.status(200).header("ETag", "\"v1\"").header("Content-Type", "text/calendar").body(ICS_V1);
        })
        .await;
    let dir = tempfile::tempdir().unwrap();
    let f = Fetcher::new(dir.path().to_path_buf());
    assert_eq!(f.fetch("work", &server.url("/cal.ics"), now()).await, FetchOutcome::Updated);
    assert_eq!(f.cached_text("work").as_deref(), Some(ICS_V1));
    assert_eq!(f.cache_meta("work").unwrap().etag.as_deref(), Some("\"v1\""));
}

#[tokio::test]
async fn second_fetch_sends_if_none_match() {
    let server = MockServer::start_async().await;
    let fresh = server
        .mock_async(|when, then| {
            when.method(GET).path("/cal.ics").header_missing("if-none-match");
            then.status(200).header("ETag", "\"v1\"").body(ICS_V1);
        })
        .await;
    let unchanged = server
        .mock_async(|when, then| {
            when.method(GET).path("/cal.ics").header("if-none-match", "\"v1\"");
            then.status(304);
        })
        .await;
    let dir = tempfile::tempdir().unwrap();
    let f = Fetcher::new(dir.path().to_path_buf());
    assert_eq!(f.fetch("work", &server.url("/cal.ics"), now()).await, FetchOutcome::Updated);
    assert_eq!(f.fetch("work", &server.url("/cal.ics"), now()).await, FetchOutcome::NotModified);
    fresh.assert_async().await;
    unchanged.assert_async().await;
    assert_eq!(f.cached_text("work").as_deref(), Some(ICS_V1));
}

async fn cached_then(status: u16, content_type: &str, body: &str) -> (FetchOutcome, Option<String>) {
    let server = MockServer::start_async().await;
    let good = server
        .mock_async(|when, then| {
            when.method(GET).path("/cal.ics");
            then.status(200).body(ICS_V1);
        })
        .await;
    let dir = tempfile::tempdir().unwrap();
    let f = Fetcher::new(dir.path().to_path_buf());
    assert_eq!(f.fetch("work", &server.url("/cal.ics"), now()).await, FetchOutcome::Updated);
    good.delete_async().await;
    let (ct, b) = (content_type.to_string(), body.to_string());
    server
        .mock_async(move |when, then| {
            when.method(GET).path("/cal.ics");
            then.status(status).header("Content-Type", ct).body(b);
        })
        .await;
    let outcome = f.fetch("work", &server.url("/cal.ics"), now()).await;
    (outcome, f.cached_text("work"))
}

#[tokio::test]
async fn server_error_keeps_last_good_copy() {
    let (outcome, cached) = cached_then(500, "text/plain", "oops").await;
    assert_eq!(outcome, FetchOutcome::Failed { reason: "the calendar server answered 500".into() });
    assert_eq!(cached.as_deref(), Some(ICS_V1));
}

#[tokio::test]
async fn html_page_keeps_last_good_copy() {
    let (outcome, cached) = cached_then(200, "text/html", "<!doctype html><html><body>Sign in</body></html>").await;
    assert_eq!(outcome, FetchOutcome::Failed { reason: "the link no longer returns a calendar".into() });
    assert_eq!(cached.as_deref(), Some(ICS_V1));
}

#[tokio::test]
async fn failure_reason_hides_the_link() {
    let dir = tempfile::tempdir().unwrap();
    let f = Fetcher::new(dir.path().to_path_buf());
    // Port 9 (discard) on localhost refuses the connection.
    let FetchOutcome::Failed { reason } = f.fetch("work", "http://127.0.0.1:9/secret-token-abc123/cal.ics", now()).await else {
        panic!("expected a failure");
    };
    assert!(!reason.contains("secret-token-abc123") && !reason.contains("127.0.0.1"), "{reason}");
    assert_eq!(f.cached_text("work"), None);
}

#[tokio::test]
async fn check_counts_upcoming_events() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/weekly.ics");
            then.status(200).body(WEEKLY);
        })
        .await;
    let dir = tempfile::tempdir().unwrap();
    let f = Fetcher::new(dir.path().to_path_buf());
    let today = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap();
    assert_eq!(f.check(&server.url("/weekly.ics"), chrono_tz::Europe::Berlin, today).await, Ok(8));
    assert_eq!(
        f.check(&server.url("/missing.ics"), chrono_tz::Europe::Berlin, today).await,
        Err("the calendar server answered 404".into())
    );
}

#[tokio::test]
async fn forget_removes_the_cache() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cal.ics");
            then.status(200).body(ICS_V1);
        })
        .await;
    let dir = tempfile::tempdir().unwrap();
    let f = Fetcher::new(dir.path().to_path_buf());
    f.fetch("work", &server.url("/cal.ics"), now()).await;
    f.forget("work");
    assert_eq!((f.cached_text("work"), f.cache_meta("work")), (None, None));
}
