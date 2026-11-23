// SPDX-License-Identifier: GPL-3.0-or-later
//! The Backlith app: the tray, the planner window and the timers, around
//! [`core::AppCore`].

pub mod commands;
pub mod core;
pub mod desktop;
pub mod timers;

use std::sync::Arc;

use backlith_sources::{KeyringStore, SecretStore};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::core::AppCore;
use crate::desktop::fake::FakeDesktop;
use crate::desktop::Desktop;

/// Opens the planner, or brings it forward. Closing it destroys the window,
/// so its web engine is not kept running.
pub fn open_planner(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("planner") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "planner", WebviewUrl::App("index.html".into()))
        .title("Backlith planner")
        .inner_size(1280.0, 820.0)
        .min_inner_size(900.0, 600.0)
        .build()?;
    Ok(())
}

fn build_tray(app: &AppHandle, core: Arc<AppCore>, paused: bool) -> tauri::Result<()> {
    let item = |id: &str, label: &str| MenuItem::with_id(app, id, label, true, None::<&str>);
    let pause = CheckMenuItem::with_id(app, "pause", "Pause", true, paused, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &item("open", "Open planner")?,
            &item("redraw", "Redraw now")?,
            &pause,
            &PredefinedMenuItem::separator(app)?,
            &item("icons", "Put my icons back")?,
            &item("wallpaper", "Restore my old wallpaper")?,
            &PredefinedMenuItem::separator(app)?,
            &item("quit", "Quit")?,
        ],
    )?;
    let icon = app.default_window_icon().cloned().ok_or_else(|| tauri::Error::AssetNotFound("app icon".into()))?;
    TrayIconBuilder::with_id("backlith")
        .icon(icon)
        .tooltip("Backlith")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
            let report = |r: Result<(), String>| {
                if let Err(e) = r {
                    eprintln!("Backlith: {e}");
                }
            };
            match event.id().as_ref() {
                "open" => report(open_planner(app).map_err(|e| e.to_string())),
                "redraw" => core.mark_dirty(),
                "pause" => {
                    let paused = core.toggle_pause();
                    let _ = pause.set_checked(paused);
                }
                "icons" => report(core.put_icons_back()),
                "wallpaper" => report(core.restore_wallpaper()),
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}

pub fn run() {
    let data_dir = dirs::data_dir().unwrap_or_else(std::env::temp_dir).join("Backlith");
    // Plan 2 puts the real Windows desktop here.
    let desktop: Arc<dyn Desktop> = Arc::new(FakeDesktop::new(None));
    let secrets: Arc<dyn SecretStore> = Arc::new(KeyringStore);
    let core = match AppCore::new(desktop, secrets, data_dir) {
        Ok(core) => Arc::new(core),
        Err(e) => {
            eprintln!("Backlith could not start: {e}");
            std::process::exit(1);
        }
    };
    let for_setup = core.clone();
    tauri::Builder::default()
        .manage(core)
        .invoke_handler(tauri::generate_handler![
            commands::planner_state,
            commands::save_layout,
            commands::apply_layout,
            commands::put_icons_back,
            commands::restore_wallpaper,
            commands::add_link,
            commands::upload,
            commands::update_source,
            commands::remove_source,
            commands::llm_prompt,
            commands::render_input,
            commands::preview_week,
        ])
        .setup(move |app| {
            let paused = for_setup.planner_state(chrono::Local::now().naive_local()).map(|s| s.settings.paused).unwrap_or(false);
            build_tray(app.handle(), for_setup.clone(), paused)?;
            if for_setup.is_first_run() {
                open_planner(app.handle())?;
            }
            timers::spawn(app.handle().clone(), for_setup.clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Backlith could not start")
        .run(|_app, event| {
            // Closing the planner must not quit Backlith: it lives in the tray.
            if let tauri::RunEvent::ExitRequested { code: None, api, .. } = event {
                api.prevent_exit();
            }
        });
}
