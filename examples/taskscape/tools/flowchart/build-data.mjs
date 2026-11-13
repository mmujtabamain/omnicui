// Builds public/data.json and public/seed-comments.json from spec/, and public/refs/*.webp from
// the recorded screenshots (spec/references, which must be captured first).
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const spec = path.resolve(here, '../../spec');
const pub = path.join(here, 'public');
const read = (p) => JSON.parse(fs.readFileSync(path.join(spec, p), 'utf8'));

// Which feature areas each screen draws on.
const AREAS = {
  'main-window': [9, 8, 11, 15], 'title-bar': [2, 9], 'task-list': [1, 7, 8], 'task-inspector': [3, 4],
  'draft-inspector': [4], 'multi-inspector': [4, 7], 'filter-panel': [5], 'command-palette': [6, 8],
  'context-menus': [6, 7], sheets: [6, 9], toast: [6, 7], trash: [7], 'attachment-lightbox': [4],
  'settings-general': [10, 11], 'settings-shortcuts': [10, 8], 'settings-capture': [10, 13],
  'settings-about': [10], 'capture-bar': [13, 14],
};

const areas = Object.fromEntries(
  fs.readdirSync(path.join(spec, 'features')).filter((f) => /^area-\d+\.json$/.test(f)).map((f) => {
    const a = read(`features/${f}`);
    return [a.area, { n: a.area, name: a.name, features: a.features.map((x) => ({ id: x.id, title: x.title })) }];
  })
);

const inv = read('inventory/screens.json');
const screens = inv.screens.map((s) => ({
  id: s.id,
  title: s.title,
  tier: s.tier,
  purpose: s.purpose,
  regions: s.regions,
  states: s.states,
  thumb: `refs/${s.states[0]}.webp`,
  widgets: s.widgets.map((w, i) => ({ key: `${s.id}-${i}`, ...w })),
  links: s.links,
  areas: (AREAS[s.id] ?? []).map((n) => areas[n]).filter(Boolean).map(({ n, name, features }) => ({ n, name, count: features.length })),
}));
const catalogue = read('catalogue.json');
fs.mkdirSync(pub, { recursive: true });
const familyOf = {};
for (const f of catalogue) for (const k of f.kinds) familyOf[k] = f.kind;
fs.writeFileSync(path.join(pub, 'data.json'), JSON.stringify({ screens, catalogue, areas, familyOf }));
const seed = fs.existsSync(path.join(spec, 'flow-comments.json')) ? read('flow-comments.json') : [];
fs.writeFileSync(path.join(pub, 'seed-comments.json'), JSON.stringify(seed));

// Screenshots as WebP, drawn in Chromium so no image tooling is needed.
const { chromium } = await import(path.resolve(here, '../reference-viewer/node_modules/playwright/index.mjs'));
const refs = path.join(spec, 'references');
const out = path.join(pub, 'refs');
fs.mkdirSync(out, { recursive: true });
const wanted = [...new Set(screens.flatMap((s) => s.states))];
const browser = await chromium.launch();
const page = await browser.newPage();
for (const st of wanted) {
  const png = fs.readFileSync(path.join(refs, st, 'shot.png')).toString('base64');
  const webp = await page.evaluate(async (b64) => {
    const img = new Image();
    img.src = `data:image/png;base64,${b64}`;
    await img.decode();
    const c = document.createElement('canvas');
    c.width = img.width;
    c.height = img.height;
    c.getContext('2d').drawImage(img, 0, 0);
    return c.toDataURL('image/webp', 0.82).split(',')[1];
  }, png);
  fs.writeFileSync(path.join(out, `${st}.webp`), Buffer.from(webp, 'base64'));
}
await browser.close();
console.log(`${screens.length} screens, ${catalogue.length} widget kinds, ${seed.length} seed comments, ${wanted.length} screenshots`);
