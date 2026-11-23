// SPDX-License-Identifier: GPL-3.0-or-later
// Small pieces of the planner's side panel.
import type { CSSProperties, ReactNode } from "react";

export const field: CSSProperties = { minHeight: 40, boxSizing: "border-box", padding: "0 10px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.16)", background: "#0D1117", color: "#E6EBF2", fontSize: 14 };
export const primary: CSSProperties = { minHeight: 44, padding: "0 16px", borderRadius: 8, border: "none", background: "#E6EBF2", color: "#0D1117", fontWeight: 600, fontSize: 14, cursor: "pointer" };
export const secondary: CSSProperties = { minHeight: 44, padding: "0 14px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.2)", background: "transparent", color: "#E6EBF2", fontWeight: 600, fontSize: 14, cursor: "pointer" };

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section style={{ display: "flex", flexDirection: "column", gap: 12, padding: 16, borderRadius: 12, background: "#11161D", border: "1px solid rgba(255,255,255,0.08)" }}>
      <h2 style={{ margin: 0, fontSize: 13, fontWeight: 600, letterSpacing: "0.06em", textTransform: "uppercase", color: "#8C96A5" }}>{title}</h2>
      {children}
    </section>
  );
}

export function Note({ tone = "muted", children }: { tone?: "muted" | "error" | "good"; children: ReactNode }) {
  const color = tone === "error" ? "#FF9F8A" : tone === "good" ? "#8FD15A" : "#8C96A5";
  return <div style={{ fontSize: 13, color, lineHeight: 1.4 }}>{children}</div>;
}
