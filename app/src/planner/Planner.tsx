// SPDX-License-Identifier: GPL-3.0-or-later
// The planner window: the canvas, the side panel, and Apply.
import { useCallback, useEffect, useState, type ReactNode } from "react";
import { api as realApi, type Api } from "../api";
import type { ApplyResult } from "../generated/ApplyResult";
import type { Layout } from "../generated/Layout";
import type { PlannerState } from "../generated/PlannerState";
import { toPx } from "../render/geometry";
import { ApplyBar, AutoArrangeDialog } from "./ApplyBar";
import { Canvas } from "./Canvas";
import { collides } from "./geometry";

const SIDE_PX = 360;
const BAR_PX = 69;
const MARGIN = 24;

function applyMessage(r: ApplyResult): string {
  const moved = r.moved === 0 ? "Everything was already in place." : r.moved === 1 ? "Moved 1 icon." : `Moved ${r.moved} icons.`;
  const left = r.overflow.reduce((n, z) => n + z.iconIds.length, 0);
  return left === 0 ? moved : `${moved} ${left === 1 ? "1 icon didn't" : `${left} icons didn't`} fit and stayed where it was.`;
}

/** A layout Apply would accept: no overlaps, and one zone for new icons if there are zones. */
export function layoutIsValid(layout: Layout, state: PlannerState): boolean {
  const rects = layout.items.map((i) => toPx(i.rect, state.workArea));
  if (rects.some((r, i) => collides(r, rects.filter((_, j) => j !== i)))) return false;
  const zones = layout.items.filter((i) => i.item.kind === "iconZone");
  return zones.length === 0 || zones.filter((z) => z.item.kind === "iconZone" && z.item.takesNewIcons).length === 1;
}

function useViewScale(screen: { w: number; h: number } | undefined): number {
  const compute = useCallback(() => {
    if (!screen) return 1;
    const w = (typeof window === "undefined" ? 1280 : window.innerWidth) - SIDE_PX - 2 * MARGIN;
    const h = (typeof window === "undefined" ? 800 : window.innerHeight) - BAR_PX - 2 * MARGIN;
    return Math.max(0.1, Math.min(w / screen.w, h / screen.h));
  }, [screen]);
  const [scale, setScale] = useState(compute);
  useEffect(() => {
    setScale(compute());
    const onResize = () => setScale(compute());
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, [compute]);
  return scale;
}

export function Planner({ api = realApi, locale, sidePanel }: { api?: Api; locale?: string; sidePanel?: (p: { state: PlannerState; layout: Layout; selectedId: string | null; setLayout: (l: Layout) => void; reload: () => Promise<void> }) => ReactNode }) {
  const [state, setState] = useState<PlannerState | null>(null);
  const [layout, setLayout] = useState<Layout | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [asking, setAsking] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      const s = await api.plannerState();
      setState(s);
      setLayout(s.settings.layout);
    } catch (e) {
      setError(String(e));
    }
  }, [api]);
  useEffect(() => {
    void reload();
  }, [reload]);
  const viewScale = useViewScale(state?.screen);

  if (!state || !layout) {
    return <div style={{ padding: 24, color: error ? "#FF9F8A" : "#8C96A5" }}>{error ?? "Loading…"}</div>;
  }

  const apply = async () => {
    setAsking(false);
    setBusy(true);
    setError(null);
    try {
      await api.saveLayout(layout);
      setMessage(applyMessage(await api.applyLayout()));
      await reload();
    } catch (e) {
      setError(typeof e === "string" ? e : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div style={{ minHeight: "100vh", background: "#0B0E13", color: "#E6EBF2", fontFamily: "Inter, system-ui, sans-serif" }}>
      <ApplyBar canApply={layoutIsValid(layout, state)} busy={busy} message={message} error={error} onApply={() => (state.autoArrange ? setAsking(true) : void apply())} />
      <div style={{ display: "flex", flexWrap: "wrap", gap: MARGIN, padding: MARGIN }}>
        <div style={{ flex: "999 1 560px", minWidth: 0 }}>
          <Canvas state={state} layout={layout} onChange={setLayout} viewScale={viewScale} selectedId={selectedId} onSelect={setSelectedId} locale={locale} />
        </div>
        <aside style={{ flex: `1 1 ${SIDE_PX - MARGIN}px`, minWidth: 0 }}>{sidePanel?.({ state, layout, selectedId, setLayout, reload })}</aside>
      </div>
      {asking && <AutoArrangeDialog onContinue={() => void apply()} onCancel={() => setAsking(false)} />}
    </div>
  );
}
