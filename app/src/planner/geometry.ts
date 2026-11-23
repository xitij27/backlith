// SPDX-License-Identifier: GPL-3.0-or-later
// Planner maths. snapRect and placeIcons follow backlith-core's
// snap_to_grid and plan_icons, so the preview matches what Apply does;
// the core stays the authority when icons are really moved.
import type { DesktopIcon } from "../generated/DesktopIcon";
import type { IconGrid } from "../generated/IconGrid";
import type { Layout } from "../generated/Layout";
import type { PointPx } from "../generated/PointPx";
import type { RectPx } from "../generated/RectPx";
import { toPx } from "../render/geometry";

export type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";
export const HANDLES: Handle[] = ["n", "s", "e", "w", "ne", "nw", "se", "sw"];

/** Height of a zone's label strip, in physical px: ZONE_LABEL_PX × scale. */
export function labelPx(grid: IconGrid): number {
  return Math.round(28 * grid.scale);
}

const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), Math.max(lo, hi));
const roundTo = (v: number, step: number) => Math.round(v / step) * step;

export function moveRect(r: RectPx, dx: number, dy: number, bounds: RectPx): RectPx {
  return {
    ...r,
    x: Math.round(clamp(r.x + dx, bounds.x, bounds.x + bounds.w - r.w)),
    y: Math.round(clamp(r.y + dy, bounds.y, bounds.y + bounds.h - r.h)),
  };
}

export function resizeRect(r: RectPx, h: Handle, dx: number, dy: number, min: { w: number; h: number }, bounds: RectPx): RectPx {
  let left = r.x;
  let top = r.y;
  let right = r.x + r.w;
  let bottom = r.y + r.h;
  if (h.includes("w")) left = clamp(left + dx, bounds.x, right - min.w);
  if (h.includes("e")) right = clamp(right + dx, left + min.w, bounds.x + bounds.w);
  if (h.includes("n")) top = clamp(top + dy, bounds.y, bottom - min.h);
  if (h.includes("s")) bottom = clamp(bottom + dy, top + min.h, bounds.y + bounds.h);
  return { x: Math.round(left), y: Math.round(top), w: Math.round(right - left), h: Math.round(bottom - top) };
}

/** Same rule as backlith-core's snap_to_grid. */
export function snapRect(r: RectPx, workArea: RectPx, grid: IconGrid): RectPx {
  const cw = Math.max(1, grid.cellW);
  const ch = Math.max(1, grid.cellH);
  const w = Math.min(Math.max(roundTo(r.w, cw), cw), workArea.w);
  const h = Math.min(Math.max(roundTo(r.h, ch), ch), workArea.h);
  const x = clamp(workArea.x + roundTo(r.x - workArea.x, cw), workArea.x, workArea.x + workArea.w - w);
  const y = clamp(workArea.y + roundTo(r.y - workArea.y, ch), workArea.y, workArea.y + workArea.h - h);
  return { x, y, w, h };
}

/** Touching edges don't collide. */
export function collides(r: RectPx, others: RectPx[]): boolean {
  return others.some((o) => r.x < o.x + o.w && o.x < r.x + r.w && r.y < o.y + o.h && o.y < r.y + r.h);
}

export function zoneCapacity(zone: RectPx, grid: IconGrid): number {
  const cols = Math.max(0, Math.floor(zone.w / Math.max(1, grid.cellW)));
  const rows = Math.max(0, Math.floor(Math.max(0, zone.h - labelPx(grid)) / Math.max(1, grid.cellH)));
  return cols * rows;
}

/** A zone grown to hold `iconCount` icons: downwards first, then to the right. Null if both are blocked. */
export function growToFit(zone: RectPx, iconCount: number, grid: IconGrid, bounds: RectPx, others: RectPx[]): RectPx | null {
  const fits = (r: RectPx) => r.y + r.h <= bounds.y + bounds.h && r.x + r.w <= bounds.x + bounds.w && !collides(r, others);
  const cols = Math.max(1, Math.floor(zone.w / grid.cellW));
  const down = { ...zone, h: labelPx(grid) + Math.ceil(iconCount / cols) * grid.cellH };
  if (fits(down)) return down;
  const rows = Math.max(1, Math.floor((zone.h - labelPx(grid)) / grid.cellH));
  const right = { ...zone, w: Math.ceil(iconCount / rows) * grid.cellW };
  if (fits(right)) return right;
  return null;
}

export type Placement = {
  /** Where each placed icon sits. Icons that overflow, or that have no zone, are absent. */
  positions: Map<string, PointPx>;
  /** Each zone's icons, in order, including those that overflow. */
  lists: Map<string, string[]>;
  overflow: Map<string, string[]>;
};

/** Same rule as backlith-core's plan_icons. */
export function placeIcons(layout: Layout, icons: DesktopIcon[], workArea: RectPx, grid: IconGrid): Placement {
  const result: Placement = { positions: new Map(), lists: new Map(), overflow: new Map() };
  const zones = layout.items.filter((i) => i.item.kind === "iconZone");
  if (zones.length === 0) return result;
  const present = new Set(icons.map((i) => i.id));
  const placed = new Set<string>();
  for (const z of zones) {
    if (z.item.kind !== "iconZone") continue;
    const list = z.item.iconOrder.filter((id) => present.has(id) && !placed.has(id));
    list.forEach((id) => placed.add(id));
    result.lists.set(z.id, list);
  }
  const inbox = zones.find((z) => z.item.kind === "iconZone" && z.item.takesNewIcons);
  if (inbox) {
    const reading = [...icons].sort((a, b) => a.position.y - b.position.y || a.position.x - b.position.x || (a.id < b.id ? -1 : 1));
    for (const icon of reading) {
      if (!placed.has(icon.id)) {
        placed.add(icon.id);
        result.lists.get(inbox.id)!.push(icon.id);
      }
    }
  }
  for (const z of zones) {
    const rect = toPx(z.rect, workArea);
    const cols = Math.max(1, Math.floor(rect.w / grid.cellW));
    const capacity = zoneCapacity(rect, grid);
    const list = result.lists.get(z.id)!;
    list.slice(0, capacity).forEach((id, i) => {
      result.positions.set(id, { x: rect.x + (i % cols) * grid.cellW, y: rect.y + labelPx(grid) + Math.floor(i / cols) * grid.cellH });
    });
    result.overflow.set(z.id, list.slice(capacity));
  }
  return result;
}

/** The first free spot, scanning the grid row by row, for a zone 3 cells wide and 3 rows tall plus its label. */
export function findFreeSpot(layout: Layout, workArea: RectPx, grid: IconGrid): RectPx | null {
  const w = 3 * grid.cellW;
  const h = labelPx(grid) + 3 * grid.cellH;
  const taken = layout.items.map((i) => toPx(i.rect, workArea));
  for (let y = workArea.y; y + h <= workArea.y + workArea.h; y += grid.cellH) {
    for (let x = workArea.x; x + w <= workArea.x + workArea.w; x += grid.cellW) {
      const spot = { x, y, w, h };
      if (!collides(spot, taken)) return spot;
    }
  }
  return null;
}
