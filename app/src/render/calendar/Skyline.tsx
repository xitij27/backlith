// SPDX-License-Identifier: GPL-3.0-or-later
// Each column fills from the bottom by how busy the day is, so the week
// reads like a skyline. docs/mockups/Skyline.dc.html.
import { fitCount, skylineFillPx } from "../fit";
import { formatHours, formatTime } from "../format";
import { theme } from "../theme";
import type { StyleProps } from "./CalendarBlock";
import { AllDayChips, DayName, LEVEL_LABEL, oneLine } from "./parts";

const HEAD_PX = 64;
const ROW_PX = 24;

const FILL = {
  light: { fill: "rgba(230,235,242,0.06)", edge: "rgba(230,235,242,0.25)" },
  steady: { fill: "rgba(230,235,242,0.10)", edge: "rgba(230,235,242,0.35)" },
  busy: { fill: "rgba(230,235,242,0.15)", edge: "rgba(230,235,242,0.50)" },
  packed: { fill: "rgba(255,159,138,0.20)", edge: theme.packed },
} as const;

export function Skyline({ week, settings, colors, height, locale }: StyleProps) {
  return (
    <div style={{ display: "grid", gridTemplateColumns: `repeat(${week.days.length}, minmax(0, 1fr))`, gap: 12, height }}>
      {week.days.map((day) => {
        const fill = skylineFillPx(day.bookedMinutes, settings.week.fullDayMinutes, height, 1);
        const { shown, more } = fitCount(height - fill - HEAD_PX - 16, ROW_PX, day.events.length);
        const packed = day.level === "packed";
        return (
          <div
            key={day.date}
            data-testid="day-column"
            style={{ position: "relative", overflow: "hidden", borderRadius: 16, background: "rgba(255,255,255,0.025)", border: `1px solid ${day.isToday ? "rgba(230,235,242,0.45)" : "rgba(255,255,255,0.07)"}`, minWidth: 0 }}
          >
            <div style={{ position: "absolute", left: 0, right: 0, bottom: 0, height: fill, boxSizing: "border-box", padding: "14px 16px", background: FILL[day.level].fill, borderTop: `2px solid ${FILL[day.level].edge}`, display: "flex", flexDirection: "column", gap: 4 }}>
              <span style={{ fontFamily: theme.display, fontSize: 44, fontWeight: 600, lineHeight: 1, color: packed ? "#FFB8A8" : theme.text }}>{formatHours(day.bookedMinutes)}</span>
              <span style={{ fontSize: 13, fontWeight: 500, color: packed ? "#FFB8A8" : theme.soft }}>{LEVEL_LABEL[day.level]}</span>
            </div>
            <div style={{ position: "relative", padding: "16px 14px", display: "flex", flexDirection: "column", gap: 12 }}>
              <div style={{ opacity: day.isPast ? 0.55 : 1, display: "flex", flexDirection: "column", gap: 6 }}>
                <DayName day={day} locale={locale} />
                <AllDayChips day={day} colors={colors} />
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                {day.events.slice(0, shown).map((e, i) => (
                  <div key={i} style={{ height: ROW_PX - 4, display: "flex", alignItems: "center", gap: 8, opacity: e.done ? 0.45 : 1, minWidth: 0 }}>
                    <span style={{ flex: "none", width: 8, height: 8, borderRadius: 4, background: colors.get(e.calendarId) ?? theme.fallbackCalendar }} />
                    <span style={{ flex: "none", fontSize: 12, color: theme.muted }}>{formatTime(e.startMinute, locale)}</span>
                    <span data-testid="event-title" style={{ ...oneLine, fontSize: 13, color: theme.text }}>
                      {settings.showTitles ? e.title : "Busy"}
                    </span>
                  </div>
                ))}
                {more > 0 && <div style={{ fontSize: 12, fontWeight: 600, color: theme.muted }}>{`+${more} more`}</div>}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
