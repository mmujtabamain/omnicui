export const meta = {
  name: 'taskscape-seams',
  description: 'Find features that fall between audit areas: four critics with different lenses read the whole index, then a merger dedupes their finds',
  phases: [{ title: 'Hunt' }, { title: 'Merge' }],
}

// args: { ref, refs, index, areas, agentTypes? }: index is a file with one line per feature (id title); F02-… means area 2
const TYPE = args.agentTypes ?? { critic: 'ts-critic', merger: 'ts-merger' }
const OPTS = { model: 'haiku', effort: 'medium' }

const FEATURE = {
  type: 'object',
  properties: {
    area: { type: 'integer', description: 'The audit area number it belongs to (see the index).' },
    title: { type: 'string' },
    trigger: { type: 'string' },
    context: { type: 'string' },
    result: { type: 'string' },
    exact: { type: 'array', items: { type: 'object', properties: { what: { type: 'string' }, value: { type: 'string' } }, required: ['what', 'value'] } },
    edge_cases: { type: 'array', items: { type: 'string' } },
    sources: { type: 'array', items: { type: 'string' } },
    why_missed: { type: 'string', description: 'Which areas it spans, or why the per-area audit could not see it.' },
  },
  required: ['area', 'title', 'trigger', 'context', 'result', 'exact', 'sources', 'why_missed'],
}
const FINDS = { type: 'object', properties: { features: { type: 'array', items: FEATURE } }, required: ['features'] }

const LENSES = [
  ['keys', 'Keys and modifiers everywhere: every shortcut, every modifier-click (Option, Shift, Command, Control), every key that behaves differently by focus (Enter, Escape, Tab, arrows, Backspace, Space), how shortcuts are shown (keycaps, key labels, tooltips), and what changes while a modifier is held.'],
  ['flows', 'Flows that cross areas: things one part of the window does to another (selection and the inspector, search and the list, the palette running commands, menus opening sheets, sheets creating data, undo across all of them), and the order of focus as the user moves through them.'],
  ['state', 'What is remembered and restored: settings, the last project and list, panel and split layout, collapsed rows, theme, window size; what happens on first launch, with no data, and after deleting the thing in view.'],
  ['edges', 'Empty, error, limit and boundary states: no lists, no tasks, no matches, very long titles, many tasks, done and trashed items, failed commands, disabled controls, confirmations, and what the app does when an action cannot be done.'],
]

const hunt = ([key, lens]) => `The features of a desktop task-manager app (the "reference") were audited area by area. The index of every feature found is ${args.index}: one line per feature, "id title", where F02-… is area 2. Read it all first. The areas: ${args.areas}.

Your lens: ${lens}

Find features the index lacks. You may read anything in the reference repo (${args.ref}) and the recordings of the running app in ${args.refs} (each folder: shot.png, an image; tree.txt, React's component tree; meta.json). Grep is the fastest way to find handlers (onKeyDown, altKey, metaKey, shiftKey, addEventListener, localStorage, setSetting). Only report a feature that is really missing from the index, with exact values and file:line sources. Return nothing rather than near-duplicates.`

const merge = (finds) => `Several critics each looked for features missing from an audit index, through different lenses. Their finds:
${JSON.stringify(finds)}

Merge them: one entry per distinct feature, combining exact values and sources. Drop any that duplicates another find. Keep strings, keys and numbers exactly.`

phase('Hunt')
const finds = await parallel(
  LENSES.map((l) => () => agent(hunt(l), { ...OPTS, agentType: TYPE.critic, schema: FINDS, label: `seams: ${l[0]}`, phase: 'Hunt' }))
)
const all = finds.filter(Boolean).flatMap((f, i) => f.features.map((x) => ({ ...x, lens: LENSES[i][0] })))
log(`${all.length} candidate features from ${finds.filter(Boolean).length} critics`)
if (!all.length) return { features: [] }
phase('Merge')
return await agent(merge(all), { ...OPTS, agentType: TYPE.merger, schema: FINDS, label: 'seams: merge', phase: 'Merge' })
