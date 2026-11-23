// SPDX-License-Identifier: GPL-3.0-or-later
//! A pretend Windows desktop: a 1920×1080 screen with a 48 px taskbar, a
//! 100 px icon grid and twelve icons, two of them named "New folder".

use std::path::Path;
use std::sync::Mutex;

use backlith_core::{DesktopIcon, IconGrid, PointPx, RectPx};

use super::Desktop;

struct State {
    icons: Vec<DesktopIcon>,
    auto_arrange: bool,
    wallpaper: Option<String>,
}

pub struct FakeDesktop {
    state: Mutex<State>,
}

const NAMES: [&str; 10] = ["Recycle Bin", "Projects", "Downloads", "Notes.txt", "Report.pdf", "Photos", "Terminal", "Music", "Budget.xlsx", "Slides.pptx"];

impl FakeDesktop {
    pub fn new(wallpaper: Option<String>) -> Self {
        let mut icons: Vec<DesktopIcon> = NAMES
            .iter()
            .enumerate()
            .map(|(i, n)| DesktopIcon { id: format!("C:\\Users\\me\\Desktop\\{n}"), name: n.to_string(), position: PointPx { x: 0, y: i as i32 * 100 } })
            .collect();
        icons.push(DesktopIcon { id: "C:\\Users\\me\\Desktop\\New folder".into(), name: "New folder".into(), position: PointPx { x: 100, y: 0 } });
        icons.push(DesktopIcon { id: "C:\\Users\\Public\\Desktop\\New folder".into(), name: "New folder".into(), position: PointPx { x: 100, y: 100 } });
        Self { state: Mutex::new(State { icons, auto_arrange: true, wallpaper }) }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Desktop for FakeDesktop {
    fn screen(&self) -> anyhow::Result<RectPx> {
        Ok(RectPx { x: 0, y: 0, w: 1920, h: 1080 })
    }
    fn work_area(&self) -> anyhow::Result<RectPx> {
        Ok(RectPx { x: 0, y: 0, w: 1920, h: 1032 })
    }
    fn icon_grid(&self) -> anyhow::Result<IconGrid> {
        Ok(IconGrid { cell_w: 100, cell_h: 100, scale: 1.0 })
    }
    fn list_icons(&self) -> anyhow::Result<Vec<DesktopIcon>> {
        Ok(self.state().icons.clone())
    }
    fn auto_arrange(&self) -> anyhow::Result<bool> {
        Ok(self.state().auto_arrange)
    }
    fn set_auto_arrange(&self, on: bool) -> anyhow::Result<()> {
        self.state().auto_arrange = on;
        Ok(())
    }
    fn move_icons(&self, moves: &[(String, PointPx)]) -> anyhow::Result<()> {
        let mut s = self.state();
        if s.auto_arrange {
            anyhow::bail!("icons can't be moved while Auto arrange is on");
        }
        for (id, at) in moves {
            if let Some(icon) = s.icons.iter_mut().find(|i| &i.id == id) {
                icon.position = *at;
            }
        }
        Ok(())
    }
    fn current_wallpaper(&self) -> anyhow::Result<Option<String>> {
        Ok(self.state().wallpaper.clone())
    }
    fn set_wallpaper(&self, png: &Path) -> anyhow::Result<()> {
        eprintln!("[pretend desktop] wallpaper set to {}", png.display());
        self.state().wallpaper = Some(png.display().to_string());
        Ok(())
    }
    fn battery_saver(&self) -> bool {
        false
    }
    fn session_locked(&self) -> bool {
        false
    }
}
