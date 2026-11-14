// Signing in and then saving works in a real browser: the app's first write after sign-in carries
// the session's new CSRF token (a stale one made every save fail, 2026-11-14). It marks
// "Cut a spare key" done through the bulk bar, checks the API, then marks it not done again.
//   node apps/web/frontend/tests/sign-in-then-save.mjs <app url>   (demo data loaded)
import { chromium } from "playwright";

const base = (process.argv[2] ?? "http://127.0.0.1:8000").replace(/\/$/, "");
const part = (name) => `[data-part~="${name}"]`;
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1180, height: 760 } });
const refused = [];
page.on("response", (r) => { if (r.request().method() !== "GET" && r.status() === 403) refused.push(r.url()); });
await page.goto(`${base}/signin`);
await page.locator(part("app.signin.emailField")).fill("demo@taskscape.test");
await page.locator(part("app.signin.passwordField")).fill("taskscape-demo");
await page.locator(part("app.signin.submitButton")).click();
await page.locator(part("app.tasklist.checkCut")).click();
await page.locator(part("app.tasklist.bulkDone")).click();
await page.waitForTimeout(1200);
const done = await page.evaluate(async () => {
  const lists = await (await fetch("/api/lists")).json();
  const tasks = await (await fetch(`/api/tasks?list=${lists[0].id}`)).json();
  return tasks.find((t) => t.title === "Cut a spare key")?.done;
});
// Put it back as it was.
await page.locator(part("app.tasklist.checkCut")).click().catch(() => {});
await page.locator(part("app.tasklist.bulkUndone")).click().catch(() => {});
await browser.close();
const problems = [done !== true && `the task is not done on the server (done=${done})`, refused.length && `${refused.length} write(s) refused: ${refused[0]}`].filter(Boolean);
console.log(problems.length ? `FAIL ${problems.join("; ")}` : "PASS signed in, saved through the bulk bar, the API has it");
process.exit(problems.length ? 1 : 0);
