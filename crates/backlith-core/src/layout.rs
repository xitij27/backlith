// SPDX-License-Identifier: GPL-3.0-or-later
//! The layout of the background: what sits behind the tools, and where each
//! tool and icon zone is. Rectangles are stored as fractions of the work
//! area (the screen minus the taskbar), so a layout survives a change of
//! resolution or scaling.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::week::WeekOptions;

/// A rectangle in fractions (0..1) of the work area.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RectF {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// A rectangle in physical screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RectPx {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PointPx {
    pub x: i32,
    pub y: i32,
}

/// Windows' desktop icon grid, in physical pixels, and the display scaling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IconGrid {
    pub cell_w: i32,
    pub cell_h: i32,
    pub scale: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CalendarStyle {
    TimeGrid,
    DayStacks,
    Skyline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarToolSettings {
    pub style: CalendarStyle,
    pub week: WeekOptions,
    /// The calendars shown. Empty means every enabled calendar.
    pub calendar_ids: Vec<String>,
    pub show_titles: bool,
}

impl Default for CalendarToolSettings {
    fn default() -> Self {
        Self { style: CalendarStyle::TimeGrid, week: WeekOptions::default(), calendar_ids: Vec::new(), show_titles: true }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ZoneSettings {
    pub name: String,
    /// New desktop icons are moved into this zone. Exactly one zone has it.
    pub takes_new_icons: bool,
    /// Icon ids in the order they fill the zone, row by row.
    pub icon_order: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ItemKind {
    Calendar(CalendarToolSettings),
    IconZone(ZoneSettings),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LayoutItem {
    pub id: String,
    pub rect: RectF,
    pub item: ItemKind,
}

/// What sits behind the tools.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum Base {
    Color { hex: String },
    Image { path: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Layout {
    pub version: u32,
    pub base: Base,
    pub items: Vec<LayoutItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LayoutError {
    #[error("A layout can hold only one calendar.")]
    TwoCalendars,
    #[error("“{0}” and “{1}” overlap.")]
    Overlap(String, String),
    #[error("“{0}” goes past the edge of the screen.")]
    OutOfBounds(String),
    #[error("Exactly one icon zone must take new icons.")]
    NewIconsZone,
}

/// Slack for comparing fractions, far below a pixel on any screen.
const EPS: f64 = 1e-9;

pub fn to_px(r: &RectF, work_area: &RectPx) -> RectPx {
    let (w, h) = (f64::from(work_area.w), f64::from(work_area.h));
    RectPx {
        x: work_area.x + (r.x * w).round() as i32,
        y: work_area.y + (r.y * h).round() as i32,
        w: (r.w * w).round() as i32,
        h: (r.h * h).round() as i32,
    }
}

pub fn to_fraction(r: &RectPx, work_area: &RectPx) -> RectF {
    let (w, h) = (f64::from(work_area.w.max(1)), f64::from(work_area.h.max(1)));
    RectF {
        x: f64::from(r.x - work_area.x) / w,
        y: f64::from(r.y - work_area.y) / h,
        w: f64::from(r.w) / w,
        h: f64::from(r.h) / h,
    }
}

fn round_to(value: i32, step: i32) -> i32 {
    ((f64::from(value) / f64::from(step)).round() as i32) * step
}

/// Rounds a rectangle onto the icon grid, at least one cell each way, inside the work area.
pub fn snap_to_grid(r: &RectPx, work_area: &RectPx, grid: &IconGrid) -> RectPx {
    let (cw, ch) = (grid.cell_w.max(1), grid.cell_h.max(1));
    let w = round_to(r.w, cw).max(cw).min(work_area.w);
    let h = round_to(r.h, ch).max(ch).min(work_area.h);
    let x = (work_area.x + round_to(r.x - work_area.x, cw)).clamp(work_area.x, work_area.x + work_area.w - w);
    let y = (work_area.y + round_to(r.y - work_area.y, ch)).clamp(work_area.y, work_area.y + work_area.h - h);
    RectPx { x, y, w, h }
}

fn overlaps(a: &RectF, b: &RectF) -> bool {
    a.x < b.x + b.w - EPS && b.x < a.x + a.w - EPS && a.y < b.y + b.h - EPS && b.y < a.y + a.h - EPS
}

/// Pairs of item ids whose rectangles overlap. Touching edges don't count.
pub fn find_overlaps(layout: &Layout) -> Vec<(String, String)> {
    let items = &layout.items;
    let mut pairs = Vec::new();
    for (i, a) in items.iter().enumerate() {
        for b in &items[i + 1..] {
            if overlaps(&a.rect, &b.rect) {
                pairs.push((a.id.clone(), b.id.clone()));
            }
        }
    }
    pairs
}

pub fn validate_layout(layout: &Layout) -> Result<(), LayoutError> {
    if layout.items.iter().filter(|i| matches!(i.item, ItemKind::Calendar(_))).count() > 1 {
        return Err(LayoutError::TwoCalendars);
    }
    if let Some(bad) = layout.items.iter().find(|i| {
        let r = &i.rect;
        r.w <= 0.0 || r.h <= 0.0 || r.x < -EPS || r.y < -EPS || r.x + r.w > 1.0 + EPS || r.y + r.h > 1.0 + EPS
    }) {
        return Err(LayoutError::OutOfBounds(bad.id.clone()));
    }
    if let Some((a, b)) = find_overlaps(layout).into_iter().next() {
        return Err(LayoutError::Overlap(a, b));
    }
    let zones: Vec<&ZoneSettings> = layout
        .items
        .iter()
        .filter_map(|i| match &i.item {
            ItemKind::IconZone(z) => Some(z),
            ItemKind::Calendar(_) => None,
        })
        .collect();
    if !zones.is_empty() && zones.iter().filter(|z| z.takes_new_icons).count() != 1 {
        return Err(LayoutError::NewIconsZone);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) const WA: RectPx = RectPx { x: 0, y: 0, w: 1920, h: 1032 };
    pub(crate) const GRID: IconGrid = IconGrid { cell_w: 100, cell_h: 100, scale: 1.0 };

    pub(crate) fn zone(id: &str, rect: RectF, takes_new: bool, order: &[&str]) -> LayoutItem {
        LayoutItem {
            id: id.into(),
            rect,
            item: ItemKind::IconZone(ZoneSettings {
                name: id.into(),
                takes_new_icons: takes_new,
                icon_order: order.iter().map(|s| s.to_string()).collect(),
            }),
        }
    }
    pub(crate) fn calendar(id: &str, rect: RectF) -> LayoutItem {
        LayoutItem { id: id.into(), rect, item: ItemKind::Calendar(CalendarToolSettings::default()) }
    }
    pub(crate) fn layout(items: Vec<LayoutItem>) -> Layout {
        Layout { version: 1, base: Base::Color { hex: "#0D1117".into() }, items }
    }
    fn f(x: f64, y: f64, w: f64, h: f64) -> RectF {
        RectF { x, y, w, h }
    }

    #[test]
    fn fractions_round_trip_within_a_pixel() {
        let wa = RectPx { x: 0, y: 40, w: 2560, h: 1400 };
        for r in [RectPx { x: 0, y: 40, w: 300, h: 1400 }, RectPx { x: 333, y: 777, w: 1001, h: 99 }] {
            let back = to_px(&to_fraction(&r, &wa), &wa);
            assert!((back.x - r.x).abs() <= 1 && (back.y - r.y).abs() <= 1 && (back.w - r.w).abs() <= 1 && (back.h - r.h).abs() <= 1, "{r:?} → {back:?}");
        }
    }

    #[test]
    fn snapping_aligns_to_cells() {
        assert_eq!(snap_to_grid(&RectPx { x: 130, y: 40, w: 290, h: 310 }, &WA, &GRID), RectPx { x: 100, y: 0, w: 300, h: 300 });
        // At least one cell, and never past the work area.
        assert_eq!(snap_to_grid(&RectPx { x: 1880, y: 1000, w: 20, h: 20 }, &WA, &GRID), RectPx { x: 1820, y: 932, w: 100, h: 100 });
    }

    #[test]
    fn touching_edges_do_not_overlap() {
        let a = calendar("cal", f(0.0, 0.0, 0.5, 1.0));
        let b = zone("zone", f(0.5, 0.0, 0.5, 1.0), true, &[]);
        assert!(find_overlaps(&layout(vec![a.clone(), b])).is_empty());
        let c = zone("zone", f(0.5 - 1.0 / 1920.0, 0.0, 0.5, 1.0), true, &[]);
        assert_eq!(find_overlaps(&layout(vec![a, c])), vec![("cal".to_string(), "zone".to_string())]);
    }

    #[test]
    fn layout_rules() {
        let two = layout(vec![calendar("a", f(0.0, 0.0, 0.4, 1.0)), calendar("b", f(0.5, 0.0, 0.4, 1.0))]);
        assert_eq!(validate_layout(&two), Err(LayoutError::TwoCalendars));
        let none_new = layout(vec![zone("z", f(0.0, 0.0, 0.2, 1.0), false, &[])]);
        assert_eq!(validate_layout(&none_new), Err(LayoutError::NewIconsZone));
        let two_new = layout(vec![zone("z1", f(0.0, 0.0, 0.2, 0.5), true, &[]), zone("z2", f(0.0, 0.5, 0.2, 0.5), true, &[])]);
        assert_eq!(validate_layout(&two_new), Err(LayoutError::NewIconsZone));
        let past = layout(vec![calendar("cal", f(0.7, 0.0, 0.4, 1.0))]);
        assert_eq!(validate_layout(&past), Err(LayoutError::OutOfBounds("cal".into())));
        let overlap = layout(vec![calendar("cal", f(0.1, 0.0, 0.5, 1.0)), zone("z", f(0.0, 0.0, 0.2, 1.0), true, &[])]);
        assert_eq!(validate_layout(&overlap), Err(LayoutError::Overlap("cal".into(), "z".into())));
        let fine = layout(vec![zone("z", f(0.0, 0.0, 0.2, 1.0), true, &[]), calendar("cal", f(0.25, 0.0, 0.75, 1.0))]);
        assert_eq!(validate_layout(&fine), Ok(()));
        assert_eq!(validate_layout(&layout(vec![])), Ok(()));
    }

    #[test]
    fn items_serialise_with_a_kind_tag() {
        let json = serde_json::to_value(zone("z", f(0.0, 0.0, 0.2, 1.0), true, &["a"])).unwrap();
        assert_eq!(json["item"]["kind"], "iconZone");
        assert_eq!(json["item"]["takesNewIcons"], true);
    }
}
