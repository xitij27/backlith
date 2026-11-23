// SPDX-License-Identifier: GPL-3.0-or-later
import { fireEvent, render, screen } from "@testing-library/react";
import { vi } from "vitest";
import type { Api } from "../../api";
import type { Layout } from "../../generated/Layout";
import { findFreeSpot } from "../geometry";
import { SidePanel } from "../SidePanel";
import { plannerState, twoZones } from "./fixture";

const WA = { x: 0, y: 0, w: 1920, h: 1032 };
const GRID = { cellW: 100, cellH: 100, scale: 1 };

function setup(layout: Layout, selectedId: string | null = null) {
  const state = plannerState(layout);
  const setLayout = vi.fn<(l: Layout) => void>();
  render(<SidePanel api={{} as Api} state={state} layout={layout} selectedId={selectedId} setLayout={setLayout} reload={async () => {}} />);
  return setLayout;
}
const zone = (l: Layout, id: string) => l.items.find((i) => i.id === id)!;

test("findFreeSpot scans the grid for room", () => {
  expect(findFreeSpot(twoZones(), WA, GRID)).toEqual({ x: 1500, y: 400, w: 300, h: 328 });
  expect(findFreeSpot(plannerState().settings.layout, WA, GRID)).toBeNull();
});

test("adding an icon zone puts it in free space", () => {
  const setLayout = setup(twoZones());
  fireEvent.click(screen.getByRole("button", { name: "Add icon zone" }));
  const added = setLayout.mock.calls[0][0].items.at(-1)!;
  expect(added.item.kind).toBe("iconZone");
  expect(added.rect.x * 1920).toBeCloseTo(1500);
  expect(added.rect.y * 1032).toBeCloseTo(400);
});

test("no room says so", () => {
  const setLayout = setup(plannerState().settings.layout);
  fireEvent.click(screen.getByRole("button", { name: "Add icon zone" }));
  expect(screen.getByText("No free space for a new zone. Make room first.")).toBeInTheDocument();
  expect(setLayout).not.toHaveBeenCalled();
});

test("renaming a zone", () => {
  const setLayout = setup(plannerState().settings.layout, "zone-1");
  fireEvent.change(screen.getByLabelText("Zone name"), { target: { value: "Work files" } });
  const z = zone(setLayout.mock.calls[0][0], "zone-1").item;
  expect(z.kind === "iconZone" && z.name).toBe("Work files");
});

test("new icons go to one zone only", () => {
  const setLayout = setup(twoZones(), "zone-2");
  fireEvent.click(screen.getByLabelText("New icons go here"));
  const next = setLayout.mock.calls[0][0];
  const z1 = zone(next, "zone-1").item;
  const z2 = zone(next, "zone-2").item;
  expect(z1.kind === "iconZone" && z1.takesNewIcons).toBe(false);
  expect(z2.kind === "iconZone" && z2.takesNewIcons).toBe(true);
});

test("removing the new-icons zone hands its job to another zone", () => {
  const setLayout = setup(twoZones(), "zone-1");
  fireEvent.click(screen.getByRole("button", { name: "Remove zone" }));
  const next = setLayout.mock.calls[0][0];
  expect(next.items.some((i) => i.id === "zone-1")).toBe(false);
  const z2 = zone(next, "zone-2").item;
  expect(z2.kind === "iconZone" && z2.takesNewIcons).toBe(true);
});
