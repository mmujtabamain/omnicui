// Builds the app for production into dist/ (app.js, app.css, index.html): `npm run build`.
import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const here = dirname(fileURLToPath(import.meta.url));
export const dist = join(here, "dist");
export const page = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>Todo</title><link rel="stylesheet" href="/app.css"></head><body><div id="root"></div><script src="/app.js"></script></body></html>`;

export async function buildApp() {
  const started = Date.now();
  await build({
    entryPoints: [join(here, "src/main.tsx")],
    bundle: true, minify: true, outdir: dist, entryNames: "app", format: "iife",
    define: { "process.env.NODE_ENV": '"production"' }, logLevel: "warning",
  });
  writeFileSync(join(dist, "index.html"), page);
  return Date.now() - started;
}

if (import.meta.url === `file://${process.argv[1]}`) console.log(`built dist/ in ${await buildApp()} ms`);
