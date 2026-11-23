// SPDX-License-Identifier: GPL-3.0-or-later
import type { DesktopIcon } from "../../generated/DesktopIcon";
import type { Layout } from "../../generated/Layout";
import type { PlannerState } from "../../generated/PlannerState";
import type { RenderInput } from "../../generated/RenderInput";
import mockup from "../../render/__tests__/mockup-week.json";

const input = mockup as unknown as RenderInput;

export const ICON_NAMES = ["Recycle Bin", "Projects", "Downloads", "Notes.txt", "Report.pdf", "Photos", "Terminal", "Music"];

export function icons(): DesktopIcon[] {
  return ICON_NAMES.map((n, i) => ({ id: `C:\\Users\\me\\Desktop\\${n}`, name: n, position: { x: 0, y: i * 100 } }));
}

export function plannerState(layout: Layout = structuredClone(input.layout)): PlannerState {
  return {
    settings: { version: 1, layout, calendars: structuredClone(input.calendars), startAtSignIn: true, paused: false },
    icons: icons(),
    screen: input.screen,
    workArea: input.workArea,
    grid: { cellW: 100, cellH: 100, scale: 1 },
    week: structuredClone(input.week),
    sources: [],
    autoArrange: true,
    firstRun: true,
  };
}

/** A layout with the calendar narrowed to x 200–1500 and a second, empty zone at x 1600–1900. */
export function twoZones(): Layout {
  const layout: Layout = structuredClone(input.layout);
  for (const item of layout.items) {
    if (item.item.kind === "calendar") item.rect = { x: 200 / 1920, y: 0, w: 1300 / 1920, h: 1 };
  }
  layout.items.push({ id: "zone-2", rect: { x: 1600 / 1920, y: 0, w: 300 / 1920, h: 328 / 1032 }, item: { kind: "iconZone", name: "Inbox", takesNewIcons: false, iconOrder: [] } });
  return layout;
}
