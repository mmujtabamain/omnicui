import ELK from 'elkjs/lib/elk.bundled.js';

const elk = new ELK();

// Lays nodes out left to right with ELK's layered algorithm; returns React Flow positions.
export async function layout(nodes, edges, { direction = 'RIGHT', spacing = 40 } = {}) {
  const graph = {
    id: 'root',
    layoutOptions: {
      'elk.algorithm': 'layered',
      'elk.direction': direction,
      'elk.layered.spacing.nodeNodeBetweenLayers': String(spacing * 2),
      'elk.spacing.nodeNode': String(spacing),
      'elk.layered.nodePlacement.strategy': 'BRANDES_KOEPF',
      'elk.edgeRouting': 'SPLINES',
    },
    children: nodes.map((n) => ({ id: n.id, width: n.width ?? 240, height: n.height ?? 90 })),
    edges: edges.map((e) => ({ id: e.id, sources: [e.source], targets: [e.target] })),
  };
  const out = await elk.layout(graph);
  const pos = Object.fromEntries(out.children.map((c) => [c.id, { x: c.x, y: c.y }]));
  return nodes.map((n) => ({ ...n, position: pos[n.id] ?? { x: 0, y: 0 } }));
}
