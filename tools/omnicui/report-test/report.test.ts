// Opens the last run's report.html (run `make check` or `bin/omnicui check` first) in Chromium and:
//   - runs axe with the WCAG 2.0, 2.1 and 2.2 A and AA rules; any violation fails;
//   - checks it has one h1, a main landmark, a caption on every table and a text alternative on every image;
//   - saves screenshots at desktop and phone widths, in light, dark and forced colours, to omnicui/.omnicui/report-shots/ (in the repository).
import { chromium } from "playwright";
import { AxeBuilder } from "@axe-core/playwright";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const report = join(here, "..", "..", "..", "omnicui", ".omnicui", "run", "report.html");
const label = process.env.REPORT_LABEL ?? "report";
const shots = join(here, "..", "..", "..", "omnicui", ".omnicui", "report-shots", label);
if (!existsSync(report)) {
  console.error(`no ${report}: run \`bin/omnicui check\` first`);
  process.exit(2);
}
mkdirSync(shots, { recursive: true });

const browser = await chromium.launch();
let failures = 0;
const results: Record<string, unknown> = {};
const cases = [
  { name: "desktop-light", viewport: { width: 1280, height: 900 }, colorScheme: "light" as const, forcedColors: "none" as const },
  { name: "desktop-dark", viewport: { width: 1280, height: 900 }, colorScheme: "dark" as const, forcedColors: "none" as const },
  { name: "phone-light", viewport: { width: 390, height: 844 }, colorScheme: "light" as const, forcedColors: "none" as const },
  { name: "desktop-forced-colors", viewport: { width: 1280, height: 900 }, colorScheme: "light" as const, forcedColors: "active" as const },
];
for (const c of cases) {
  const context = await browser.newContext({ viewport: c.viewport, colorScheme: c.colorScheme, forcedColors: c.forcedColors, reducedMotion: "reduce" });
  const page = await context.newPage();
  await page.goto(pathToFileURL(report).href);
  const axe = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"]).analyze();
  const structure = await page.evaluate(() => ({
    h1: document.querySelectorAll("h1").length,
    main: document.querySelectorAll("main").length,
    tablesWithoutCaption: [...document.querySelectorAll("table")].filter((t) => !t.querySelector("caption")).length,
    imagesWithoutAlt: [...document.querySelectorAll("img")].filter((i) => !i.getAttribute("alt")).length,
    horizontalScroll: document.documentElement.scrollWidth > window.innerWidth + 1,
  }));
  const problems = [
    ...axe.violations.map((v) => `axe ${v.id} (${v.impact}): ${v.help} — ${v.nodes.length} node(s)`),
    ...(structure.h1 !== 1 ? [`${structure.h1} h1 elements`] : []),
    ...(structure.main !== 1 ? [`${structure.main} main landmarks`] : []),
    ...(structure.tablesWithoutCaption ? [`${structure.tablesWithoutCaption} tables without a caption`] : []),
    ...(structure.imagesWithoutAlt ? [`${structure.imagesWithoutAlt} images without alt text`] : []),
    ...(structure.horizontalScroll ? ["the page scrolls sideways"] : []),
  ];
  await page.screenshot({ path: join(shots, `${c.name}.png`), fullPage: false });
  results[c.name] = { axePasses: axe.passes.length, axeViolations: axe.violations.length, ...structure, problems };
  console.log(`${problems.length ? "FAIL" : "pass"}  ${c.name}: axe ${axe.passes.length} rules passed, ${axe.violations.length} violations${problems.length ? "\n      " + problems.join("\n      ") : ""}`);
  failures += problems.length ? 1 : 0;
  await context.close();
}
await browser.close();
writeFileSync(join(shots, "results.json"), JSON.stringify(results, null, 2) + "\n");
console.log(`${label}: ` + (failures ? `${failures} of ${cases.length} failed` : `all ${cases.length} passed`));
process.exit(failures ? 1 : 0);
