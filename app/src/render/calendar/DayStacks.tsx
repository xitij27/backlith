// SPDX-License-Identifier: GPL-3.0-or-later
// One card per day: hours booked large, events in order, and gaps of an
// hour or more shown as free time. docs/mockups/DayStacks.dc.html.
import { fitCount } from "../fit";
import { formatHours, formatTime } from "../format";
import { ink, theme, tint } from "../theme";
import type { StyleProps } from "./CalendarBlock";
import { AllDayChips, DayName, FullnessBar, LEVEL_LABEL, levelColor, oneLine } from "./parts";

/** A card's head: day name, hours, level, bar and all-day chips. */
export const DAY_STACK_HEAD_PX = 148;
/** One event or free-time row, with the gap under it. */
export const ITEM_PX = 64;

type Item =
  | { kind: "event"; start: number; end: number; title: string; calendarId: string; done: boolean; counts: boolean }
  | { kind: "free"; start: number; end: number; done: boolean };

export function DayStacks({ week, settings, colors, height, locale }: StyleProps) {
  return (
    <div style={{ display: "grid", gridTemplateColumns: `repeat(${week.days.length}, minmax(0, 1fr))`, gap: 12, height }}>
      {week.days.map((day) => {
        const items: Item[] = [
          ...day.events.map((e): Item => ({ kind: "event", start: e.startMinute, end: e.endMinute, title: e.title, calendarId: e.calendarId, done: e.done, counts: e.countsAsBusy })),
          ...day.freeGaps.map((g): Item => ({ kind: "free", start: g.startMinute, end: g.endMinute, done: day.isPast || (day.isToday && g.endMinute <= week.nowMinute) })),
        ].sort((a, b) => a.start - b.start || (a.kind === "free" ? -1 : 1));
        const { shown, more } = fitCount(height - DAY_STACK_HEAD_PX, ITEM_PX, items.length);
        return (
          <div
            key={day.date}
            data-testid="day-card"
            style={{
              boxSizing: "border-box",
              padding: "16px 12px",
              borderRadius: 16,
              background: "rgba(255,255,255,0.035)",
              border: `1px solid ${day.isToday ? "rgba(230,235,242,0.45)" : theme.panelBorder}`,
              display: "flex",
              flexDirection: "column",
              minWidth: 0,
              overflow: "hidden",
            }}
          >
            <div style={{ height: DAY_STACK_HEAD_PX - 32, display: "flex", flexDirection: "column", gap: 8, opacity: day.isPast ? 0.55 : 1 }}>
              <DayName day={day} locale={locale} />
              <div style={{ display: "flex", alignItems: "baseline", gap: 10 }}>
                <span style={{ fontFamily: theme.display, fontSize: 32, fontWeight: 600, lineHeight: 1, color: day.level === "packed" ? theme.packed : theme.text }}>{formatHours(day.bookedMinutes)}</span>
                <span style={{ fontSize: 13, fontWeight: 500, color: levelColor(day) }}>{LEVEL_LABEL[day.level]}</span>
              </div>
              <FullnessBar day={day} fullDay={settings.week.fullDayMinutes} />
              <AllDayChips day={day} colors={colors} />
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {items.slice(0, shown).map((it, i) => {
                const range = `${formatTime(it.start, locale)} – ${formatTime(it.end % 1440, locale)}`;
                if (it.kind === "free") {
                  return (
                    <div key={i} style={{ height: ITEM_PX - 8, boxSizing: "border-box", padding: "8px 12px", borderRadius: 10, border: "1px dashed rgba(255,255,255,0.18)", display: "flex", flexDirection: "column", justifyContent: "center", gap: 2, fontSize: 12, color: theme.muted, opacity: it.done ? 0.45 : 1 }}>
                      <span style={{ fontWeight: 600, color: theme.soft }}>{`${formatHours(it.end - it.start)} free`}</span>
                      <span style={oneLine}>{range}</span>
                    </div>
                  );
                }
                const color = colors.get(it.calendarId) ?? theme.fallbackCalendar;
                return (
                  <div key={i} style={{ height: ITEM_PX - 8, boxSizing: "border-box", padding: "8px 12px", borderRadius: 10, background: tint(color, it.counts ? 0.2 : 0.08), color: ink(color), opacity: it.done ? 0.45 : 1, display: "flex", flexDirection: "column", justifyContent: "center", gap: 2, minWidth: 0 }}>
                    <span style={{ ...oneLine, fontSize: 12, opacity: 0.8 }}>{range}</span>
                    <span data-testid="event-title" style={{ ...oneLine, fontSize: 14, fontWeight: 600 }}>
                      {settings.showTitles ? it.title : "Busy"}
                    </span>
                  </div>
                );
              })}
              {more > 0 && <div style={{ fontSize: 12, fontWeight: 600, color: theme.muted, padding: "4px 12px" }}>{`+${more} more`}</div>}
              {items.length === 0 && <div style={{ fontSize: 13, color: theme.muted, padding: "4px 2px" }}>Nothing planned</div>}
            </div>
          </div>
        );
      })}
    </div>
  );
}
