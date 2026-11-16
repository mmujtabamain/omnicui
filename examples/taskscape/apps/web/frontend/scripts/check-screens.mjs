// Checks every screen that has both a contract and a harness entry, several at a time, and prints one line each:
//   node apps/web/frontend/scripts/check-screens.mjs [--jobs 6] [name…]
// Each check has its own database, server and output folder (check-screen.mjs), so they run side by side.
// Writes .checks/screens-summary.json ({ name: { ok, contract } }). Exit code 0 when all pass.
import { spawn } from "node:child_process";
import { existsSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const ui = resolve(here, "..");
const root = resolve(ui, "../../..");
const args = process.argv.slice(2);
const jobs = args[0] === "--jobs" ? Number(args.splice(0, 2)[1]) : 6;
const contracts = join(root, "omnicui/contracts");
const names = args.length
  ? args
  : readdirSync(contracts)
      .filter((f) => f.endsWith(".contract.yaml"))
      .map((f) => f.slice(0, -".contract.yaml".length))
      .filter((n) => existsSync(join(ui, "src/harness", `${n}.tsx`)))
      .sort();
const summary = {};
const started = Date.now();
const run = (name) => new Promise((done) => {
  const p = spawn(process.execPath, [join(here, "check-screen.mjs"), name], { cwd: root });
  let out = "";
  p.stdout.on("data", (d) => (out += d));
  p.stderr.on("data", (d) => (out += d));
  p.on("close", (code) => {
    const contract = out.match(/^CONTRACT: (.*)$/m)?.[1] ?? out.split("\n").filter(Boolean).slice(-1)[0];
    const fails = out.split("\n").filter((l) => /^- (FAIL|BROKEN) /.test(l)).map((l) => l.slice(0, 200));
    summary[name] = { ok: code === 0, contract, fails };
    console.log(`${code === 0 ? "PASS" : "FAIL"}  ${name.padEnd(20)} ${contract}`);
    for (const f of fails.slice(0, 3)) console.log(`      ${f}`);
    done();
  });
});
const queue = [...names];
await Promise.all(Array.from({ length: jobs }, async () => { while (queue.length) await run(queue.shift()); }));
writeFileSync(join(root, ".checks/screens-summary.json"), JSON.stringify(summary, null, 1));
const failed = Object.values(summary).filter((s) => !s.ok).length;
console.log(`${names.length - failed} of ${names.length} pass (${Math.round((Date.now() - started) / 60000)} min)`);
process.exit(failed ? 1 : 0);
