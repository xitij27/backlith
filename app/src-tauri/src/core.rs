// SPDX-License-Identifier: GPL-3.0-or-later
//! Everything the planner's commands, the tray and the timers do, kept out
//! of Tauri so it can be tested on its own.
//!
//! Files in the data folder: `settings.json`, `undo.json`, `base/` (a copy
//! of the wallpaper from before Backlith), `calendars/` (uploaded files)
//! and `cache/` (fetched calendar links). Calendar links themselves are in
//! the secret store only.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use backlith_core::{
    build_week, check_schedule, decide_redraw, expand_schedule, first_run_layout, llm_prompt, load_json, plan_icons, read_ics,
    save_json, secret_key, validate_layout, week_dates, CalendarSource, CalendarToolSettings, DateRange, DesktopIcon, IcsError,
    IconGrid, ItemKind, Layout, Occurrence, RectPx, RedrawDecision, RedrawState, RenderInput, Settings, SourceKind, UndoRecord,
    WeekView, ZoneOverflow, SETTINGS_VERSION,
};
use backlith_sources::{normalize_link, FetchOutcome, Fetcher, SecretStore, MAX_CALENDAR_BYTES};
use chrono::{Duration, NaiveDate, NaiveDateTime};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::desktop::Desktop;

/// Calendar colours offered to new sources, chosen so neighbours differ in
/// lightness and not only in hue.
pub const PALETTE: [&str; 8] = ["#5A8CF0", "#F5B65C", "#4FBF9F", "#E06C9F", "#9B8CF0", "#F08A5A", "#8FD15A", "#5AC8E0"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SourceStatus {
    pub id: String,
    pub ok: bool,
    /// For example "unreachable since 10:20".
    pub message: Option<String>,
    pub event_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlannerState {
    pub settings: Settings,
    pub icons: Vec<DesktopIcon>,
    pub screen: RectPx,
    pub work_area: RectPx,
    pub grid: IconGrid,
    pub week: WeekView,
    pub sources: Vec<SourceStatus>,
    pub auto_arrange: bool,
    pub first_run: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApplyResult {
    pub moved: usize,
    pub overflow: Vec<ZoneOverflow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum UploadError {
    /// A calendar with this name exists; upload again with `replace` to replace it.
    Exists { name: String },
    Invalid { errors: Vec<String> },
    TooBig,
}

struct CoreState {
    settings: Settings,
    first_run: bool,
    /// When each failing link first failed, and why.
    failures: HashMap<String, (NaiveDateTime, String)>,
    redraw: RedrawState,
}

pub struct AppCore {
    desktop: Arc<dyn Desktop>,
    secrets: Arc<dyn SecretStore>,
    fetcher: Fetcher,
    data_dir: PathBuf,
    tz: Tz,
    state: Mutex<CoreState>,
}

/// The computer's time zone, or UTC if it can't be read.
fn system_zone() -> Tz {
    iana_time_zone::get_timezone().ok().and_then(|name| name.parse().ok()).unwrap_or(chrono_tz::UTC)
}

fn local_now() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

fn calendar_settings(layout: &Layout) -> CalendarToolSettings {
    layout
        .items
        .iter()
        .find_map(|i| match &i.item {
            ItemKind::Calendar(c) => Some(c.clone()),
            ItemKind::IconZone(_) => None,
        })
        .unwrap_or_default()
}

fn extension(file_name: &str) -> String {
    Path::new(file_name).extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase()
}

fn upcoming(today: NaiveDate) -> DateRange {
    DateRange { from: today, to: today + Duration::weeks(8) }
}

impl AppCore {
    pub fn new(desktop: Arc<dyn Desktop>, secrets: Arc<dyn SecretStore>, data_dir: PathBuf) -> anyhow::Result<Self> {
        let saved: Option<Settings> = load_json(&data_dir.join("settings.json"))?;
        let (settings, first_run) = match saved {
            Some(s) => (s, false),
            None => {
                let icons = desktop.list_icons()?;
                let base = Self::keep_wallpaper_copy(&*desktop, &data_dir);
                let layout = first_run_layout(&icons, &desktop.work_area()?, &desktop.icon_grid()?, base);
                (Settings::new(layout), true)
            }
        };
        Ok(Self {
            fetcher: Fetcher::new(data_dir.join("cache")),
            desktop,
            secrets,
            data_dir,
            tz: system_zone(),
            state: Mutex::new(CoreState { settings, first_run, failures: HashMap::new(), redraw: RedrawState::default() }),
        })
    }

    /// Uses `tz` as the viewer's zone instead of the computer's.
    pub fn with_time_zone(mut self, tz: Tz) -> Self {
        self.tz = tz;
        self
    }

    /// Copies the current wallpaper into `base/`, so the base survives the
    /// user changing or deleting the original.
    fn keep_wallpaper_copy(desktop: &dyn Desktop, data_dir: &Path) -> Option<String> {
        let original = PathBuf::from(desktop.current_wallpaper().ok()??);
        let name = original.file_name()?;
        let dir = data_dir.join("base");
        std::fs::create_dir_all(&dir).ok()?;
        let copy = dir.join(name);
        std::fs::copy(&original, &copy).ok()?;
        Some(copy.display().to_string())
    }

    fn lock(&self) -> MutexGuard<'_, CoreState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn save(&self, state: &CoreState) -> Result<(), String> {
        save_json(&self.data_dir.join("settings.json"), &state.settings).map_err(|e| e.to_string())
    }

    pub fn is_first_run(&self) -> bool {
        self.lock().first_run
    }

    fn calendars_dir(&self) -> PathBuf {
        self.data_dir.join("calendars")
    }

    /// One source's occurrences within `range`.
    fn occurrences(&self, source: &CalendarSource, range: &DateRange) -> Vec<Occurrence> {
        let text = match &source.kind {
            SourceKind::Link => self.fetcher.cached_text(&source.id),
            SourceKind::IcsFile { file } | SourceKind::Schedule { file } => std::fs::read_to_string(self.calendars_dir().join(file)).ok(),
        };
        let Some(text) = text else { return Vec::new() };
        match &source.kind {
            SourceKind::Schedule { .. } => match check_schedule(text.as_bytes()) {
                Ok(file) => expand_schedule(&file, &source.id, self.tz, range),
                Err(_) => Vec::new(),
            },
            _ => read_ics(&text, &source.id, self.tz, range).unwrap_or_default(),
        }
    }

    fn week(&self, settings: &Settings, now: NaiveDateTime) -> WeekView {
        let tool = calendar_settings(&settings.layout);
        let (from, to) = week_dates(now, tool.week.mode);
        let range = DateRange { from: from - Duration::days(1), to: to + Duration::days(1) };
        let occurrences: Vec<Occurrence> = settings
            .calendars
            .iter()
            .filter(|s| s.enabled && (tool.calendar_ids.is_empty() || tool.calendar_ids.contains(&s.id)))
            .flat_map(|s| self.occurrences(s, &range))
            .collect();
        build_week(&occurrences, now, &tool.week)
    }

    fn status(&self, state: &CoreState, source: &CalendarSource, today: NaiveDate) -> SourceStatus {
        let failure = state.failures.get(&source.id);
        SourceStatus {
            id: source.id.clone(),
            ok: failure.is_none(),
            message: failure.map(|(since, _)| format!("unreachable since {}", since.format("%H:%M"))),
            event_count: self.occurrences(source, &upcoming(today)).len(),
        }
    }

    pub fn planner_state(&self, now: NaiveDateTime) -> anyhow::Result<PlannerState> {
        let state = self.lock();
        let sources = state.settings.calendars.iter().map(|s| self.status(&state, s, now.date())).collect();
        Ok(PlannerState {
            settings: state.settings.clone(),
            icons: self.desktop.list_icons()?,
            screen: self.desktop.screen()?,
            work_area: self.desktop.work_area()?,
            grid: self.desktop.icon_grid()?,
            week: self.week(&state.settings, now),
            sources,
            auto_arrange: self.desktop.auto_arrange()?,
            first_run: state.first_run,
        })
    }

    pub fn save_layout(&self, layout: Layout) -> Result<(), String> {
        validate_layout(&layout).map_err(|e| e.to_string())?;
        let mut state = self.lock();
        state.settings.layout = layout;
        state.settings.version = SETTINGS_VERSION;
        self.save(&state)?;
        state.first_run = false;
        state.redraw.dirty = true;
        Ok(())
    }

    fn undo_path(&self) -> PathBuf {
        self.data_dir.join("undo.json")
    }

    pub fn apply_layout(&self) -> Result<ApplyResult, String> {
        let fail = |e: anyhow::Error| e.to_string();
        let icons = self.desktop.list_icons().map_err(fail)?;
        if !self.undo_path().exists() {
            let record = UndoRecord {
                version: SETTINGS_VERSION,
                icons: icons.iter().map(|i| (i.id.clone(), i.position)).collect(),
                wallpaper: self.desktop.current_wallpaper().map_err(fail)?,
                taken_at: local_now(),
            };
            save_json(&self.undo_path(), &record).map_err(|e| e.to_string())?;
        }
        if self.desktop.auto_arrange().map_err(fail)? {
            self.desktop.set_auto_arrange(false).map_err(fail)?;
        }
        let mut state = self.lock();
        let plan = plan_icons(
            &state.settings.layout,
            &icons,
            &self.desktop.work_area().map_err(fail)?,
            &self.desktop.icon_grid().map_err(fail)?,
        );
        self.desktop.move_icons(&plan.moves).map_err(fail)?;
        state.first_run = false;
        state.redraw.dirty = true;
        self.save(&state)?;
        Ok(ApplyResult { moved: plan.moves.len(), overflow: plan.overflow })
    }

    fn undo_record(&self) -> Result<UndoRecord, String> {
        load_json(&self.undo_path())
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Backlith hasn't changed anything yet, so there is nothing to put back.".to_string())
    }

    pub fn put_icons_back(&self) -> Result<(), String> {
        let record = self.undo_record()?;
        self.desktop.move_icons(&record.icons).map_err(|e| e.to_string())
    }

    pub fn restore_wallpaper(&self) -> Result<(), String> {
        let record = self.undo_record()?;
        let path = record.wallpaper.ok_or_else(|| "Backlith has no record of your old wallpaper.".to_string())?;
        self.desktop.set_wallpaper(Path::new(&path)).map_err(|e| e.to_string())?;
        self.lock().settings.paused = true;
        Ok(())
    }

    fn new_id(state: &CoreState) -> String {
        let base = format!("src-{:x}", chrono::Utc::now().timestamp_millis());
        let mut id = base.clone();
        let mut n = 1;
        while state.settings.calendars.iter().any(|s| s.id == id) {
            n += 1;
            id = format!("{base}-{n}");
        }
        id
    }

    pub async fn add_link(&self, name: String, color: String, url: String) -> Result<SourceStatus, String> {
        let url = normalize_link(&url).map_err(|e| e.to_string())?;
        let now = local_now();
        // Network first, with no lock held across the await.
        let event_count = self.fetcher.check(&url, self.tz, now.date()).await?;
        let id = Self::new_id(&self.lock());
        self.secrets.set(&secret_key(&id), &url).map_err(|e| e.to_string())?;
        if let FetchOutcome::Failed { reason } = self.fetcher.fetch(&id, &url, now).await {
            let _ = self.secrets.delete(&secret_key(&id));
            return Err(reason);
        }
        let mut state = self.lock();
        state.settings.calendars.push(CalendarSource { id: id.clone(), name, color, enabled: true, kind: SourceKind::Link });
        if let Err(e) = self.save(&state) {
            state.settings.calendars.retain(|s| s.id != id);
            let _ = self.secrets.delete(&secret_key(&id));
            return Err(e);
        }
        state.redraw.dirty = true;
        Ok(SourceStatus { id, ok: true, message: None, event_count })
    }

    pub fn upload(&self, file_name: String, bytes: Vec<u8>, replace: bool) -> Result<SourceStatus, UploadError> {
        let today = local_now().date();
        let (name, kind_is_schedule) = match extension(&file_name).as_str() {
            "json" => match check_schedule(&bytes) {
                Ok(file) => (file.name, true),
                Err(errors) => return Err(UploadError::Invalid { errors: errors.iter().map(ToString::to_string).collect() }),
            },
            "ics" => {
                if bytes.len() > MAX_CALENDAR_BYTES {
                    return Err(UploadError::TooBig);
                }
                let text = String::from_utf8_lossy(&bytes);
                match read_ics(&text, "upload", self.tz, &upcoming(today)) {
                    Ok(_) => {}
                    Err(IcsError::NotCalendar) => return Err(UploadError::Invalid { errors: vec!["This file is not a calendar.".into()] }),
                    Err(e @ IcsError::Parse(_)) => return Err(UploadError::Invalid { errors: vec![format!("This calendar file {e}.")] }),
                }
                let stem = Path::new(&file_name).file_stem().and_then(|s| s.to_str()).unwrap_or("Calendar").to_string();
                (stem, false)
            }
            _ => return Err(UploadError::Invalid { errors: vec!["Use a .json schedule or an .ics calendar file.".into()] }),
        };

        let mut state = self.lock();
        let existing = state
            .settings
            .calendars
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(&name) && !matches!(s.kind, SourceKind::Link));
        if existing.is_some() && !replace {
            return Err(UploadError::Exists { name });
        }
        let id = match existing {
            Some(i) => state.settings.calendars[i].id.clone(),
            None => Self::new_id(&state),
        };
        let file = format!("{id}.{}", if kind_is_schedule { "json" } else { "ics" });
        let write = std::fs::create_dir_all(self.calendars_dir()).and_then(|()| std::fs::write(self.calendars_dir().join(&file), &bytes));
        if write.is_err() {
            return Err(UploadError::Invalid { errors: vec!["Backlith could not save the file on this computer.".into()] });
        }
        let kind = if kind_is_schedule { SourceKind::Schedule { file } } else { SourceKind::IcsFile { file } };
        match existing {
            Some(i) => state.settings.calendars[i].kind = kind,
            None => {
                let used: Vec<&str> = state.settings.calendars.iter().map(|s| s.color.as_str()).collect();
                let color = PALETTE.iter().find(|c| !used.contains(c)).unwrap_or(&PALETTE[0]).to_string();
                state.settings.calendars.push(CalendarSource { id: id.clone(), name, color, enabled: true, kind });
            }
        }
        self.save(&state).map_err(|e| UploadError::Invalid { errors: vec![e] })?;
        state.redraw.dirty = true;
        let source = state.settings.calendars.iter().find(|s| s.id == id).expect("just added").clone();
        Ok(self.status(&state, &source, today))
    }

    pub fn update_source(&self, id: String, name: String, color: String, enabled: bool) -> Result<(), String> {
        let mut state = self.lock();
        let source = state.settings.calendars.iter_mut().find(|s| s.id == id).ok_or("That calendar is gone.")?;
        source.name = name;
        source.color = color;
        source.enabled = enabled;
        self.save(&state)?;
        state.redraw.dirty = true;
        Ok(())
    }

    pub fn remove_source(&self, id: String) -> Result<(), String> {
        let mut state = self.lock();
        let i = state.settings.calendars.iter().position(|s| s.id == id).ok_or("That calendar is gone.")?;
        let source = state.settings.calendars.remove(i);
        self.save(&state)?;
        match source.kind {
            SourceKind::Link => {
                self.secrets.delete(&secret_key(&id)).map_err(|e| e.to_string())?;
                self.fetcher.forget(&id);
            }
            SourceKind::IcsFile { file } | SourceKind::Schedule { file } => {
                let _ = std::fs::remove_file(self.calendars_dir().join(file));
            }
        }
        state.failures.remove(&id);
        state.redraw.dirty = true;
        Ok(())
    }

    pub fn llm_prompt(&self) -> String {
        llm_prompt()
    }

    pub fn render_input(&self, now: NaiveDateTime) -> anyhow::Result<RenderInput> {
        let state = self.lock();
        Ok(RenderInput {
            layout: state.settings.layout.clone(),
            week: self.week(&state.settings, now),
            calendars: state.settings.calendars.clone(),
            screen: self.desktop.screen()?,
            work_area: self.desktop.work_area()?,
            scale: self.desktop.icon_grid()?.scale,
        })
    }

    /// Fetches every enabled calendar link. A failure keeps the cached copy
    /// and records when the link first failed.
    pub async fn refresh_links(&self, now: NaiveDateTime) {
        let links: Vec<String> = self
            .lock()
            .settings
            .calendars
            .iter()
            .filter(|s| s.enabled && matches!(s.kind, SourceKind::Link))
            .map(|s| s.id.clone())
            .collect();
        for id in links {
            let Ok(Some(url)) = self.secrets.get(&secret_key(&id)) else {
                self.lock().failures.entry(id).or_insert((now, "the link is missing from the secret store".into()));
                continue;
            };
            let outcome = self.fetcher.fetch(&id, &url, now).await;
            let mut state = self.lock();
            match outcome {
                FetchOutcome::Updated => {
                    state.failures.remove(&id);
                    state.redraw.dirty = true;
                }
                FetchOutcome::NotModified => {
                    state.failures.remove(&id);
                }
                FetchOutcome::Failed { reason } => {
                    state.failures.entry(id).or_insert((now, reason));
                }
            }
        }
    }

    /// Called every minute: the render input when the wallpaper is due a redraw.
    pub fn tick(&self, now: NaiveDateTime) -> Option<RenderInput> {
        {
            let mut state = self.lock();
            state.redraw.locked = self.desktop.session_locked();
            state.redraw.battery_saver = self.desktop.battery_saver();
            state.redraw.paused = state.settings.paused;
            if decide_redraw(now, &state.redraw) != RedrawDecision::Now {
                return None;
            }
            state.redraw.last_redraw = Some(now);
            state.redraw.dirty = false;
        }
        self.render_input(now).ok()
    }

    pub fn mark_dirty(&self) {
        self.lock().redraw.dirty = true;
    }

    /// Pauses or resumes redrawing. Returns whether Backlith is now paused.
    pub fn toggle_pause(&self) -> bool {
        let mut state = self.lock();
        state.settings.paused = !state.settings.paused;
        if !state.settings.paused {
            state.redraw.dirty = true;
        }
        let _ = self.save(&state);
        state.settings.paused
    }
}

#[cfg(test)]
#[path = "core_tests.rs"]
mod tests;
