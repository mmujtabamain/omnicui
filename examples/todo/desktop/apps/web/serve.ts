// Builds the app and serves it on 127.0.0.1 (PORT, or a free port): `npm start`. Every path that
// isn't a file is the app, so a reload of ?task=t1 or ?settings works.
import { createServer } from "node:http";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { buildApp, dist, page } from "./build.ts";
import { serveFile } from "./static.ts";

await buildApp();
const server = createServer((req, res) => {
  const name = new URL(req.url ?? "/", "http://x").pathname;
  if (name !== "/" && existsSync(join(dist, name))) return serveFile(res, dist, name);
  res.writeHead(200, { "content-type": "text/html" }).end(page);
});
await new Promise<void>((ok) => server.listen(Number(process.env.PORT ?? 0), "127.0.0.1", ok));
console.log(`The todo web app is at http://127.0.0.1:${(server.address() as { port: number }).port}/ (Ctrl+C to stop).`);
