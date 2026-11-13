// Copies the reference app's frontend into a cache folder outside any repo, installs its
// own dependencies there, and adds the viewer's entry pages. The reference repo itself is
// only read.
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const ref = process.env.TASKSCAPE_REFERENCE;
if (!ref || !fs.existsSync(path.join(ref, 'common-ui'))) {
  console.error('Set TASKSCAPE_REFERENCE to the reference app repo.');
  process.exit(2);
}
export const copy =
  process.env.VIEWER_CACHE ?? path.join(os.homedir(), '.cache', 'taskscape-reference');

const parts = ['package.json', 'package-lock.json', 'main', 'tray', 'common-ui'];
const skip = new Set(['node_modules', 'src-tauri', 'dist', '.DS_Store']);

fs.mkdirSync(copy, { recursive: true });
for (const p of parts) {
  fs.rmSync(path.join(copy, p), { recursive: true, force: true });
  fs.cpSync(path.join(ref, p), path.join(copy, p), {
    recursive: true,
    filter: (src) => !skip.has(path.basename(src)),
  });
}

const lock = fs.readFileSync(path.join(copy, 'package-lock.json'), 'utf8');
const stamp = path.join(copy, 'node_modules', '.viewer-lock');
if (!fs.existsSync(stamp) || fs.readFileSync(stamp, 'utf8') !== lock) {
  execFileSync('npm', ['ci', '--no-audit', '--no-fund', '--ignore-scripts'], {
    cwd: copy,
    stdio: 'inherit',
  });
  fs.writeFileSync(stamp, lock);
}

// The app subsets its icon font at dev time with a Python script; the full font draws
// the same glyphs.
const font = path.join(copy, 'node_modules', 'material-symbols', 'material-symbols-outlined.woff2');
for (const app of ['main', 'tray']) {
  const dir = path.join(copy, app, 'src', 'assets', 'fonts');
  fs.mkdirSync(dir, { recursive: true });
  fs.copyFileSync(font, path.join(dir, 'material-symbols-subset.woff2'));
}

const viewer = path.join(copy, 'viewer');
fs.rmSync(viewer, { recursive: true, force: true });
fs.cpSync(path.join(here, 'mock'), viewer, { recursive: true });
fs.writeFileSync(
  path.join(viewer, 'catalog.json'),
  execFileSync('python3', [path.join(here, 'catalog.py'), path.join(ref, 'common', 'src', 'hotkeys.rs')])
);

// Entry pages: the app's own HTML with the mock loaded first.
const inject = (html, extra) =>
  html.replace(
    '<script type="module" src="/src/',
    `${extra}<script type="module" src="/@fs${viewer}/ipc.js"></script>\n    <script type="module" src="/src/`
  );
const mainHtml = fs.readFileSync(path.join(copy, 'main', 'index.html'), 'utf8');
fs.writeFileSync(path.join(copy, 'main', 'viewer.html'), inject(mainHtml, ''));
const panelsHtml = fs.readFileSync(path.join(copy, 'main', 'panels.html'), 'utf8');
fs.writeFileSync(
  path.join(copy, 'main', 'viewer-settings.html'),
  inject(panelsHtml, `<script>window.__PANEL__ = { kind: 'settings' };</script>\n    `)
);
const trayHtml = fs.readFileSync(path.join(copy, 'tray', 'index.html'), 'utf8');
fs.writeFileSync(path.join(copy, 'tray', 'viewer.html'), inject(trayHtml, ''));

console.log(`reference copy ready: ${copy}`);
