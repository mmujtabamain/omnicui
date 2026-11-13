// The tauri target's adapter. omnicui starts it once per check (`adapter:` in target.yaml) and
// sends JSON-RPC 2.0 requests, one per line, on standard input; every reply is one line on
// standard output. Logs go to standard error only, because anything else on standard output
// breaks the conversation.
//
// Taskscape is a Tauri app whose frontend runs in macOS's WKWebView, which has no WebDriver. So
// this target runs the app's real frontend (built once by setup.sh with the app's own Vite and
// Tailwind: build/dist/) in Chromium through Playwright, with the page's user agent saying
// macOS, as WKWebView's does, so that the app takes its macOS paths (⌘ shortcuts, the window
// controls on the left). Every Tauri command goes, through Tauri's own mockIPC (harness.ts), to
// server/: a Rust server that answers it with the app's taskscape_common Store on a fresh
// database the fixture loader (../../fixtures) writes for every mount. Each reply is sent only
// once the page has settled: nothing animating, fonts loaded, no command in flight, two frames
// painted.
import { spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, existsSync } from "node:fs";
import { createInterface } from "node:readline";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Page, type CDPSession, type ElementHandle } from "playwright";

const here = dirname(fileURLToPath(import.meta.url));
const log = (...a: unknown[]) => console.error("[tauri]", ...a);
const started = performance.now();

// protocol: error codes
const NO_COMPONENT = -32001, NO_PART = -32002, STEP_FAILED = -32003, NO_METHOD = -32601;
class RpcError extends Error {
  code: number;
  constructor(code: number, message: string) {
    super(message);
    this.code = code;
  }
}

// --- the app's backend: the Rust server over the real store ---------------------------------
const dist = join(here, "build", "dist");
const serverBin = join(here, "build", "cargo", "release", "taskscape-tauri-target-server");
for (const f of [join(dist, "main", "index.html"), join(dist, "tray", "index.html"), serverBin])
  if (!existsSync(f)) {
    log(`${f} is missing: run ./setup.sh in this folder first (it builds the harness and the server once)`);
    process.exit(1);
  }
const data = mkdtempSync(join(tmpdir(), "omnicui-tauri-"));
const server = spawn(serverBin, [dist, data], { stdio: ["ignore", "pipe", "inherit"] });
const port: number = await new Promise((ok, fail) => {
  createInterface({ input: server.stdout! }).once("line", (line) => {
    const m = /^listening (\d+)$/.exec(line);
    if (m) ok(Number(m[1]));
    else fail(new Error(`the server said ${line}`));
  });
  server.once("exit", (code) => fail(new Error(`the server exited with ${code}`)));
});
// The server goes with the adapter, however the adapter ends.
process.on("exit", () => server.kill());
server.on("exit", (code) => {
  log(`the store server exited (${code}); the adapter stops so that omnicui reports this target as broken`);
  process.exit(3);
});

const browser = await chromium.launch({ headless: true });
// If Chromium dies, every later request would fail one by one as if the components were wrong.
// Exit instead: omnicui then reports this target as broken, once, with these lines.
browser.on("disconnected", () => {
  log("Chromium has disconnected (it crashed or was closed); the adapter stops so that omnicui reports this target as broken");
  process.exit(3);
});
// WKWebView's user agent on macOS; the app reads only "Mac" from it (lib/platform.ts).
const MAC_UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)";
let page: Page;
let cdp: CDPSession;
let mounted = "";

async function freshPage(viewport: { width: number; height: number }, colorScheme: "light" | "dark", surface?: "main" | "tray") {
  await page?.context().close();
  const context = await browser.newContext({
    viewport, deviceScaleFactor: 1, colorScheme, reducedMotion: "no-preference", userAgent: MAC_UA,
    permissions: ["clipboard-read", "clipboard-write"],
  });
  page = await context.newPage();
  page.on("pageerror", (e) => log("page error:", e.message));
  page.on("console", (m) => { if (m.type() === "error") log("console:", m.text()); });
  if (surface) await page.goto(`http://127.0.0.1:${port}/${surface}/index.html`);
  cdp = await context.newCDPSession(page);
}

/**
 * Waits until nothing is animating, fonts are loaded, no Tauri command is in flight
 * (`window.omnicuiBusy`, harness.ts), and two more frames have been painted.
 */
async function settle() {
  await page.evaluate(async () => {
    await document.fonts.ready;
    const busy = (window as any).omnicuiBusy as (() => boolean) | undefined;
    const busyUntil = performance.now() + 10000;
    // A command often starts one task after the input that caused it: look twice.
    for (let quiet = 0; quiet < 2; ) {
      if (busy?.()) {
        quiet = 0;
        if (performance.now() > busyUntil) throw new Error("the app was still busy after 10 s (a Tauri command in flight)");
      } else quiet++;
      await new Promise((r) => setTimeout(r, 10));
    }
    const deadline = performance.now() + 5000;
    while (performance.now() < deadline) {
      const running = document.getAnimations().filter((a) => a.playState === "running" && a.effect?.getComputedTiming().endTime !== Infinity);
      if (running.length === 0) break;
      await Promise.race([Promise.all(running.map((a) => a.finished.catch(() => null))), new Promise((r) => setTimeout(r, 100))]);
    }
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
  });
}

// Parts. The app tags an element `data-part="<name>"` (the contract's part name, without the
// component's id: names are unique across the product, SPEC-PRODUCT.md). A part that repeats,
// a row or a tab, is tagged by its kind (`data-part="row"`), and its pieces by the kind and the
// piece (`data-part="rowCheck"`): the contract's `row2Check` is the second row's check, in the
// order they are on the page. This lookup runs in the page, so traces find a part every frame.
const FIND = String.raw`(full) => {
  const name = full.replace(/^taskscape\.[a-zA-Z]+\./, "");
  const q = (root, n) => root.querySelector('[data-part="' + n + '"]');
  const own = q(document, name);
  if (own) return own;
  const m = /^([a-z][a-zA-Z]*?)(\d+)([A-Z][a-zA-Z]*)?$/.exec(name);
  if (!m) return null;
  const item = document.querySelectorAll('[data-part="' + m[1] + '"]')[Number(m[2]) - 1];
  if (!item || !m[3]) return item ?? null;
  return q(item, m[1] + m[3]);
}`;

async function locate(part: string): Promise<ElementHandle<Element>> {
  const handle = await page.evaluateHandle(([find, p]) => (0, eval)(find)(p), [FIND, part] as const);
  const el = handle.asElement();
  if (!el) throw new RpcError(NO_PART, `no part ${part}`);
  return el as ElementHandle<Element>;
}

/** The accessibility node Chromium computes for a part: its role, name, states and value. */
async function axNode(part: string) {
  const el = await locate(part);
  const { node } = await cdp.send("DOM.describeNode", { objectId: await objectId(el) });
  const { nodes } = await cdp.send("Accessibility.getPartialAXTree", { backendNodeId: node.backendNodeId, fetchRelatives: false });
  return nodes[0];
}

async function objectId(el: ElementHandle<Element>): Promise<string> {
  // Tag it, then resolve it through CDP: Playwright's handles don't expose a CDP object id.
  const mark = `omnicui-${Math.random().toString(36).slice(2)}`;
  await el.evaluate((e, m) => e.setAttribute("data-omnicui-probe", m), mark);
  const { root } = await cdp.send("DOM.getDocument", { depth: 0 });
  const { nodeId } = await cdp.send("DOM.querySelector", { nodeId: root.nodeId, selector: `[data-omnicui-probe="${mark}"]` });
  await el.evaluate((e) => e.removeAttribute("data-omnicui-probe"));
  const { object } = await cdp.send("DOM.resolveNode", { nodeId });
  return object.objectId!;
}

const prop = (node: any, name: string) => node?.properties?.find((p: any) => p.name === name)?.value?.value;

/**
 * The window chrome in this screenshot that a native window would draw itself: Taskscape hides
 * macOS's title bar and draws its own (data-part="titlebar", "settingsTitlebar") with its own
 * traffic lights ("windowControls"). Where the bar is the window's whole top edge (the main window,
 * Settings) it is reported whole, and omnicui cuts it off before comparing; on the title bar
 * mounted on its own, only the window controls are, and omnicui masks them.
 */
async function titlebarRegions() {
  const box = async (part: string) => {
    const el = await page.$(`[data-part="${part}"]`);
    const b = await el?.boundingBox();
    return b && b.width > 0 ? { kind: "titlebar", x: Math.round(b.x), y: Math.round(b.y), width: Math.round(b.width), height: Math.round(b.height) } : null;
  };
  const whole = mounted === "taskscape.window" ? await box("titlebar") : mounted === "taskscape.settings" ? await box("settingsTitlebar") : null;
  const regions = whole ? [whole] : [await box("windowControls")];
  return regions.filter((r) => r !== null);
}

// protocol: Space is sent as " " to Playwright; every other key name is the same.
const playwrightKey = (key: string) => key.split("+").map((k) => (k === "Space" ? " " : k)).join("+");

type Params = Record<string, any>;

async function step(method: string, p: Params, settled = true) {
  switch (method) {
    case "press": await (await locate(p.part)).click(); break;
    case "hover": await (await locate(p.part)).hover(); break;
    case "focus": await (await locate(p.part)).focus(); break;
    // The key goes wherever focus really is, so a part that can't be focused fails honestly.
    // A key with no part goes to whatever has focus.
    case "key": if (p.part !== undefined) await locate(p.part); await page.keyboard.press(playwrightKey(p.key)); break;
    // Focus first, as a user tabs or clicks in: Playwright's pressSequentially on a field that
    // isn't focused puts the caret at the start itself, which no user's focus does. Focused, the
    // field keeps the caret where its own focus put it (the end, in this app).
    case "type": { const field = await locate(p.part); await field.focus(); await page.keyboard.type(p.text); break; }
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
    log(`ready in ${Math.round(performance.now() - started)} ms (store server on port ${port}, Chromium)`);
    return {
      protocol: "1",
      can: ["mount", "press", "focus", "key", "type", "drag", "hover", "read", "events", "screenshot", "trace", "trace.opacity", "trace.scale", "record", "probe", "bounds", "role", "focused", "selected", "do.summon"],
      cannot: {
        "do.back": "a desktop window has no system back",
        "do.scroll": "not implemented in this target",
        "do.swipe": "a desktop window has no swipe",
        "do.refresh": "the app has no pull to refresh",
        "do.pick": "the app has no picker",
      },
      // The host this runs on; the page itself says macOS (MAC_UA), as the app only ships there.
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
    // The capture bar is the tray app's: its own bundle and stylesheet.
    await freshPage(p.viewport, p.colorScheme, p.component === "taskscape.capture" ? "tray" : "main");
    mounted = p.component;
    const err = await page.evaluate(([c, props]) => (window as any).omnicuiMount(c, props), [p.component, p.props ?? {}] as const);
    if (err) throw new RpcError(NO_COMPONENT, err);
    await settle();
    return null;
  },
  // protocol: do. `summon` is the tray's global shortcut (⌘↩), which the OS delivers to the tray
  // process, not the page: what Rust does then, the reveal, is done here instead.
  async do(p) {
    if (p.name !== "summon") throw new RpcError(NO_METHOD, `do.${p.name} is not something this target does`);
    await page.evaluate((n) => (window as any).omnicuiDo(n), p.name);
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
    // A text box's text is what is typed in it. Otherwise `text` is what the part shows:
    // innerText without what only a screen reader gets (visually hidden: a 1×1 box that clips its
    // content) and without icon glyphs, which Material Symbols draws from their ligature names
    // (`aria-hidden` spans: "more_horiz" is a glyph, not words on screen).
    if (p.field === "text") return loc.evaluate((el) => {
      if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) return el.value;
      const hidden = [...el.querySelectorAll<HTMLElement>("*")].filter((d) => {
        if (d.getAttribute("aria-hidden") === "true" || d.classList.contains("material-symbols-outlined")) return true;
        const r = d.getBoundingClientRect(), s = getComputedStyle(d);
        return r.width <= 1 && r.height <= 1 && (s.overflow === "hidden" || s.clip !== "auto" || s.clipPath !== "none");
      });
      const saved = hidden.map((d) => d.style.display);
      hidden.forEach((d) => (d.style.display = "none"));
      // A flex row of spans ("1", "/", "3") comes out of innerText a line each; what is on
      // screen is one line, so line breaks between the pieces become one space.
      const text = (el as HTMLElement).innerText.replace(/\s*\n\s*/g, " ").trim();
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
    return { path: p.path, regions: await titlebarRegions() };
  },
  // contract: trace[]. Samples the part's box on every animation frame while the steps run; or,
  // with property: opacity, its opacity as drawn (its own times its ancestors'); or, with
  // property: scale, its scale as drawn (its width on screen over its layout width). The part is
  // looked up at each frame, so one that appears during the steps (a sheet, a dialog) is traced
  // from its first frame on screen.
  async trace(p) {
    await page.evaluate(([sel, part, point, property]) => {
      const t = ((window as any).__trace = { samples: [] as Record<string, number>[], on: true, start: performance.now() });
      const opacity = (el: Element) => {
        let v = 1;
        for (let n: Element | null = el; n; n = n.parentElement) v *= Number(getComputedStyle(n).opacity);
        return v;
      };
      const find = (0, eval)(sel);
      const frame = (now: number) => {
        const el = find(part) as HTMLElement | null;
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
    }, [FIND, p.part, p.point, p.property ?? "position"] as const);
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
  // as one jump 600 ms after the press: examples/todo/GAPS-todo.txt.) held: false, a frame for every 16 ms.
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
  server.removeAllListeners("exit");
  server.kill();
  rmSync(data, { recursive: true, force: true });
  process.exit(0);
});
log("ready on port", port);
