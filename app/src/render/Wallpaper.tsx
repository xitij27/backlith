// SPDX-License-Identifier: GPL-3.0-or-later
// The whole background: the base, the icon zone panels and the tools. The
// planner shows it as a live preview, and plan 2 draws it to the picture.
import type { RenderInput } from "../generated/RenderInput";
import { CalendarBlock } from "./calendar/CalendarBlock";
import { toPx, type Box } from "./geometry";
import { theme } from "./theme";
import { ZonePanel } from "./ZonePanel";

export function Wallpaper({ input, baseImageUrl, locale }: { input: RenderInput; baseImageUrl?: string; locale?: string }) {
  const scale = input.scale || 1;
  const base = input.layout.base;
  const box = (px: { x: number; y: number; w: number; h: number }): Box => ({
    left: (px.x - input.screen.x) / scale,
    top: (px.y - input.screen.y) / scale,
    width: px.w / scale,
    height: px.h / scale,
  });
  return (
    <div
      data-testid="wallpaper"
      style={{
        position: "relative",
        width: input.screen.w / scale,
        height: input.screen.h / scale,
        overflow: "hidden",
        background: base.kind === "color" ? base.hex : theme.ground,
        color: theme.text,
        fontFamily: theme.body,
      }}
    >
      {base.kind === "image" && baseImageUrl && (
        <img src={baseImageUrl} alt="" style={{ position: "absolute", inset: 0, width: "100%", height: "100%", objectFit: "cover" }} />
      )}
      {input.layout.items.map((item) => {
        const b = box(toPx(item.rect, input.workArea));
        return item.item.kind === "iconZone" ? (
          <ZonePanel key={item.id} id={item.id} name={item.item.name} box={b} />
        ) : (
          <CalendarBlock key={item.id} settings={item.item} week={input.week} calendars={input.calendars} box={b} locale={locale} />
        );
      })}
    </div>
  );
}
