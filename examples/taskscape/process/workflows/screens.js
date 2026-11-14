export const meta = {
  name: 'taskscape-screen-contracts',
  description: 'Screen contracts: two read-only writers draft each web screen\'s contract from the inventory, features and decisions (one by states, one by features), a critic on high-value screens, then a merger',
  phases: [{ title: 'Write' }, { title: 'Critique' }, { title: 'Merge' }],
}

// args: { root, screens: [{ id, title, high, inventory?, features?, notes?, brief? }] }
//   inventory/features/notes: files under spec/; brief: text for a screen with no inventory (sign-in).
const OPTS = { model: 'haiku', effort: 'high' }
const CONTRACT = {
  type: 'object',
  properties: {
    yaml: { type: 'string', description: 'The whole contract file, valid YAML, in the omnicui format.' },
    notes: { type: 'string', description: 'What it is based on, the harness props it needs and how the harness must interpret them, and what was left out on purpose (with the reason).' },
  },
  required: ['yaml', 'notes'],
}
const CRITIQUE = {
  type: 'object',
  properties: { issues: { type: 'array', items: { type: 'object', properties: { about: { type: 'string' }, problem: { type: 'string' }, fix: { type: 'string' } }, required: ['about', 'problem', 'fix'] } } },
  required: ['issues'],
}
const R = args.root

const RULES = `Hard rules of this omnicui version (a contract that breaks one is refused):
- Part roles are only: button, switch, checkbox, slider, textbox, status, heading, generic. Map every other ARIA role onto them (option, tab, menuitem, treeitem, link → button; listbox, list, listitem, toolbar, menu, tree, tablist, dialog, img, group → generic) and put the real role in a comment (# aria: tab).
- A prop has only type and default; explain allowed values in a comment. Names camelCase. Valid YAML; comments start with #; quote any string with a colon.
- Every step with an action names its part, key presses included ({ action: key, part: search, key: Escape }).
- drag acts only on slider parts: leave drag-and-drop out of the tests (note it in a comment).
- trace and record need steps that start the motion.
- Every test has at least one expect, screenshot, trace or record.
- An event's last values have the declared type (a plain string for a string field).
- Named inputs available in steps: selectToggle, modifierHold, confirm, dismiss, toggleBold, toggleItalic, toggleUnderline, submit.`

const SCREEN = `How a screen contract works here:
- id: "app.<screen id>". The screen is one React component of the web app, mounted alone by a harness, signed in as the demo account on the real API with one demo data set (spec/screens/web.md lists the data).
- Props set where the screen starts: always \`dataset\` (string, default standard), plus the few context props the screen needs to be mounted alone, by name, never by id (for example \`list: "Errands"\`, \`task: "Rewrite the hero section"\`, \`open: "rename"\`). Say in notes exactly what each prop makes the harness do.
- Parts are the screen's regions and controls, named by what they are (search, newTask, row, footer, doneCount), found by data-part="app.<screen id>.<part>". When several elements share a part (rows), the first one is the part; name a specific one as its own part only when a test needs it (rowGroceries).
- Tests reach every state of the screen the way a user would (press, type, key, named inputs) and check it: visible text, names, roles, checked/selected/focused, and that data changed (a renamed task shows its new title; a deleted one is gone). Each reference state gets a screenshot test.
- viewports: the size the screen has in the app (the inventory measures it); colorSchemes [light, dark] where the reference has a dark state, else [light].
- Leave out anything the web drops (spec/screens/web.md) and follow every decision in spec/decisions.md; where the web differs, test the web behaviour.`

const reads = (s) => [
  ...(s.inventory ? [`${R}/spec/inventory/${s.inventory} (the screen inventory: regions, widgets, states, measurements)`] : []),
  ...(s.features ? [`${R}/spec/inventory/${s.features} (the screen's feature titles with ids; Grep ${R}/spec/features/area-*.json for a feature's full record when you need its detail)`] : []),
  ...(s.notes ? [`${R}/spec/screens/${s.notes} (the team's review notes on this screen and how each was settled)`] : []),
  `${R}/spec/screens/web.md (what the web version drops or changes, and the demo data)`,
  `${R}/spec/decisions.md (product decisions; they override the reference)`,
  `${R}/spec/library/contract-format.txt (the omnicui contract format)`,
  `${R}/spec/library/index.txt (the widgets the screen is built from)`,
].map((l) => `- ${l}`).join('\n')

const writer = (s, lens) => `You write the behaviour contract for one screen of a task manager's web app, ${s.title} (id "app.${s.id}"), in the omnicui contract format. The screen will be rebuilt from this contract and checked against it, so it must state exactly what the screen shows and does.
${s.brief ? `\nThe screen: ${s.brief}\n` : ''}
Read only these files:
${reads(s)}

${SCREEN}

${RULES}

Your lens: ${lens}
Keep your work within budget: read each file once; do not open source code.`

const critic = (s, A, B) => `You check two independent drafts of the omnicui contract for the screen ${s.title} (id "app.${s.id}").

Draft A:
${A.yaml}
Notes A: ${A.notes}

Draft B:
${B.yaml}
Notes B: ${B.notes}

You may read: ${s.features ? `${R}/spec/inventory/${s.features}, ` : ''}${R}/spec/screens/web.md, ${R}/spec/decisions.md, ${R}/spec/library/contract-format.txt.

Report: states or features one draft tests and the other misses; features from the feature list that neither covers and that a test could check; tests that contradict a decision or test something the web drops; format mistakes; props whose harness meaning is unclear; tests that could not be measured.

${RULES}`

const merger = (s, A, B, C) => `You merge two independent drafts of the omnicui contract for the screen ${s.title} (id "app.${s.id}") into one valid contract.${C ? ' A critic checked both; apply its fixes.' : ''}

Draft A:
${A.yaml}
Notes A: ${A.notes}

Draft B:
${B.yaml}
Notes B: ${B.notes}
${C ? `\nCritique: ${JSON.stringify(C.issues)}\n` : ''}
Rules: one set of props, events and parts (the clearer names, used everywhere); the union of the tests with duplicates removed; every field, role and action one the format allows. In notes, give the merged harness meaning of every prop. Return the whole contract.

${SCREEN}

${RULES}`

const retry = async (fn) => (await fn()) ?? (await fn())
return await pipeline(
  args.screens,
  async (s) => {
    const [A, B] = await parallel([
      () => retry(() => agent(writer(s, 'states first: walk every state the inventory lists and every state a user can reach from them, and test each.'), { ...OPTS, agentType: 'ts-solver', schema: CONTRACT, label: `A ${s.id}`, phase: 'Write' })),
      () => retry(() => agent(writer(s, 'features first: walk the feature list and turn every feature a test can observe into a test.'), { ...OPTS, agentType: 'ts-solver', schema: CONTRACT, label: `B ${s.id}`, phase: 'Write' })),
    ])
    return { A, B }
  },
  async ({ A, B }, s) => (A && B && s.high ? { A, B, C: await agent(critic(s, A, B), { ...OPTS, agentType: 'ts-critic', schema: CRITIQUE, label: `critic ${s.id}`, phase: 'Critique' }) } : { A, B, C: null }),
  async ({ A, B, C }, s) => {
    if (!A || !B) return { name: s.id, failed: true, contract: A ?? B }
    const M = await agent(merger(s, A, B, C), { ...OPTS, agentType: 'ts-merger', schema: CONTRACT, label: `merge ${s.id}`, phase: 'Merge' })
    return { name: s.id, id: `app.${s.id}`, contract: M, issues: C ? C.issues.length : null }
  }
)
