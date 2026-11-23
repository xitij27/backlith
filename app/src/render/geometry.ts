// SPDX-License-Identifier: GPL-3.0-or-later
import type { RectF } from "../generated/RectF";
import type { RectPx } from "../generated/RectPx";

/** Same rounding as backlith-core's `to_px`. */
export function toPx(r: RectF, workArea: RectPx): RectPx {
  return {
    x: workArea.x + Math.round(r.x * workArea.w),
    y: workArea.y + Math.round(r.y * workArea.h),
    w: Math.round(r.w * workArea.w),
    h: Math.round(r.h * workArea.h),
  };
}

export function toFraction(r: RectPx, workArea: RectPx): RectF {
  const w = Math.max(1, workArea.w);
  const h = Math.max(1, workArea.h);
  return { x: (r.x - workArea.x) / w, y: (r.y - workArea.y) / h, w: r.w / w, h: r.h / h };
}

/** A box in CSS pixels, for absolute positioning. */
export type Box = { left: number; top: number; width: number; height: number };
