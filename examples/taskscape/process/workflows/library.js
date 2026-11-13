export const meta = {
  name: 'taskscape-library',
  description: 'Widget library design per catalogue family: designer A keeps the reference distinctions, designer B consolidates as shadcn would, a critic on high-value families, then a merger settles the widgets and their variant axes',
  phases: [{ title: 'Design' }, { title: 'Critique' }, { title: 'Merge' }],
}

// args: { brief, decisions, families: [{ id, title, file, high }] }
const TYPE = args.agentTypes ?? { solver: 'ts-solver', critic: 'ts-critic', merger: 'ts-merger' }
const OPTS = { model: 'haiku', effort: 'high' }

const WIDGET = {
  type: 'object',
  properties: {
    name: { type: 'string', description: 'PascalCase component name, e.g. Button, IconButton, MenuItem.' },
    purpose: { type: 'string' },
    base: { type: 'string', description: 'The shadcn/ui or Radix primitive it follows, or "none".' },
    variants: {
      type: 'array',
      description: 'Variant axes, shadcn style (cva): e.g. variant: default|ghost|danger, size: sm|md|lg.',
      items: {
        type: 'object',
        properties: {
          axis: { type: 'string' },
          values: { type: 'array', items: { type: 'object', properties: { value: { type: 'string' }, when: { type: 'string', description: 'When to use it, and its look.' } }, required: ['value', 'when'] } },
          default: { type: 'string' },
        },
        required: ['axis', 'values', 'default'],
      },
    },
    props: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, type: { type: 'string' }, description: { type: 'string' } }, required: ['name', 'type', 'description'] } },
    states: { type: 'array', items: { type: 'string' } },
    role: { type: 'string', description: 'The ARIA role it exposes.' },
    keyboard: { type: 'string', description: 'Its keyboard behaviour.' },
    replaces: { type: 'array', items: { type: 'string' }, description: 'The catalogue variant names this widget covers, exactly as written in the catalogue.' },
    platform: { type: 'string', description: 'Where web and desktop differ for this widget, or "same".' },
    open: { type: 'string', description: 'Anything a product decision must settle, or "".' },
  },
  required: ['name', 'purpose', 'base', 'variants', 'props', 'states', 'role', 'keyboard', 'replaces', 'platform', 'open'],
}
const DESIGN = {
  type: 'object',
  properties: {
    widgets: { type: 'array', items: WIDGET },
    notWidgets: { type: 'array', items: { type: 'object', properties: { variant: { type: 'string' }, reason: { type: 'string' } }, required: ['variant', 'reason'] }, description: 'Catalogue variants that are not library widgets (layout wrappers, screen-specific compositions, things dropped on both versions), with why.' },
  },
  required: ['widgets', 'notWidgets'],
}
const CRITIQUE = {
  type: 'object',
  properties: { issues: { type: 'array', items: { type: 'object', properties: { about: { type: 'string' }, problem: { type: 'string' }, fix: { type: 'string' } }, required: ['about', 'problem', 'fix'] } } },
  required: ['issues'],
}

const COMMON = (f) => `You design part of a widget library for the rebuild of a task-manager app. The library is a separate package built the way shadcn/ui is: one component per file, variants through cva, Radix primitives where shadcn uses them, design tokens as CSS variables. The apps may use nothing else for UI: no raw buttons, inputs or menus.

The rebuild: ${args.brief}
Decisions already made: ${args.decisions}

Family "${f.title}". Read ${f.file} first and only that file: it is the family's catalogue, every distinct widget variant found in the reference app, with its look, states, behaviour and the screens that use it.

Turn it into library widgets: name each component, give it variant axes (not one variant per catalogue row), its props, states, ARIA role and keyboard behaviour, and list in replaces the exact catalogue variant names it covers. Every catalogue variant must be covered by a widget or listed in notWidgets with a reason. Note where web and desktop differ.`

const designerA = (f) => `${COMMON(f)}

Your lens: keep every distinction the reference makes that a user can see or feel (a size, a tone, a behaviour). Merge two catalogue variants only when they really are the same control.`

const designerB = (f) => `${COMMON(f)}

Your lens: consolidate as shadcn/ui would. Prefer few components with clear variant axes; fold one-off differences (an odd icon weight, a stray radius) into the nearest standard variant, and say what you folded.`

const critic = (f, A, B) => `You check two independent designs of the "${f.title}" family of a shadcn-style widget library for a task-manager app's rebuild. A kept the reference's distinctions; B consolidated.

A: ${JSON.stringify(A)}

B: ${JSON.stringify(B)}

The family catalogue they worked from is ${f.file}; open only that. Find catalogue variants neither covers, widgets that merge controls with different behaviour, variant axes that are missing or pointless, and names or roles that do not match what the control does.`

const merger = (f, A, B, C) => `You settle the "${f.title}" family of a shadcn-style widget library for a task-manager app's rebuild, from two independent designs.${C ? ' A critic checked both against the catalogue; apply its fixes.' : ''}

A (kept the reference's distinctions): ${JSON.stringify(A)}

B (consolidated as shadcn would): ${JSON.stringify(B)}
${C ? `\nCritique: ${JSON.stringify(C.issues)}\n` : ''}
Rules: take B's consolidation where A's distinction is not visible or felt by a user; keep A's distinction where it is. One component per job; variant axes, not copies. The union of both designs' replaces and notWidgets must all appear in your answer, so no catalogue variant is lost. Keep catalogue variant names exactly as written.`

const results = await pipeline(
  args.families,
  (f) =>
    parallel([
      () => agent(designerA(f), { ...OPTS, agentType: TYPE.solver, schema: DESIGN, label: `A ${f.id}`, phase: 'Design' }),
      () => agent(designerB(f), { ...OPTS, agentType: TYPE.solver, schema: DESIGN, label: `B ${f.id}`, phase: 'Design' }),
    ]),
  async ([A, B], f) => {
    if (!A || !B || !f.high) return { A, B, C: null }
    const C = await agent(critic(f, A, B), { ...OPTS, agentType: TYPE.critic, schema: CRITIQUE, label: `critic ${f.id}`, phase: 'Critique' })
    return { A, B, C }
  },
  async ({ A, B, C }, f) => {
    if (!A || !B) return { id: f.id, failed: true }
    const M = await agent(merger(f, A, B, C), { ...OPTS, agentType: TYPE.merger, schema: DESIGN, label: `merge ${f.id}`, phase: 'Merge' })
    return { id: f.id, design: M, counts: { A: A.widgets.length, B: B.widgets.length, issues: C ? C.issues.length : null } }
  }
)
return results
