// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use backlith_core::{load_json, plan_icons, secret_key, Base, ItemKind, LayoutError, UndoRecord};
use backlith_sources::{MemoryStore, SecretStore};
use chrono::{Duration, NaiveDateTime};
use httpmock::prelude::*;
use tempfile::TempDir;

use super::*;
use crate::desktop::fake::FakeDesktop;

const WEEKLY: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:w\r\nDTSTART;TZID=Europe/Berlin:20261102T100000\r\nDTEND;TZID=Europe/Berlin:20261102T110000\r\nRRULE:FREQ=WEEKLY\r\nSUMMARY:Weekly\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
const EXAMPLE: &str = include_str!("../../../schemas/schedule-v1.example.json");

fn at(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
}

struct Env {
    core: AppCore,
    desktop: Arc<FakeDesktop>,
    secrets: Arc<MemoryStore>,
    dir: TempDir,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let wallpaper = dir.path().join("old-wallpaper.jpg");
    std::fs::write(&wallpaper, b"not really a jpeg").unwrap();
    let desktop = Arc::new(FakeDesktop::new(Some(wallpaper.display().to_string())));
    let secrets = Arc::new(MemoryStore::default());
    let core = AppCore::new(desktop.clone(), secrets.clone(), dir.path().join("data")).unwrap().with_time_zone(chrono_tz::Europe::Berlin);
    Env { core, desktop, secrets, dir }
}

fn positions(d: &FakeDesktop) -> Vec<(String, backlith_core::PointPx)> {
    let mut v: Vec<_> = d.list_icons().unwrap().into_iter().map(|i| (i.id, i.position)).collect();
    v.sort();
    v
}

fn applied(e: &Env) {
    let state = e.core.planner_state(at("2026-11-25T13:40")).unwrap();
    e.core.save_layout(state.settings.layout).unwrap();
    e.core.apply_layout().unwrap();
}

async fn calendar_server() -> MockServer {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cal.ics");
            then.status(200).body(WEEKLY);
        })
        .await;
    server
}

#[test]
fn first_run_proposes_a_layout_that_fits_every_icon() {
    let e = env();
    let state = e.core.planner_state(at("2026-11-25T13:40")).unwrap();
    assert!(state.first_run && e.core.is_first_run());
    assert_eq!(state.icons.len(), 12);
    assert!(plan_icons(&state.settings.layout, &state.icons, &state.work_area, &state.grid).overflow.is_empty());
}

#[test]
fn first_run_keeps_a_copy_of_the_wallpaper() {
    let e = env();
    let state = e.core.planner_state(at("2026-11-25T13:40")).unwrap();
    let Base::Image { path } = state.settings.layout.base else { panic!("expected an image base") };
    assert!(path.starts_with(&e.dir.path().join("data").join("base").display().to_string()), "{path}");
    assert_eq!(std::fs::read(&path).unwrap(), b"not really a jpeg");
}

#[test]
fn save_layout_refuses_bad_layouts() {
    let e = env();
    let mut layout = e.core.planner_state(at("2026-11-25T13:40")).unwrap().settings.layout;
    let cal = layout.items.iter().find(|i| matches!(i.item, ItemKind::Calendar(_))).unwrap().clone();
    layout.items.push(backlith_core::LayoutItem { id: "second".into(), ..cal });
    assert_eq!(e.core.save_layout(layout), Err(LayoutError::TwoCalendars.to_string()));
}

#[test]
fn apply_takes_the_undo_record_only_once() {
    let e = env();
    let original = positions(&e.desktop);
    applied(&e);
    assert_ne!(positions(&e.desktop), original);
    e.desktop.move_icons(&[("C:\\Users\\me\\Desktop\\Music".into(), backlith_core::PointPx { x: 1500, y: 500 })]).unwrap();
    e.core.apply_layout().unwrap();
    let record: UndoRecord = load_json(&e.dir.path().join("data").join("undo.json")).unwrap().unwrap();
    let mut kept = record.icons.clone();
    kept.sort();
    assert_eq!(kept, original);
    assert!(record.wallpaper.unwrap().ends_with("old-wallpaper.jpg"));
}

#[test]
fn apply_turns_off_auto_arrange() {
    let e = env();
    assert!(e.desktop.auto_arrange().unwrap());
    applied(&e);
    assert!(!e.desktop.auto_arrange().unwrap());
    assert!(!e.core.is_first_run());
}

#[test]
fn put_icons_back_restores_positions() {
    let e = env();
    let original = positions(&e.desktop);
    applied(&e);
    e.core.put_icons_back().unwrap();
    assert_eq!(positions(&e.desktop), original);
}

#[test]
fn restore_wallpaper_puts_the_old_one_back() {
    let e = env();
    assert!(e.core.restore_wallpaper().is_err(), "nothing to restore before the first apply");
    applied(&e);
    e.desktop.set_wallpaper(std::path::Path::new("/tmp/backlith-wallpaper-a.png")).unwrap();
    e.core.restore_wallpaper().unwrap();
    assert!(e.desktop.current_wallpaper().unwrap().unwrap().ends_with("old-wallpaper.jpg"));
}

#[tokio::test]
async fn add_link_keeps_the_url_out_of_settings() {
    let e = env();
    let server = calendar_server().await;
    let url = server.url("/cal.ics");
    let status = e.core.add_link("Work".into(), "#F5B65C".into(), url.clone()).await.unwrap();
    assert!(status.ok);
    assert_eq!(status.event_count, 8);
    let settings = std::fs::read_to_string(e.dir.path().join("data").join("settings.json")).unwrap();
    assert!(!settings.contains("http"), "{settings}");
    assert_eq!(e.secrets.get(&secret_key(&status.id)).unwrap(), Some(url));
}

#[tokio::test]
async fn add_link_refuses_what_is_not_a_link() {
    let e = env();
    let err = e.core.add_link("Work".into(), "#F5B65C".into(), "my calendar".into()).await.unwrap_err();
    assert_eq!(err, "That doesn't look like a calendar link. It should start with https:// or webcal://.");
}

#[test]
fn upload_reports_schedule_errors() {
    let e = env();
    let bad = r#"{ "backlith": 1, "name": "Gym", "events": [ { "title": "Gym", "date": "2026-11-24", "start": "10am", "end": "11:00" } ] }"#;
    assert_eq!(
        e.core.upload("gym.json".into(), bad.as_bytes().to_vec(), false),
        Err(UploadError::Invalid { errors: vec!["Event 1 (Gym): `start` \"10am\" is not a time like 09:30.".into()] })
    );
}

#[test]
fn upload_same_name_needs_replace() {
    let e = env();
    e.core.upload("uni.json".into(), EXAMPLE.as_bytes().to_vec(), false).unwrap();
    assert_eq!(
        e.core.upload("uni.json".into(), EXAMPLE.as_bytes().to_vec(), false),
        Err(UploadError::Exists { name: "University timetable, winter 2026/27".into() })
    );
    e.core.upload("uni.json".into(), EXAMPLE.as_bytes().to_vec(), true).unwrap();
    assert_eq!(e.core.planner_state(at("2026-11-25T13:40")).unwrap().settings.calendars.len(), 1);
}

#[test]
fn upload_rejects_other_files() {
    let e = env();
    assert_eq!(
        e.core.upload("timetable.pdf".into(), b"%PDF-1.7".to_vec(), false),
        Err(UploadError::Invalid { errors: vec!["Use a .json schedule or an .ics calendar file.".into()] })
    );
    assert_eq!(
        e.core.upload("cal.ics".into(), b"<html>Sign in</html>".to_vec(), false),
        Err(UploadError::Invalid { errors: vec!["This file is not a calendar.".into()] })
    );
}

#[test]
fn uploaded_schedule_shows_in_the_week() {
    let e = env();
    e.core.upload("uni.json".into(), EXAMPLE.as_bytes().to_vec(), false).unwrap();
    let week = e.core.planner_state(at("2026-11-25T13:40")).unwrap().week;
    let lectures = week.days.iter().flat_map(|d| &d.events).filter(|ev| ev.title == "Machine Learning (lecture)").count();
    assert_eq!(lectures, 2, "Monday 23 and Thursday 26 November");
}

#[test]
fn switched_off_sources_leave_the_week() {
    let e = env();
    let status = e.core.upload("uni.json".into(), EXAMPLE.as_bytes().to_vec(), false).unwrap();
    e.core.update_source(status.id.clone(), "Uni".into(), "#4FBF9F".into(), false).unwrap();
    let state = e.core.planner_state(at("2026-11-25T13:40")).unwrap();
    assert_eq!(state.week.total_booked_minutes, 0);
    assert_eq!(state.settings.calendars[0].name, "Uni");
}

#[tokio::test]
async fn remove_source_deletes_its_secret() {
    let e = env();
    let server = calendar_server().await;
    let status = e.core.add_link("Work".into(), "#F5B65C".into(), server.url("/cal.ics")).await.unwrap();
    e.core.remove_source(status.id.clone()).unwrap();
    assert_eq!(e.secrets.get(&secret_key(&status.id)).unwrap(), None);
    assert!(e.core.planner_state(at("2026-11-25T13:40")).unwrap().settings.calendars.is_empty());
}

#[tokio::test]
async fn sources_report_since_when() {
    let e = env();
    let server = MockServer::start_async().await;
    let good = server
        .mock_async(|when, then| {
            when.method(GET).path("/cal.ics");
            then.status(200).body(WEEKLY);
        })
        .await;
    let status = e.core.add_link("Work".into(), "#F5B65C".into(), server.url("/cal.ics")).await.unwrap();
    good.delete_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cal.ics");
            then.status(500);
        })
        .await;
    e.core.refresh_links(at("2026-11-25T10:20")).await;
    e.core.refresh_links(at("2026-11-25T10:35")).await;
    let state = e.core.planner_state(at("2026-11-25T10:40")).unwrap();
    let s = state.sources.iter().find(|s| s.id == status.id).unwrap();
    assert!(!s.ok);
    assert_eq!(s.message.as_deref(), Some("unreachable since 10:20"));
    assert!(state.week.days.iter().flat_map(|d| &d.events).any(|ev| ev.title == "Weekly"), "events kept from the cache");
}

#[test]
fn tick_redraws_when_dirty_then_waits() {
    let e = env();
    let now = at("2026-11-25T13:40");
    applied(&e);
    assert!(e.core.tick(now).is_some());
    assert!(e.core.tick(now + Duration::minutes(1)).is_none());
    e.core.mark_dirty();
    assert!(e.core.tick(now + Duration::minutes(2)).is_some());
    assert!(e.core.toggle_pause());
    e.core.mark_dirty();
    assert!(e.core.tick(now + Duration::minutes(3)).is_none());
    assert!(!e.core.toggle_pause());
    assert!(e.core.tick(now + Duration::minutes(4)).is_some());
}

#[test]
fn preview_week_uses_the_given_settings_without_saving() {
    let e = env();
    e.core.upload("uni.json".into(), EXAMPLE.as_bytes().to_vec(), false).unwrap();
    let tool = backlith_core::CalendarToolSettings {
        week: backlith_core::WeekOptions { show_weekend: false, ..Default::default() },
        ..Default::default()
    };
    let week = e.core.preview_week(at("2026-11-25T13:40"), tool);
    assert_eq!(week.days.len(), 5);
    assert_eq!(e.core.planner_state(at("2026-11-25T13:40")).unwrap().week.days.len(), 7, "settings unchanged");
}

#[test]
fn no_redraw_before_the_first_apply() {
    // Until then the user's own wallpaper must stay, so the undo record keeps it.
    let e = env();
    assert!(e.core.tick(at("2026-11-25T13:40")).is_none());
    applied(&e);
    assert!(e.core.tick(at("2026-11-25T13:41")).is_some());
}

#[test]
fn restoring_the_wallpaper_stays_paused_after_a_restart() {
    let e = env();
    applied(&e);
    e.core.restore_wallpaper().unwrap();
    let again = AppCore::new(e.desktop.clone(), e.secrets.clone(), e.dir.path().join("data")).unwrap();
    assert!(again.planner_state(at("2026-11-25T13:40")).unwrap().settings.paused);
    assert!(again.tick(at("2026-11-25T13:40")).is_none());
}

#[test]
fn restore_uses_the_saved_copy_when_the_original_is_gone() {
    let e = env();
    applied(&e);
    std::fs::remove_file(e.dir.path().join("old-wallpaper.jpg")).unwrap();
    e.core.restore_wallpaper().unwrap();
    let now_set = e.desktop.current_wallpaper().unwrap().unwrap();
    assert!(now_set.starts_with(&e.dir.path().join("data").join("base").display().to_string()), "{now_set}");
}

#[test]
fn a_damaged_settings_file_starts_fresh_and_is_kept_aside() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("settings.json"), "{ \"version\": 1, \"layo").unwrap();
    let core = AppCore::new(Arc::new(FakeDesktop::new(None)), Arc::new(MemoryStore::default()), data.clone()).unwrap();
    assert!(core.is_first_run());
    let kept = std::fs::read_dir(&data).unwrap().filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().starts_with("settings.json.damaged"));
    assert!(kept, "the damaged file is kept for the user");
}

#[test]
fn an_unreadable_calendar_file_says_so_plainly() {
    let e = env();
    let broken = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:x\r\n";
    assert_eq!(
        e.core.upload("cal.ics".into(), broken.as_bytes().to_vec(), false),
        Err(UploadError::Invalid { errors: vec!["This calendar file could not be read.".into()] })
    );
}
