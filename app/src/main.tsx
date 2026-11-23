// SPDX-License-Identifier: GPL-3.0-or-later
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource/inter/400.css";
import "@fontsource/inter/600.css";
import "@fontsource/space-grotesk/600.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <p>Backlith</p>
  </StrictMode>,
);
