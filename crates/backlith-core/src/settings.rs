// SPDX-License-Identifier: GPL-3.0-or-later
//! What Backlith keeps on disk: settings, the record that undoes its icon
//! moves and wallpaper change, and what the renderer is given to draw.
//!
//! A calendar link is never kept here. Its URL lives in the secret store,
//! under [`secret_key`].

use std::path::Path;

use chrono::NaiveDateTime;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::{Layout, PointPx, RectPx};
use crate::week::WeekView;

/// The settings format this build reads and writes.
pub const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum SourceKind {
    /// A calendar link. Its URL is in the secret store.
    Link,
    /// An uploaded `.ics` file, copied into Backlith's folder.
    IcsFile { file: String },
    /// An uploaded Backlith schedule, copied into Backlith's folder.
    Schedule { file: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CalendarSource {
    pub id: String,
    pub name: String,
    /// `#RRGGBB`.
    pub color: String,
    pub enabled: bool,
    pub kind: SourceKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Settings {
    pub version: u32,
    pub layout: Layout,
    pub calendars: Vec<CalendarSource>,
    pub start_at_sign_in: bool,
    pub paused: bool,
}

impl Settings {
    pub fn new(layout: Layout) -> Self {
        Self { version: SETTINGS_VERSION, layout, calendars: Vec::new(), start_at_sign_in: true, paused: false }
    }
}

/// Taken once, before Backlith first moves icons or changes the wallpaper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UndoRecord {
    pub version: u32,
    pub icons: Vec<(String, PointPx)>,
    pub wallpaper: Option<String>,
    pub taken_at: NaiveDateTime,
}

/// Everything the renderer needs to draw the background.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RenderInput {
    pub layout: Layout,
    pub week: WeekView,
    pub calendars: Vec<CalendarSource>,
    pub screen: RectPx,
    pub work_area: RectPx,
    pub scale: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SettingsError {
    #[error("could not read or write the file: {0}")]
    Io(String),
    #[error("the file is damaged: {0}")]
    Corrupt(String),
    #[error("the file was written by a newer Backlith (format {0})")]
    NewerVersion(u32),
}

/// The secret-store key holding a calendar link's URL.
pub fn secret_key(source_id: &str) -> String {
    format!("calendar-link:{source_id}")
}

/// A missing file is `Ok(None)`. A `version` newer than this build's is refused.
pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, SettingsError> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(SettingsError::Io(e.to_string())),
    };
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| SettingsError::Corrupt(e.to_string()))?;
    if let Some(v) = value.get("version").and_then(serde_json::Value::as_u64) {
        if v > u64::from(SETTINGS_VERSION) {
            return Err(SettingsError::NewerVersion(u32::try_from(v).unwrap_or(u32::MAX)));
        }
    }
    serde_json::from_value(value).map(Some).map_err(|e| SettingsError::Corrupt(e.to_string()))
}

/// Writes `<name>.tmp` beside the file, then renames it over the file, so a
/// failed save never leaves a half-written file.
pub fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<(), SettingsError> {
    let io = |e: std::io::Error| SettingsError::Io(e.to_string());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|e| SettingsError::Corrupt(e.to_string()))?;
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    std::fs::write(&tmp, text).map_err(io)?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        io(e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Base;

    fn settings() -> Settings {
        let mut s = Settings::new(Layout { version: 1, base: Base::Color { hex: "#0D1117".into() }, items: vec![] });
        s.calendars.push(CalendarSource { id: "work".into(), name: "Work".into(), color: "#F5B65C".into(), enabled: true, kind: SourceKind::Link });
        s
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("settings.json");
        save_json(&path, &settings()).unwrap();
        assert_eq!(load_json::<Settings>(&path).unwrap(), Some(settings()));
        assert!(!dir.path().join("nested").join("settings.json.tmp").exists());
    }

    #[test]
    fn missing_file_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_json::<Settings>(&dir.path().join("nope.json")).unwrap(), None);
    }

    #[test]
    fn newer_version_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let mut v = serde_json::to_value(settings()).unwrap();
        v["version"] = 2.into();
        std::fs::write(&path, v.to_string()).unwrap();
        assert_eq!(load_json::<Settings>(&path), Err(SettingsError::NewerVersion(2)));
    }

    #[test]
    fn damaged_file_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{ \"version\": 1, \"layout\": ").unwrap();
        assert!(matches!(load_json::<Settings>(&path), Err(SettingsError::Corrupt(_))));
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_save_leaves_the_old_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        save_json(&path, &settings()).unwrap();
        let before = std::fs::read_to_string(&path).unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
        let mut changed = settings();
        changed.paused = true;
        let result = save_json(&path, &changed);
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(result, Err(SettingsError::Io(_))));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    }

    #[test]
    fn settings_never_hold_a_link() {
        let json = serde_json::to_string(&settings()).unwrap();
        assert!(!json.contains("http") && !json.contains("webcal"), "{json}");
        assert_eq!(secret_key("work"), "calendar-link:work");
    }
}
