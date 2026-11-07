// The page the web adapter opens. It mounts any of the app's components, fresh, and records every
// event the component reports. The registry: one line per component in `components`.
import { createRoot, type Root } from "react-dom/client";
import { flushSync } from "react-dom";
import type { ComponentType } from "react";
import { Counter, Switch } from "web-app/components";
import { EVENTS } from "web-app/omnicui";

const components: Record<string, ComponentType<any>> = {
  "ui.counter": Counter,
  "ui.switch": Switch,
};

let root: Root | null = null;
let events: { name: string; fields: Record<string, unknown> }[] = [];

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
    flushSync(() => root!.render(<C {...props} {...handlers} />));
    return null;
  },
  omnicuiEvents() {
    return events.splice(0);
  },
});
