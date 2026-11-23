// SPDX-License-Identifier: GPL-3.0-or-later
import type { CalendarStyle } from "../generated/CalendarStyle";
import type { CalendarToolSettings } from "../generated/CalendarToolSettings";
import type { WeekMode } from "../generated/WeekMode";
import { field } from "./ui";

const row = { display: "flex", flexDirection: "column", gap: 6, fontSize: 14 } as const;
const check = { display: "flex", alignItems: "center", gap: 10, minHeight: 32, fontSize: 14 } as const;

export function CalendarSettings({ settings, onChange }: { settings: CalendarToolSettings; onChange: (s: CalendarToolSettings) => void }) {
  const week = (patch: Partial<CalendarToolSettings["week"]>) => onChange({ ...settings, week: { ...settings.week, ...patch } });
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <div style={row}>
        <label htmlFor="cal-style">Style</label>
        <select id="cal-style" value={settings.style} onChange={(e) => onChange({ ...settings, style: e.target.value as CalendarStyle })} style={field}>
          <option value="timeGrid">Time grid</option>
          <option value="dayStacks">Day stacks</option>
          <option value="skyline">Load skyline</option>
        </select>
      </div>
      <div style={row}>
        <label htmlFor="cal-week">Week</label>
        <select id="cal-week" value={settings.week.mode} onChange={(e) => week({ mode: e.target.value as WeekMode })} style={field}>
          <option value="thisWeek">This week (Mon–Sun)</option>
          <option value="next7Days">Next 7 days</option>
        </select>
      </div>
      <div style={row}>
        <label htmlFor="cal-full-day">Full day (hours)</label>
        <input
          id="cal-full-day"
          type="number"
          min={1}
          max={16}
          step={1}
          value={Math.round(settings.week.fullDayMinutes / 60)}
          onChange={(e) => {
            const hours = Number(e.target.value);
            if (Number.isFinite(hours) && hours >= 1 && hours <= 16) week({ fullDayMinutes: Math.round(hours) * 60 });
          }}
          style={{ ...field, width: 96 }}
        />
      </div>
      <label style={check}>
        <input type="checkbox" checked={settings.week.showWeekend} onChange={(e) => week({ showWeekend: e.target.checked })} style={{ width: 20, height: 20 }} />
        Show the weekend
      </label>
      <label style={check}>
        <input type="checkbox" checked={settings.showTitles} onChange={(e) => onChange({ ...settings, showTitles: e.target.checked })} style={{ width: 20, height: 20 }} />
        Show event titles
      </label>
    </div>
  );
}
