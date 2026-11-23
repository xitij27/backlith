// SPDX-License-Identifier: GPL-3.0-or-later
// Dragging by pointer, so mouse, pen and touch behave the same. Distances
// are reported in screen pixels, undoing the canvas's scale.
import { useRef } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";

export function usePointerDrag(opts: { scale: number; onMove: (dx: number, dy: number) => void; onEnd: (dx: number, dy: number) => void }) {
  const start = useRef<{ x: number; y: number; id: number } | null>(null);
  const delta = (e: ReactPointerEvent) => {
    const s = start.current!;
    return [(e.clientX - s.x) / opts.scale, (e.clientY - s.y) / opts.scale] as const;
  };
  return {
    onPointerDown(e: ReactPointerEvent) {
      if (e.pointerType === "mouse" && e.button !== 0) return;
      e.stopPropagation();
      e.preventDefault();
      (e.currentTarget as Element).setPointerCapture?.(e.pointerId);
      start.current = { x: e.clientX, y: e.clientY, id: e.pointerId };
    },
    onPointerMove(e: ReactPointerEvent) {
      if (start.current?.id !== e.pointerId) return;
      opts.onMove(...delta(e));
    },
    onPointerUp(e: ReactPointerEvent) {
      if (start.current?.id !== e.pointerId) return;
      const [dx, dy] = delta(e);
      start.current = null;
      opts.onEnd(dx, dy);
    },
    onPointerCancel() {
      if (!start.current) return;
      start.current = null;
      opts.onEnd(0, 0);
    },
  };
}
