// SPDX-License-Identifier: GPL-3.0-or-later
// The wallpaper's look, from docs/mockups/.

export const theme = {
  ground: "#0D1117",
  /** Dark backing behind every tool, so text reads over any base image. */
  panel: "rgba(13,17,23,0.72)",
  panelBorder: "rgba(255,255,255,0.08)",
  zone: "rgba(255,255,255,0.035)",
  line: "rgba(255,255,255,0.06)",
  track: "rgba(255,255,255,0.08)",
  text: "#E6EBF2",
  muted: "#8C96A5",
  soft: "#C9D2DE",
  packed: "#FF9F8A",
  now: "#FF7A66",
  fallbackCalendar: "#8C96A5",
  body: "Inter, system-ui, sans-serif",
  display: "'Space Grotesk', Inter, system-ui, sans-serif",
} as const;

/** `#RRGGBB` at the given opacity, as `#RRGGBBAA`. */
export function tint(hex: string, opacity: number): string {
  const a = Math.round(Math.min(1, Math.max(0, opacity)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `${hex.slice(0, 7)}${a}`;
}

/** A light version of a calendar colour, for text on its tint. */
export function ink(hex: string): string {
  return `color-mix(in srgb, ${hex.slice(0, 7)} 35%, #FFFFFF)`;
}
