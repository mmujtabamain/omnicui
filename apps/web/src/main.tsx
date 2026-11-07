// The web app: the counter and the switch, a dark colour scheme toggle, and the events they report.
// `npm start` (or `run.sh app web`) builds it and serves it on 127.0.0.1.
import { createRoot } from "react-dom/client";
import { useState } from "react";
import { Counter, Switch } from "./components/index.ts";

const DARK = "--bg: #1b1d19; --fg: #ecebe6; --accent: #8ab4f8; --off: #6f746a; --thumb: #ffffff;";
const LIGHT = "--bg: #fbfaf7; --fg: #1d1f1b; --accent: #1f5fbf; --off: #8a8f84; --thumb: #ffffff;";

function Show() {
  const [events, setEvents] = useState<string[]>([]);
  const [dark, setDark] = useState(matchMedia("(prefers-color-scheme: dark)").matches);
  const log = (name: string) => (fields: object) => setEvents((e) => [`${name} ${JSON.stringify(fields)}`, ...e].slice(0, 8));
  document.documentElement.setAttribute("style", dark ? DARK : LIGHT);
  const caption = { fontSize: 12, opacity: 0.6, margin: "16px 0 6px" };
  return (
    <main style={{ padding: 24, fontFamily: "DejaVu Sans" }}>
      <div style={caption}>Counter</div>
      <Counter start={0} onChanged={log("changed")} />
      <div style={caption}>Switch</div>
      <Switch on={false} label="Dark mode" onToggled={log("toggled")} />
      <hr style={{ margin: "24px 0 12px" }} />
      <label><input type="checkbox" checked={dark} onChange={(e) => setDark(e.target.checked)} /> Dark colour scheme</label>
      <p>Events (newest first):</p>
      <pre aria-live="polite">{events.join("\n")}</pre>
    </main>
  );
}

createRoot(document.getElementById("root")!).render(<Show />);
