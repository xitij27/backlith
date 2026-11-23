// SPDX-License-Identifier: GPL-3.0-or-later
// The screen at small scale: the live wallpaper preview, a frame over each
// tool and zone, and the icons where they will sit. Changes go to
// `onChange`; nothing is applied until the planner's Apply.
import { useState } from "react";
import type { Layout } from "../generated/Layout";
import type { PlannerState } from "../generated/PlannerState";
import type { PointPx } from "../generated/PointPx";
import type { RectPx } from "../generated/RectPx";
import type { RenderInput } from "../generated/RenderInput";
import { toFraction, toPx } from "../render/geometry";
import { Wallpaper } from "../render/Wallpaper";
import { collides, growToFit, labelPx, moveRect, placeIcons, resizeRect, snapRect, type Handle } from "./geometry";
import { IconLayer } from "./IconLayer";
import { ItemFrame } from "./ItemFrame";

export function Canvas(props: { state: PlannerState; layout: Layout; onChange: (layout: Layout) => void; viewScale: number; selectedId?: string | null; onSelect?: (id: string) => void; locale?: string }) {
  const { state, layout, onChange, viewScale: k, locale } = props;
  const { workArea: wa, grid, screen } = state;
  const [live, setLive] = useState<{ id: string; rect: RectPx } | null>(null);
  const [noRoom, setNoRoom] = useState<string | null>(null);

  const items = layout.items.map((i) => ({ item: i, rect: toPx(i.rect, wa) }));
  const placement = placeIcons(layout, state.icons, wa, grid);
  const input: RenderInput = { layout, week: state.week, calendars: state.settings.calendars, screen, workArea: wa, scale: grid.scale };
  const origin = { x: screen.x, y: screen.y };
  const minSize = { w: grid.cellW, h: grid.cellH };

  const others = (id: string) => items.filter((o) => o.item.id !== id).map((o) => o.rect);
  const withRect = (id: string, r: RectPx): Layout => ({ ...layout, items: layout.items.map((i) => (i.id === id ? { ...i, rect: toFraction(r, wa) } : i)) });
  /** Commits a new rectangle, or snaps back if it would land on another item. */
  const commit = (id: string, r: RectPx, from: RectPx) => {
    setLive(null);
    if (r.x === from.x && r.y === from.y && r.w === from.w && r.h === from.h) return;
    if (collides(r, others(id))) return;
    onChange(withRect(id, r));
  };

  const dropIcon = (iconId: string, center: PointPx) => {
    const target = items.find(({ item, rect }) => item.item.kind === "iconZone" && center.x >= rect.x && center.x < rect.x + rect.w && center.y >= rect.y && center.y < rect.y + rect.h);
    if (!target) return;
    const cols = Math.max(1, Math.floor(target.rect.w / grid.cellW));
    const col = Math.min(cols - 1, Math.max(0, Math.floor((center.x - target.rect.x) / grid.cellW)));
    const row = Math.max(0, Math.floor((center.y - target.rect.y - labelPx(grid)) / grid.cellH));
    const lists = new Map([...placement.lists].map(([z, ids]) => [z, ids.filter((x) => x !== iconId)]));
    const list = lists.get(target.item.id) ?? [];
    list.splice(Math.min(list.length, row * cols + col), 0, iconId);
    lists.set(target.item.id, list);
    onChange({
      ...layout,
      items: layout.items.map((i) => (i.item.kind === "iconZone" ? { ...i, item: { ...i.item, iconOrder: lists.get(i.id) ?? [] } } : i)),
    });
  };

  return (
    <div style={{ position: "relative", width: screen.w * k, height: screen.h * k, overflow: "hidden", borderRadius: 8, boxShadow: "0 0 0 1px rgba(255,255,255,0.12)", userSelect: "none" }}>
      <div aria-hidden="true" style={{ position: "absolute", left: 0, top: 0, transform: `scale(${k * grid.scale})`, transformOrigin: "0 0", pointerEvents: "none" }}>
        <Wallpaper input={input} locale={locale} />
      </div>
      {items.map(({ item, rect }) => {
        const shown = live?.id === item.id ? live.rect : rect;
        return (
          <ItemFrame
            key={item.id}
            id={item.id}
            label={item.item.kind === "iconZone" ? item.item.name : "Calendar"}
            rect={shown}
            origin={origin}
            scale={k}
            selected={props.selectedId === item.id}
            invalid={live?.id === item.id && collides(live.rect, others(item.id))}
            onSelect={() => props.onSelect?.(item.id)}
            onMove={(dx, dy) => setLive({ id: item.id, rect: snapRect(moveRect(rect, dx, dy, wa), wa, grid) })}
            onMoveEnd={(dx, dy) => commit(item.id, snapRect(moveRect(rect, dx, dy, wa), wa, grid), rect)}
            onResize={(h: Handle, dx, dy) => setLive({ id: item.id, rect: snapRect(resizeRect(rect, h, dx, dy, minSize, wa), wa, grid) })}
            onResizeEnd={(h: Handle, dx, dy) => commit(item.id, snapRect(resizeRect(rect, h, dx, dy, minSize, wa), wa, grid), rect)}
          />
        );
      })}
      <IconLayer icons={state.icons} positions={placement.positions} grid={grid} origin={origin} scale={k} onDrop={dropIcon} />
      {items.map(({ item, rect }) => {
        const extra = placement.overflow.get(item.id)?.length ?? 0;
        if (item.item.kind !== "iconZone" || extra === 0) return null;
        const total = (placement.lists.get(item.id)?.length ?? 0);
        const grow = () => {
          const r = growToFit(rect, total, grid, wa, others(item.id));
          if (r) {
            setNoRoom(null);
            onChange(withRect(item.id, r));
          } else {
            setNoRoom(item.id);
          }
        };
        return (
          <div key={`over-${item.id}`} style={{ position: "absolute", left: (rect.x - origin.x) * k, top: (rect.y + rect.h - origin.y) * k + 4, zIndex: 4, display: "flex", gap: 8, alignItems: "center", background: "#161B22", border: "1px solid #FF9F8A", borderRadius: 8, padding: "6px 8px", fontSize: 12, color: "#E6EBF2" }}>
            <span>{extra === 1 ? "1 icon doesn't fit" : `${extra} icons don't fit`}</span>
            <button type="button" onClick={grow} style={{ minHeight: 32, padding: "0 10px", borderRadius: 6, border: "none", background: "#E6EBF2", color: "#0D1117", fontWeight: 600, cursor: "pointer" }}>
              Make bigger
            </button>
            {noRoom === item.id && <span style={{ color: "#FF9F8A" }}>No room to grow this zone.</span>}
          </div>
        );
      })}
    </div>
  );
}
