// Checks every widget that has a harness entry, several at a time, and prints one line each:
//   node packages/ui/scripts/check-all.mjs [--jobs 6] [name…]
// Writes .checks/summary.json ({ name: { ok, contract } }). Exit code 0 when all pass.
import { spawn } from "node:child_process";
import { readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const args = process.argv.slice(2);
const jobs = args[0] === "--jobs" ? Number(args.splice(0, 2)[1]) : 6;
const names = args.length ? args : readdirSync(join(here, "../src/harness")).filter((f) => f.endsWith(".tsx")).map((f) => f.slice(0, -4)).sort();
const summary = {};
const run = (name) => new Promise((done) => {
  const p = spawn(process.execPath, [join(here, "check-widget.mjs"), name], { cwd: root });
  let out = "";
  p.stdout.on("data", (d) => (out += d));
  p.stderr.on("data", (d) => (out += d));
  p.on("close", (code) => {
    const contract = out.match(/^CONTRACT: (.*)$/m)?.[1] ?? out.split("\n").filter(Boolean).slice(-1)[0];
    const types = /^TYPES: ok/m.test(out), lint = /^LINT: ok/m.test(out);
    summary[name] = { ok: code === 0, types, lint, contract };
    console.log(`${code === 0 ? "PASS" : "FAIL"}  ${name.padEnd(20)} ${types ? "" : "types! "}${lint ? "" : "lint! "}${contract}`);
    done();
  });
});
const queue = [...names];
await Promise.all(Array.from({ length: jobs }, async () => { while (queue.length) await run(queue.shift()); }));
writeFileSync(join(root, ".checks/summary.json"), JSON.stringify(summary, null, 1));
const failed = Object.values(summary).filter((s) => !s.ok).length;
console.log(`${names.length - failed} of ${names.length} pass`);
process.exit(failed ? 1 : 0);
