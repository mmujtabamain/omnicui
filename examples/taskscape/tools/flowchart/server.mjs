// Serves the built page and its comments API. Comments live in a local SQLite file next to the
// app (db.sqlite3, as a Django project keeps its database; COMMENTS_DB overrides the path).
// The first start loads the seed comments (dist/seed-comments.json) into an empty database.
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import Database from 'better-sqlite3';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.join(here, 'dist');
const port = Number(process.env.PORT ?? 8080);
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.webp': 'image/webp', '.png': 'image/png', '.svg': 'image/svg+xml' };
const log = (...a) => console.log(new Date().toISOString(), ...a);

function sqliteStore(file) {
  const db = new Database(file);
  db.pragma('journal_mode = WAL');
  db.exec(`CREATE TABLE IF NOT EXISTS flow_comments (
    id TEXT PRIMARY KEY, node TEXT NOT NULL, parent TEXT, author TEXT NOT NULL,
    tag TEXT NOT NULL DEFAULT 'general', text TEXT NOT NULL,
    resolved INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL)`);
  const row = (r) => r && { ...r, resolved: !!r.resolved };
  const insert = db.prepare('INSERT INTO flow_comments (id,node,parent,author,tag,text,resolved,created_at) VALUES (@id,@node,@parent,@author,@tag,@text,@resolved,@created_at)');
  return {
    all: () => db.prepare('SELECT * FROM flow_comments ORDER BY created_at').all().map(row),
    count: () => db.prepare('SELECT count(*) AS n FROM flow_comments').get().n,
    add: (c) => (insert.run({ ...c, resolved: c.resolved ? 1 : 0 }), c),
    seed: db.transaction((rows) => rows.forEach((c) => insert.run({ ...c, resolved: c.resolved ? 1 : 0 }))),
    update: (id, patch) => {
      if (typeof patch.resolved === 'boolean') db.prepare('UPDATE flow_comments SET resolved = ? WHERE id = ?').run(patch.resolved ? 1 : 0, id);
      return row(db.prepare('SELECT * FROM flow_comments WHERE id = ?').get(id));
    },
  };
}

const store = sqliteStore(process.env.COMMENTS_DB ?? path.join(here, 'db.sqlite3'));
if (store.count() === 0) {
  const seed = path.join(dist, 'seed-comments.json');
  if (fs.existsSync(seed)) {
    const rows = JSON.parse(fs.readFileSync(seed, 'utf8'));
    store.seed(rows);
    log(`seeded ${rows.length} comments`);
  }
}

const TAGS = new Set(['web', 'desktop', 'both', 'general']);
const body = (req) =>
  new Promise((resolve, reject) => {
    let s = '';
    req.on('data', (d) => {
      s += d;
      if (s.length > 100_000) reject(new Error('too large'));
    });
    req.on('end', () => {
      try { resolve(s ? JSON.parse(s) : {}); } catch (e) { reject(e); }
    });
  });
const send = (res, code, data) => {
  res.writeHead(code, { 'content-type': 'application/json' });
  res.end(JSON.stringify(data));
};

http
  .createServer(async (req, res) => {
    const url = new URL(req.url, 'http://x');
    try {
      if (url.pathname === '/api/comments' && req.method === 'GET') return send(res, 200, store.all());
      if (url.pathname === '/api/comments' && req.method === 'POST') {
        const b = await body(req);
        const text = String(b.text ?? '').trim();
        if (!b.node || !text) return send(res, 400, { error: 'node and text are required' });
        const c = {
          id: `c-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`,
          node: String(b.node).slice(0, 200),
          parent: b.parent ? String(b.parent) : null,
          author: String(b.author || 'Anonymous').slice(0, 60),
          tag: TAGS.has(b.tag) ? b.tag : 'general',
          text: text.slice(0, 5000),
          resolved: false,
          created_at: Date.now(),
        };
        log('comment', c.node, c.author);
        return send(res, 201, store.add(c));
      }
      const m = url.pathname.match(/^\/api\/comments\/([\w-]+)$/);
      if (m && req.method === 'PATCH') {
        const b = await body(req);
        const c = store.update(m[1], { resolved: typeof b.resolved === 'boolean' ? b.resolved : undefined });
        return c ? send(res, 200, c) : send(res, 404, { error: 'no such comment' });
      }
      if (url.pathname.startsWith('/api/')) return send(res, 404, { error: 'not found' });
      let file = path.join(dist, decodeURIComponent(url.pathname));
      if (!file.startsWith(dist) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) file = path.join(dist, 'index.html');
      res.writeHead(200, { 'content-type': TYPES[path.extname(file)] ?? 'application/octet-stream' });
      fs.createReadStream(file).pipe(res);
    } catch (e) {
      log('error', req.method, url.pathname, e.message);
      send(res, 500, { error: 'server error' });
    }
  })
  .listen(port, () => log(`flowchart on :${port} (comments in SQLite)`));
