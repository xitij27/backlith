// SPDX-License-Identifier: GPL-3.0-or-later
import { fireEvent, render, screen } from "@testing-library/react";
import { vi } from "vitest";
import type { Layout } from "../../generated/Layout";
import { Canvas } from "../Canvas";
import { plannerState, twoZones } from "./fixture";

function drag(el: Element, from: [number, number], to: [number, number], pointerType = "mouse") {
  fireEvent.pointerDown(el, { clientX: from[0], clientY: from[1], pointerId: 1, pointerType, button: 0 });
  fireEvent.pointerMove(el, { clientX: to[0], clientY: to[1], pointerId: 1, pointerType });
  fireEvent.pointerUp(el, { clientX: to[0], clientY: to[1], pointerId: 1, pointerType });
}

function zoneRect(layout: Layout, id: string) {
  return layout.items.find((i) => i.id === id)!.rect;
}

function setup(layout?: Layout) {
  const state = plannerState(layout);
  const onChange = vi.fn<(l: Layout) => void>();
  render(<Canvas state={state} layout={state.settings.layout} onChange={onChange} viewScale={1} locale="en-GB" />);
  return { state, onChange };
}

test("dragging a zone by pointer moves it on the grid", () => {
  const { onChange } = setup();
  drag(screen.getByTestId("frame-zone-1"), [50, 500], [180, 540]);
  expect(onChange).toHaveBeenCalledTimes(1);
  const r = zoneRect(onChange.mock.calls[0][0], "zone-1");
  expect(r.x).toBeCloseTo(100 / 1920);
  expect(r.y).toBeCloseTo(0);
});

test("a drop onto another item snaps back", () => {
  const { onChange } = setup();
  const frame = screen.getByTestId("frame-zone-1");
  drag(frame, [50, 500], [1050, 500]);
  expect(onChange).not.toHaveBeenCalled();
  expect(frame.style.left).toBe("0px");
});

test("touch works like the mouse", () => {
  const { onChange } = setup();
  drag(screen.getByTestId("frame-zone-1"), [50, 500], [180, 540], "touch");
  expect(zoneRect(onChange.mock.calls[0][0], "zone-1").x).toBeCloseTo(100 / 1920);
});

test("dragging an icon into another zone updates icon_order", () => {
  const { onChange } = setup(twoZones());
  drag(screen.getByTestId("icon-Photos"), [50, 28 + 5 * 100 + 50], [1650, 78]);
  const next = onChange.mock.calls[0][0];
  const zone2 = next.items.find((i) => i.id === "zone-2")!.item;
  const zone1 = next.items.find((i) => i.id === "zone-1")!.item;
  expect(zone2.kind === "iconZone" && zone2.iconOrder).toEqual(["C:\\Users\\me\\Desktop\\Photos"]);
  expect(zone1.kind === "iconZone" && zone1.iconOrder).not.toContain("C:\\Users\\me\\Desktop\\Photos");
});

test("an overfull zone shows its count and Make bigger", () => {
  const layout = plannerState().settings.layout;
  layout.items.find((i) => i.id === "zone-1")!.rect.h = 328 / 1032;
  const { onChange } = setup(layout);
  expect(screen.getByText("5 icons don't fit")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Make bigger" }));
  const r = zoneRect(onChange.mock.calls[0][0], "zone-1");
  expect(r.h * 1032).toBeCloseTo(828);
});
