// Records each state in states.mjs from the running viewer (serve.mjs):
//   spec/references/<state>/shot.png      the window
//   spec/references/<state>/tree.txt      React's component tree: name, props, box
//   spec/references/<state>/elements.json interactive and text elements: role, label, box, colours, type
//   spec/references/<state>/meta.json     the commands the app sent, and any step that failed
// usage: node capture.mjs [state-id ...]
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';
import { states } from './states.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const out = path.resolve(here, '../../spec/references');
const T = 1794301200000; // 2026-11-11 09:00 UTC: every capture reads the same "2d ago"
const MAC = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)';
const pages = {
  main: 'http://localhost:1420/viewer.html',
  settings: 'http://localhost:1420/viewer-settings.html',
  tray: 'http://localhost:1421/viewer.html',
};

function dumpTree() {
  const root = document.getElementById('root');
  const key = Object.keys(root).find((k) => k.startsWith('__reactContainer$'));
  const lines = [];
  const nameOf = (f) => {
    const t = f.type;
    if (!t || typeof t === 'string') return null;
    if (typeof t === 'function') return t.displayName || t.name || null;
    if (t.render) return t.displayName || t.render.displayName || t.render.name || null;
    if (t.type) return t.displayName || t.type.displayName || t.type.name || null;
    return null;
  };
  const val = (v) => {
    if (v == null) return null;
    if (typeof v === 'string') return JSON.stringify(v.length > 60 ? v.slice(0, 60) + '…' : v);
    if (typeof v === 'number' || typeof v === 'boolean') return String(v);
    if (typeof v === 'function') return 'ƒ';
    if (Array.isArray(v)) return `[${v.length}]`;
    if (v.$$typeof) return '<el>';
    if (typeof v === 'object' && v.id && v.title) return `{task ${JSON.stringify(v.title)}}`;
    if (typeof v === 'object' && v.id && v.name) return `{${JSON.stringify(v.name)}}`;
    return '{…}';
  };
  const propsOf = (p) =>
    Object.entries(p || {})
      .filter(([k]) => k !== 'children' && k !== 'key' && k !== 'ref')
      .map(([k, v]) => [k, val(v)])
      .filter(([, v]) => v !== null)
      .map(([k, v]) => `${k}=${v}`)
      .join(' ');
  const firstHost = (f) => {
    for (let c = f.child; c; c = c.child) if (c.stateNode instanceof Element) return c.stateNode;
    return null;
  };
  const boxOf = (el) => {
    if (!el) return '';
    const r = el.getBoundingClientRect();
    if (!r.width && !r.height) return ' @hidden';
    return ` @${Math.round(r.x)},${Math.round(r.y)} ${Math.round(r.width)}x${Math.round(r.height)}`;
  };
  const skip = /^(Provider|Consumer|Fragment|StrictMode|Suspense|Context|_|LazyMotion|MotionConfig|PresenceChild|PopChild)/;
  const walk = (f, depth) => {
    for (let c = f; c; c = c.sibling) {
      const n = nameOf(c);
      let d = depth;
      if (n && !skip.test(n)) {
        lines.push(`${'  '.repeat(depth)}${n} ${propsOf(c.memoizedProps)}${boxOf(firstHost(c))}`.trimEnd());
        d = depth + 1;
      }
      if (c.child) walk(c.child, d);
    }
  };
  // The container holds the HostRoot fiber; its stateNode.current is the committed tree
  // (the container's own fiber can be the stale alternate).
  walk(root[key].stateNode.current, 0);
  return lines.join('\n');
}

function dumpElements() {
  const sel = 'button, a, input, textarea, select, [role], [contenteditable="true"], [tabindex], h1, h2, h3, p, span, label, kbd';
  const out = [];
  for (const el of document.querySelectorAll(sel)) {
    const r = el.getBoundingClientRect();
    if (!r.width || !r.height) continue;
    const cs = getComputedStyle(el);
    if (cs.visibility === 'hidden' || Number(cs.opacity) === 0) continue;
    const own = [...el.childNodes].filter((n) => n.nodeType === 3).map((n) => n.textContent.trim()).join(' ').trim();
    const interactive = el.matches('button, a, input, textarea, select, [role], [contenteditable="true"], [tabindex]');
    if (!interactive && !own) continue;
    out.push({
      tag: el.tagName.toLowerCase(),
      role: el.getAttribute('role') ?? undefined,
      label: el.getAttribute('aria-label') ?? el.getAttribute('title') ?? undefined,
      text: (interactive ? el.innerText : own).trim().replace(/\s+/g, ' ').slice(0, 80) || undefined,
      state: ['aria-pressed', 'aria-selected', 'aria-checked', 'aria-expanded', 'disabled']
        .filter((a) => el.hasAttribute(a))
        .map((a) => `${a}=${el.getAttribute(a)}`)
        .join(' ') || undefined,
      box: [r.x, r.y, r.width, r.height].map(Math.round),
      color: cs.color,
      bg: cs.backgroundColor === 'rgba(0, 0, 0, 0)' ? undefined : cs.backgroundColor,
      font: `${cs.fontWeight} ${cs.fontSize}/${cs.lineHeight}`,
      radius: cs.borderRadius === '0px' ? undefined : cs.borderRadius,
      transform: cs.textTransform === 'none' ? undefined : cs.textTransform,
    });
  }
  return out;
}

async function capture(browser, s) {
  const dir = path.join(out, s.id);
  fs.mkdirSync(dir, { recursive: true });
  const ctx = await browser.newContext({
    viewport: s.viewport ?? (s.window === 'tray' ? { width: 760, height: 420 } : { width: 1180, height: 760 }),
    deviceScaleFactor: 1,
    userAgent: MAC,
    colorScheme: s.theme ?? 'light',
    reducedMotion: 'no-preference',
  });
  const page = await ctx.newPage();
  const problems = [];
  page.on('pageerror', (e) => problems.push(`pageerror: ${e.message}`));
  page.on('console', (m) => m.type() === 'warning' && m.text().includes('[viewer]') && problems.push(m.text()));
  await page.goto(`${pages[s.window ?? 'main']}?fixture=${s.fixture ?? 'standard'}&t=${T}`);
  if (s.window === 'tray') {
    await page.waitForTimeout(800);
    await page.evaluate(() => window.__viewer.emit('mini-shown', 'top'));
  }
  await page.waitForTimeout(s.window === 'main' || !s.window ? 2200 : 1200);
  try {
    await s.steps(page);
    await page.waitForTimeout(500);
  } catch (e) {
    problems.push(`step failed: ${e.message.split('\n')[0]}`);
  }
  await page.screenshot({ path: path.join(dir, 'shot.png') });
  fs.writeFileSync(path.join(dir, 'tree.txt'), await page.evaluate(dumpTree));
  fs.writeFileSync(path.join(dir, 'elements.json'), JSON.stringify(await page.evaluate(dumpElements)));
  const log = await page.evaluate(() => window.__viewer.log.map((l) => l.cmd));
  fs.writeFileSync(
    path.join(dir, 'meta.json'),
    JSON.stringify({ id: s.id, areas: s.areas, window: s.window ?? 'main', theme: s.theme ?? 'light', fixture: s.fixture ?? 'standard', commands: log, problems }, null, 1)
  );
  await ctx.close();
  return problems;
}

const only = process.argv.slice(2);
const todo = only.length ? states.filter((s) => only.includes(s.id)) : states;
const browser = await chromium.launch();
const queue = [...todo];
const failed = [];
await Promise.all(
  Array.from({ length: 4 }, async () => {
    for (let s; (s = queue.shift()); ) {
      const problems = await capture(browser, s);
      console.log(`${problems.length ? '!!' : 'ok'} ${s.id}${problems.length ? ': ' + problems.join(' | ') : ''}`);
      if (problems.length) failed.push(s.id);
    }
  })
);
await browser.close();
console.log(`${todo.length - failed.length}/${todo.length} clean`);
