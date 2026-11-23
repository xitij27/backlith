// SPDX-License-Identifier: GPL-3.0-or-later
// The desktop icons, drawn as labelled outlines where they will sit.
// Drag one into a zone to move it there.
import { useState } from "react";
import type { DesktopIcon } from "../generated/DesktopIcon";
import type { IconGrid } from "../generated/IconGrid";
import type { PointPx } from "../generated/PointPx";
import { usePointerDrag } from "./usePointerDrag";

function Icon({ icon, at, grid, origin, scale, onDrop }: { icon: DesktopIcon; at: PointPx; grid: IconGrid; origin: { x: number; y: number }; scale: number; onDrop: (id: string, center: PointPx) => void }) {
  const [offset, setOffset] = useState<[number, number] | null>(null);
  const drag = usePointerDrag({
    scale,
    onMove: (dx, dy) => setOffset([dx, dy]),
    onEnd: (dx, dy) => {
      setOffset(null);
      if (dx !== 0 || dy !== 0) onDrop(icon.id, { x: at.x + dx + grid.cellW / 2, y: at.y + dy + grid.cellH / 2 });
    },
  });
  const [dx, dy] = offset ?? [0, 0];
  return (
    <div
      data-testid={`icon-${icon.name}`}
      title={icon.name}
      {...drag}
      style={{
        position: "absolute",
        left: `${(at.x + dx - origin.x) * scale}px`,
        top: `${(at.y + dy - origin.y) * scale}px`,
        width: `${grid.cellW * scale}px`,
        height: `${grid.cellH * scale}px`,
        boxSizing: "border-box",
        padding: 4,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 4,
        cursor: "grab",
        touchAction: "none",
        zIndex: offset ? 3 : 2,
      }}
    >
      <span style={{ width: "42%", aspectRatio: "1", borderRadius: 6, border: "1.5px solid #A9BBD0", background: "rgba(169,187,208,0.12)" }} />
      <span style={{ maxWidth: "100%", fontSize: 11, color: "#E6EBF2", textShadow: "0 1px 2px #000", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{icon.name}</span>
    </div>
  );
}

export function IconLayer(props: { icons: DesktopIcon[]; positions: Map<string, PointPx>; grid: IconGrid; origin: { x: number; y: number }; scale: number; onDrop: (id: string, center: PointPx) => void }) {
  return (
    <>
      {props.icons.map((icon) => (
        <Icon key={icon.id} icon={icon} at={props.positions.get(icon.id) ?? icon.position} grid={props.grid} origin={props.origin} scale={props.scale} onDrop={props.onDrop} />
      ))}
    </>
  );
}
