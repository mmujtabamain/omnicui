// A load that looks like one load (Mujtaba's recording, 2026-11-16): the first paint is already in the right
// theme, and coming back to the tab neither blanks the page nor replays its fade-in.
//   node apps/web/frontend/tests/steady-load.mjs <app url>
import { chromium } from "playwright";

const base = (process.argv[2] ?? "http://127.0.0.1:8000").replace(/\/$/, "");
const browser = await chromium.launch();
const fails = [];
const check = (ok, what) => { if (!ok) fails.push(what); };

// 1. Dark system: dark from the first paint, before any of the app's code has run.
const dark = await browser.newContext({ colorScheme: "dark" });
const page = await dark.newPage();
await page.addInitScript(() => {
  document.addEventListener("readystatechange", () => {
    if (document.readyState === "interactive") window.__firstPaintDark = document.documentElement.classList.contains("dark");
  });
});
await page.goto(`${base}/signin`);
await page.waitForTimeout(1500);
check(await page.evaluate(() => window.__firstPaintDark === true), "a dark system's first paint is not dark");

// 2. Coming back to the tab after the data went stale: the page stays, the fade-in does not replay.
await page.waitForTimeout(5500);
const events = await page.evaluate(async () => {
  const log = [];
  const root = document.getElementById("root");
  new MutationObserver(() => { if (root.childElementCount === 0) log.push("root emptied"); }).observe(root, { childList: true });
  document.addEventListener("animationstart", (e) => { if (e.animationName === "app-in") log.push("fade-in replayed"); }, true);
  Object.defineProperty(document, "visibilityState", { value: "hidden", configurable: true });
  window.dispatchEvent(new Event("visibilitychange"));
  Object.defineProperty(document, "visibilityState", { value: "visible", configurable: true });
  window.dispatchEvent(new Event("visibilitychange"));
  await new Promise((done) => setTimeout(done, 1500));
  return log;
});
check(events.length === 0, `on tab focus: ${events.join(", ")}`);
await browser.close();

if (fails.length) {
  console.log(`FAIL ${fails.join("; ")}`);
  process.exit(1);
}
console.log("PASS dark from the first paint; tab focus keeps the page and does not replay the fade-in");
