// The signed-out page is quiet and steady: it asks who is signed in once, asks for nothing that
// needs an account, and never blanks while it sits there (the "blinking" sign-in page, 2026-11-14).
//   node apps/web/frontend/tests/signed-out.mjs <app url>
import { chromium } from "playwright";

const base = (process.argv[2] ?? "http://127.0.0.1:8000").replace(/\/$/, "");
const browser = await chromium.launch();
const page = await browser.newPage();
const calls = [];
page.on("request", (r) => { const u = new URL(r.url()); if (u.pathname.startsWith("/api/")) calls.push(u.pathname); });
await page.goto(`${base}/signin`);
await page.waitForSelector("#root *");
let blanks = 0;
for (let i = 0; i < 40; i++) {
  await page.waitForTimeout(100);
  if ((await page.evaluate(() => document.getElementById("root").childElementCount)) === 0) blanks++;
}
await browser.close();
const me = calls.filter((c) => c === "/api/auth/me").length;
const needsAccount = calls.filter((c) => !c.startsWith("/api/auth/"));
const problems = [
  me !== 1 && `asked who is signed in ${me} times (want 1)`,
  needsAccount.length && `asked for ${[...new Set(needsAccount)].join(", ")} while signed out`,
  blanks && `the page was blank in ${blanks} of 40 samples`,
].filter(Boolean);
console.log(problems.length ? `FAIL ${problems.join("; ")}` : `PASS signed-out page: 1 /api/auth/me, nothing else, never blank (4 s)`);
process.exit(problems.length ? 1 : 0);
