// SPDX-License-Identifier: GPL-3.0-or-later
import { render, screen, within } from "@testing-library/react";
import type { CalendarStyle } from "../../generated/CalendarStyle";
import type { CalendarToolSettings } from "../../generated/CalendarToolSettings";
import type { RenderInput } from "../../generated/RenderInput";
import type { WeekView } from "../../generated/WeekView";
import { DAY_STACK_HEAD_PX, DayStacks, ITEM_PX } from "../calendar/DayStacks";
import { fitCount, lanes, skylineFillPx, visibleHours } from "../fit";
import { formatHours, formatTime } from "../format";
import { Wallpaper } from "../Wallpaper";
import fixture from "./mockup-week.json";

const mockup = fixture as unknown as RenderInput;

function withCalendar(style: CalendarStyle, extra: Partial<CalendarToolSettings> = {}): RenderInput {
  const input: RenderInput = structuredClone(mockup);
  for (const item of input.layout.items) {
    if (item.item.kind === "calendar") Object.assign(item.item, { style, ...extra });
  }
  return input;
}

const eventCount = mockup.week.days.reduce((n, d) => n + d.events.length, 0);

test("formatHours", () => {
  expect(formatHours(270)).toBe("4h 30m");
  expect(formatHours(480)).toBe("8h");
  expect(formatHours(0)).toBe("0h");
});

test("formatTime follows the locale", () => {
  expect(formatTime(570, "en-GB")).toBe("09:30");
  expect(formatTime(570, "en-US")).toBe("9:30 AM");
});

test("time grid has a column per day and one now line", () => {
  render(<Wallpaper input={withCalendar("timeGrid")} locale="en-GB" />);
  const columns = screen.getAllByTestId("day-column");
  expect(columns).toHaveLength(7);
  const lines = screen.getAllByTestId("now-line");
  expect(lines).toHaveLength(1);
  expect(columns[2]).toContainElement(lines[0]);
});

test("visibleHours widens for an early event", () => {
  expect(visibleHours(mockup.week)).toEqual([7, 22]);
  const early: WeekView = structuredClone(mockup.week);
  early.days[3].events[0].startMinute = 360;
  expect(visibleHours(early)).toEqual([6, 22]);
});

test("hidden titles show Busy only", () => {
  render(<Wallpaper input={withCalendar("timeGrid", { showTitles: false })} locale="en-GB" />);
  expect(screen.queryByText("Standup")).toBeNull();
  expect(screen.queryByText("Hackathon prep")).toBeNull();
  expect(screen.getAllByText("Busy")).toHaveLength(eventCount);
});

test("day stacks show free gaps", () => {
  render(<Wallpaper input={withCalendar("dayStacks")} locale="en-GB" />);
  const wednesday = screen.getAllByTestId("day-card")[2];
  expect(within(wednesday).getByText("2h free")).toBeInTheDocument();
  expect(within(wednesday).getByText("Hackathon prep")).toBeInTheDocument();
});

test("day stacks cut off with +N more", () => {
  expect(fitCount(300, 64, 8)).toEqual({ shown: 3, more: 5 });
  expect(fitCount(600, 64, 8)).toEqual({ shown: 8, more: 0 });
  const settings = (withCalendar("dayStacks").layout.items.find((i) => i.item.kind === "calendar")!.item) as CalendarToolSettings;
  const colors = new Map(mockup.calendars.map((c) => [c.id, c.color]));
  render(<DayStacks week={mockup.week} settings={settings} colors={colors} height={500} locale="en-GB" />);
  const wednesday = screen.getAllByTestId("day-card")[2];
  const items = mockup.week.days[2].events.length + mockup.week.days[2].freeGaps.length;
  const { more } = fitCount(500 - DAY_STACK_HEAD_PX, ITEM_PX, items);
  expect(more).toBeGreaterThan(0);
  expect(within(wednesday).getByText(`+${more} more`)).toBeInTheDocument();
});

test("skyline fill follows booked time", () => {
  expect(skylineFillPx(480, 480, 856, 1)).toBe(445);
  expect(skylineFillPx(60, 480, 856, 1)).toBe(88);
});

test("packed day is marked", () => {
  render(<Wallpaper input={withCalendar("skyline")} locale="en-GB" />);
  const wednesday = screen.getAllByTestId("day-column")[2];
  expect(within(wednesday).getByText("Packed")).toBeInTheDocument();
  expect(within(wednesday).getByText("8h")).toBeInTheDocument();
});

test("zones draw their labels at their rects", () => {
  render(<Wallpaper input={mockup} locale="en-GB" />);
  const zone = screen.getByTestId("zone-zone-1");
  expect(within(zone).getByText("Downloads")).toBeInTheDocument();
  expect(zone.style.left).toBe("0px");
  expect(zone.style.top).toBe("0px");
  expect(zone.style.width).toBe("100px");
  expect(zone.style.height).toBe("1032px");
});

test("long titles end with an ellipsis", () => {
  render(<Wallpaper input={withCalendar("timeGrid")} locale="en-GB" />);
  for (const title of screen.getAllByTestId("event-title")) {
    expect(title.style.textOverflow).toBe("ellipsis");
    expect(title.style.whiteSpace).toBe("nowrap");
  }
});

test("week header sums", () => {
  render(<Wallpaper input={mockup} locale="en-GB" />);
  expect(screen.getByText("30h")).toBeInTheDocument();
  expect(screen.getByText("Wed · 8h")).toBeInTheDocument();
  expect(screen.getByText("Sun · 1h")).toBeInTheDocument();
  expect(screen.getByText("Week 48")).toBeInTheDocument();
  expect(screen.getByText("23 – 29 November")).toBeInTheDocument();
});

test("the base colour fills the screen", () => {
  render(<Wallpaper input={mockup} locale="en-GB" />);
  const root = screen.getByTestId("wallpaper");
  expect(root.style.width).toBe("1920px");
  expect(root.style.height).toBe("1080px");
  expect(root.style.background).toContain("rgb(13, 17, 23)");
});

test("hidden titles hide all-day titles too", () => {
  const input = withCalendar("timeGrid", { showTitles: false });
  input.week.days[3].allDay = [{ calendarId: "personal", title: "Holiday" }];
  render(<Wallpaper input={input} locale="en-GB" />);
  expect(screen.queryByText("Holiday")).toBeNull();
  expect(screen.getByText("All day")).toBeInTheDocument();
});

test("lanes split overlapping events", () => {
  const ev = (startMinute: number, endMinute: number) => ({ startMinute, endMinute });
  expect(lanes([ev(600, 660), ev(630, 720), ev(780, 840)])).toEqual([
    { lane: 0, of: 2 },
    { lane: 1, of: 2 },
    { lane: 0, of: 1 },
  ]);
  expect(lanes([ev(600, 660), ev(660, 720)])).toEqual([
    { lane: 0, of: 1 },
    { lane: 0, of: 1 },
  ]);
});

test("overlapping events sit side by side in the time grid", () => {
  const input = withCalendar("timeGrid");
  const thursday = input.week.days[3];
  thursday.events.push({ ...thursday.events[1], title: "Overlap", startMinute: 850, endMinute: 920 });
  render(<Wallpaper input={input} locale="en-GB" />);
  expect(screen.getByText("Dentist").closest("[data-lane]")).toHaveAttribute("data-lane", "0/2");
  expect(screen.getByText("Overlap").closest("[data-lane]")).toHaveAttribute("data-lane", "1/2");
  expect(screen.getAllByText("Standup")[3].closest("[data-lane]")).toHaveAttribute("data-lane", "0/1");
});
