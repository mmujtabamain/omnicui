// node --test static.test.ts
import { test } from "node:test";
import assert from "node:assert/strict";
import { createServer, type Server } from "node:http";
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { serveFile } from "./static.ts";

async function serving(root: string): Promise<[Server, string]> {
  const server = createServer((req, res) => serveFile(res, root, new URL(req.url ?? "/", "http://x").pathname));
  await new Promise<void>((ok) => server.listen(0, "127.0.0.1", ok));
  return [server, `http://127.0.0.1:${(server.address() as { port: number }).port}`];
}

test("a file is served with its content type", async () => {
  const root = mkdtempSync(join(tmpdir(), "static-"));
  writeFileSync(join(root, "app.js"), "ok()");
  const [server, url] = await serving(root);
  try {
    const r = await fetch(`${url}/app.js`);
    assert.equal(r.status, 200);
    assert.equal(r.headers.get("content-type"), "text/javascript");
    assert.equal(await r.text(), "ok()");
  } finally {
    server.close();
  }
});

// The regression: a missing file used to crash the server (ERR_HTTP_HEADERS_SENT).
test("a missing file, a folder and a path outside the root are 404, and the server keeps serving", async () => {
  const parent = mkdtempSync(join(tmpdir(), "static-"));
  const root = join(parent, "dist");
  mkdirSync(join(root, "sub"), { recursive: true });
  mkdirSync(join(parent, "dist-other"));
  writeFileSync(join(parent, "dist-other", "secret.txt"), "no");
  writeFileSync(join(parent, "secret.txt"), "no");
  writeFileSync(join(root, "app.css"), "body{}");
  const [server, url] = await serving(root);
  try {
    for (const path of ["/favicon.ico", "/sub", "/../secret.txt", "/%2e%2e/secret.txt", "/../dist-other/secret.txt"]) {
      const r = await fetch(url + path);
      assert.equal(r.status, 404, path);
    }
    assert.equal((await fetch(`${url}/app.css`)).status, 200, "still serving after the 404s");
  } finally {
    server.close();
  }
});
