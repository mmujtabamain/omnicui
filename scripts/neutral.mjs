// node scripts/neutral.mjs: omnicui is its own project and names no host that uses it, anywhere in
// the repository (this file names the one it looks for).
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

const root = process.cwd();
const skip = new Set(['node_modules', 'dist', 'target', 'bin', '.git', 'neutral.mjs', '.omnicui']);
const hits = [];
const walk = (d) => {
  for (const n of readdirSync(d)) {
    if (skip.has(n) || (n.startsWith('.') && n !== '.github' && n !== '.gitignore')) continue;
    const p = join(d, n);
    const st = statSync(p);
    if (st.isDirectory()) walk(p);
    else if (st.size < 2_000_000 && !/\.(png|gif|ttf|jar)$/.test(n)) {
      readFileSync(p, 'utf8').split('\n').forEach((l, i) => {
        if (/dev.?bench/i.test(l)) hits.push(`${relative(root, p)}:${i + 1}: ${l.trim().slice(0, 100)}`);
      });
    }
  }
};
walk(root);
if (hits.length) {
  console.error(`omnicui is host-neutral, but these name DevBench:\n${hits.join('\n')}`);
  process.exit(1);
}
console.log('check:neutral: no host names anywhere');
