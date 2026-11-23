// SPDX-License-Identifier: GPL-3.0-or-later
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";
import type { Api } from "../../api";
import { Planner } from "../Planner";
import { plannerState } from "./fixture";

function fakeApi(extra: Partial<Record<keyof Api, unknown>> = {}) {
  return {
    plannerState: vi.fn(async () => plannerState()),
    updateSource: vi.fn(async () => {}),
    putIconsBack: vi.fn(async () => {}),
    restoreWallpaper: vi.fn(async () => {}),
    ...extra,
  } as unknown as Api;
}

test("unsaved layout changes survive a calendar source change", async () => {
  const api = fakeApi();
  render(<Planner api={api} locale="en-GB" />);
  const style = (await screen.findByLabelText("Style")) as HTMLSelectElement;
  fireEvent.change(style, { target: { value: "skyline" } });
  fireEvent.click(screen.getByLabelText("Show Work · Outlook"));
  await waitFor(() => expect(api.plannerState).toHaveBeenCalledTimes(2));
  expect((screen.getByLabelText("Style") as HTMLSelectElement).value).toBe("skyline");
});

test("the planner can put icons back and restore the old wallpaper", async () => {
  const api = fakeApi();
  render(<Planner api={api} locale="en-GB" />);
  fireEvent.click(await screen.findByRole("button", { name: "Put my icons back" }));
  expect(await screen.findByText("Your icons are back where they were.")).toBeInTheDocument();
  expect(api.putIconsBack).toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Restore my old wallpaper" }));
  expect(await screen.findByText("Your old wallpaper is back. Backlith is paused until you resume it from the tray.")).toBeInTheDocument();
});

test("undo errors are shown", async () => {
  const api = fakeApi({ putIconsBack: vi.fn(async () => Promise.reject("Backlith hasn't changed anything yet, so there is nothing to put back.")) });
  render(<Planner api={api} locale="en-GB" />);
  fireEvent.click(await screen.findByRole("button", { name: "Put my icons back" }));
  expect(await screen.findByText("Backlith hasn't changed anything yet, so there is nothing to put back.")).toBeInTheDocument();
});
