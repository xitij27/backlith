// SPDX-License-Identifier: GPL-3.0-or-later
//! Desktop icons and icon zones: how many icons a zone holds, where each
//! icon goes, and the layout proposed on first run.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::{
    to_fraction, to_px, Base, CalendarToolSettings, IconGrid, ItemKind, Layout, LayoutItem, PointPx, RectPx, ZoneSettings,
};

/// Height of a zone's label strip at 100% scaling, in pixels.
pub const ZONE_LABEL_PX: i32 = 28;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DesktopIcon {
    /// The item's full parsing name: unique, unlike its display name.
    pub id: String,
    pub name: String,
    pub position: PointPx,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ZoneOverflow {
    pub zone_id: String,
    pub icon_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IconPlan {
    /// Icons to move, and where to. Icons already in place are left out.
    pub moves: Vec<(String, PointPx)>,
    /// Icons that don't fit in their zone. They are not moved.
    pub overflow: Vec<ZoneOverflow>,
}

fn label_px(grid: &IconGrid) -> i32 {
    (f64::from(ZONE_LABEL_PX) * grid.scale).round() as i32
}

fn columns(zone: &RectPx, grid: &IconGrid) -> i32 {
    (zone.w / grid.cell_w.max(1)).max(0)
}

pub fn zone_capacity(zone: &RectPx, grid: &IconGrid) -> usize {
    let rows = ((zone.h - label_px(grid)).max(0) / grid.cell_h.max(1)).max(0);
    (columns(zone, grid) * rows) as usize
}

/// Icons in reading order: top to bottom, then left to right.
fn reading_order(icons: &[DesktopIcon]) -> Vec<&DesktopIcon> {
    let mut v: Vec<&DesktopIcon> = icons.iter().collect();
    v.sort_by(|a, b| (a.position.y, a.position.x, &a.id).cmp(&(b.position.y, b.position.x, &b.id)));
    v
}

pub fn plan_icons(layout: &Layout, icons: &[DesktopIcon], work_area: &RectPx, grid: &IconGrid) -> IconPlan {
    let zones: Vec<(&str, RectPx, &ZoneSettings)> = layout
        .items
        .iter()
        .filter_map(|i| match &i.item {
            ItemKind::IconZone(z) => Some((i.id.as_str(), to_px(&i.rect, work_area), z)),
            ItemKind::Calendar(_) => None,
        })
        .collect();
    if zones.is_empty() {
        return IconPlan::default();
    }

    let present: HashMap<&str, &DesktopIcon> = icons.iter().map(|i| (i.id.as_str(), i)).collect();
    let mut placed: HashSet<&str> = HashSet::new();
    // Each zone's icons: its listed ones first, skipping any listed earlier.
    let mut lists: Vec<Vec<&str>> = zones
        .iter()
        .map(|(_, _, z)| {
            z.icon_order
                .iter()
                .map(String::as_str)
                .filter(|id| present.contains_key(id) && placed.insert(id))
                .collect()
        })
        .collect();
    if let Some(inbox) = zones.iter().position(|(_, _, z)| z.takes_new_icons) {
        for icon in reading_order(icons) {
            if placed.insert(icon.id.as_str()) {
                lists[inbox].push(icon.id.as_str());
            }
        }
    }

    let label = label_px(grid);
    let mut plan = IconPlan::default();
    for ((zone_id, rect, _), list) in zones.iter().zip(lists) {
        let cols = columns(rect, grid).max(1) as usize;
        let capacity = zone_capacity(rect, grid);
        for (i, id) in list.iter().enumerate() {
            if i >= capacity {
                continue;
            }
            let at = PointPx {
                x: rect.x + (i % cols) as i32 * grid.cell_w,
                y: rect.y + label + (i / cols) as i32 * grid.cell_h,
            };
            if present[id].position != at {
                plan.moves.push((id.to_string(), at));
            }
        }
        if list.len() > capacity {
            plan.overflow.push(ZoneOverflow {
                zone_id: zone_id.to_string(),
                icon_ids: list[capacity..].iter().map(|s| s.to_string()).collect(),
            });
        }
    }
    plan
}

/// One zone on the left sized for the icons there now, and the calendar
/// filling the rest, one cell to its right.
pub fn first_run_layout(icons: &[DesktopIcon], work_area: &RectPx, grid: &IconGrid, current_wallpaper: Option<String>) -> Layout {
    let (cw, ch) = (grid.cell_w.max(1), grid.cell_h.max(1));
    let rows = ((work_area.h - label_px(grid)) / ch).max(1) as usize;
    let needed = icons.len().div_ceil(rows).max(1) as i32;
    let widest = ((f64::from(work_area.w) * 0.4) as i32 / cw).max(1);
    let cols = needed.min(widest);

    let zone = RectPx { x: work_area.x, y: work_area.y, w: cols * cw, h: work_area.h };
    let cal_x = zone.x + (cols + 1) * cw;
    let calendar = RectPx { x: cal_x, y: work_area.y, w: work_area.x + work_area.w - cal_x, h: work_area.h };

    Layout {
        version: 1,
        base: match current_wallpaper {
            Some(path) => Base::Image { path },
            None => Base::Color { hex: "#0D1117".into() },
        },
        items: vec![
            LayoutItem {
                id: "zone-1".into(),
                rect: to_fraction(&zone, work_area),
                item: ItemKind::IconZone(ZoneSettings {
                    name: "Shortcuts".into(),
                    takes_new_icons: true,
                    icon_order: reading_order(icons).iter().map(|i| i.id.clone()).collect(),
                }),
            },
            LayoutItem {
                id: "calendar".into(),
                rect: to_fraction(&calendar, work_area),
                item: ItemKind::Calendar(CalendarToolSettings::default()),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::tests::{calendar, layout, zone, GRID, WA};
    use crate::layout::{validate_layout, RectF};

    fn icon(id: &str, name: &str, x: i32, y: i32) -> DesktopIcon {
        DesktopIcon { id: id.into(), name: name.into(), position: PointPx { x, y } }
    }
    /// A zone at the given pixel rectangle.
    fn zone_px(id: &str, r: RectPx, takes_new: bool, order: &[&str]) -> crate::layout::LayoutItem {
        zone(id, to_fraction(&r, &WA), takes_new, order)
    }
    fn target(plan: &IconPlan, id: &str) -> Option<PointPx> {
        plan.moves.iter().find(|(i, _)| i == id).map(|(_, p)| *p)
    }

    #[test]
    fn capacity_leaves_room_for_the_label() {
        assert_eq!(zone_capacity(&RectPx { x: 0, y: 0, w: 300, h: 328 }, &GRID), 9);
        assert_eq!(zone_capacity(&RectPx { x: 0, y: 0, w: 300, h: 327 }, &GRID), 6);
        let hidpi = IconGrid { cell_w: 150, cell_h: 150, scale: 1.5 };
        assert_eq!(zone_capacity(&RectPx { x: 0, y: 0, w: 300, h: 342 }, &hidpi), 4);
    }

    #[test]
    fn icons_fill_row_by_row_in_order() {
        let l = layout(vec![zone_px("z", RectPx { x: 0, y: 0, w: 300, h: 328 }, true, &["b", "a"])]);
        let plan = plan_icons(&l, &[icon("a", "A", 900, 900), icon("b", "B", 800, 800)], &WA, &GRID);
        assert_eq!(target(&plan, "b"), Some(PointPx { x: 0, y: 28 }));
        assert_eq!(target(&plan, "a"), Some(PointPx { x: 100, y: 28 }));
        assert!(plan.overflow.is_empty());
    }

    #[test]
    fn unlisted_icons_go_to_the_new_icons_zone() {
        let l = layout(vec![
            zone_px("left", RectPx { x: 0, y: 0, w: 300, h: 328 }, false, &["a"]),
            zone_px("inbox", RectPx { x: 1500, y: 0, w: 300, h: 328 }, true, &["b"]),
        ]);
        // c is above d on screen, so c comes first.
        let icons = [icon("a", "A", 0, 0), icon("b", "B", 0, 100), icon("d", "D", 500, 600), icon("c", "C", 900, 300)];
        let plan = plan_icons(&l, &icons, &WA, &GRID);
        assert_eq!(target(&plan, "a"), Some(PointPx { x: 0, y: 28 }));
        assert_eq!(target(&plan, "b"), Some(PointPx { x: 1500, y: 28 }));
        assert_eq!(target(&plan, "c"), Some(PointPx { x: 1600, y: 28 }));
        assert_eq!(target(&plan, "d"), Some(PointPx { x: 1700, y: 28 }));
    }

    #[test]
    fn overflow_is_reported_not_moved() {
        let ids: Vec<String> = (0..10).map(|i| format!("i{i}")).collect();
        let order: Vec<&str> = ids.iter().map(String::as_str).collect();
        let l = layout(vec![zone_px("z", RectPx { x: 0, y: 0, w: 300, h: 328 }, true, &order)]);
        let icons: Vec<DesktopIcon> = ids.iter().map(|id| icon(id, id, 1000, 1000)).collect();
        let plan = plan_icons(&l, &icons, &WA, &GRID);
        assert_eq!(plan.moves.len(), 9);
        assert_eq!(plan.overflow, vec![ZoneOverflow { zone_id: "z".into(), icon_ids: vec!["i9".into()] }]);
        assert_eq!(target(&plan, "i9"), None);
    }

    #[test]
    fn icons_already_in_place_are_not_moved() {
        let l = layout(vec![zone_px("z", RectPx { x: 0, y: 0, w: 300, h: 328 }, true, &["a", "b"])]);
        let plan = plan_icons(&l, &[icon("a", "A", 0, 28), icon("b", "B", 0, 500)], &WA, &GRID);
        assert_eq!(plan.moves, vec![("b".to_string(), PointPx { x: 100, y: 28 })]);
    }

    #[test]
    fn same_name_different_id_both_placed() {
        let l = layout(vec![zone_px("z", RectPx { x: 0, y: 0, w: 300, h: 328 }, true, &[])]);
        let icons = [
            icon("C:\\Users\\k\\Desktop\\New folder", "New folder", 900, 100),
            icon("C:\\Users\\Public\\Desktop\\New folder", "New folder", 900, 200),
        ];
        let plan = plan_icons(&l, &icons, &WA, &GRID);
        assert_eq!(plan.moves.len(), 2);
        assert_ne!(plan.moves[0].1, plan.moves[1].1);
    }

    #[test]
    fn no_zones_means_no_moves() {
        let l = layout(vec![calendar("cal", RectF { x: 0.0, y: 0.0, w: 1.0, h: 1.0 })]);
        assert_eq!(plan_icons(&l, &[icon("a", "A", 5, 5)], &WA, &GRID), IconPlan::default());
    }

    #[test]
    fn first_run_fits_the_icons_on_the_left() {
        let icons: Vec<DesktopIcon> = (0..12).map(|i| icon(&format!("i{i}"), "x", 0, i * 100)).collect();
        let l = first_run_layout(&icons, &WA, &GRID, Some("C:\\wall.jpg".into()));
        assert_eq!(l.base, Base::Image { path: "C:\\wall.jpg".into() });
        assert_eq!(validate_layout(&l), Ok(()));
        let zone = l.items.iter().find(|i| matches!(i.item, ItemKind::IconZone(_))).unwrap();
        assert_eq!(to_px(&zone.rect, &WA), RectPx { x: 0, y: 0, w: 200, h: 1032 });
        let cal = l.items.iter().find(|i| matches!(i.item, ItemKind::Calendar(_))).unwrap();
        assert_eq!(to_px(&cal.rect, &WA), RectPx { x: 300, y: 0, w: 1620, h: 1032 });
        assert!(plan_icons(&l, &icons, &WA, &GRID).overflow.is_empty());
        let no_wallpaper = first_run_layout(&icons, &WA, &GRID, None);
        assert_eq!(no_wallpaper.base, Base::Color { hex: "#0D1117".into() });
    }
}
