export const meta = {
  name: 'taskscape-fix-contracts',
  description: 'Fix contracts that omnicui refused: a read-only fixer opens only its contract, and gets the errors and the format in its prompt',
  phases: [{ title: 'Fix' }],
}

// args: { formatFile, items: [{ name, file, errors }] }: file is the refused contract
const TYPE = args.agentTypes ?? { relay: 'ts-solver', fixer: 'ts-solver' }
const RELAY = { type: 'object', properties: { text: { type: 'string', description: "The file's whole content, exactly as written." } }, required: ['text'] }
const CONTRACT = { type: 'object', properties: { yaml: { type: 'string' }, notes: { type: 'string' } }, required: ['yaml', 'notes'] }
const RULES = `Hard rules of this omnicui version (a contract that breaks one is refused):
- Part roles are only: button, switch, checkbox, slider, textbox, status, heading, generic. Map every other ARIA role onto them: option, tab, menuitem, treeitem, link to button; listbox, list, listitem, toolbar, menu, tree, tablist, dialog, banner, img, group to generic. Put the real ARIA role in a YAML comment on the part (# aria: option), and test it through the fields the format allows.
- A prop has only type and default. No description, enum or other fields; explain allowed values in a comment.
- Only the fields, actions and roles the format lists. Names camelCase. The file must be valid YAML (comments start with #, never //): quote any string that holds a colon.
- Every step with an action names its part, key presses included ({ action: key, part: field, key: Escape }).
- drag acts only on slider parts; there is no drag on a button or generic part, so leave drag behaviour out (note it in a comment).
- trace and record need steps that start the motion: a part animating as it mounts cannot be traced; open it through a step instead, or use a screenshot.
- Every test has at least one expect, screenshot, trace or record; a contract has at least one test.
- An event's last values must have the field's declared type: a string field gets a plain string, never { contains: … } or { matches: … } (those work only on part text and name).`
const relay = (file, label) =>
  agent(`Read ${file} and return its whole content exactly as written, with nothing added, left out or reworded. Open nothing else.`, { model: 'haiku', effort: 'low', agentType: TYPE.relay, schema: RELAY, label, phase: 'Fix' }).then((r) => r?.text)

phase('Fix')
const FORMAT = await relay(args.formatFile, 'relay format')
return await parallel(
  args.items.map((it) => async () => {
    const F = await agent(`omnicui refused the contract in ${it.file}. Read that file (and only that file), fix every error below and change nothing else, then return the whole contract.

omnicui's errors:
${it.errors}

${RULES}

The format:
${FORMAT}`, { model: 'haiku', effort: 'high', agentType: TYPE.fixer, schema: CONTRACT, label: `fix ${it.name}`, phase: 'Fix' })
    return F ? { name: it.name, id: `ui.${it.name}`, contract: F } : { name: it.name, failed: true }
  })
)
