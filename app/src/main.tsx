// SPDX-License-Identifier: GPL-3.0-or-later
import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource/inter/400.css";
import "@fontsource/inter/600.css";
import "@fontsource/space-grotesk/600.css";
import { api } from "./api";
import type { PlannerState } from "./generated/PlannerState";

// Shows the planner's state as JSON. The real planner replaces this in Task 9.
function StateView() {
  const [state, setState] = useState<PlannerState | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    api.plannerState().then(setState, (e) => setError(String(e)));
  }, []);
  return <pre style={{ whiteSpace: "pre-wrap" }}>{error ?? JSON.stringify(state, null, 2)}</pre>;
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <StateView />
  </StrictMode>,
);
