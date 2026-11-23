// SPDX-License-Identifier: GPL-3.0-or-later
//! Everything Backlith needs from the desktop it runs on. Windows gets a
//! real implementation in plan 2; Linux has a pretend desktop for
//! building and testing the planner.

use std::path::Path;

use backlith_core::{DesktopIcon, IconGrid, PointPx, RectPx};

pub mod fake;

pub trait Desktop: Send + Sync {
    /// The main screen, in physical pixels.
    fn screen(&self) -> anyhow::Result<RectPx>;
    /// The main screen minus the taskbar.
    fn work_area(&self) -> anyhow::Result<RectPx>;
    fn icon_grid(&self) -> anyhow::Result<IconGrid>;
    fn list_icons(&self) -> anyhow::Result<Vec<DesktopIcon>>;
    fn auto_arrange(&self) -> anyhow::Result<bool>;
    fn set_auto_arrange(&self, on: bool) -> anyhow::Result<()>;
    fn move_icons(&self, moves: &[(String, PointPx)]) -> anyhow::Result<()>;
    fn current_wallpaper(&self) -> anyhow::Result<Option<String>>;
    fn set_wallpaper(&self, png: &Path) -> anyhow::Result<()>;
    fn battery_saver(&self) -> bool;
    fn session_locked(&self) -> bool;
}
