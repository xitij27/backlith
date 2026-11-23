# Backlith

The monolith behind your windows.

A desktop background that does useful work. It holds tools, laid out by the app as one background, not as a set of widgets, so nothing has to be placed by hand. The first tool shows the coming week from all of your calendars, and how busy each day is going to be.

Windows comes first. Ubuntu may follow.

## Status

- **Plan 1, built:** everything that can be built and tested on Ubuntu. That covers the calendar engine, the schedule format, the layout and icon-zone maths, fetching calendar links, the three calendar styles and the planner app running against a pretend desktop. It lives on the branch `core-and-planner`.
- **Plan 2, not started:** the Windows parts, which wait for a Windows machine. These are moving real icons, drawing the wallpaper as a picture and setting it, starting at sign-in, and the installer.

The design is `docs/superpowers/specs/2026-11-22-backlith-v1-design.md`, and the plans are in `docs/superpowers/plans/`. How it got here, and why, is in `docs/decisions.md`.

## How it works

Backlith is one Tauri app.
- **The tray part:** a small Rust program that starts at sign-in. It fetches calendars and keeps desktop icons inside their zones.
- **The wallpaper:** every 5 minutes, or when something changes, Backlith draws the layout as a picture and sets it as the normal Windows wallpaper. No web engine stays running in between.
- **The planner:** a window where you place the calendar and the icon zones, pick the calendar's style and add calendars.

## The code

| Folder | What it is |
|---|---|
| `crates/backlith-core` | Pure logic with no I/O: the week and busy sums, reading ICS, the schedule format, layouts, icon zones, settings and redraw timing. |
| `crates/backlith-sources` | Fetching calendar links with a cache, and the secret store that keeps links out of every file. |
| `app/src-tauri` | The Tauri app: `AppCore` (all command logic), the tray, the planner window, the timers, and the `Desktop` trait with a pretend desktop for Linux. |
| `app/src/render` | The wallpaper in React: base, zone panels, and the three calendar styles. |
| `app/src/planner` | The planner window. |
| `app/src/generated` | TypeScript types generated from the Rust types by `ts-rs`. Don't edit by hand. |
| `schemas/` | The schedule format's JSON Schema and an example. |
| `docs/mockups/` | The calendar mockups the styles follow. |

## Developing on Ubuntu

Set-up, once:

```bash
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
cd app && pnpm install
```

Checks:

```bash
cargo test --workspace        # Rust; also regenerates app/src/generated
cd app && pnpm test           # interface tests
cd app && pnpm build          # type-check and build the interface
```

`cd app && pnpm tauri dev` runs the app against the pretend desktop. That's only for development: the real behaviour can only be checked on Windows.

The interface tests use `app/src/render/__tests__/mockup-week.json`, which is made by `cargo run -p backlith-core --example mockup_render_input > app/src/render/__tests__/mockup-week.json`.

## Secrets

Calendar links (secret ICS addresses) give full read access to a calendar. Backlith keeps them only in the system's secret store: Windows Credential Manager, or the Secret Service on Linux. They are never in its files, logs or the wallpaper.

## License

Copyright (C) 2026 Kshitij Parashar

Backlith is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. It is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See [LICENSE](LICENSE) for the full text.
