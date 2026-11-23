// SPDX-License-Identifier: GPL-3.0-or-later
// Typed wrappers around the app's commands (app/src-tauri/src/commands.rs).
import { invoke } from "@tauri-apps/api/core";
import type { ApplyResult } from "./generated/ApplyResult";
import type { Layout } from "./generated/Layout";
import type { PlannerState } from "./generated/PlannerState";
import type { RenderInput } from "./generated/RenderInput";
import type { SourceStatus } from "./generated/SourceStatus";

export const api = {
  plannerState: () => invoke<PlannerState>("planner_state"),
  saveLayout: (layout: Layout) => invoke<void>("save_layout", { layout }),
  applyLayout: () => invoke<ApplyResult>("apply_layout"),
  putIconsBack: () => invoke<void>("put_icons_back"),
  restoreWallpaper: () => invoke<void>("restore_wallpaper"),
  addLink: (name: string, color: string, url: string) => invoke<SourceStatus>("add_link", { name, color, url }),
  /** Rejects with an UploadError. */
  upload: (fileName: string, bytes: Uint8Array, replace: boolean) => invoke<SourceStatus>("upload", { fileName, bytes: Array.from(bytes), replace }),
  updateSource: (id: string, name: string, color: string, enabled: boolean) => invoke<void>("update_source", { id, name, color, enabled }),
  removeSource: (id: string) => invoke<void>("remove_source", { id }),
  llmPrompt: () => invoke<string>("llm_prompt"),
  renderInput: () => invoke<RenderInput>("render_input"),
};

export type Api = typeof api;
