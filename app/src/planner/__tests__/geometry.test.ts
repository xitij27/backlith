// SPDX-License-Identifier: GPL-3.0-or-later
import { collides, growToFit, moveRect, placeIcons, resizeRect, snapRect } from "../geometry";
import { icons, plannerState } from "./fixture";

const WA = { x: 0, y: 0, w: 1920, h: 1032 };
const GRID = { cellW: 100, cellH: 100, scale: 1 };

test("snapRect matches the Rust rule", () => {
  expect(snapRect({ x: 130, y: 40, w: 290, h: 310 }, WA, GRID)).toEqual({ x: 100, y: 0, w: 300, h: 300 });
  expect(snapRect({ x: 1880, y: 1000, w: 20, h: 20 }, WA, GRID)).toEqual({ x: 1820, y: 932, w: 100, h: 100 });
});

test("touching edges do not collide", () => {
  expect(collides({ x: 0, y: 0, w: 100, h: 100 }, [{ x: 100, y: 0, w: 100, h: 100 }])).toBe(false);
  expect(collides({ x: 0, y: 0, w: 101, h: 100 }, [{ x: 100, y: 0, w: 100, h: 100 }])).toBe(true);
});

test("moveRect stays inside the bounds", () => {
  expect(moveRect({ x: 1800, y: 900, w: 200, h: 200 }, 500, 500, WA)).toEqual({ x: 1720, y: 832, w: 200, h: 200 });
  expect(moveRect({ x: 10, y: 10, w: 200, h: 200 }, -50, -50, WA)).toEqual({ x: 0, y: 0, w: 200, h: 200 });
});

test("resizeRect keeps a minimum size", () => {
  expect(resizeRect({ x: 0, y: 0, w: 300, h: 300 }, "se", 100, 50, { w: 100, h: 100 }, WA)).toEqual({ x: 0, y: 0, w: 400, h: 350 });
  expect(resizeRect({ x: 100, y: 100, w: 300, h: 300 }, "nw", 250, 250, { w: 100, h: 100 }, WA)).toEqual({ x: 300, y: 300, w: 100, h: 100 });
});

test("growToFit grows down first", () => {
  expect(growToFit({ x: 0, y: 0, w: 300, h: 328 }, 12, GRID, WA, [])).toEqual({ x: 0, y: 0, w: 300, h: 428 });
});

test("growToFit grows right when down is blocked", () => {
  const below = { x: 0, y: 328, w: 300, h: 704 };
  expect(growToFit({ x: 0, y: 0, w: 300, h: 328 }, 12, GRID, WA, [below])).toEqual({ x: 0, y: 0, w: 400, h: 328 });
});

test("growToFit returns null when boxed in", () => {
  const others = [{ x: 0, y: 328, w: 1920, h: 704 }, { x: 300, y: 0, w: 1620, h: 328 }];
  expect(growToFit({ x: 0, y: 0, w: 300, h: 328 }, 12, GRID, WA, others)).toBeNull();
});

test("placeIcons fills zones row by row and reports overflow", () => {
  const state = plannerState();
  const placed = placeIcons(state.settings.layout, icons(), WA, GRID);
  expect(placed.positions.get("C:\\Users\\me\\Desktop\\Recycle Bin")).toEqual({ x: 0, y: 28 });
  expect(placed.positions.get("C:\\Users\\me\\Desktop\\Projects")).toEqual({ x: 0, y: 128 });
  expect(placed.overflow.get("zone-1") ?? []).toEqual([]);
});
