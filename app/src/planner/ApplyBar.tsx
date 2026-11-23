// SPDX-License-Identifier: GPL-3.0-or-later
// The bar over the canvas: what's going on, and Apply. The first apply
// asks before Backlith turns off "Auto arrange icons".
export function ApplyBar(props: { canApply: boolean; busy: boolean; message: string | null; error: string | null; onApply: () => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 16, padding: "12px 20px", borderBottom: "1px solid rgba(255,255,255,0.08)" }}>
      <h1 style={{ margin: 0, fontSize: 18, fontWeight: 600 }}>Backlith planner</h1>
      <div role="status" style={{ flex: 1, fontSize: 13, color: props.error ? "#FF9F8A" : "#8C96A5" }}>
        {props.error ?? props.message ?? "Drag tools and zones to arrange your background. Nothing changes until you press Apply."}
      </div>
      <button
        type="button"
        onClick={props.onApply}
        disabled={!props.canApply || props.busy}
        style={{ minHeight: 44, padding: "0 22px", borderRadius: 8, border: "none", background: props.canApply ? "#E6EBF2" : "#3A4250", color: "#0D1117", fontWeight: 600, fontSize: 14, cursor: props.canApply ? "pointer" : "not-allowed" }}
      >
        Apply
      </button>
    </div>
  );
}

export function AutoArrangeDialog({ onContinue, onSaveOnly, onCancel }: { onContinue: () => void; onSaveOnly: () => void; onCancel: () => void }) {
  return (
    <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.55)", display: "grid", placeItems: "center", zIndex: 10 }}>
      <div role="dialog" aria-modal="true" aria-labelledby="auto-arrange-title" style={{ width: 420, maxWidth: "calc(100vw - 32px)", background: "#161B22", border: "1px solid rgba(255,255,255,0.12)", borderRadius: 12, padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
        <h2 id="auto-arrange-title" style={{ margin: 0, fontSize: 17 }}>Turn off Auto arrange?</h2>
        <p style={{ margin: 0, fontSize: 14, lineHeight: 1.5, color: "#C9D2DE" }}>Backlith will turn off “Auto arrange icons” so your icons stay in their zones.</p>
        <p style={{ margin: 0, fontSize: 13, lineHeight: 1.5, color: "#8C96A5" }}>To keep Auto arrange, save the layout without moving icons. Windows then decides where icons go, so they may cover your tools.</p>
        <div style={{ display: "flex", gap: 12, justifyContent: "flex-end", flexWrap: "wrap" }}>
          <button type="button" onClick={onCancel} style={{ minHeight: 44, padding: "0 18px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.2)", background: "transparent", color: "#E6EBF2", fontWeight: 600, cursor: "pointer" }}>
            Cancel
          </button>
          <button type="button" onClick={onSaveOnly} style={{ minHeight: 44, padding: "0 18px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.2)", background: "transparent", color: "#E6EBF2", fontWeight: 600, cursor: "pointer" }}>
            Save without moving icons
          </button>
          <button type="button" onClick={onContinue} autoFocus style={{ minHeight: 44, padding: "0 18px", borderRadius: 8, border: "none", background: "#E6EBF2", color: "#0D1117", fontWeight: 600, cursor: "pointer" }}>
            Continue
          </button>
        </div>
      </div>
    </div>
  );
}
