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

/**
 * Side-by-side lanes for one day's events, in input order: events that
 * overlap, directly or through others, share a group and each gets its
 * own lane, so none is drawn on top of another.
 */
export function lanes(events: { startMinute: number; endMinute: number }[]): { lane: number; of: number }[] {
  const out = events.map(() => ({ lane: 0, of: 1 }));
  const order = events.map((_, i) => i).sort((a, b) => events[a].startMinute - events[b].startMinute || events[a].endMinute - events[b].endMinute);
  let group: number[] = [];
  let laneEnds: number[] = [];
  let groupEnd = -1;
  const close = () => group.forEach((i) => (out[i].of = laneEnds.length));
  for (const i of order) {
    const e = events[i];
    if (group.length > 0 && e.startMinute >= groupEnd) {
      close();
      group = [];
      laneEnds = [];
    }
    let lane = laneEnds.findIndex((end) => end <= e.startMinute);
    if (lane === -1) lane = laneEnds.push(0) - 1;
    laneEnds[lane] = e.endMinute;
    out[i].lane = lane;
    group.push(i);
    groupEnd = Math.max(groupEnd, e.endMinute);
  }
  close();
  return out;
}
