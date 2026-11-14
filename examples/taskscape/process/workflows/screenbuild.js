export const meta = {
  name: 'taskscape-screens',
  description: 'Web screens: per screen, a builder writes it from its contract with widgets only (pass 1); an independent second builder runs the contract, compares the screenshots with the reference and fixes what is wrong (pass 2)',
  phases: [{ title: 'Build' }, { title: 'Fix' }],
}

// args: { root, pg, python, screens: [{ id, folder, component, inventory?, features?, notes?, refs: [state], uses?: [folder], note? }] }
//   id: contract name (tasklist); folder: src/screens/<folder>; component: TaskList;
//   refs: reference states (spec/references/<state>/shot.png); uses: other screen folders it composes;
//   own: the only files under src/screens/ it may write, when it shares a folder (default: the whole folder).
const OPTS = { model: 'haiku', effort: 'high', agentType: 'ts-builder' }
const RESULT = {
  type: 'object',
  properties: {
    passed: { type: 'boolean', description: 'The check exits 0: types, lint and every contract test.' },
    summary: { type: 'string', description: 'What you built or changed; the last check result line; how it compares with the reference; anything still wrong, with the reason.' },
    needs: { type: 'string', description: 'Anything you needed outside your files (a widget that cannot do something, a shared hook or store field, an API change, a contract that cannot be met). Empty if none.' },
  },
  required: ['passed', 'summary', 'needs'],
}
const R = args.root
const F = `${R}/apps/web/frontend`
const check = (s) => `cd ${R} && PATH=$HOME/.local/node22/bin:$PATH SCREENS_DATABASE_URL=${args.pg} TASKSCAPE_PYTHON=${args.python} node apps/web/frontend/scripts/check-screen.mjs ${s.id}`
const files = (s) => s.own
  ? `${s.own.map((o) => `${F}/src/screens/${o}`).join(', ')} (your screen, exported as ${s.component}) and ${F}/src/harness/${s.id}.tsx`
  : `${F}/src/screens/${s.folder}/ (your screen, exported as ${s.component} from ${s.folder}/${s.component}.tsx) and ${F}/src/harness/${s.id}.tsx`
const refs = (s) => s.refs.map((r) => `${R}/spec/references/${r}/shot.png`).join(', ')
const reads = (s) => [
  `${F}/AGENTS.md (how a screen is written: follow it exactly)`,
  `${R}/omnicui/contracts/${s.id}.contract.yaml (the contract the screen must pass)`,
  `${R}/spec/screens/contract-notes-${s.id}.txt (how the contract was written and what each of its props must make the harness do)`,
  ...(s.inventory ? [`${R}/spec/inventory/${s.inventory} (the reference screen: regions, widgets, states, measurements)`] : []),
  ...(s.features ? [`${R}/spec/inventory/${s.features} (its feature titles with ids; Grep ${R}/spec/features/area-*.json for a feature's full record)`] : []),
  ...(s.notes ? [`${R}/spec/screens/${s.notes} (the team's review notes and how each was settled)`] : []),
  `${R}/spec/screens/web.md and ${R}/spec/decisions.md (what the web changes; product decisions)`,
  `${F}/src/api/queries.ts, ${F}/src/api/endpoints.ts, ${F}/src/api/types.ts, ${F}/src/state/ui.ts, ${F}/src/harness/lib/AppHarness.tsx`,
  `${R}/packages/ui/src/components/ui/ (the widgets; read the files of those you use)`,
  ...(s.uses ?? []).map((u) => `${F}/src/screens/${u}/ (a screen you compose)`),
  ...(s.refs.length ? [`the reference screenshots (Read the PNGs): ${refs(s)}`] : []),
].map((l) => `- ${l}`).join('\n')

const rules = (s) => `Rules:
- Write only ${files(s)}. Shared code, widgets, the contract and other screens are read-only: if you need one changed, say so in "needs" and work around it meanwhile.
- Read only what is listed above. Do not look for an older implementation anywhere else.
- Draw only with widgets from @taskscape/ui (the lint refuses raw elements); layout with div/span/section and token utilities.
- Keep replies short: you have a limited budget. Read each file once.${s.note ? `\n- ${s.note}` : ''}`

const builder = (s) => `You build one screen of a task manager's web app, ${s.component} (contract "app.${s.id}"), so that it passes its contract and looks like the reference.

Read:
${reads(s)}

Then write ${files(s)}: every region, state and behaviour the contract tests and the features list, with real data from the API, and the harness entry that mounts it in the contract's starting states.

Check: ${check(s)}
Run it, fix what fails, run it again; at most 5 runs. Then open the light sheet it names and compare it with the reference screenshots: layout, spacing, sizes, colours, text. Fix what differs; run the check once more if you changed anything.

${rules(s)}`

const fixer = (s, first) => `You review and finish one screen of a task manager's web app, ${s.component} (contract "app.${s.id}"). Another builder wrote it; you check it independently.

Read:
${reads(s)}
- ${files(s)} (the screen as built)

The builder said: ${first}

Run: ${check(s)}
1. Fix every failing test, type error and lint error. Never remove behaviour to make a test pass.
2. Open both sheets it names and the reference screenshots. Fix every visible difference from the reference (layout, spacing, sizes, colours, text, icons) and anything the features list says the screen does that it does not.
3. Run the check again. At most 5 runs in all.

${rules(s)}`

phase('Build')
return await pipeline(
  args.screens,
  // A repair (s.repair: what was built and what still fails) skips the build and goes to an independent fixer.
  (s) => (s.repair ? { passed: false, summary: s.repair, needs: '' } : agent(builder(s), { ...OPTS, schema: RESULT, label: `build ${s.id}`, phase: 'Build' })),
  async (B, s) => {
    const first = B ? `${B.passed ? 'passed' : 'not passing'}. ${B.summary}${B.needs ? ` Needs: ${B.needs}` : ''}` : 'nothing (the first builder did not finish).'
    const X = await agent(fixer(s, first), { ...OPTS, schema: RESULT, label: `fix ${s.id}`, phase: 'Fix' })
    return { id: s.id, build: B, fix: X }
  }
)
