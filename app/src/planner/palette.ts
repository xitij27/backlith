// SPDX-License-Identifier: GPL-3.0-or-later
// Calendar colours, the same as PALETTE in app/src-tauri/src/core.rs:
// neighbours differ in lightness, not only in hue.
export const PALETTE = ["#5A8CF0", "#F5B65C", "#4FBF9F", "#E06C9F", "#9B8CF0", "#F08A5A", "#8FD15A", "#5AC8E0"];

export function nextColor(used: string[]): string {
  return PALETTE.find((c) => !used.includes(c)) ?? PALETTE[0];
}
