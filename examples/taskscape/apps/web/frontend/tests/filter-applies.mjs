// Filter & Sort changes the list you see (it used to change only the panel's own hidden copy,
// 2026-11-16): signs in, opens Filter & Sort, shows Done only, and counts the rows of the real list.
//   node apps/web/frontend/tests/filter-applies.mjs <app url>   (demo data loaded)
import { chromium } from "playwright";

const base = (process.argv[2] ?? "http://127.0.0.1:8000").replace(/\/$/, "");
const part = (name) => `[data-part~="${name}"]`;
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
await page.goto(`${base}/signin`);
await page.locator(part("app.signin.emailField")).fill("demo@taskscape.test");
await page.locator(part("app.signin.passwordField")).fill("taskscape-demo");
await page.locator(part("app.signin.submitButton")).click();
const rows = page.locator(`${part("app.mainwindow.taskPane")} [data-task-id]`);
await rows.first().waitFor();
const before = await rows.count();
await page.locator(part("app.tasklist.sortFilter")).click();
await page.locator(part("app.filterpanel.showDone")).click();
await page.waitForTimeout(400);
const titles = await rows.allInnerTexts();
await page.locator(part("app.filterpanel.showAll")).click().catch(() => {});
await browser.close();
// Done only: the done tasks, plus the parents that hold one (Buy groceries holds Oat milk).
const expected = ["Post the parcel to Mum", "Buy groceries for the week", "Oat milk"];
const shown = titles.map((t) => t.split("\n")[0].trim());
const ok = before > shown.length && expected.every((e) => shown.some((s) => s.startsWith(e))) && shown.length === expected.length;
console.log(ok ? `PASS Filter & Sort filters the list (${before} rows, Done only shows ${shown.length})` : `FAIL rows before ${before}, Done only shows: ${JSON.stringify(shown)}`);
process.exit(ok ? 0 : 1);
