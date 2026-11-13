import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Background, Controls, Handle, MarkerType, MiniMap, Position, ReactFlow, ReactFlowProvider, useReactFlow } from '@xyflow/react';
import { layout } from './layout.js';
import { Comments } from './Comments.jsx';

// Node keys used by comments: screen:<id>, widget:<family>, variant:<family>/<variant>,
// use:<screen>/<kind>/<variant> (a widget as used on one screen), link:<from>><to>.
const key = {
  screen: (id) => `screen:${id}`,
  widget: (kind) => `widget:${kind}`,
  variant: (kind, v) => `variant:${kind}/${v}`,
  link: (a, b) => `link:${a}>${b}`,
  use: (screen, w) => `use:${screen}/${w.kind}/${w.variant || w.component}`,
};

function ScreenNode({ data }) {
  return (
    <div className={`node screen ${data.tier} ${data.focus ? 'focus' : ''}`}>
      <Handle type="target" position={Position.Left} />
      {data.thumb && <img src={data.thumb} alt="" />}
      <div className="node-body">
        <div className="node-title">{data.label}</div>
        <div className="node-sub">{data.sub}</div>
      </div>
      {data.comments > 0 && <span className="pill">{data.comments}</span>}
      <Handle type="source" position={Position.Right} />
    </div>
  );
}
function WidgetNode({ data }) {
  return (
    <div className={`node widget ${data.focus ? 'focus' : ''}`}>
      <Handle type="target" position={Position.Left} />
      <div className="node-kind">{data.kind}</div>
      <div className="node-title">{data.label}</div>
      <div className="node-sub">{data.sub}</div>
      {data.comments > 0 && <span className="pill">{data.comments}</span>}
      <Handle type="source" position={Position.Right} />
    </div>
  );
}
function RegionNode({ data }) {
  return <div className="region-head">{data.label} <span className="muted">· {data.count}</span></div>;
}
const nodeTypes = { screen: ScreenNode, widget: WidgetNode, region: RegionNode };

const edge = (id, source, target, label, opts = {}) => ({
  id, source, target, label, type: 'default',
  markerEnd: { type: MarkerType.ArrowClosed, width: 16, height: 16 },
  labelBgPadding: [4, 2], labelBgBorderRadius: 4, ...opts,
});

function buildApp(data, counts) {
  const nodes = data.screens.map((s) => ({
    id: s.id, type: 'screen', width: 230, height: 168,
    data: { label: s.title, sub: `${s.widgets.length} widgets · ${s.links.length} links`, tier: s.tier, thumb: s.thumb, comments: counts[key.screen(s.id)] ?? 0, node: key.screen(s.id) },
  }));
  const pairs = new Map();
  for (const s of data.screens)
    for (const l of s.links) {
      if (l.to === s.id || !data.byId[l.to]) continue;
      const k = `${s.id}>${l.to}`;
      if (!pairs.has(k)) pairs.set(k, { from: s.id, to: l.to, via: [] });
      pairs.get(k).via.push(l.via);
    }
  const edges = [...pairs.values()].map((p) => edge(key.link(p.from, p.to), p.from, p.to, undefined, { data: { via: p.via, node: key.link(p.from, p.to) } }));
  return { nodes, edges };
}

// A screen: the screen on the left, its widgets in one column per region (placed here, not by
// ELK, so a screen with sixty widgets stays readable), and the screens it leads to on the right.
function buildScreen(data, id, counts) {
  const s = data.byId[id];
  const W = 240, H = 92, GAP = 14, COL = W + 40, TOP = 60;
  const nodes = [{ id: s.id, type: 'screen', width: 260, height: 190, position: { x: 0, y: TOP }, data: { label: s.title, sub: s.purpose.slice(0, 90), tier: s.tier, thumb: s.thumb, focus: true, comments: counts[key.screen(s.id)] ?? 0, node: key.screen(s.id) } }];
  const edges = [];
  const regions = [];
  for (const w of s.widgets) {
    const r = (w.region || 'Other').split(/[(,;]/)[0].trim().slice(0, 40) || 'Other';
    let g = regions.find((x) => x.name === r);
    if (!g) regions.push((g = { name: r, items: [] }));
    g.items.push(w);
  }
  regions.forEach((g, i) => {
    const x = 320 + i * COL;
    nodes.push({ id: `r:${i}`, type: 'region', position: { x, y: 0 }, data: { label: g.name, count: g.items.length }, draggable: false, selectable: false });
    g.items.forEach((w, j) => {
      const nid = `w:${w.key}`;
      nodes.push({ id: nid, type: 'widget', width: W, height: H, position: { x, y: TOP + j * (H + GAP) }, data: { kind: w.kind, label: w.purpose, sub: `${w.component} · ${w.variant}`, comments: counts[key.use(s.id, w)] ?? 0, node: key.use(s.id, w), widget: w, screen: s.id } });
    });
  });
  const seen = [];
  const rx = 320 + regions.length * COL + 80;
  for (const l of s.links) {
    if (l.to === s.id || !data.byId[l.to]) continue;
    const t = data.byId[l.to];
    if (!seen.includes(t.id)) {
      nodes.push({ id: `to:${t.id}`, type: 'screen', width: 230, height: 168, position: { x: rx, y: TOP + seen.length * 190 }, data: { label: t.title, sub: 'leads here', tier: t.tier, thumb: t.thumb, comments: counts[key.screen(t.id)] ?? 0, node: key.screen(t.id), go: t.id } });
      seen.push(t.id);
    }
    edges.push(edge(`l:${s.id}>${t.id}:${edges.length}`, s.id, `to:${t.id}`, undefined, { className: 'faint', data: { node: key.link(s.id, t.id), via: [l.via] } }));
  }
  return { nodes, edges, placed: true };
}

// A widget family: its variants in a grid, and the screens that use them on the right.
function buildWidget(data, kind, counts) {
  const fam = data.catalogue.find((f) => f.kind === kind);
  const W = 240, H = 92, GAP = 14, COLS = 4, TOP = 120;
  const nodes = [{ id: `k:${kind}`, type: 'widget', width: 4 * (W + GAP), height: 90, position: { x: 0, y: 0 }, data: { kind: 'widget family', label: fam?.title ?? kind, sub: fam?.description ?? '', focus: true, comments: counts[key.widget(kind)] ?? 0, node: key.widget(kind) } }];
  const edges = [];
  const used = [];
  (fam?.variants ?? []).forEach((v, i) => {
    const vid = `v:${i}`;
    nodes.push({ id: vid, type: 'widget', width: W, height: H, position: { x: (i % COLS) * (W + GAP), y: TOP + Math.floor(i / COLS) * (H + GAP) }, data: { kind: v.kind, label: v.name, sub: v.description, comments: counts[key.variant(kind, v.name)] ?? 0, node: key.variant(kind, v.name), variant: v } });
    for (const sid of v.usedIn ?? []) {
      if (!data.byId[sid]) continue;
      if (!used.includes(sid)) used.push(sid);
      edges.push(edge(`u:${vid}>${sid}`, vid, `s:${sid}`, undefined, { className: 'faint' }));
    }
  });
  used.forEach((sid, i) => {
    const s = data.byId[sid];
    nodes.push({ id: `s:${sid}`, type: 'screen', width: 230, height: 168, position: { x: COLS * (W + GAP) + 160, y: TOP + i * 190 }, data: { label: s.title, sub: 'uses it', tier: s.tier, thumb: s.thumb, comments: counts[key.screen(sid)] ?? 0, node: key.screen(sid), go: sid } });
  });
  return { nodes, edges, placed: true };
}

function Canvas({ data, view, setView, select, counts, filter, focusId }) {
  const [graph, setGraph] = useState({ nodes: [], edges: [] });
  const rf = useReactFlow();
  useEffect(() => {
    let built = view.kind === 'app' ? buildApp(data, counts) : view.kind === 'screen' ? buildScreen(data, view.id, counts) : buildWidget(data, view.id, counts);
    if (filter) {
      const f = filter.toLowerCase();
      built = { ...built, nodes: built.nodes.map((n) => ({ ...n, className: JSON.stringify(n.data).toLowerCase().includes(f) ? 'hit' : 'dim' })) };
    }
    (built.placed ? Promise.resolve(built.nodes) : layout(built.nodes, built.edges, { spacing: view.kind === 'app' ? 60 : 24 })).then((nodes) => {
      setGraph({ nodes, edges: built.edges });
      setTimeout(() => (built.placed ? rf.fitView({ padding: 0.05, duration: 300, maxZoom: 1, nodes: nodes.slice(0, 12) }) : rf.fitView({ padding: 0.12, duration: 300 })), 30);
    });
  }, [data, view, counts, filter, rf]);
  // On the app map, only the selected screen's links stand out.
  const edges = useMemo(
    () => (view.kind !== 'app' ? graph.edges : graph.edges.map((e) => ({ ...e, className: !focusId ? 'faint' : e.source === focusId || e.target === focusId ? 'lit' : 'gone', label: focusId && (e.source === focusId || e.target === focusId) ? (e.data?.via ?? [])[0]?.slice(0, 40) : undefined }))),
    [graph.edges, focusId, view.kind]
  );
  return (
    <ReactFlow
      nodes={graph.nodes}
      edges={edges}
      nodeTypes={nodeTypes}
      onNodeClick={(_, n) => select({ node: n.data.node, data: n.data })}
      onNodeDoubleClick={(_, n) => {
        if (n.type === 'screen') setView({ kind: 'screen', id: n.data.go ?? n.id });
        else if (n.data.widget) setView({ kind: 'widget', id: data.familyOf[n.data.widget.kind] ?? 'structure' });
      }}
      onEdgeClick={(_, e) => e.data?.node && select({ node: e.data.node, data: { via: e.data.via, link: true } })}
      minZoom={0.1}
      fitView
      proOptions={{ hideAttribution: true }}
    >
      <Background gap={24} />
      <Controls showInteractive={false} />
      <MiniMap pannable zoomable nodeStrokeWidth={3} />
    </ReactFlow>
  );
}

function Details({ data, sel, view, setView }) {
  if (!sel) return <p className="muted">Click a screen, widget or link to see it here and comment on it. Double-click a screen to open it.</p>;
  const [kind, rest] = sel.node.split(/:(.*)/s);
  if (kind === 'screen') {
    const s = data.byId[rest];
    return (
      <div className="details">
        <h2>{s.title}</h2>
        <p>{s.purpose}</p>
        {view.kind !== 'screen' || view.id !== s.id ? <button className="link" onClick={() => setView({ kind: 'screen', id: s.id })}>Open this screen →</button> : null}
        <div className="shots">{s.states.map((st) => <a key={st} href={`refs/${st}.webp`} target="_blank" rel="noreferrer"><img src={`refs/${st}.webp`} alt={st} title={st} /></a>)}</div>
        <h3>Regions</h3>
        <ul>{s.regions.map((r) => <li key={r.name}><b>{r.name}:</b> {r.contents}</li>)}</ul>
        <h3>Widgets ({s.widgets.length})</h3>
        <ul className="compact">{s.widgets.map((w) => <li key={w.key}><span className="tag">{w.kind}</span> {w.variant || w.component} <span className="muted">×{w.count}</span></li>)}</ul>
        <h3>Leads to</h3>
        <ul className="compact">{s.links.filter((l) => data.byId[l.to]).map((l, i) => <li key={i}><button className="link" onClick={() => setView({ kind: 'screen', id: l.to })}>{data.byId[l.to].title}</button> <span className="muted">via {l.via}</span></li>)}</ul>
        <h3>Features</h3>
        {s.areas.map((a) => (
          <details key={a.n}>
            <summary>{a.n}. {a.name} <span className="muted">({a.count})</span></summary>
            <ul className="compact">{data.areas[a.n].features.map((f) => <li key={f.id}><span className="tag">{f.id}</span> {f.title}</li>)}</ul>
          </details>
        ))}
      </div>
    );
  }
  if (kind === 'use' && sel.data.widget) {
    const w = sel.data.widget;
    return (
      <div className="details">
        <h2><span className="tag">{w.kind}</span> {w.variant || w.component}</h2>
        <p>{w.purpose}</p>
        <dl>
          <dt>Component</dt><dd>{w.component}</dd>
          <dt>Text</dt><dd>{w.text || '—'}</dd>
          <dt>States</dt><dd>{w.states.join(', ') || '—'}</dd>
          <dt>Behaviour</dt><dd>{w.behaviour}</dd>
          <dt>Look</dt><dd>{w.look || '—'}</dd>
          <dt>Region</dt><dd>{w.region} · ×{w.count}</dd>
        </dl>
        <button className="link" onClick={() => setView({ kind: 'widget', id: data.familyOf[w.kind] ?? 'structure' })}>This widget family across the app →</button>
      </div>
    );
  }
  if (kind === 'widget' || kind === 'variant') {
    const k = kind === 'widget' ? rest : rest.split('/')[0];
    const fam = data.catalogue.find((f) => f.kind === k);
    const v = sel.data.variant;
    return (
      <div className="details">
        <h2>{v ? v.name : fam?.title ?? k}</h2>
        <p>{v ? v.description : fam?.description}</p>
        {v && <dl><dt>Looks</dt><dd>{v.look || '—'}</dd><dt>States</dt><dd>{(v.states ?? []).join(', ') || '—'}</dd><dt>Behaviour</dt><dd>{v.behaviour || '—'}</dd><dt>Used in</dt><dd>{(v.usedIn ?? []).map((id) => data.byId[id]?.title ?? id).join(', ')}</dd></dl>}
        {!v && fam && <ul className="compact">{fam.variants.map((x) => <li key={x.name}>{x.name} <span className="muted">in {(x.usedIn ?? []).length} screens</span></li>)}</ul>}
      </div>
    );
  }
  if (kind === 'link') {
    const [a, b] = rest.split('>');
    return (
      <div className="details">
        <h2>{data.byId[a]?.title} → {data.byId[b]?.title}</h2>
        <ul>{(sel.data.via ?? []).map((v, i) => <li key={i}>{v}</li>)}</ul>
      </div>
    );
  }
  return null;
}

export default function App() {
  const [data, setData] = useState(null);
  const [comments, setComments] = useState([]);
  const [view, setView] = useState({ kind: 'app' });
  const [sel, setSel] = useState(null);
  const [filter, setFilter] = useState('');
  const load = useCallback(() => fetch('api/comments').then((r) => (r.ok ? r.json() : [])).then(setComments).catch(() => {}), []);
  useEffect(() => {
    fetch('data.json').then((r) => r.json()).then((d) => setData({ ...d, byId: Object.fromEntries(d.screens.map((s) => [s.id, s])) }));
    load();
  }, [load]);
  const counts = useMemo(() => {
    const c = {};
    for (const x of comments) if (!x.resolved) c[x.node] = (c[x.node] ?? 0) + 1;
    return c;
  }, [comments]);
  if (!data) return <div className="loading">Loading…</div>;
  const crumbs = [{ label: 'App', v: { kind: 'app' } }];
  if (view.kind === 'screen') crumbs.push({ label: data.byId[view.id].title, v: view });
  if (view.kind === 'widget') crumbs.push({ label: `Widget: ${view.id}`, v: view });
  return (
    <div className="shell">
      <header>
        <strong>Taskscape map</strong>
        <nav>{crumbs.map((c, i) => <button key={i} className={i === crumbs.length - 1 ? 'crumb on' : 'crumb'} onClick={() => setView(c.v)}>{c.label}</button>)}</nav>
        <select value={view.kind === 'widget' ? view.id : ''} onChange={(e) => e.target.value && setView({ kind: 'widget', id: e.target.value })}>
          <option value="">Widgets…</option>
          {data.catalogue.map((f) => <option key={f.kind} value={f.kind}>{f.title} ({f.variants.length})</option>)}
        </select>
        <input placeholder="Highlight…" value={filter} onChange={(e) => setFilter(e.target.value)} />
        <span className="muted small">{data.screens.length} screens · {data.catalogue.reduce((n, f) => n + f.variants.length, 0)} widget variants · {comments.filter((c) => !c.resolved).length} open comments</span>
      </header>
      <main>
        <ReactFlowProvider>
          <Canvas focusId={sel?.node?.startsWith('screen:') ? sel.node.slice(7) : null} data={data} view={view} setView={(v) => { setView(v); setSel(v.kind === 'screen' ? { node: key.screen(v.id), data: {} } : null); }} select={setSel} counts={counts} filter={filter} />
        </ReactFlowProvider>
      </main>
      <aside>
        <Details data={data} sel={sel} view={view} setView={(v) => { setView(v); if (v.kind === 'screen') setSel({ node: key.screen(v.id), data: {} }); }} />
        {sel && <Comments node={sel.node} comments={comments} reload={load} />}
        {!sel && <AllComments comments={comments} data={data} pick={(node) => setSel({ node, data: {} })} />}
      </aside>
    </div>
  );
}

function AllComments({ comments, pick }) {
  const open = comments.filter((c) => !c.resolved && !c.parent);
  return (
    <div className="all">
      <h3>Open comments ({open.length})</h3>
      <ul className="compact">{open.slice(-40).reverse().map((c) => <li key={c.id}><button className="link" onClick={() => pick(c.node)}>{c.node}</button> <span className={`chip ${c.tag}`}>{c.tag}</span> {c.text.slice(0, 120)}</li>)}</ul>
    </div>
  );
}
