export const meta = {
  name: 'taskscape-inventory',
  description: 'Screen inventory: solver A from code and component trees, solver B from screenshots and measurements only, a critic on high-tier screens, then a merger',
  phases: [{ title: 'Solve' }, { title: 'Critique' }, { title: 'Merge' }],
}

// args: { refs, screenList: "id: name; …", screens: [{ id, name, tier, files, states }] }
const TYPE = args.agentTypes ?? { solver: 'ts-solver', critic: 'ts-critic', merger: 'ts-merger' }
const OPTS = { model: 'haiku', effort: 'medium' }

const KINDS = ['button', 'icon-button', 'toolbar-button', 'key-action', 'link', 'tab', 'segmented-control', 'text-field',
  'search-field', 'rich-text-editor', 'checkbox', 'round-check', 'toggle', 'select', 'menu', 'menu-item', 'context-menu',
  'dropdown', 'sheet', 'dialog', 'toast', 'popover', 'tooltip', 'badge', 'keycap', 'hotkey-hint', 'progress-bar',
  'spinner', 'empty-state', 'list-row', 'task-row', 'section-header', 'label', 'divider', 'surface', 'title-bar',
  'window-controls', 'nav-item', 'setting-row', 'settings-group', 'thumbnail', 'lightbox', 'icon', 'other']

const WIDGET = {
  type: 'object',
  properties: {
    kind: { type: 'string', enum: KINDS, description: 'The generic kind of control.' },
    component: { type: 'string', description: 'The component name in the code, or "?" if not known.' },
    variant: { type: 'string', description: 'What sets this use apart: the props or visible differences (size, tone, filled, icon-only, danger, active...).' },
    purpose: { type: 'string', description: 'What it is for on this screen.' },
    text: { type: 'string', description: 'Its label, icon name, placeholder and tooltip, exactly.' },
    states: { type: 'array', items: { type: 'string' }, description: 'States it shows: hover, pressed, active, selected, disabled, focus, open, error, empty...' },
    behaviour: { type: 'string', description: 'What using it does: click, keys, typing, drag.' },
    look: { type: 'string', description: 'Measured look where known: size, colours, font, radius, spacing.' },
    count: { type: 'integer', description: 'How many times it appears on the screen.' },
    region: { type: 'string' },
    sources: { type: 'array', items: { type: 'string' } },
  },
  required: ['kind', 'component', 'variant', 'purpose', 'text', 'states', 'behaviour', 'count', 'region', 'sources'],
}
const SCREEN = {
  type: 'object',
  properties: {
    purpose: { type: 'string', description: 'What the screen is for, in one or two sentences.' },
    regions: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, contents: { type: 'string' } }, required: ['name', 'contents'] } },
    widgets: { type: 'array', items: WIDGET },
    links: {
      type: 'array',
      items: { type: 'object', properties: { to: { type: 'string', description: 'A screen id from the list.' }, via: { type: 'string', description: 'The control or gesture.' } }, required: ['to', 'via'] },
    },
    notes: { type: 'string' },
  },
  required: ['purpose', 'regions', 'widgets', 'links'],
}
const CRITIQUE = {
  type: 'object',
  properties: {
    issues: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          kind: { type: 'string', enum: ['only_in_A', 'only_in_B', 'conflict', 'missed_by_both', 'wrong_kind', 'wrong_variant', 'wrong_link'] },
          about: { type: 'string' },
          verdict: { type: 'string' },
          evidence: { type: 'string' },
        },
        required: ['kind', 'about', 'verdict', 'evidence'],
      },
    },
  },
  required: ['issues'],
}

const COMMON = (s) => `Screen "${s.id}": ${s.name}.

The app's screens, for links (id: name): ${args.screenList}.

List every UI element the screen shows or can show, one entry per distinct widget and variant (a group of identical controls is one entry with a count). Include plain elements styled as controls and name them by what they are. For each: its kind, its variant, what it is for here, its exact text (label, icon, placeholder, tooltip), the states it shows, what using it does, how many there are, the region it sits in, and sources. Also give the screen's purpose, its regions, and every link to another screen with the control or gesture that leads there. Be exhaustive and exact.`

const solverA = (s) => `You inventory one screen of a desktop task-manager app (the "reference") from its source code and React component trees. ${COMMON(s)}

Read ONLY these, completely:
${s.files.map((f) => `- ${f}`).join('\n')}
and the component tree of each recorded state of this screen (tree.txt: component names, props, on-screen boxes):
${s.states.map((st) => `- ${args.refs}/${st}/tree.txt`).join('\n')}
Name each widget's component as the code does, and take its variant from its props. Cite file:line or state:<id>.`

const solverB = (s) => `You inventory one screen of a desktop task-manager app (the "reference") from what it looks like. You do NOT read its source code or component trees. ${COMMON(s)}

Open ONLY these recordings of the running app. Each folder holds shot.png (an image of the window; open it) and elements.json (every visible control and text: role, label, text, state, box [x,y,w,h], colour, background, font, radius):
${s.states.map((st) => `- ${args.refs}/${st}/ (state "${st}")`).join('\n')}
Name widgets by what they look like and do; component is "?". Put measured colours, sizes, fonts and radii in "look". Cite state:<id>.`

const critic = (s, A, B) => `You are the critic for the inventory of screen "${s.id}" (${s.name}) of a desktop task-manager app. A worked from the code and component trees; B from screenshots and measurements only.

A: ${JSON.stringify(A)}

B: ${JSON.stringify(B)}

Find widgets only one of them has, conflicting kinds or variants, wrong links, and widgets both missed. Settle each against the sources; you may open only:
${[...s.files, ...s.states.map((st) => `${args.refs}/${st}/`)].map((f) => `- ${f}`).join('\n')}`

const merger = (s, A, B, C) => `You merge two independent inventories of screen "${s.id}" (${s.name}) of a desktop task-manager app into one. A worked from the code (its component names and props are authoritative for component and variant); B from screenshots and measurements (its measured look values are authoritative for look).${C ? ' A critic checked both; its verdicts take precedence.' : ''}

A: ${JSON.stringify(A)}

B: ${JSON.stringify(B)}
${C ? `\nCritique: ${JSON.stringify(C.issues)}\n` : ''}
Rules: one entry per distinct widget and variant; match A's and B's entries for the same widget and combine them (A's component and variant, B's look, both sources); keep anything only one found unless shown wrong; union the links (screen ids from the list only) and regions; write the purpose in one or two plain sentences. Keep strings exact.`

// A solver that answers without reading returns almost nothing; give it one more try.
const solve = async (prompt, opts) => {
  const r = await agent(prompt, opts)
  if (r && r.widgets.length >= 3) return r
  return agent(`${prompt}\n\nOpen every listed file before answering; this screen has many widgets.`, { ...opts, label: `${opts.label} (retry)` })
}

const results = await pipeline(
  args.screens,
  (s) =>
    parallel([
      () => solve(solverA(s), { ...OPTS, agentType: TYPE.solver, schema: SCREEN, label: `A ${s.id}`, phase: 'Solve' }),
      () => solve(solverB(s), { ...OPTS, agentType: TYPE.solver, schema: SCREEN, label: `B ${s.id}`, phase: 'Solve' }),
    ]),
  async ([A, B], s) => {
    if (!A || !B || s.tier !== 'high') return { A, B, C: null }
    const C = await agent(critic(s, A, B), { ...OPTS, agentType: TYPE.critic, schema: CRITIQUE, label: `critic ${s.id}`, phase: 'Critique' })
    return { A, B, C }
  },
  async ({ A, B, C }, s) => {
    if (!A || !B) return { id: s.id, failed: true, A, B }
    const M = await agent(merger(s, A, B, C), { ...OPTS, agentType: TYPE.merger, schema: SCREEN, label: `merge ${s.id}`, phase: 'Merge' })
    return { id: s.id, name: s.name, tier: s.tier, counts: { A: A.widgets.length, B: B.widgets.length, issues: C ? C.issues.length : null }, merged: M }
  }
)
return results
