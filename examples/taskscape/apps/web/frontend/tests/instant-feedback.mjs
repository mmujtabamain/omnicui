// Changes show at once and are taken back if the server refuses them (review note #11961, 2026-11-16).
// Writes are held for 2 s and then answered by the test (nothing reaches the server, so the demo data is
// untouched): the done count must move within 400 ms of Mark done; a refused write must put it back and say so.
//   node apps/web/frontend/tests/instant-feedback.mjs <app url>   (demo data loaded)
import { chromium } from "playwright";

const base = (process.argv[2] ?? "http://127.0.0.1:8000").replace(/\/$/, "");
const part = (name) => `[data-part~="${name}"]`;
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const fails = [];
const check = (ok, what) => { if (!ok) fails.push(what); };

await page.goto(`${base}/signin`);
await page.locator(part("app.signin.emailField")).fill("demo@taskscape.test");
await page.locator(part("app.signin.passwordField")).fill("taskscape-demo");
await page.locator(part("app.signin.submitButton")).click();
const count = page.locator(part("app.tasklist.doneCount"));
await count.waitFor();
await page.locator("#boot").waitFor({ state: "detached" });
const doneOf = async () => Number((await count.innerText()).match(/(\d+)\s*\//)?.[1] ?? NaN);

let answer = 200;
await page.route("**/api/tasks/bulk", async (route) => {
  await new Promise((done) => setTimeout(done, 2000));
  if (answer === 200) await route.fulfill({ status: 200, contentType: "application/json", body: "{}" });
  else await route.fulfill({ status: 500, contentType: "application/json", body: '{"detail":"refused for a test"}' });
});

// 1. Mark done: the count moves before the server answers.
const before = await doneOf();
await page.locator(part("app.tasklist.checkCut")).click();
await page.locator(part("app.tasklist.bulkDone")).click();
await page.waitForTimeout(400);
check((await doneOf()) === before + 1, `done count did not move at once (${before} → ${await doneOf()})`);
await page.waitForTimeout(3000);

// 2. Refused: the change shows, then is put back, and a toast says so.
answer = 500;
await page.keyboard.press("Escape"); // drop the picks left from step 1
const start = await doneOf();
await page.locator(part("app.tasklist.checkBike")).click();
await page.locator(part("app.tasklist.bulkDone")).click();
await page.waitForTimeout(400);
check((await doneOf()) === start + 1, `refused write: count did not move at once (${start} → ${await doneOf()})`);
await page.waitForTimeout(2500);
check((await doneOf()) === start, `refused write was not put back (count ${await doneOf()}, was ${start})`);
check(await page.getByText("Couldn't save that change").isVisible(), "no message after a refused write");
await browser.close();

if (fails.length) {
  console.log(`FAIL ${fails.join("; ")}`);
  process.exit(1);
}
console.log("PASS Mark done shows within 400 ms; a refused write is put back with a message");
