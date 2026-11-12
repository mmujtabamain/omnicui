// The page the web adapter opens. It mounts any of the app's components, fresh, and records every
// event the component reports. The registry: one line per component in `components`.
import { createRoot, type Root } from "react-dom/client";
import { flushSync } from "react-dom";
import type { ComponentType } from "react";
import { RedesignedCounter, RedesignedSwitch } from "web-app/components";
import { EVENTS } from "web-app/omnicui";

const components: Record<string, ComponentType<any>> = {
  "ui.counter": RedesignedCounter,
  "ui.switch": RedesignedSwitch,
};

let root: Root | null = null;
let events: { name: string; fields: Record<string, unknown> }[] = [];

// Scenarios for the readiness hook (OMNICUI_DEFECTS), standing in for an app whose data arrives
// after it mounts: the component appears only 400 ms after mount.
//   web.harness.late       the page says nothing, so reads made at once find nothing (the tests fail)
//   web.harness.late-busy  the page says it is busy until then (window.omnicuiBusy), so the
//                          adapter waits and the tests pass
const scenario = (name: string) => String((globalThis as any).__OMNICUI_DEFECTS__ ?? "").split(",").includes(name);
let pendingUntil = 0;
(window as any).omnicuiBusy = () => scenario("web.harness.late-busy") && performance.now() < pendingUntil;

Object.assign(window, {
  omnicuiMount(component: string, props: Record<string, unknown>): string | null {
    const C = components[component];
    if (!C) return `no component ${component}`;
    root?.unmount();
    events = [];
    const handlers = Object.fromEntries(
      (EVENTS[component] ?? []).map((name) => ["on" + name[0].toUpperCase() + name.slice(1), (fields: Record<string, unknown>) => events.push({ name, fields })]),
    );
    root = createRoot(document.getElementById("root")!);
    const show = () => flushSync(() => root!.render(<C {...props} {...handlers} />));
    if (scenario("web.harness.late") || scenario("web.harness.late-busy")) {
      pendingUntil = performance.now() + 400;
      setTimeout(show, 400);
    } else {
      show();
    }
    return null;
  },
  omnicuiEvents() {
    return events.splice(0);
  },
});
