// SPDX-License-Identifier: GPL-3.0-or-later
import "@testing-library/jest-dom/vitest";

// jsdom has no PointerEvent: give tests one that carries pointer fields.
if (typeof window !== "undefined" && !("PointerEvent" in window)) {
  class PointerEventStandIn extends MouseEvent {
    pointerId: number;
    pointerType: string;
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init);
      this.pointerId = init.pointerId ?? 0;
      this.pointerType = init.pointerType ?? "mouse";
    }
  }
  (window as unknown as { PointerEvent: typeof PointerEventStandIn }).PointerEvent = PointerEventStandIn;
}
