// SPDX-License-Identifier: GPL-3.0-or-later
import type { CalendarSource } from "../../generated/CalendarSource";
import type { CalendarToolSettings } from "../../generated/CalendarToolSettings";
import type { WeekView } from "../../generated/WeekView";
import type { Box } from "../geometry";
import { theme } from "../theme";
import { DayStacks } from "./DayStacks";
import { Skyline } from "./Skyline";
import { TimeGrid } from "./TimeGrid";
import { HEADER_PX, WeekHeader } from "./WeekHeader";

const PAD = 24;

export function CalendarBlock(props: { settings: CalendarToolSettings; week: WeekView; calendars: CalendarSource[]; box: Box; locale?: string }) {
  const { settings, week, calendars, box, locale } = props;
  const colors = new Map(calendars.map((c) => [c.id, c.color]));
  const height = Math.max(0, box.height - 2 * PAD - HEADER_PX);
  const body = { week, settings, colors, height, locale };
  return (
    <div
      data-testid="calendar"
      style={{
        position: "absolute",
        left: `${box.left}px`,
        top: `${box.top}px`,
        width: `${box.width}px`,
        height: `${box.height}px`,
        boxSizing: "border-box",
        padding: PAD,
        borderRadius: 20,
        background: theme.panel,
        border: `1px solid ${theme.panelBorder}`,
        overflow: "hidden",
      }}
    >
      <WeekHeader week={week} calendars={calendars} locale={locale} />
      {settings.style === "timeGrid" && <TimeGrid {...body} />}
      {settings.style === "dayStacks" && <DayStacks {...body} />}
      {settings.style === "skyline" && <Skyline {...body} />}
    </div>
  );
}

export type StyleProps = {
  week: WeekView;
  settings: CalendarToolSettings;
  colors: Map<string, string>;
  /** CSS px available below the week header. */
  height: number;
  locale?: string;
};
