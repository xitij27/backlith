// SPDX-License-Identifier: GPL-3.0-or-later
import type { CalendarSource } from "../../generated/CalendarSource";
import type { DayRef } from "../../generated/DayRef";
import type { WeekView } from "../../generated/WeekView";
import { formatHours, isoWeek, rangeLabel, weekdayShort } from "../format";
import { theme } from "../theme";

/** Height of the week header, including the space under it. */
export const HEADER_PX = 76;

const eyebrow = { fontSize: 13, fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase" as const, color: theme.muted };
const big = { fontFamily: theme.display, fontSize: 34, fontWeight: 600, lineHeight: 1.1 };

export function WeekHeader({ week, calendars, locale }: { week: WeekView; calendars: CalendarSource[]; locale?: string }) {
  const first = week.days[0]?.date;
  const last = week.days[week.days.length - 1]?.date;
  const day = (d: DayRef | null) => (d ? `${weekdayShort(d.date, locale)} · ${formatHours(d.bookedMinutes)}` : "—");
  return (
    <div style={{ height: HEADER_PX - 20, marginBottom: 20, display: "flex", justifyContent: "space-between", alignItems: "flex-end", gap: 32, minWidth: 0 }}>
      <div style={{ display: "flex", flexDirection: "column", gap: 4, minWidth: 0 }}>
        <div style={eyebrow}>{first ? `Week ${isoWeek(first)}` : ""}</div>
        <div style={big}>{first && last ? rangeLabel(first, last, locale) : ""}</div>
      </div>
      <div style={{ display: "flex", alignItems: "flex-end", gap: 36 }}>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 4 }}>
          <div style={big}>{formatHours(week.totalBookedMinutes)}</div>
          <div style={{ fontSize: 13, color: theme.muted }}>booked this week</div>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: 13, color: theme.muted }}>
          <div>
            Busiest <span style={{ color: theme.packed, fontWeight: 600 }}>{day(week.busiest)}</span>
          </div>
          <div>
            Lightest <span style={{ color: theme.text, fontWeight: 600 }}>{day(week.lightest)}</span>
          </div>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: 13, color: theme.muted }}>
          {calendars
            .filter((c) => c.enabled)
            .map((c) => (
              <div key={c.id} style={{ display: "flex", alignItems: "center", gap: 8, whiteSpace: "nowrap" }}>
                <span style={{ width: 10, height: 10, borderRadius: 3, background: c.color }} />
                {c.name}
              </div>
            ))}
        </div>
      </div>
    </div>
  );
}
