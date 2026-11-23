// SPDX-License-Identifier: GPL-3.0-or-later
//! Prints the render input for the sample week in `docs/mockups/`, as JSON.
//! The interface's tests use it as a fixture:
//! `cargo run -p backlith-core --example mockup_render_input > app/src/render/__tests__/mockup-week.json`

use backlith_core::*;
use chrono::NaiveDateTime;

fn at(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
}

fn main() {
    let rows: &[(&str, &[(&str, &str, &str, &str)])] = &[
        ("2026-11-23", &[("09:00", "09:30", "Standup", "work"), ("10:00", "12:00", "Lecture · Machine Learning", "work"), ("14:00", "15:00", "1:1 with supervisor", "work"), ("18:00", "19:00", "Gym", "personal")]),
        ("2026-11-24", &[("09:00", "09:30", "Standup", "work"), ("11:00", "12:30", "Design review", "work"), ("13:00", "14:00", "Lunch with the team", "personal"), ("16:00", "17:30", "Lab session", "work")]),
        ("2026-11-25", &[("09:00", "09:30", "Standup", "work"), ("10:00", "11:30", "Lecture · Databases", "work"), ("12:00", "13:00", "Lunch", "personal"), ("13:30", "15:00", "Project sync", "work"), ("15:30", "17:00", "Deep work", "work"), ("19:00", "21:00", "Hackathon prep", "personal")]),
        ("2026-11-26", &[("09:00", "09:30", "Standup", "work"), ("14:00", "15:00", "Dentist", "personal")]),
        ("2026-11-27", &[("09:00", "09:30", "Standup", "work"), ("10:00", "12:00", "Lecture · Machine Learning", "work"), ("13:00", "17:00", "Hackathon", "personal")]),
        ("2026-11-28", &[("11:00", "13:00", "Football", "personal"), ("20:00", "22:00", "Dinner with friends", "personal")]),
        ("2026-11-29", &[("16:00", "17:00", "Call home", "personal")]),
    ];
    let occurrences: Vec<Occurrence> = rows
        .iter()
        .flat_map(|(d, evs)| {
            evs.iter().map(move |(s, e, title, cal)| Occurrence {
                calendar_id: cal.to_string(),
                title: title.to_string(),
                location: None,
                start: at(&format!("{d}T{s}")),
                end: at(&format!("{d}T{e}")),
                all_day: false,
                counts_as_busy: true,
            })
        })
        .collect();
    let week = build_week(&occurrences, at("2026-11-25T13:40"), &WeekOptions::default());
    let work_area = RectPx { x: 0, y: 0, w: 1920, h: 1032 };
    let grid = IconGrid { cell_w: 100, cell_h: 100, scale: 1.0 };
    let icons: Vec<DesktopIcon> = ["Recycle Bin", "Projects", "Downloads", "Notes.txt", "Report.pdf", "Photos", "Terminal", "Music"]
        .iter()
        .enumerate()
        .map(|(i, n)| DesktopIcon { id: format!("C:\\Users\\me\\Desktop\\{n}"), name: n.to_string(), position: PointPx { x: 0, y: i as i32 * 100 } })
        .collect();
    let mut layout = first_run_layout(&icons, &work_area, &grid, None);
    if let Some(ItemKind::IconZone(z)) = layout.items.iter_mut().map(|i| &mut i.item).find(|k| matches!(k, ItemKind::IconZone(_))) {
        z.name = "Downloads".into();
    }
    let input = RenderInput {
        layout,
        week,
        calendars: vec![
            CalendarSource { id: "work".into(), name: "Work · Outlook".into(), color: "#F5B65C".into(), enabled: true, kind: SourceKind::Link },
            CalendarSource { id: "personal".into(), name: "Personal · Google".into(), color: "#5A8CF0".into(), enabled: true, kind: SourceKind::Link },
        ],
        screen: RectPx { x: 0, y: 0, w: 1920, h: 1080 },
        work_area,
        scale: 1.0,
    };
    println!("{}", serde_json::to_string_pretty(&input).unwrap());
}
