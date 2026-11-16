// The app's first moments and its type (review notes #11956, #11959, #11960, 2026-11-16): the loading veil
// stays up while the tasks load and lifts after; the sign-in page lifts it at once; the text is Inter; and
// text is selectable only where it is edited.
//   node apps/web/frontend/tests/boot-and-type.mjs <app url>   (demo data loaded)
import { chromium } from "playwright";

const base = (process.argv[2] ?? "http://127.0.0.1:8000").replace(/\/$/, "");
const part = (name) => `[data-part~="${name}"]`;
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const fails = [];
const check = (ok, what) => { if (!ok) fails.push(what); };

await page.goto(`${base}/signin`);
await page.locator(part("app.signin.emailField")).waitFor();
await page.waitForTimeout(800);
check((await page.locator("#boot").count()) === 0, "the veil is still up on the sign-in page");
await page.locator(part("app.signin.emailField")).fill("demo@taskscape.test");
await page.locator(part("app.signin.passwordField")).fill("taskscape-demo");
await page.locator(part("app.signin.submitButton")).click();
await page.locator(part("app.mainwindow.taskPane")).waitFor();

// A slow task list: the veil covers the window until it arrives.
await page.route("**/api/tasks**", async (route) => {
  await new Promise((done) => setTimeout(done, 1500));
  await route.continue();
});
await page.reload();
await page.waitForTimeout(700);
check((await page.locator("#boot").count()) === 1, "no veil while the tasks load");
await page.locator(`${part("app.mainwindow.taskPane")} [data-task-id]`).first().waitFor();
await page.waitForTimeout(1200);
check((await page.locator("#boot").count()) === 0, "the veil stays up after the tasks arrived");

const type = await page.evaluate(() => {
  const row = document.querySelector("[data-task-id]");
  const input = document.querySelector("input");
  return {
    font: getComputedStyle(document.body).fontFamily,
    inter: document.fonts.check('14px "Inter"'),
    row: row ? getComputedStyle(row).userSelect : "",
    cursor: row ? getComputedStyle(row).cursor : "",
    input: input ? getComputedStyle(input).userSelect : "",
  };
});
check(type.font.startsWith('Inter') || type.font.startsWith('"Inter"'), `body font is ${type.font}`);
check(type.inter, "Inter is not loaded");
check(type.row === "none" && type.cursor === "default", `a task row is selectable (${type.row}, ${type.cursor})`);
check(type.input === "text", `the search field is not selectable (${type.input})`);
await browser.close();

if (fails.length) {
  console.log(`FAIL ${fails.join("; ")}`);
  process.exit(1);
}
console.log("PASS veil while loading, gone after; Inter; selection only where text is edited");
