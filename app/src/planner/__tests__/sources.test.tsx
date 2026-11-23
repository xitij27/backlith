// SPDX-License-Identifier: GPL-3.0-or-later
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";
import type { Api } from "../../api";
import type { PlannerState } from "../../generated/PlannerState";
import { SourcesPanel } from "../SourcesPanel";
import { plannerState } from "./fixture";

const writeText = vi.fn(async () => {});
Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });

function setup(overrides: Partial<Record<keyof Api, unknown>> = {}, state: PlannerState = plannerState()) {
  const api = {
    addLink: vi.fn(),
    upload: vi.fn(),
    updateSource: vi.fn(async () => {}),
    removeSource: vi.fn(async () => {}),
    llmPrompt: vi.fn(async () => "PROMPT"),
    ...overrides,
  } as unknown as Api;
  const reload = vi.fn(async () => {});
  render(<SourcesPanel api={api} state={state} reload={reload} />);
  return { api, reload };
}

function chooseFile(name: string, text = "{}") {
  const input = screen.getByLabelText("Choose a file") as HTMLInputElement;
  fireEvent.change(input, { target: { files: [new File([text], name)] } });
}

beforeEach(() => writeText.mockClear());

test("adding a link shows events found", async () => {
  const { api, reload } = setup({ addLink: vi.fn(async () => ({ id: "src-1", ok: true, message: null, eventCount: 42 })) });
  fireEvent.change(screen.getByLabelText("Name"), { target: { value: "Work" } });
  fireEvent.change(screen.getByLabelText("Calendar link"), { target: { value: "https://example.com/work.ics" } });
  fireEvent.click(screen.getByRole("button", { name: "Add" }));
  expect(await screen.findByText("42 events in the next 8 weeks")).toBeInTheDocument();
  expect(api.addLink).toHaveBeenCalledWith("Work", expect.stringMatching(/^#[0-9A-F]{6}$/), "https://example.com/work.ics");
  expect(reload).toHaveBeenCalled();
});

test("a bad link shows why", async () => {
  setup({ addLink: vi.fn(async () => Promise.reject("the link no longer returns a calendar")) });
  fireEvent.change(screen.getByLabelText("Name"), { target: { value: "Work" } });
  fireEvent.change(screen.getByLabelText("Calendar link"), { target: { value: "https://example.com/x" } });
  fireEvent.click(screen.getByRole("button", { name: "Add" }));
  expect(await screen.findByText("the link no longer returns a calendar")).toBeInTheDocument();
});

test("upload errors list and copy", async () => {
  const errors = ["Event 1 (Gym): `start` \"10am\" is not a time like 09:30.", "Event 2 (Lab): unknown field `room`."];
  setup({ upload: vi.fn(async () => Promise.reject({ kind: "invalid", errors })) });
  chooseFile("gym.json");
  expect(await screen.findByText(errors[0])).toBeInTheDocument();
  expect(screen.getByText(errors[1])).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Copy errors" }));
  await waitFor(() => expect(writeText).toHaveBeenCalledWith(errors.join("\n")));
});

test("same name asks to replace", async () => {
  const upload = vi.fn(async (_n: string, _b: Uint8Array, replace: boolean) => (replace ? { id: "src-1", ok: true, message: null, eventCount: 30 } : Promise.reject({ kind: "exists", name: "Uni" })));
  setup({ upload });
  chooseFile("uni.json");
  expect(await screen.findByText("Replace “Uni”?")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Replace" }));
  await waitFor(() => expect(upload).toHaveBeenLastCalledWith("uni.json", expect.any(Uint8Array), true));
});

test("copy prompt", async () => {
  setup();
  fireEvent.click(screen.getByRole("button", { name: "Copy prompt for an LLM" }));
  expect(await screen.findByText("Copied")).toBeInTheDocument();
  expect(writeText).toHaveBeenCalledWith("PROMPT");
});

test("sources can be switched off and recoloured", async () => {
  const { api } = setup();
  fireEvent.click(screen.getByLabelText("Show Work · Outlook"));
  await waitFor(() => expect(api.updateSource).toHaveBeenCalledWith("work", "Work · Outlook", "#F5B65C", false));
  fireEvent.click(screen.getByRole("button", { name: "Colour for Work · Outlook" }));
  fireEvent.click(screen.getByRole("button", { name: "Colour #4FBF9F" }));
  await waitFor(() => expect(api.updateSource).toHaveBeenLastCalledWith("work", "Work · Outlook", "#4FBF9F", true));
});

test("accepts .json and .ics only", () => {
  const { api } = setup();
  expect((screen.getByLabelText("Choose a file") as HTMLInputElement).accept).toBe(".json,.ics");
  chooseFile("timetable.pdf");
  expect(screen.getByText("Use a .json schedule or an .ics calendar file.")).toBeInTheDocument();
  expect(api.upload).not.toHaveBeenCalled();
});

test("a failing link says since when", () => {
  const state = plannerState();
  state.sources = [{ id: "work", ok: false, message: "unreachable since 10:20", eventCount: 12 }];
  setup({}, state);
  expect(screen.getByText("unreachable since 10:20")).toBeInTheDocument();
});
