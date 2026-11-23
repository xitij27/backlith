// SPDX-License-Identifier: GPL-3.0-or-later
//! The planner's commands: thin wrappers over [`AppCore`]. Their names are
//! what `app/src/api.ts` invokes.

use std::sync::Arc;

use backlith_core::{Layout, RenderInput};
use chrono::NaiveDateTime;
use tauri::State;

use crate::core::{AppCore, ApplyResult, PlannerState, SourceStatus, UploadError};

type Core<'a> = State<'a, Arc<AppCore>>;

fn now() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

#[tauri::command]
pub fn planner_state(core: Core<'_>) -> Result<PlannerState, String> {
    core.planner_state(now()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_layout(core: Core<'_>, layout: Layout) -> Result<(), String> {
    core.save_layout(layout)
}

#[tauri::command]
pub fn apply_layout(core: Core<'_>) -> Result<ApplyResult, String> {
    core.apply_layout()
}

#[tauri::command]
pub fn put_icons_back(core: Core<'_>) -> Result<(), String> {
    core.put_icons_back()
}

#[tauri::command]
pub fn restore_wallpaper(core: Core<'_>) -> Result<(), String> {
    core.restore_wallpaper()
}

#[tauri::command]
pub async fn add_link(core: Core<'_>, name: String, color: String, url: String) -> Result<SourceStatus, String> {
    core.add_link(name, color, url).await
}

#[tauri::command]
pub fn upload(core: Core<'_>, file_name: String, bytes: Vec<u8>, replace: bool) -> Result<SourceStatus, UploadError> {
    core.upload(file_name, bytes, replace)
}

#[tauri::command]
pub fn update_source(core: Core<'_>, id: String, name: String, color: String, enabled: bool) -> Result<(), String> {
    core.update_source(id, name, color, enabled)
}

#[tauri::command]
pub fn remove_source(core: Core<'_>, id: String) -> Result<(), String> {
    core.remove_source(id)
}

#[tauri::command]
pub fn llm_prompt(core: Core<'_>) -> String {
    core.llm_prompt()
}

#[tauri::command]
pub fn render_input(core: Core<'_>) -> Result<RenderInput, String> {
    core.render_input(now()).map_err(|e| e.to_string())
}
