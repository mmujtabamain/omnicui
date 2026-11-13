// One image per 24 states, for checking a capture at a glance: node contact-sheet.mjs <out-prefix>
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = path.dirname(fileURLToPath(import.meta.url));
const refs = path.resolve(here, '../../spec/references');
const ids = fs.readdirSync(refs).filter((d) => fs.existsSync(path.join(refs, d, 'shot.png'))).sort();
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1800, height: 1000 } });
for (let i = 0; i < ids.length; i += 24) {
  const cells = ids.slice(i, i + 24).map((id) =>
    `<figure><img src="data:image/png;base64,${fs.readFileSync(path.join(refs, id, 'shot.png')).toString('base64')}"><figcaption>${id}</figcaption></figure>`).join('');
  await page.setContent(`<style>body{margin:8px;font:13px sans-serif;display:grid;grid-template-columns:repeat(4,1fr);gap:8px}
    figure{margin:0}img{width:100%;border:1px solid #ccc}figcaption{padding:2px 0}</style>${cells}`);
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${process.argv[2]}-${i / 24 + 1}.png`, fullPage: true });
}
await browser.close();
