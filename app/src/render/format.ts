// SPDX-License-Identifier: GPL-3.0-or-later
import type { Level } from "../generated/Level";

/** 270 → "4h 30m", 480 → "8h". */
export function formatHours(minutes: number): string {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return m ? `${h}h ${m}m` : `${h}h`;
}

/** A time of day in the locale's own style: "09:30" in en-GB, "9:30 AM" in en-US. */
export function formatTime(minuteOfDay: number, locale?: string): string {
  const cycle = new Intl.DateTimeFormat(locale, { hour: "numeric" }).resolvedOptions().hourCycle;
  const hour = cycle === "h23" || cycle === "h24" ? "2-digit" : "numeric";
  const d = new Date(2000, 0, 1, Math.floor(minuteOfDay / 60), minuteOfDay % 60);
  return new Intl.DateTimeFormat(locale, { hour, minute: "2-digit" }).format(d);
}

export const LEVEL_LABEL: Record<Level, string> = {
  light: "Light",
  steady: "Steady",
  busy: "Busy",
  packed: "Packed",
};

/** "2026-11-25" → a local Date at midnight. */
export function parseDate(iso: string): Date {
  const [y, m, d] = iso.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export function weekdayShort(iso: string, locale?: string): string {
  return new Intl.DateTimeFormat(locale, { weekday: "short" }).format(parseDate(iso));
}

export function dayOfMonth(iso: string): string {
  return String(parseDate(iso).getDate());
}

/** ISO 8601 week number. */
export function isoWeek(iso: string): number {
  const d = parseDate(iso);
  const thursday = new Date(d.getFullYear(), d.getMonth(), d.getDate() + 3 - ((d.getDay() + 6) % 7));
  const firstThursday = new Date(thursday.getFullYear(), 0, 4);
  return 1 + Math.round(((thursday.getTime() - firstThursday.getTime()) / 86_400_000 - 3 + ((firstThursday.getDay() + 6) % 7)) / 7);
}

/** "23 – 29 November", or "30 November – 6 December" across months. */
export function rangeLabel(first: string, last: string, locale?: string): string {
  const a = parseDate(first);
  const b = parseDate(last);
  const month = (d: Date) => new Intl.DateTimeFormat(locale, { month: "long" }).format(d);
  return a.getMonth() === b.getMonth() ? `${a.getDate()} – ${b.getDate()} ${month(b)}` : `${a.getDate()} ${month(a)} – ${b.getDate()} ${month(b)}`;
}
