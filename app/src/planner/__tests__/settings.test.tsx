// SPDX-License-Identifier: GPL-3.0-or-later
import { fireEvent, render, screen, within } from "@testing-library/react";
import { vi } from "vitest";
import type { CalendarToolSettings } from "../../generated/CalendarToolSettings";
import { CalendarSettings } from "../CalendarSettings";

const settings: CalendarToolSettings = { style: "timeGrid", week: { mode: "thisWeek", showWeekend: true, fullDayMinutes: 480 }, calendarIds: [], showTitles: true };

test("style choices", () => {
  render(<CalendarSettings settings={settings} onChange={() => {}} />);
  const options = within(screen.getByLabelText("Style")).getAllByRole("option").map((o) => o.textContent);
  expect(options).toEqual(["Time grid", "Day stacks", "Load skyline"]);
});

test("week choices", () => {
  render(<CalendarSettings settings={settings} onChange={() => {}} />);
  const options = within(screen.getByLabelText("Week")).getAllByRole("option").map((o) => o.textContent);
  expect(options).toEqual(["This week (Mon–Sun)", "Next 7 days"]);
  expect(screen.getByLabelText("Show the weekend")).toBeChecked();
  const fullDay = screen.getByLabelText("Full day (hours)") as HTMLInputElement;
  expect(fullDay.value).toBe("8");
  expect(fullDay.min).toBe("1");
  expect(fullDay.max).toBe("16");
  expect(screen.getByLabelText("Show event titles")).toBeChecked();
});

test("changes reach the layout", () => {
  const onChange = vi.fn();
  render(<CalendarSettings settings={settings} onChange={onChange} />);
  fireEvent.change(screen.getByLabelText("Style"), { target: { value: "skyline" } });
  expect(onChange).toHaveBeenLastCalledWith({ ...settings, style: "skyline" });
  fireEvent.change(screen.getByLabelText("Full day (hours)"), { target: { value: "6" } });
  expect(onChange).toHaveBeenLastCalledWith({ ...settings, week: { ...settings.week, fullDayMinutes: 360 } });
  fireEvent.click(screen.getByLabelText("Show event titles"));
  expect(onChange).toHaveBeenLastCalledWith({ ...settings, showTitles: false });
});
