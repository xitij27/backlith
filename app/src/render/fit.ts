// SPDX-License-Identifier: GPL-3.0-or-later
import type { WeekView } from "../generated/WeekView";

/** How many items of `itemPx` fit in `availablePx`; when not all do, one row is kept for "+N more". */
export function fitCount(availablePx: number, itemPx: number, total: number): { shown: number; more: number } {
  const fits = Math.max(0, Math.floor(availablePx / itemPx));
  if (total <= fits) return { shown: total, more: 0 };
  const shown = Math.max(0, fits - 1);
  return { shown, more: total - shown };
}

/** The skyline's fill: 1.25 full days fill 65% of the column, never less than 88 px. */
export function skylineFillPx(booked: number, fullDay: number, columnPx: number, scale: number): number {
  const share = Math.min(1, booked / (1.25 * Math.max(1, fullDay)));
  return Math.max(88 * scale, Math.round(share * 0.65 * columnPx));
}

/** The hours the time grid shows: 07:00 to 22:00, widened to whole hours around any event outside them. */
export function visibleHours(week: WeekView): [number, number] {
  let start = 7;
  let end = 22;
  for (const day of week.days) {
    for (const e of day.events) {
      start = Math.min(start, Math.floor(e.startMinute / 60));
      end = Math.max(end, Math.ceil(e.endMinute / 60));
    }
  }
  return [start, Math.min(24, end)];
}
