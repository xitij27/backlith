// SPDX-License-Identifier: GPL-3.0-or-later
// The outline drawn over a tool or zone: drag its body to move it, its
// handles to resize it. Handles are 44 CSS px to hit by touch.
import type { RectPx } from "../generated/RectPx";
import { HANDLES, type Handle } from "./geometry";
import { usePointerDrag } from "./usePointerDrag";

const HIT = 44;

const handlePlace: Record<Handle, { left: string; top: string; cursor: string }> = {
  n: { left: "50%", top: "0%", cursor: "ns-resize" },
  s: { left: "50%", top: "100%", cursor: "ns-resize" },
  e: { left: "100%", top: "50%", cursor: "ew-resize" },
  w: { left: "0%", top: "50%", cursor: "ew-resize" },
  ne: { left: "100%", top: "0%", cursor: "nesw-resize" },
  sw: { left: "0%", top: "100%", cursor: "nesw-resize" },
  nw: { left: "0%", top: "0%", cursor: "nwse-resize" },
  se: { left: "100%", top: "100%", cursor: "nwse-resize" },
};

function HandleDot({ handle, scale, onResize, onResizeEnd }: { handle: Handle; scale: number; onResize: (h: Handle, dx: number, dy: number) => void; onResizeEnd: (h: Handle, dx: number, dy: number) => void }) {
  const drag = usePointerDrag({ scale, onMove: (dx, dy) => onResize(handle, dx, dy), onEnd: (dx, dy) => onResizeEnd(handle, dx, dy) });
  const place = handlePlace[handle];
  return (
    <div
      aria-hidden="true"
      data-handle={handle}
      {...drag}
      style={{ position: "absolute", left: place.left, top: place.top, width: HIT, height: HIT, transform: "translate(-50%, -50%)", cursor: place.cursor, touchAction: "none", display: "grid", placeItems: "center" }}
    >
      <span style={{ width: 10, height: 10, borderRadius: 5, background: "#E6EBF2", boxShadow: "0 0 0 2px #0D1117" }} />
    </div>
  );
}

export function ItemFrame(props: {
  id: string;
  label: string;
  rect: RectPx;
  origin: { x: number; y: number };
  scale: number;
  selected: boolean;
  invalid: boolean;
  onSelect: () => void;
  onMove: (dx: number, dy: number) => void;
  onMoveEnd: (dx: number, dy: number) => void;
  onResize: (h: Handle, dx: number, dy: number) => void;
  onResizeEnd: (h: Handle, dx: number, dy: number) => void;
}) {
  const { id, label, rect, origin, scale, selected, invalid } = props;
  const drag = usePointerDrag({ scale, onMove: props.onMove, onEnd: props.onMoveEnd });
  return (
    <div
      data-testid={`frame-${id}`}
      role="group"
      aria-label={label}
      onPointerDownCapture={props.onSelect}
      {...drag}
      style={{
        position: "absolute",
        left: `${(rect.x - origin.x) * scale}px`,
        top: `${(rect.y - origin.y) * scale}px`,
        width: `${rect.w * scale}px`,
        height: `${rect.h * scale}px`,
        boxSizing: "border-box",
        border: `2px ${selected ? "solid" : "dashed"} ${invalid ? "#FF7A66" : selected ? "#E6EBF2" : "rgba(230,235,242,0.45)"}`,
        borderRadius: 8,
        cursor: "move",
        touchAction: "none",
      }}
    >
      <span style={{ position: "absolute", left: 6, top: 4, fontSize: 12, fontWeight: 600, color: "#0D1117", background: "#E6EBF2", borderRadius: 4, padding: "1px 6px", pointerEvents: "none" }}>{label}</span>
      {selected && HANDLES.map((h) => <HandleDot key={h} handle={h} scale={scale} onResize={props.onResize} onResizeEnd={props.onResizeEnd} />)}
    </div>
  );
}
