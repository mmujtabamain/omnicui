export const meta = {
  name: 'taskscape-catalogue',
  description: 'Widget catalogue: one merger per widget family across every screen, a critic on the high-value families, then a fixer applies the critique',
  phases: [{ title: 'Merge' }, { title: 'Critique' }, { title: 'Fix' }],
}

// args: { families: [{ id, title, kinds, file, high }] }: file holds every widget row of the family, from every screen
const TYPE = args.agentTypes ?? { solver: 'ts-solver', critic: 'ts-critic', merger: 'ts-merger' }
const OPTS = { model: 'haiku', effort: 'medium' }

const FAMILY = {
  type: 'object',
  properties: {
    title: { type: 'string' },
    description: { type: 'string', description: 'What this family of widgets is, in one or two sentences.' },
    variants: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          name: { type: 'string', description: 'A short, distinct name, e.g. "icon button, plain, small".' },
          kind: { type: 'string' },
          component: { type: 'string' },
          description: { type: 'string', description: 'What sets it apart and when it is used.' },
          look: { type: 'string', description: 'Size, colours, font, radius, from the measured rows.' },
          states: { type: 'array', items: { type: 'string' } },
          behaviour: { type: 'string' },
          usedIn: { type: 'array', items: { type: 'string' }, description: 'Screen ids.' },
          examples: { type: 'array', items: { type: 'string' }, description: 'A few exact labels, icons or tooltips it carries.' },
        },
        required: ['name', 'kind', 'component', 'description', 'look', 'states', 'behaviour', 'usedIn', 'examples'],
      },
    },
  },
  required: ['title', 'description', 'variants'],
}
const CRITIQUE = {
  type: 'object',
  properties: { issues: { type: 'array', items: { type: 'object', properties: { variant: { type: 'string' }, problem: { type: 'string' }, fix: { type: 'string' } }, required: ['variant', 'problem', 'fix'] } } },
  required: ['issues'],
}

const merge = (f) => `You build one family of a widget catalogue for a desktop task-manager app: "${f.title}" (kinds: ${f.kinds.join(', ')}).

Read ${f.file} first and only that file. It lists every widget of this family found on every screen of the app, one row per screen and widget, with its component name, variant, text, states, behaviour, measured look and screen.

Group the rows into distinct variants: rows that are the same widget with the same look and behaviour are one variant, used on several screens; rows that differ in a way a designer would name (size, tone, icon-only vs labelled, filled vs plain, danger) are separate variants. Do not merge widgets that behave differently. Name each variant plainly. Every row must land in exactly one variant; list the screens in usedIn.`

const critic = (f, M) => `You check one family of a widget catalogue for a desktop task-manager app: "${f.title}".

The catalogue entry:
${JSON.stringify(M)}

The rows it was built from are in ${f.file}; open only that file. Find rows that landed in no variant, variants that should be split (different behaviour or look) or merged (no real difference), wrong usedIn screens, and vague names or looks.`

const fix = (f, M, C) => `You correct one family of a widget catalogue for a desktop task-manager app: "${f.title}".

The entry:
${JSON.stringify(M)}

A critic checked it against the source rows; apply every fix below and return the whole corrected entry.
${JSON.stringify(C.issues)}`

const results = await pipeline(
  args.families,
  (f) => agent(merge(f), { ...OPTS, agentType: TYPE.solver, schema: FAMILY, label: `catalogue ${f.id}`, phase: 'Merge' }),
  async (M, f) => {
    if (!M || !f.high) return { M, C: null }
    const C = await agent(critic(f, M), { ...OPTS, agentType: TYPE.critic, schema: CRITIQUE, label: `critic ${f.id}`, phase: 'Critique' })
    return { M, C }
  },
  async ({ M, C }, f) => {
    if (!M) return { id: f.id, failed: true }
    if (!C || !C.issues.length) return { id: f.id, entry: M, issues: 0 }
    const F = await agent(fix(f, M, C), { ...OPTS, agentType: TYPE.merger, schema: FAMILY, label: `fix ${f.id}`, phase: 'Fix' })
    return { id: f.id, entry: F ?? M, issues: C.issues.length }
  }
)
return results
