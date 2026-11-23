// SPDX-License-Identifier: GPL-3.0-or-later
// Pieces the three styles share.
import type { DayView } from "../../generated/DayView";
import { dayOfMonth, LEVEL_LABEL, weekdayShort } from "../format";
import { theme } from "../theme";

export function TodayPill() {
  return <span style={{ fontSize: 11, fontWeight: 600, padding: "2px 8px", borderRadius: 999, background: theme.text, color: theme.ground }}>Today</span>;
}

export function DayName({ day, locale }: { day: DayView; locale?: string }) {
  return (
    <div style={{ display: "flex", alignItems: "baseline", gap: 8, whiteSpace: "nowrap" }}>
      <span style={{ fontSize: 13, fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase", color: theme.muted }}>{weekdayShort(day.date, locale)}</span>
      <span style={{ fontFamily: theme.display, fontSize: 26, fontWeight: 600 }}>{dayOfMonth(day.date)}</span>
      {day.isToday && <TodayPill />}
    </div>
  );
}

export function FullnessBar({ day, fullDay }: { day: DayView; fullDay: number }) {
  const width = Math.min(100, Math.round((day.bookedMinutes / Math.max(1, fullDay)) * 100));
  return (
    <div style={{ height: 6, borderRadius: 3, background: theme.track, overflow: "hidden" }}>
      <div style={{ height: 6, width: `${width}%`, borderRadius: 3, background: day.level === "packed" ? theme.packed : theme.text }} />
    </div>
  );
}

export function levelColor(day: DayView): string {
  return day.level === "packed" ? theme.packed : theme.muted;
}

export { LEVEL_LABEL };

/** Text that stays on one line and ends in an ellipsis when too long. */
export const oneLine = { whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" } as const;

/** All-day events. With titles hidden, each reads "All day": the switch is for people who share their screen. */
export function AllDayChips({ day, colors, showTitles }: { day: DayView; colors: Map<string, string>; showTitles: boolean }) {
  if (day.allDay.length === 0) return null;
  return (
    <div style={{ display: "flex", gap: 4, minWidth: 0 }}>
      {day.allDay.slice(0, 2).map((e, i) => (
        <span key={i} style={{ ...oneLine, fontSize: 11, padding: "2px 6px", borderRadius: 6, background: `${(colors.get(e.calendarId) ?? theme.fallbackCalendar).slice(0, 7)}3D`, minWidth: 0 }}>
          {showTitles ? e.title : "All day"}
        </span>
      ))}
      {day.allDay.length > 2 && <span style={{ fontSize: 11, color: theme.muted }}>+{day.allDay.length - 2}</span>}
    </div>
  );
}
