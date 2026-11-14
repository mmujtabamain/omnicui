export const meta = {
  name: 'taskscape-widgets',
  description: 'Widget library: per widget, a builder writes it against its contract (pass 1); an independent second builder checks it, looks at the screenshots and fixes what is wrong (pass 2)',
  phases: [{ title: 'Build' }, { title: 'Fix' }],
}

// args: { root, widgets: [{ name, contract, file, spec, uses? }] }
//   root: examples/taskscape; name: PaneFooter; contract: panefooter; file: pane-footer (no .tsx);
//   spec: the widget spec JSON (default spec/library/widgets/<name>.json); uses: other components/ui files it composes (kebab names); note: extra instructions.
const OPTS = { model: 'haiku', effort: 'high', agentType: 'ts-builder' }
const RESULT = {
  type: 'object',
  properties: {
    passed: { type: 'boolean', description: 'The check exits 0: types, lint and every contract test.' },
    summary: { type: 'string', description: 'What you built or changed; the last check result line; anything still wrong, with the reason.' },
    needs: { type: 'string', description: 'Anything you needed and did not have (a library, a change to another widget, a contract that cannot be met as written). Empty if none.' },
  },
  required: ['passed', 'summary', 'needs'],
}
const ui = `${args.root}/packages/ui`
const check = (w) => `cd ${args.root} && PATH=$HOME/.local/node22/bin:$PATH node packages/ui/scripts/check-widget.mjs ${w.contract}`
const files = (w) => `${ui}/src/components/ui/${w.file}.tsx and ${ui}/src/harness/${w.contract}.tsx`
const reads = (w) => [
  `${ui}/AGENTS.md (the conventions: follow them exactly)`,
  `${args.root}/omnicui/contracts/${w.contract}.contract.yaml (the contract the widget must pass)`,
  `${w.spec ?? `${args.root}/spec/library/widgets/${w.name}.json`} (the widget spec: purpose, variants with measured sizes and colours, states, keyboard, open questions)`,
  `${ui}/src/styles/tokens.css (the only colours, radii, shadows and motion you may use)`,
  `${ui}/src/components/ui/button.tsx and ${ui}/src/harness/button.tsx (a finished widget, for the conventions)`,
  ...(w.uses ?? []).map((u) => `${ui}/src/components/ui/${u}.tsx (a widget you compose)`),
].map((l) => `- ${l}`).join('\n')

const rules = (w) => `Rules:
- Write only ${files(w)}. Do not edit any other file: not the contract, not other widgets, not the styles, not package.json. If you need one changed, say so in "needs".
- Read only the files listed above, and other files under ${ui}/src/components/ui/ when you compose them. Do not look for an older implementation anywhere else.
- Keep your replies short: you have a limited budget. Read each file once.`

const builder = (w) => `You build one widget of a React widget library, ${w.name}, so that it passes its contract.

Read:
${reads(w)}

Then write ${files(w)}: the widget the shadcn way (cva variants, Radix where the spec names it, cn, tokens), every variant, state and part of the contract, and its harness entry. Use the measured sizes and colours from the spec, converted at 1rem = 14px.

Check: ${check(w)}
Run it, fix what fails, run it again; at most 4 runs. Then open the light sheet it names (Read the PNG) and fix anything that looks wrong against the spec (sizes, colours, alignment, clipped or overlapping content); run the check once more if you changed anything.

${w.note ? `\nAlso: ${w.note}\n` : ''}
${rules(w)}`

const fixer = (w, first) => `You review and finish one widget of a React widget library, ${w.name}. Another builder wrote it; you check it independently.

Read:
${reads(w)}
- ${files(w)} (the widget as built)

The builder said: ${first}

Run: ${check(w)}
1. Fix every failing test, type error and lint error, in the widget (or the harness when the harness, not the widget, is wrong). Never weaken the widget to make a test pass.
2. Open both sheets it names (Read the PNGs). Compare what you see with the spec: the variants' sizes and colours, both colour schemes, focus rings, hover states, text that is cut off, icons that show as words. Fix what is wrong.
3. Run the check again. At most 4 runs in all.
${w.note ? `\nAlso: ${w.note}\n` : ''}
${rules(w)}`

phase('Build')
return await pipeline(
  args.widgets,
  (w) => agent(builder(w), { ...OPTS, schema: RESULT, label: `build ${w.name}`, phase: 'Build' }),
  async (B, w) => {
    const first = B ? `${B.passed ? 'passed' : 'not passing'}. ${B.summary}${B.needs ? ` Needs: ${B.needs}` : ''}` : 'nothing (the first builder did not finish).'
    const F = await agent(fixer(w, first), { ...OPTS, schema: RESULT, label: `fix ${w.name}`, phase: 'Fix' })
    return { name: w.name, contract: w.contract, build: B, fix: F }
  }
)
