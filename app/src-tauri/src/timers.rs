// SPDX-License-Identifier: GPL-3.0-or-later
//! One background task: fetch calendar links every 15 minutes, and ask
//! every minute whether the wallpaper is due a redraw. Plan 2 draws the
//! picture there; for now the render input goes to the planner if open.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::core::AppCore;

const FETCH_EVERY: Duration = Duration::from_secs(15 * 60);
const TICK_EVERY: Duration = Duration::from_secs(60);

pub fn spawn(app: AppHandle, core: Arc<AppCore>) {
    tauri::async_runtime::spawn(async move {
        let mut last_fetch: Option<Instant> = None;
        loop {
            let now = chrono::Local::now().naive_local();
            if last_fetch.is_none_or(|t| t.elapsed() >= FETCH_EVERY) {
                core.refresh_links(now).await;
                last_fetch = Some(Instant::now());
            }
            if let Some(input) = core.tick(now) {
                if let Some(planner) = app.get_webview_window("planner") {
                    let _ = planner.emit("render-input", &input);
                }
            }
            tokio::time::sleep(TICK_EVERY).await;
        }
    });
}
