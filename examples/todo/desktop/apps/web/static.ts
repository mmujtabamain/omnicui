// A static file handler for the app's dev server (serve.ts) and for the omnicui web target, which
// serves its harness the same way.
//
// The file is read before any header is written, so a missing file (a favicon the browser asks
// for on its own, say) is a clean 404. Writing 200 first and then failing to read the file sent
// the headers twice and crashed the server with ERR_HTTP_HEADERS_SENT.
import type { ServerResponse } from "node:http";
import { readFileSync } from "node:fs";
import { extname, join, sep } from "node:path";

export const contentTypes: Record<string, string> = {
  ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json",
  ".ttf": "font/ttf", ".woff2": "font/woff2", ".png": "image/png", ".svg": "image/svg+xml",
};

/** Answers with the file at `pathname` inside `root`, or 404 when there is none (or it is outside root). */
export function serveFile(res: ServerResponse, root: string, pathname: string): void {
  const path = join(root, pathname);
  let body: Buffer;
  try {
    // join() has already resolved any "..", so a path outside root no longer starts with it.
    // root + sep, not root: "dist-other" starts with "dist" too.
    if (!path.startsWith(root + sep)) throw new Error("outside the served folder");
    body = readFileSync(path);
  } catch {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "content-type": contentTypes[extname(path)] ?? "application/octet-stream" }).end(body);
}
