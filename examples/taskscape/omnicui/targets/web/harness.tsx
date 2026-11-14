// The page the web adapter opens. It mounts any widget of packages/ui (or, in app mode, any screen
// of apps/web/frontend), fresh, by its contract id, and records every event it reports. Each one
// brings its own harness entry (src/harness/<name>.tsx): { id, Component, events }.
import { createRoot, type Root } from "react-dom/client";
import { flushSync } from "react-dom";
import { harnesses } from "./registry.gen";
import { TooltipProvider } from "../../../packages/ui/src/components/ui/tooltip";

declare const OMNICUI_APP: boolean;
type Harness = { id: string; Component: (props: any) => any; events: string[] };
const byId = Object.fromEntries((harnesses as Harness[]).map((h) => [h.id, h]));

let root: Root | null = null;
let events: { name: string; fields: Record<string, unknown> }[] = [];

Object.assign(window, {
  async omnicuiMount(component: string, props: Record<string, unknown>): Promise<string | null> {
    const h = byId[component];
    if (!h) return `no component ${component}`;
    root?.unmount();
    events = [];
    // A screen starts from a demo data set, signed in as the demo account.
    if (OMNICUI_APP) {
      const r = await fetch("/api/demo/reset", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ dataset: props.dataset ?? "standard" }) });
      if (!r.ok) return `demo reset failed: ${r.status} ${await r.text()}`;
    }
    const emit = (name: string, fields: Record<string, unknown> = {}) => events.push({ name, fields });
    root = createRoot(document.getElementById("root")!);
    flushSync(() => root!.render(<TooltipProvider delayDuration={0}><h.Component {...props} emit={emit} /></TooltipProvider>));
    return null;
  },
  omnicuiEvents() {
    return events.splice(0);
  },
});
