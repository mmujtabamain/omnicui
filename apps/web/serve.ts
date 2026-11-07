// Builds the web app for production, serves it on 127.0.0.1 and opens it in your browser:
// `npm start` (or `run.sh app web`). It keeps serving until Ctrl+C.
// OMNICUI_DEFECTS=… switches on the components' deliberate defects; NO_OPEN=1 doesn't open a browser.
import { createServer } from "node:http";
import { spawn } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";
import { serveFile } from "./static.ts";

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, "dist");
await build({
  entryPoints: [join(here, "src/main.tsx")],
  bundle: true, minify: true, outdir: dist, entryNames: "app", format: "iife",
  loader: { ".ttf": "file" }, assetNames: "[name]", define: { "process.env.NODE_ENV": '"production"' }, logLevel: "warning",
});
const page = `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Web app</title>
<link rel="stylesheet" href="./app.css"><script>globalThis.__OMNICUI_DEFECTS__ = ${JSON.stringify(process.env.OMNICUI_DEFECTS ?? "")};</script></head>
<body><div id="root"></div><script src="./app.js"></script></body></html>`;
const server = createServer((req, res) => {
  const name = new URL(req.url ?? "/", "http://x").pathname;
  if (name === "/") return res.writeHead(200, { "content-type": "text/html" }).end(page);
  serveFile(res, dist, name);
});
await new Promise<void>((ok) => server.listen(Number(process.env.PORT ?? 0), "127.0.0.1", ok));
const url = `http://127.0.0.1:${(server.address() as { port: number }).port}/`;
console.log(`The web app is at ${url} (Ctrl+C to stop).`);
if (process.env.NO_OPEN !== "1") {
  const [cmd, args] = process.platform === "win32" ? ["cmd", ["/c", "start", "", url]] : [process.platform === "darwin" ? "open" : "xdg-open", [url]];
  spawn(cmd, args, { stdio: "ignore", detached: true }).on("error", () => console.log("Open that address in a browser.")).unref();
}
