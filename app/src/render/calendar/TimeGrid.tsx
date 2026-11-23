// SPDX-License-Identifier: GPL-3.0-or-later
// Hours down the side, events as blocks at their real times, and a "now"
// line in today's column. docs/mockups/TimeGrid.dc.html.
import type { DayEvent } from "../../generated/DayEvent";
import { visibleHours } from "../fit";
import { formatHours, formatTime } from "../format";
import { ink, theme, tint } from "../theme";
import type { StyleProps } from "./CalendarBlock";
import { AllDayChips, DayName, FullnessBar, LEVEL_LABEL, levelColor, oneLine } from "./parts";

const DAY_HEAD_PX = 92;
const GUTTER_PX = 56;

function EventBlock({ e, top, height, color, showTitles, locale }: { e: DayEvent; top: number; height: number; color: string; showTitles: boolean; locale?: string }) {
  const short = height < 34;
  return (
    <div
      style={{
        position: "absolute",
        left: 4,
        right: 4,
        top,
        height: Math.max(2, height),
        boxSizing: "border-box",
        padding: short ? "2px 8px" : "6px 8px",
        borderRadius: 6,
        background: tint(color, e.countsAsBusy ? 0.2 : 0.08),
        border: e.countsAsBusy ? "none" : `1px dashed ${tint(color, 0.5)}`,
        color: ink(color),
        opacity: e.done ? 0.45 : 1,
        overflow: "hidden",
        fontSize: 12.5,
        lineHeight: 1.3,
      }}
    >
      <div data-testid="event-title" style={{ ...oneLine, fontWeight: 600 }}>
        {showTitles ? e.title : "Busy"}
      </div>
      {!short && <div style={{ ...oneLine, opacity: 0.8, fontSize: 11.5 }}>{`${formatTime(e.startMinute, locale)} – ${formatTime(e.endMinute % 1440, locale)}`}</div>}
    </div>
  );
}

export function TimeGrid({ week, settings, colors, height, locale }: StyleProps) {
  const [from, to] = visibleHours(week);
  const bodyHeight = Math.max(0, height - DAY_HEAD_PX);
  const hourPx = bodyHeight / Math.max(1, to - from);
  const columns = `repeat(${week.days.length}, minmax(0, 1fr))`;
  const y = (minute: number) => (minute / 60 - from) * hourPx;
  const hours = Array.from({ length: to - from + 1 }, (_, i) => from + i);
  return (
    <div>
      <div style={{ display: "grid", gridTemplateColumns: `${GUTTER_PX}px ${columns}`, height: DAY_HEAD_PX }}>
        <div />
        {week.days.map((day) => (
          <div key={day.date} style={{ padding: "0 10px", display: "flex", flexDirection: "column", gap: 6, opacity: day.isPast ? 0.55 : 1, minWidth: 0 }}>
            <DayName day={day} locale={locale} />
            <FullnessBar day={day} fullDay={settings.week.fullDayMinutes} />
            <div style={{ fontSize: 12, color: levelColor(day) }}>{`${formatHours(day.bookedMinutes)} · ${LEVEL_LABEL[day.level]}`}</div>
            <AllDayChips day={day} colors={colors} />
          </div>
        ))}
      </div>
      <div style={{ position: "relative", height: bodyHeight }}>
        {hours.map((h) => (
          <div key={h}>
            <div style={{ position: "absolute", left: 0, width: GUTTER_PX - 12, top: y(h * 60) - 7, fontSize: 11, textAlign: "right", color: theme.muted }}>{formatTime((h % 24) * 60, locale)}</div>
            <div style={{ position: "absolute", left: GUTTER_PX, right: 0, top: y(h * 60), borderTop: `1px solid ${theme.line}` }} />
          </div>
        ))}
        <div style={{ position: "absolute", left: GUTTER_PX, right: 0, top: 0, bottom: 0, display: "grid", gridTemplateColumns: columns }}>
          {week.days.map((day) => (
            <div key={day.date} data-testid="day-column" style={{ position: "relative", borderLeft: `1px solid ${theme.line}`, background: day.isToday ? "rgba(255,255,255,0.035)" : "transparent" }}>
              {day.events.map((e, i) => (
                <EventBlock
                  key={i}
                  e={e}
                  top={y(e.startMinute) + 1}
                  height={y(e.endMinute) - y(e.startMinute) - 3}
                  color={colors.get(e.calendarId) ?? theme.fallbackCalendar}
                  showTitles={settings.showTitles}
                  locale={locale}
                />
              ))}
              {day.isToday && (
                <div data-testid="now-line" style={{ position: "absolute", left: 0, right: 0, top: y(week.nowMinute), height: 2, background: theme.now }}>
                  <div style={{ position: "absolute", left: -5, top: -4, width: 10, height: 10, borderRadius: 5, background: theme.now }} />
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
