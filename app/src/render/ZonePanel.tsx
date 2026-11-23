// SPDX-License-Identifier: GPL-3.0-or-later
// The soft labelled panel painted behind an icon zone. Its label strip is
// 28 CSS px tall: ZONE_LABEL_PX in backlith-core.
import type { Box } from "./geometry";
import { theme } from "./theme";

export function ZonePanel({ id, name, box }: { id: string; name: string; box: Box }) {
  return (
    <div
      data-testid={`zone-${id}`}
      style={{
        position: "absolute",
        left: `${box.left}px`,
        top: `${box.top}px`,
        width: `${box.width}px`,
        height: `${box.height}px`,
        boxSizing: "border-box",
        borderRadius: 16,
        background: theme.zone,
        border: `1px solid ${theme.panelBorder}`,
      }}
    >
      <div
        style={{
          height: 28,
          padding: "0 12px",
          display: "flex",
          alignItems: "center",
          fontSize: 12,
          fontWeight: 600,
          letterSpacing: "0.08em",
          textTransform: "uppercase",
          color: theme.muted,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {name}
      </div>
    </div>
  );
}
