// SPDX-License-Identifier: GPL-3.0-or-later
// The planner's side panel: the calendar's settings, icon zones, and the
// calendars Backlith reads.
import { useState } from "react";
import type { Api } from "../api";
import type { Layout } from "../generated/Layout";
import type { LayoutItem } from "../generated/LayoutItem";
import type { PlannerState } from "../generated/PlannerState";
import type { ZoneSettings } from "../generated/ZoneSettings";
import { toFraction } from "../render/geometry";
import { CalendarSettings } from "./CalendarSettings";
import { findFreeSpot } from "./geometry";
import { SourcesPanel } from "./SourcesPanel";
import { field, Note, primary, secondary, Section } from "./ui";

type Props = { api: Api; state: PlannerState; layout: Layout; selectedId: string | null; setLayout: (l: Layout) => void; reload: () => Promise<void> };

function zoneOf(item: LayoutItem): ZoneSettings | null {
  return item.item.kind === "iconZone" ? item.item : null;
}

export function SidePanel({ api, state, layout, selectedId, setLayout, reload }: Props) {
  const [noRoom, setNoRoom] = useState(false);
  const calendar = layout.items.find((i) => i.item.kind === "calendar");
  const selected = layout.items.find((i) => i.id === selectedId && i.item.kind === "iconZone");
  const zones = layout.items.filter((i) => i.item.kind === "iconZone");

  const setItem = (id: string, f: (i: LayoutItem) => LayoutItem) => setLayout({ ...layout, items: layout.items.map((i) => (i.id === id ? f(i) : i)) });
  const setZone = (id: string, patch: Partial<ZoneSettings>) => setItem(id, (i) => ({ ...i, item: { ...(i.item as { kind: "iconZone" } & ZoneSettings), ...patch } }));

  const addZone = () => {
    const spot = findFreeSpot(layout, state.workArea, state.grid);
    if (!spot) {
      setNoRoom(true);
      return;
    }
    setNoRoom(false);
    let n = zones.length + 1;
    while (layout.items.some((i) => i.id === `zone-${n}`)) n += 1;
    setLayout({
      ...layout,
      items: [...layout.items, { id: `zone-${n}`, rect: toFraction(spot, state.workArea), item: { kind: "iconZone", name: `Zone ${n}`, takesNewIcons: zones.length === 0, iconOrder: [] } }],
    });
  };

  const takeNewIcons = (id: string) =>
    setLayout({ ...layout, items: layout.items.map((i) => (i.item.kind === "iconZone" ? { ...i, item: { ...i.item, takesNewIcons: i.id === id } } : i)) });

  const removeZone = (id: string) => {
    const gone = layout.items.find((i) => i.id === id);
    const items = layout.items.filter((i) => i.id !== id);
    if (gone && zoneOf(gone)?.takesNewIcons) {
      const heir = items.find((i) => i.item.kind === "iconZone");
      if (heir && heir.item.kind === "iconZone") heir.item = { ...heir.item, takesNewIcons: true };
    }
    setLayout({ ...layout, items });
  };

  const zone = selected ? zoneOf(selected) : null;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      {calendar && calendar.item.kind === "calendar" && (
        <Section title="Calendar">
          <CalendarSettings settings={calendar.item} onChange={(s) => setItem(calendar.id, (i) => ({ ...i, item: { kind: "calendar", ...s } }))} />
        </Section>
      )}
      <Section title="Icon zones">
        {selected && zone ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            <label htmlFor="zone-name" style={{ fontSize: 14 }}>
              Zone name
            </label>
            <input id="zone-name" value={zone.name} onChange={(e) => setZone(selected.id, { name: e.target.value })} style={field} />
            <label style={{ display: "flex", alignItems: "center", gap: 10, fontSize: 14, minHeight: 32 }}>
              <input type="checkbox" checked={zone.takesNewIcons} disabled={zone.takesNewIcons} onChange={() => takeNewIcons(selected.id)} style={{ width: 20, height: 20 }} />
              New icons go here
            </label>
            <div>
              <button type="button" style={secondary} onClick={() => removeZone(selected.id)}>
                Remove zone
              </button>
            </div>
          </div>
        ) : (
          <Note>Select a zone on the screen to rename it or choose where new icons go.</Note>
        )}
        <div>
          <button type="button" style={primary} onClick={addZone}>
            Add icon zone
          </button>
        </div>
        {noRoom && <Note tone="error">No free space for a new zone. Make room first.</Note>}
      </Section>
      <SourcesPanel api={api} state={state} reload={reload} />
    </div>
  );
}
