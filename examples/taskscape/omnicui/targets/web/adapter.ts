// The web target's adapter. omnicui starts it once per check (`adapter:` in target.yaml) and sends
// JSON-RPC 2.0 requests, one per line, on standard input; every reply is one line on standard
// output. Logs go to standard error only, because anything else on standard output breaks the
// conversation.
//
// It builds the widget library (packages/ui) with every widget's harness entry
// (packages/ui/src/harness/*.tsx) for production with esbuild, compiles its Tailwind CSS, serves
// both from a local HTTP server, and performs every step in a real Chromium through Playwright. Each reply is sent
// only once the page has settled: no running animations, fonts loaded, the app not busy (see
// settle()), two frames painted.
import { createServer, request as httpRequest, type IncomingMessage, type ServerResponse } from "node:http";
import { readFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync, readdirSync, existsSync, copyFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { execFileSync, spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";
import { chromium, type Page, type CDPSession } from "playwright";

const here = dirname(fileURLToPath(import.meta.url));
const log = (...a: unknown[]) => console.error("[web]", ...a);

// protocol: error codes
const NO_COMPONENT = -32001, NO_PART = -32002, STEP_FAILED = -32003, NO_METHOD = -32601;
class RpcError extends Error {
  code: number;
  constructor(code: number, message: string) {
    super(message);
    this.code = code;
  }
}

// --- build what ships, and serve it ------------------------------------------------------------
// Two targets share this adapter. "web" (no argument) mounts the widgets of packages/ui; "app"
// (targets/web-app) mounts the screens of apps/web/frontend on a real API with demo data.
const APP = process.argv[2] === "app";
const ui = join(here, "../../../packages/ui");
const frontend = join(here, "../../../apps/web/frontend");
const source = APP ? join(frontend, "src") : join(ui, "src");
// Every adapter process builds into a folder of its own, so several checks can run side by side.
const dist = mkdtempSync(join(tmpdir(), "omnicui-web-"));
process.on("exit", () => rmSync(dist, { recursive: true, force: true }));
for (const sig of ["SIGTERM", "SIGINT", "SIGHUP"] as const) process.on(sig, () => process.exit(1));
// One harness entry per widget or screen (src/harness/<name>.tsx, default export { id, Component, events }).
// OMNICUI_WIDGETS=button,icon builds only those, so a widget still being written elsewhere cannot break this build.
const only = process.env.OMNICUI_WIDGETS?.split(",").map((w) => w.trim()).filter(Boolean);
const entries = (existsSync(join(source, "harness")) ? readdirSync(join(source, "harness")).filter((f) => f.endsWith(".tsx")).sort() : [])
  .filter((f) => !only || only.includes(f.replace(/\.tsx$/, "")));
const registry = join(dist, "registry.gen.ts");
writeFileSync(registry,
  entries.map((f, i) => `import h${i} from ${JSON.stringify(join(source, "harness", f))};`).join("\n") +
  `\nexport const harnesses = [${entries.map((_, i) => `h${i}`).join(", ")}];\n`);
await build({
  entryPoints: [join(here, "harness.tsx")],
  bundle: true, minify: true, outdir: dist, entryNames: "harness", format: "iife",
  loader: { ".woff2": "file", ".ttf": "file" }, assetNames: "[name]", logLevel: "warning",
  nodePaths: [join(ui, "node_modules"), join(frontend, "node_modules"), join(here, "../../../node_modules")],
  define: { "process.env.NODE_ENV": '"production"', "OMNICUI_APP": String(APP) },
  plugins: [{ name: "registry", setup: (b) => b.onResolve({ filter: /^\.\/registry\.gen$/ }, () => ({ path: registry })) }],
});
execFileSync(join(here, "../../../node_modules/.bin/tailwindcss"), ["-i", APP ? join(frontend, "src/styles.css") : join(ui, "src/styles/index.css"), "-o", join(dist, "harness.css"), "--minify"], { stdio: ["ignore", "ignore", "inherit"] });
writeFileSync(join(dist, "harness.html"), readFileSync(join(here, "harness.html")));
// Media the contracts name (fixtures/photo.png…) are served from ./fixtures; fixtures/missing.* is a 404.
mkdirSync(join(dist, "fixtures"), { recursive: true });
for (const f of readdirSync(join(here, "fixtures"))) copyFileSync(join(here, "fixtures", f), join(dist, "fixtures", f));
// The icon font sits next to the styles (packages/ui/scripts/fonts.mjs); the CSS asks for ./fonts/.
mkdirSync(join(dist, "fonts"), { recursive: true });
for (const f of readdirSync(join(ui, "src/styles/fonts"))) copyFileSync(join(ui, "src/styles/fonts", f), join(dist, "fonts", f));
const TYPES: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2", ".png": "image/png", ".svg": "image/svg+xml", ".mp4": "video/mp4", ".mp3": "audio/mpeg", ".pdf": "application/pdf" };
const api = APP ? await startApi() : null;
const server = createServer((req, res) => {
  if (api && /^\/(api|media)\//.test(req.url ?? "")) return proxy(req, res, api);
  // fixtures/held-open.*: the response never comes, for a loading state that lasts.
  if (/\/fixtures\/held-open\./.test(req.url ?? "")) return;
  const path = join(dist, new URL(req.url ?? "/", "http://x").pathname);
  if (!path.startsWith(dist) || !existsSync(path)) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { "content-type": TYPES[path.slice(path.lastIndexOf("."))] ?? "application/octet-stream" });
  res.end(readFileSync(path));
});
await new Promise<void>((ok) => server.listen(0, "127.0.0.1", ok));
const port = (server.address() as { port: number }).port;

const browser = await chromium.launch({ headless: true });
// If Chromium dies (a crashed GPU process, a broken install), every later request would fail one
// by one as if the components were wrong. Exit instead: omnicui then reports this target as
// broken, once, with these lines from standard error.
browser.on("disconnected", () => {
  log("Chromium has disconnected (it crashed or was closed); the adapter stops so that omnicui reports this target as broken");
  process.exit(3);
});
let page: Page;
let cdp: CDPSession;

async function freshPage(viewport: { width: number; height: number }, colorScheme: "light" | "dark") {
  await page?.context().close();
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1, colorScheme, reducedMotion: "no-preference" });
  page = await context.newPage();
  await page.addInitScript(`globalThis.__OMNICUI_DEFECTS__ = ${JSON.stringify(process.env.OMNICUI_DEFECTS ?? "")};`);
  page.on("pageerror", (e) => log("page error:", e.message));
  await page.goto(`http://127.0.0.1:${port}/harness.html`);
  cdp = await context.newCDPSession(page);
}

/**
 * Waits until nothing is animating, fonts are loaded, the app is not busy, and two more frames
 * have been painted. Busy: the app may define `window.omnicuiBusy = () => boolean`, true while it
 * waits for something outside the page (a network reply, say); it is waited for, up to 10 s.
 */
async function settle() {
  await page.evaluate(async () => {
    await document.fonts.ready;
    const busy = (window as any).omnicuiBusy as (() => boolean) | undefined;
    const busyUntil = performance.now() + 10000;
    while (busy?.()) {
      if (performance.now() > busyUntil) throw new Error("the app was still busy after 10 s (window.omnicuiBusy)");
      await new Promise((r) => setTimeout(r, 10));
    }
    const deadline = performance.now() + 5000;
    while (performance.now() < deadline) {
      // An endless animation (a spinner) never finishes; waiting on it would only use up the deadline.
      const running = document.getAnimations().filter((a) => a.playState === "running" && a.effect?.getTiming().iterations !== Infinity);
      if (running.length === 0) break;
      await Promise.race([Promise.all(running.map((a) => a.finished.catch(() => null))), new Promise((r) => setTimeout(r, 100))]);
    }
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
  });
}

// A part ("ui.button.label") is the element whose data-part is its full name or, shadcn's
// convention, whose data-slot is its own name ("label"). data-part settles a clash with a nested widget.
const quote = (s: string) => s.replace(/"/g, '\\"');
const selector = (part: string) => {
  const own = part.slice(part.lastIndexOf(".") + 1);
  return `[data-part="${quote(part)}"], [data-slot="${quote(own)}"]`;
};

async function locate(part: string) {
  const loc = page.locator(selector(part));
  if ((await loc.count()) === 0) throw new RpcError(NO_PART, `no part ${part}`);
  return loc.first();
}

/** The accessibility node Chromium computes for a part: its role, name, states and value. */
async function axNode(part: string) {
  await locate(part);
  const { root } = await cdp.send("DOM.getDocument", { depth: 0 });
  const { nodeId } = await cdp.send("DOM.querySelector", { nodeId: root.nodeId, selector: selector(part) });
  const { nodes } = await cdp.send("Accessibility.getPartialAXTree", { nodeId, fetchRelatives: false });
  return nodes[0];
}

const prop = (node: any, name: string) => node?.properties?.find((p: any) => p.name === name)?.value?.value;

// protocol: Space is sent as " " to Playwright; every other key name is the same.
const playwrightKey = (key: string) => key.split("+").map((k) => (k === "Space" ? " " : k)).join("+");

type Params = Record<string, any>;

async function step(method: string, p: Params, settled = true) {
  switch (method) {
    // A disabled control is still pressed where a user would press it, by the mouse: Playwright's
    // click() would wait for it to become enabled instead.
    case "press": {
      const loc = await locate(p.part);
      for (let i = 0; i < (p.times ?? 1); i++) {
        if (await loc.isDisabled()) {
          const box = await loc.boundingBox();
          if (box) await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
        } else await loc.click();
      }
      break;
    }
    case "hover": await (await locate(p.part)).hover(); break;
    case "focus": await (await locate(p.part)).focus(); break;
    // The key goes wherever focus really is, so a part that can't be focused fails honestly.
    // A key with no part goes to whatever has focus.
    case "key": if (p.part !== undefined) await locate(p.part); await page.keyboard.press(playwrightKey(p.key)); break;
    // Focus first, as a user tabs or clicks in: Playwright's pressSequentially on a field that
    // isn't focused puts the caret at the start itself, which no user's focus does. Focused, the
    // field keeps the caret where its own focus put it (the end, in this app).
    case "type": { const field = await locate(p.part); await field.focus(); await field.pressSequentially(p.text); break; }
    case "drag": {
      const box = await (await locate(p.part)).boundingBox();
      if (!box) throw new RpcError(STEP_FAILED, `${p.part} is not visible`);
      const x = box.x + box.width / 2, y = box.y + box.height / 2;
      await page.mouse.move(x, y);
      await page.mouse.down();
      await page.mouse.move(x + p.dx, y + p.dy, { steps: 10 });
      await page.mouse.up();
      break;
    }
    default: throw new RpcError(NO_METHOD, `unknown step ${method}`);
  }
  if (settled) await settle();
}

const methods: Record<string, (p: Params) => Promise<unknown>> = {
  // protocol: the first request; which constructs this target can perform on this run.
  async advertise() {
    return {
      protocol: "1",
      can: ["mount", "press", "focus", "key", "type", "drag", "hover", "read", "events", "screenshot", "trace", "trace.opacity", "trace.scale", "record", "probe", "bounds", "role", "focused", "selected", "do.back", "do.contextMenu", "do.togglePlayback"],
      cannot: {
        "do.scroll": "not implemented in this target yet",
        "do.swipe": "a desktop browser has no swipe",
        "do.refresh": "the app has no pull to refresh",
        "do.pick": "the app has no picker",
      },
      // The web runs on whatever the host is; here, the OS this adapter runs on.
      platform: { os: ({ darwin: "macos", win32: "windows" } as Record<string, string>)[process.platform] ?? process.platform },
    };
  },
  // protocol: the render check's picture: white, a black 32×32 square at 8,8, a red one 8 px from
  // the bottom-right corner, at the viewport asked for.
  async probe(p) {
    await freshPage(p.viewport, "light");
    await page.setContent(`<!doctype html><body style="margin:0;background:#fff">
      <div style="position:fixed;left:8px;top:8px;width:32px;height:32px;background:#000"></div>
      <div style="position:fixed;right:8px;bottom:8px;width:32px;height:32px;background:#f00"></div></body>`);
    mkdirSync(dirname(p.path), { recursive: true });
    await page.screenshot({ path: p.path });
    return p.path;
  },
  // protocol: where a part is on screen, in CSS pixels; a part that is not rendered is 0×0.
  async bounds(p) {
    const box = await (await locate(p.part)).boundingBox();
    return box ? { x: box.x, y: box.y, width: box.width, height: box.height } : { x: 0, y: 0, width: 0, height: 0 };
  },
  // contract: tests[].props, viewports, colorSchemes. A fresh page for every mount.
  async mount(p) {
    await freshPage(p.viewport, p.colorScheme);
    const err = await page.evaluate(([c, props]) => (window as any).omnicuiMount(c, props), [p.component, p.props ?? {}] as const);
    if (err) throw new RpcError(NO_COMPONENT, err);
    await settle();
    return null;
  },
  // protocol: do. back is the browser's back button (history.back(), which the app's History
  // API routing answers with popstate); contextMenu is a right click; togglePlayback plays or pauses
  // the part's video or audio.
  async do(p) {
    if (p.name === "back") await page.goBack({ waitUntil: "commit" });
    // contextMenu: a right click on the part, as a mouse user opens a context menu.
    else if (p.name === "contextMenu") await (await locate(p.part)).click({ button: "right" });
    // togglePlayback: what the media element's own play/pause control does.
    else if (p.name === "togglePlayback") await (await locate(p.part)).evaluate(async (el) => {
      const m = (el.matches("video, audio") ? el : el.querySelector("video, audio")) as HTMLMediaElement | null;
      if (!m) throw new Error("no video or audio element in this part");
      if (m.paused) await m.play().catch(() => {}); else m.pause();
    });
    else throw new RpcError(NO_METHOD, `do.${p.name} is not something this target does`);
    await settle();
    return null;
  },
  press: (p) => step("press", p),
  hover: (p) => step("hover", p),
  focus: (p) => step("focus", p),
  key: (p) => step("key", p),
  type: (p) => step("type", p),
  drag: (p) => step("drag", p),
  // contract: expect[] with part:. text is what is shown; the rest is what assistive tech sees.
  async read(p) {
    const loc = await locate(p.part);
    // A text box's text is what is typed in it, as egui (AccessKit's value) and JavaFX (getText) read it.
    // `text` is what the part shows. innerText also has text only a screen reader gets (visually
    // hidden: a 1×1 box that clips its content), which is left out here.
    if (p.field === "text") return loc.evaluate((el) => {
      if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) return el.value;
      const hidden = [...el.querySelectorAll<HTMLElement>("*")].filter((d) => {
        const r = d.getBoundingClientRect(), s = getComputedStyle(d);
        return r.width <= 1 && r.height <= 1 && (s.overflow === "hidden" || s.clip !== "auto" || s.clipPath !== "none");
      });
      const saved = hidden.map((d) => d.style.display);
      hidden.forEach((d) => (d.style.display = "none"));
      const text = (el as HTMLElement).innerText.trim();
      hidden.forEach((d, i) => (d.style.display = saved[i]));
      return text;
    });
    if (p.field === "focused") return loc.evaluate((el) => el === document.activeElement);
    // A chosen chip, tab or segment: aria-pressed, aria-selected or aria-checked on a button.
    if (p.field === "selected") return loc.evaluate((el) => ["aria-pressed", "aria-selected", "aria-checked"].some((a) => el.getAttribute(a) === "true"));
    const node = await axNode(p.part);
    switch (p.field) {
      case "name": return node?.name?.value ?? null;
      case "checked": { const v = prop(node, "checked"); return v === undefined ? null : v === "true" || v === true; }
      case "disabled": return prop(node, "disabled") === true;
      case "value": { const v = node?.value?.value ?? prop(node, "valuenow"); return v === undefined ? null : Number(v); }
      // Chromium's role names are WAI-ARIA's, except a few of its own.
      case "role": { const r = String(node?.role?.value ?? ""); return ({ StaticText: "text", none: "generic", LabelText: "label" } as Record<string, string>)[r] ?? r.toLowerCase(); }
    }
    throw new RpcError(STEP_FAILED, `unknown field ${p.field}`);
  },
  // contract: expect[] with event:
  async events() {
    return page.evaluate(() => (window as any).omnicuiEvents());
  },
  // contract: screenshot: true. The path is chosen by omnicui.
  async screenshot(p) {
    await settle();
    mkdirSync(dirname(p.path), { recursive: true });
    await page.screenshot({ path: p.path, caret: "hide" });
    return p.path;
  },
  // contract: trace[]. Samples the part's box on every animation frame while the steps run; or,
  // with property: opacity, its opacity as drawn (its own times its ancestors'); or, with
  // property: scale, its scale as drawn (its width on screen over its layout width). The part is
  // looked up at each frame, so one that appears during the steps (a sheet, a dialog) is traced
  // from its first frame on screen.
  async trace(p) {
    await page.evaluate(([sel, point, property]) => {
      const t = ((window as any).__trace = { samples: [] as Record<string, number>[], on: true, start: performance.now() });
      const opacity = (el: Element) => {
        let v = 1;
        for (let n: Element | null = el; n; n = n.parentElement) v *= Number(getComputedStyle(n).opacity);
        return v;
      };
      const frame = (now: number) => {
        const el = document.querySelector(sel) as HTMLElement | null;
        const ms = now - t.start;
        if (el && property === "opacity") {
          t.samples.push({ ms, x: 0, y: 0, value: opacity(el) });
        } else if (el && property === "scale") {
          const w = el.offsetWidth;
          if (w > 0) t.samples.push({ ms, x: 0, y: 0, value: el.getBoundingClientRect().width / w });
        } else if (el) {
          const r = el.getBoundingClientRect();
          t.samples.push({ ms, x: point === "center" ? r.left + r.width / 2 : r.left, y: point === "center" ? r.top + r.height / 2 : r.top });
        }
        if (t.on) requestAnimationFrame(frame);
      };
      requestAnimationFrame(frame);
    }, [selector(p.part), p.point, p.property ?? "position"] as const);
    for (const s of p.steps) await step(s.method, s);
    await page.waitForTimeout(150);
    const samples: Record<string, number>[] = await page.evaluate(() => {
      const t = (window as any).__trace;
      t.on = false;
      return t.samples;
    });
    if (samples.length === 0) throw new RpcError(NO_PART, `no part ${p.part} (not on screen at any frame)`);
    const t0 = samples[0].ms;
    return samples.map((s) => ({ ...s, ms: s.ms - t0 }));
  },
  // contract: record: true. A clip on the page's own animation clock, frame by frame: right after
  // each step every animation it started (CSS transitions and animations, Web Animations) is held
  // at its start, then moved on 16 ms at a time, with a screenshot at each. So a 150 ms transition
  // gives ten frames whatever the machine's load. (Chromium's screencast, which the reference
  // web target records with, sent frames late or coalesced them here, and a 150 ms switch came out
  // as one jump 600 ms after the press: GAPS-todo.txt.) held: false, a frame for every 16 ms.
  async record(p) {
    mkdirSync(p.path, { recursive: true });
    const frames: { ms: number; file: string }[] = [];
    const inputs: number[] = [];
    let ms = 0;
    const shot = async () => {
      const file = join(p.path, `${frames.length}.png`);
      await page.screenshot({ path: file, caret: "hide" });
      frames.push({ ms, file });
      ms += 16;
    };
    for (let i = 0; i < 4; i++) await shot();
    for (const s of p.steps) {
      inputs.push(ms);
      await step(s.method, s, false);
      // Hold what the step started at its start; getAnimations() flushes style, so transitions
      // that the step's changes start exist by now. Returns how long the longest one runs.
      const length: number = await page.evaluate(() => {
        let end = 0;
        for (const a of document.getAnimations()) {
          a.pause();
          a.currentTime = 0;
          const t = a.effect?.getComputedTiming();
          end = Math.max(end, Number(t?.endTime ?? 0));
        }
        return Number.isFinite(end) ? Math.min(end, 5000) : 1000;
      });
      for (let t = 0; t <= length + 32; t += 16) {
        await page.evaluate((at) => document.getAnimations().forEach((a) => { a.currentTime = at; }), t);
        await shot();
      }
      await page.evaluate(() => document.getAnimations().forEach((a) => a.finish()));
      await settle();
    }
    for (let i = 0; i < 4; i++) await shot();
    return { frames, inputs, held: false };
  },
};

// --- the conversation -------------------------------------------------------------------------
const send = (msg: object) => process.stdout.write(JSON.stringify({ jsonrpc: "2.0", ...msg }) + "\n");
let queue: Promise<unknown> = Promise.resolve();
createInterface({ input: process.stdin }).on("line", (line) => {
  queue = queue.then(async () => {
    let req: { id?: number; method?: string; params?: Params };
    try {
      req = JSON.parse(line);
    } catch {
      return send({ id: null, error: { code: -32700, message: "parse error" } });
    }
    const fn = req.method ? methods[req.method] : undefined;
    if (!fn) return send({ id: req.id, error: { code: NO_METHOD, message: `unknown method ${req.method}` } });
    try {
      send({ id: req.id, result: (await fn(req.params ?? {})) ?? null });
    } catch (e: any) {
      const code = e instanceof RpcError ? e.code : STEP_FAILED;
      log(req.method, "failed:", e?.message ?? e);
      send({ id: req.id, error: { code, message: String(e?.message ?? e).split("\n")[0] } });
    }
  });
}).on("close", async () => {
  await queue;
  await browser.close();
  server.close();
  process.exit(0);
});
log("ready on port", port);

// --- the API behind the screens (app mode) -------------------------------------------------------
// The screens talk to the real Django API, on a database of this run's own that is dropped on exit,
// with DEMO_RESET on so every mount starts from a demo data set (props.dataset, default standard).
// Needs SCREENS_DATABASE_URL (a Postgres server, e.g. postgres://postgres@127.0.0.1:5432) and,
// unless python3 is the right interpreter, TASKSCAPE_PYTHON.
async function startApi(): Promise<number> {
  const backend = join(here, "../../../apps/web/backend");
  const python = process.env.TASKSCAPE_PYTHON ?? "python3";
  const server = process.env.SCREENS_DATABASE_URL?.replace(/\/+$/, "");
  if (!server) throw new Error("SCREENS_DATABASE_URL is not set: the screens need a Postgres server for their API");
  const db = `taskscape_screens_${process.pid}`;
  const admin = (sql: string) => execFileSync(python, ["-c",
    `import psycopg, sys; c = psycopg.connect(sys.argv[1], autocommit=True); c.execute(sys.argv[2])`, `${server}/postgres`, sql], { stdio: ["ignore", "ignore", "inherit"] });
  admin(`DROP DATABASE IF EXISTS ${db}`);
  admin(`CREATE DATABASE ${db}`);
  const env = { ...process.env, DATABASE_URL: `${server}/${db}`, DEBUG: "1", DEMO_RESET: "1", MEDIA_ROOT: join(dist, "media") };
  execFileSync(python, ["manage.py", "migrate", "--noinput"], { cwd: backend, env, stdio: ["ignore", "ignore", "inherit"] });
  const port = await new Promise<number>((ok) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = (s.address() as { port: number }).port; s.close(() => ok(p)); }); });
  const django = spawn(python, ["manage.py", "runserver", `127.0.0.1:${port}`, "--noreload", "--nothreading"], { cwd: backend, env, stdio: ["ignore", "ignore", "pipe"] });
  django.stderr.on("data", (d) => { const l = String(d).trim(); if (/error|exception|traceback/i.test(l)) log("api:", l.slice(0, 2000)); });
  process.on("exit", () => {
    django.kill();
    try { admin(`DROP DATABASE IF EXISTS ${db} WITH (FORCE)`); } catch { /* the server may already be gone */ }
  });
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(`http://127.0.0.1:${port}/api/auth/csrf`)).ok) return port; } catch { /* not up yet */ }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error("the API did not start in 30 s");
}

function proxy(req: IncomingMessage, res: ServerResponse, port: number) {
  const out = httpRequest({ host: "127.0.0.1", port, path: req.url, method: req.method, headers: req.headers }, (r) => {
    res.writeHead(r.statusCode ?? 502, r.headers);
    r.pipe(res);
  });
  out.on("error", (e) => { res.writeHead(502); res.end(String(e)); });
  req.pipe(out);
}
