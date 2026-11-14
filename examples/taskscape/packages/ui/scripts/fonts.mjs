// Copies the icon font from the material-symbols package next to the styles, where both the
// frontend's Vite build and the omnicui web target pick it up. Runs after every install.
import { copyFileSync, mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const pkg = dirname(createRequire(import.meta.url).resolve("material-symbols/package.json"));
const out = join(here, "../src/styles/fonts");
mkdirSync(out, { recursive: true });
copyFileSync(join(pkg, "material-symbols-outlined.woff2"), join(out, "material-symbols-outlined.woff2"));
