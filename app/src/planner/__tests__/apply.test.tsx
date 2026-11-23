// SPDX-License-Identifier: GPL-3.0-or-later
import { fireEvent, render, screen } from "@testing-library/react";
import { vi } from "vitest";
import type { Api } from "../../api";
import { Planner } from "../Planner";
import { plannerState } from "./fixture";

function fakeApi(autoArrange: boolean) {
  const calls: string[] = [];
  const state = { ...plannerState(), autoArrange };
  const api = {
    plannerState: vi.fn(async () => state),
    saveLayout: vi.fn(async () => {
      calls.push("save");
    }),
    applyLayout: vi.fn(async () => {
      calls.push("apply");
      return { moved: 8, overflow: [] };
    }),
  } as unknown as Api;
  return { api, calls };
}

test("first apply asks about auto arrange", async () => {
  const { api, calls } = fakeApi(true);
  render(<Planner api={api} locale="en-GB" />);
  fireEvent.click(await screen.findByRole("button", { name: "Apply" }));
  expect(screen.getByText("Backlith will turn off “Auto arrange icons” so your icons stay in their zones.")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Continue" }));
  expect(await screen.findByText("Moved 8 icons.")).toBeInTheDocument();
  expect(calls).toEqual(["save", "apply"]);
});

test("cancel leaves everything alone", async () => {
  const { api, calls } = fakeApi(true);
  render(<Planner api={api} locale="en-GB" />);
  fireEvent.click(await screen.findByRole("button", { name: "Apply" }));
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(calls).toEqual([]);
});

test("later applies go straight through", async () => {
  const { api, calls } = fakeApi(false);
  render(<Planner api={api} locale="en-GB" />);
  fireEvent.click(await screen.findByRole("button", { name: "Apply" }));
  expect(await screen.findByText("Moved 8 icons.")).toBeInTheDocument();
  expect(calls).toEqual(["save", "apply"]);
});
