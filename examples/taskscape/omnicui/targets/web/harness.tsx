// The page the web adapter opens. It mounts any widget of packages/ui, fresh, by its contract id,
// and records every event the widget reports. Each widget brings its own harness entry
// (packages/ui/src/harness/<name>.tsx): { id, Component, events }.
import { createRoot, type Root } from "react-dom/client";
import { flushSync } from "react-dom";
import { harnesses } from "./registry.gen";
import { TooltipProvider } from "../../../packages/ui/src/components/ui/tooltip";

type Harness = { id: string; Component: (props: any) => any; events: string[] };
const byId = Object.fromEntries((harnesses as Harness[]).map((h) => [h.id, h]));

let root: Root | null = null;
let events: { name: string; fields: Record<string, unknown> }[] = [];

Object.assign(window, {
  omnicuiMount(component: string, props: Record<string, unknown>): string | null {
    const h = byId[component];
    if (!h) return `no component ${component}`;
    root?.unmount();
    events = [];
    const emit = (name: string, fields: Record<string, unknown> = {}) => events.push({ name, fields });
    root = createRoot(document.getElementById("root")!);
    flushSync(() => root!.render(<TooltipProvider delayDuration={0}><h.Component {...props} emit={emit} /></TooltipProvider>));
    return null;
  },
  omnicuiEvents() {
    return events.splice(0);
  },
});
