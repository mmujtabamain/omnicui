// The web target's adapter. omnicui starts it once per check (`adapter:` in target.yaml) and sends
// JSON-RPC 2.0 requests, one per line, on standard input; every reply is one line on standard
// output. Logs go to standard error only, because anything else on standard output breaks the
// conversation.
//
// It builds the app's components (apps/web, imported as the package web-app) for production with esbuild, serves the bundle from a local HTTP
// server, and performs every step in a real Chromium through Playwright. Each reply is sent
// only once the page has settled: no running animations, fonts loaded, the app not busy (see
// settle()), two frames painted.
import { createServer } from "node:http";
import { readFileSync, mkdirSync, writeFileSync, realpathSync } from "node:fs";
import { createInterface } from "node:readline";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";
import { serveFile } from "web-app/static";
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
const dist = join(here, "dist");
// React comes from the app, so that its components run with the React they ship with, and only one.
const app = realpathSync(join(here, "node_modules", "web-app"));
const appReact = {
  name: "app-react",
  setup(b: import("esbuild").PluginBuild) {
    b.onResolve({ filter: /^react(-dom)?(\/.*)?$/ }, (args) =>
      args.pluginData === "app" ? undefined : b.resolve(args.path, { resolveDir: app, kind: args.kind, pluginData: "app" }));
  },
};
await build({
  entryPoints: [join(here, "harness.tsx")],
  plugins: [appReact],
  bundle: true, minify: true, outdir: dist, entryNames: "harness", format: "iife",
  loader: { ".ttf": "file" }, assetNames: "[name]", define: { "process.env.NODE_ENV": '"production"' }, logLevel: "warning",
});
writeFileSync(join(dist, "harness.html"), readFileSync(join(here, "harness.html")));
const server = createServer((req, res) => serveFile(res, dist, new URL(req.url ?? "/", "http://x").pathname));
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
      const running = document.getAnimations().filter((a) => a.playState === "running");
      if (running.length === 0) break;
      await Promise.race([Promise.all(running.map((a) => a.finished.catch(() => null))), new Promise((r) => setTimeout(r, 100))]);
    }
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
  });
}

const selector = (part: string) => `[data-part="${part.replace(/"/g, '\\"')}"]`;

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

async function step(method: string, p: Params) {
  switch (method) {
    case "press": await (await locate(p.part)).click(); break;
    case "hover": await (await locate(p.part)).hover(); break;
    case "focus": await (await locate(p.part)).focus(); break;
    // The key goes wherever focus really is, so a part that can't be focused fails honestly.
    case "key": await locate(p.part); await page.keyboard.press(playwrightKey(p.key)); break;
    case "type": await (await locate(p.part)).pressSequentially(p.text); break;
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
  await settle();
}

const methods: Record<string, (p: Params) => Promise<unknown>> = {
  // protocol: the first request; which constructs this target can perform on this run.
  async advertise() {
    return { protocol: "1", can: ["mount", "press", "focus", "key", "type", "drag", "hover", "read", "events", "screenshot", "trace", "trace.opacity", "trace.scale", "record", "probe", "bounds", "role", "focused", "selected"] };
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
  // contract: record: true. Chromium's screencast sends a frame whenever the page changes. The
  // reply says when each step's input was delivered, on the frames' clock (the screencast's
  // timestamps are wall-clock seconds), so that clips can be lined up at the input.
  async record(p) {
    mkdirSync(p.path, { recursive: true });
    const frames: { ms: number; file: string }[] = [];
    let first = -1;
    const writes: Promise<void>[] = [];
    const onFrame = (f: any) => {
      const ts = f.metadata.timestamp * 1000;
      if (first < 0) first = ts;
      const file = join(p.path, `${frames.length}.png`);
      frames.push({ ms: ts - first, file });
      writeFileSync(file, Buffer.from(f.data, "base64"));
      writes.push(cdp.send("Page.screencastFrameAck", { sessionId: f.sessionId }).then(() => undefined).catch(() => undefined));
    };
    cdp.on("Page.screencastFrame", onFrame);
    await cdp.send("Page.startScreencast", { format: "png", everyNthFrame: 1 });
    await page.waitForTimeout(150);
    const inputs: number[] = [];
    for (const s of p.steps) {
      inputs.push(first < 0 ? 0 : Date.now() - first);
      await step(s.method, s);
    }
    await page.waitForTimeout(300);
    await cdp.send("Page.stopScreencast");
    cdp.off("Page.screencastFrame", onFrame);
    await Promise.all(writes);
    // held: frames come only when the picture changes, so a gap means "unchanged until then".
    return { frames, inputs, held: true };
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
