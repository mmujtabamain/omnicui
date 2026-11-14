// Checks one widget the way its review will: types, the lint rules, then its omnicui contract in
// Chromium. Prints a short report and draws every screenshot on one sheet to look at.
//
//   node packages/ui/scripts/check-widget.mjs <contract name> [--test <name>]
//
// <contract name> is the contract's file name without .contract.yaml (button, panefooter…); the
// harness entry src/harness/<contract name>.tsx mounts the widget. Exit code 0 when every test passes.
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const ui = resolve(here, "..");
const root = resolve(ui, "../..");
const project = join(root, "omnicui");
const [name, ...rest] = process.argv.slice(2);
if (!name) { console.error("usage: check-widget.mjs <contract name> [--test <name>]"); process.exit(2); }
const contract = join(project, "contracts", `${name}.contract.yaml`);
const harness = join(ui, "src/harness", `${name}.tsx`);
for (const f of [contract, harness]) if (!existsSync(f)) { console.error(`missing ${f}`); process.exit(2); }
const out = join(root, ".checks", name);
mkdirSync(out, { recursive: true });
let ok = true;

// 1. Types: the harness entry and everything it imports.
const tsconfig = join(out, "tsconfig.json");
writeFileSync(tsconfig, JSON.stringify({ extends: join(ui, "tsconfig.json"), files: [harness], include: [] }));
const tsc = spawnSync(join(root, "node_modules/.bin/tsc"), ["-p", tsconfig], { encoding: "utf8" });
const typeErrors = (tsc.stdout + tsc.stderr).trim();
console.log(typeErrors ? `TYPES: errors\n${typeErrors.split("\n").slice(0, 30).join("\n")}` : "TYPES: ok");
if (typeErrors) ok = false;

// 2. Lint: the widget's own files.
const files = readFileSync(harness, "utf8").match(/from "\.\.\/components\/ui\/[^"]+"/g)?.map((m) => join(ui, "src/components/ui", m.slice(25, -1) + ".tsx")) ?? [];
const lint = spawnSync(join(root, "node_modules/.bin/eslint"), ["--no-warn-ignored", harness, ...files.filter(existsSync)], { cwd: root, encoding: "utf8" });
const lintText = (lint.stdout + lint.stderr).trim();
console.log(lint.status === 0 ? "LINT: ok" : `LINT: errors\n${lintText.split("\n").slice(0, 30).join("\n")}`);
if (lint.status !== 0) ok = false;

// 3. The contract, on the web target, with only this widget built.
const run = join(out, "run");
const check = spawnSync(join(root, "../../bin/omnicui"), ["--project", project, "check", "web", name, "--out", run, ...rest], {
  cwd: project, encoding: "utf8", env: { ...process.env, OMNICUI_WIDGETS: name }, maxBuffer: 64 << 20,
});
const results = existsSync(join(run, "results.json")) ? JSON.parse(readFileSync(join(run, "results.json"), "utf8")) : null;
if (!results) {
  console.log(`CONTRACT: omnicui did not finish\n${(check.stderr || check.stdout).split("\n").slice(-40).join("\n")}`);
  process.exit(1);
}
const s = results.summary;
console.log(`CONTRACT: ${s.pass} pass, ${s.fail} fail, ${s.broken} broken, ${s.notMeasured} not measured, ${s.notApplicable} not applicable`);
for (const r of results.results) {
  if (["pass", "not applicable"].includes(r.status)) continue;
  // Not measured and not advertised (an action this target cannot do: a swipe in a desktop browser) are reported, not failed.
  if (!["not measured", "not advertised"].includes(r.status)) ok = false;
  console.log(`- ${r.status.toUpperCase()} ${r.test}: ${(r.messages ?? []).join(" / ").slice(0, 500)}`);
  const errs = (r.stderr ?? []).filter((l) => /error|failed/i.test(l)).slice(0, 2);
  for (const l of errs) console.log(`    ${l.slice(0, 300)}`);
}
for (const p of (results.problems ?? []).filter((p) => p.file?.endsWith(`/${name}.contract.yaml`))) console.log(`PROBLEM: ${JSON.stringify(p).slice(0, 400)}`);

// 4. Every screenshot on one sheet per colour scheme.
const shots = join(run, "web", name);
if (existsSync(shots)) {
  const { chromium } = await import("playwright");
  const browser = await chromium.launch();
  for (const scheme of ["light", "dark"]) {
    const cells = readdirSync(shots).sort().flatMap((t) =>
      readdirSync(join(shots, t)).filter((f) => f.endsWith(`-${scheme}.png`)).map((f) =>
        `<figure><img src="data:image/png;base64,${readFileSync(join(shots, t, f)).toString("base64")}"><figcaption>${t}</figcaption></figure>`));
    if (!cells.length) continue;
    const page = await browser.newPage({ viewport: { width: 1600, height: 900 } });
    await page.setContent(`<style>body{margin:8px;font:12px sans-serif;display:grid;grid-template-columns:repeat(5,1fr);gap:6px;background:#fff}
      figure{margin:0}img{width:100%;border:1px solid #bbb}figcaption{padding:2px 0;overflow-wrap:anywhere}</style>${cells.join("")}`);
    await page.screenshot({ path: join(out, `sheet-${scheme}.png`), fullPage: true });
    await page.close();
    console.log(`SHEET (${scheme}): ${join(out, `sheet-${scheme}.png`)} (${cells.length} screenshots)`);
  }
  await browser.close();
}
process.exit(ok ? 0 : 1);
