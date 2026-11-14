// The screenshots of one widget's omnicui run on one sheet, to look at all its states at once:
//   node widget-sheet.mjs <component> <out.png> [light|dark]
// It reads omnicui/.omnicui/run/web/<component>/<test>/<viewport>-<scheme>.png.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = path.dirname(fileURLToPath(import.meta.url));
const [component, out, scheme = 'light'] = process.argv.slice(2);
const dir = path.resolve(here, '../../omnicui/.omnicui/run/web', component);
const shots = fs.readdirSync(dir).sort().flatMap((t) =>
  fs.readdirSync(path.join(dir, t)).filter((f) => f.endsWith(`-${scheme}.png`)).map((f) => [t, path.join(dir, t, f)]));
const cells = shots.map(([t, f]) =>
  `<figure><img src="data:image/png;base64,${fs.readFileSync(f).toString('base64')}"><figcaption>${t}</figcaption></figure>`).join('');
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 900 } });
await page.setContent(`<style>body{margin:8px;font:11px sans-serif;display:grid;grid-template-columns:repeat(6,1fr);gap:6px}
  figure{margin:0}img{width:100%;border:1px solid #ccc}figcaption{padding:2px 0;overflow-wrap:anywhere}</style>${cells}`);
await page.waitForTimeout(300);
await page.screenshot({ path: out, fullPage: true });
await browser.close();
console.log(`${shots.length} screenshots → ${out}`);
