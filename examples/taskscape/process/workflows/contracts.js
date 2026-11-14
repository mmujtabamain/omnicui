export const meta = {
  name: 'taskscape-contracts',
  description: 'Widget contracts: two web-only writers draft each widget contract independently, a critic on high-value widgets, then a merger writes one contract',
  phases: [{ title: 'Write' }, { title: 'Critique' }, { title: 'Merge' }],
}

// args: { formatFile, inputs, widgets: [{ name, id, high, file }] }: file holds the widget's library
// entry (variants, props, states, role, keyboard, platform); formatFile is omnicui's contract
// reference. The writers have no file access, so a read-only relay hands each file to them.
const TYPE = args.agentTypes ?? { relay: 'ts-solver', writer: 'ts-writer', critic: 'ts-writer', merger: 'ts-merger' }
const OPTS = { model: 'haiku', effort: 'high' }

const CONTRACT = {
  type: 'object',
  properties: {
    yaml: { type: 'string', description: 'The whole contract file, valid YAML, in the format given.' },
    notes: { type: 'string', description: 'What you based it on (references used), and anything left out on purpose.' },
  },
  required: ['yaml', 'notes'],
}
const CRITIQUE = {
  type: 'object',
  properties: { issues: { type: 'array', items: { type: 'object', properties: { about: { type: 'string' }, problem: { type: 'string' }, fix: { type: 'string' } }, required: ['about', 'problem', 'fix'] } } },
  required: ['issues'],
}

const RELAY = { type: 'object', properties: { text: { type: 'string', description: 'The file\'s whole content, exactly as written.' } }, required: ['text'] }
const relay = (file, label) => agent(`Read ${file} and return its whole content exactly as written, with nothing added, left out or reworded. Open nothing else.`, { model: 'haiku', effort: 'low', agentType: TYPE.relay, schema: RELAY, label, phase: 'Write' }).then((r) => r?.text)

const RULES = `Hard rules of this omnicui version (a contract that breaks one is refused):
- Part roles are only: button, switch, checkbox, slider, textbox, status, heading, generic. Map every other ARIA role onto them: option, tab, menuitem, treeitem, link to button; listbox, list, listitem, toolbar, menu, tree, tablist, dialog, banner, img, group to generic. Put the real ARIA role in a YAML comment on the part (# aria: option), and test it through the fields the format allows.
- A prop has only type and default. No description, enum or other fields; explain allowed values in a comment.
- Only the fields, actions and roles the format lists. Names camelCase. The file must be valid YAML (comments start with #, never //): quote any string that holds a colon.
- Every step with an action names its part, key presses included ({ action: key, part: field, key: Escape }).
- An event's last values must have the field's declared type: a string field gets a plain string, never { contains: … } or { matches: … } (those work only on part text and name).`

const writer = (w, lens) => `You write the behaviour contract for one widget of a UI library, in the omnicui contract format. Two implementations (a React web one and a native desktop one) will be checked against it, so it must say what an ideal, idiomatic widget of this kind does, not how anything was built before.

The widget, ${w.name} (contract id "${w.id}"):
${w.spec}

Named inputs available in steps (platforms.yaml): ${args.inputs}

The contract format (omnicui's own reference):
${FORMAT}

${RULES}
${DECISIONS ? `\nProduct decisions it must follow:\n${DECISIONS}\n` : ''}
Write one contract: props for its variants and states, the events it emits, its parts with roles, and tests that cover every variant, state and keyboard behaviour in the spec. Use keyboard: true where Enter/Space must do what a press does. Add screenshot tests for each variant at light and dark. Keep names camelCase. Research idiomatic behaviour on the web (WAI-ARIA Authoring Practices for the role, Radix and shadcn/ui docs for the component) and follow it where the spec is silent.

Your lens: ${lens}`

const critic = (w, A, B) => `You check two independent drafts of the omnicui contract for the widget ${w.name}.

The widget: ${w.spec}

Draft A:
${A.yaml}

Draft B:
${B.yaml}

The format: ${FORMAT}

Find tests one draft has and the other lacks, behaviour that contradicts WAI-ARIA practice for its role, format mistakes (unknown fields, roles or actions; names that are not camelCase), variants or states the spec lists that neither tests, and tests that cannot be measured. You may check WAI-ARIA and Radix docs on the web.`

const merger = (w, A, B, C) => `You merge two independent drafts of the omnicui contract for the widget ${w.name} (id "${w.id}") into one valid contract.${C ? ' A critic checked both; apply its fixes.' : ''}

Draft A:
${A.yaml}

Draft B:
${B.yaml}
${C ? `\nCritique: ${JSON.stringify(C.issues)}\n` : ''}
The format: ${FORMAT}

Rules: one set of props, events and parts (pick the clearer names and use them everywhere); the union of the tests, with duplicates removed; every field, role and action must be one the format lists. Return the whole contract as YAML.

${RULES}`

const FORMAT = await relay(args.formatFile, 'relay format')
if (!FORMAT) throw new Error('could not read the contract format')
const DECISIONS = args.decisionsFile ? await relay(args.decisionsFile, 'relay decisions') : args.decisions
const results = await pipeline(
  args.widgets,
  async (w0) => ({ ...w0, spec: await relay(w0.file, `relay ${w0.name}`) }),
  async (w) => {
    if (!w.spec) return { w, A: null, B: null }
    const [A, B] = await parallel([
      () => agent(writer(w, 'start from the WAI-ARIA pattern for its role, then fit the spec to it.'), { ...OPTS, agentType: TYPE.writer, schema: CONTRACT, label: `A ${w.name}`, phase: 'Write' }),
      () => agent(writer(w, 'start from the spec and the shadcn/Radix component it follows, then check it against WAI-ARIA.'), { ...OPTS, agentType: TYPE.writer, schema: CONTRACT, label: `B ${w.name}`, phase: 'Write' }),
    ])
    return { w, A, B }
  },
  async ({ w, A, B }) => {
    if (!A || !B || !w.high) return { w, A, B, C: null }
    const C = await agent(critic(w, A, B), { ...OPTS, agentType: TYPE.critic, schema: CRITIQUE, label: `critic ${w.name}`, phase: 'Critique' })
    return { w, A, B, C }
  },
  async ({ w, A, B, C }) => {
    if (!A || !B) return { name: w.name, failed: true }
    const M = await agent(merger(w, A, B, C), { ...OPTS, agentType: TYPE.merger, schema: CONTRACT, label: `merge ${w.name}`, phase: 'Merge' })
    return { name: w.name, id: w.id, contract: M, issues: C ? C.issues.length : null }
  }
)
return results
