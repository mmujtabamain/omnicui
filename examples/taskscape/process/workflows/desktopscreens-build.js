export const meta = {
  name: 'taskscape-desktop-screens',
  description: 'Desktop screens (egui): per screen, a builder ports the web screen against its contract on the app core and the widget library, in a build slot of its own; a second builder fixes what still fails',
  phases: [{ title: 'Build' }, { title: 'Fix' }],
}

// args: { root, slots, screens: [{ contract, struct, web, newArgs, note? }] }
//   contract: tasklist; struct: TaskList (the screen's type, which the main window composes); web: the
//   web screen folders (apps/web/frontend/src/screens/<web>/); newArgs: what new() takes ("" or "pane: usize").
const OPTS = { model: 'haiku', effort: 'high', agentType: 'ts-builder' }
const RESULT = {
  type: 'object',
  properties: {
    passed: { type: 'boolean', description: 'The check exits 0: it builds and every contract test passes.' },
    summary: { type: 'string', description: 'What you built or changed; the last CONTRACT line; anything still wrong, with the reason.' },
    needs: { type: 'string', description: 'Anything you needed and did not have (a change to another widget, a contract that cannot be met as written on egui). Empty if none.' },
  },
  required: ['passed', 'summary', 'needs'],
}
const R = args.root
const A = `${R}/apps/desktop/app`
const UI = `${R}/apps/desktop/ui`
const check = (w, slot) => `cd ${R} && python3 apps/desktop/scripts/check-widget.py ${w.contract} --slot ${slot}`
const reads = (w) => [
  `${A}/AGENTS.md and ${UI}/AGENTS.md (the conventions: follow them exactly)`,
  `${R}/omnicui/contracts/${w.contract}.contract.yaml (the contract; tests tagged only: [web] do not apply)`,
  `${R}/spec/screens/contract-notes-${w.contract}.txt (what each prop makes the harness do), if it exists`,
  `${UI}/src/omnicui/${w.contract}.rs (the types generated from it: props, events, part ids)`,
  `${A}/src/app.rs, ${A}/src/data.rs and ${A}/src/state.rs (the app you draw on: its data, actions and UI state)`,
  `${R}/apps/web/frontend/src/screens/${w.web}/ (the web screen you are porting: its layout, sizes, copy and behaviour; start from the file named like the screen)`,
  `${R}/apps/web/frontend/src/harness/${w.contract}.tsx (how the web test set the screen up from its props), if it exists`,
  `${R}/spec/decisions.md (desktop decisions win over the web's behaviour)`,
  `${UI}/src/widgets/ (the finished widgets: use them where the web uses the same widget; read only those you use)`,
].map((l) => `- ${l}`).join('\n')

const build = (w, slot) => `You build one screen of a desktop app (egui): ${w.contract} (contract app.${w.contract}), by porting the web screen onto the desktop app's core and widget library, so that it passes its omnicui contract on the desktop target.

Read these first:
${reads(w)}

The screen's type is \`pub struct ${w.struct}\` with \`pub fn new(${w.newArgs}) -> Self\` and \`pub fn show(&mut self, app: &mut crate::App, ui: &mut egui::Ui, look: taskscape_ui::Look) -> Vec<the contract's event type>\`: the main window will call exactly that, so keep the names. ${w.note ?? ''}

Write only ${A}/drafts/${w.contract}.rs. If it already exists, an earlier attempt wrote it: read it and continue. Check it with exactly this command, and run no other command:
  ${check(w, slot)}
Run it with the Bash tool's timeout set to 600000 (it waits its turn for the shared build); never in the background. Use --test "<part of a test name>" while you work on a few tests, and run the whole contract before you finish. Repeat until it exits 0, then look at two or three screenshots it names (Read the PNG) and fix what looks wrong next to the web screen. Read each file once.`

const fix = (w, slot, first) => `A first builder ported the screen ${w.contract} (contract app.${w.contract}) to egui as ${A}/drafts/${w.contract}.rs; its check still fails. Its report: ${first?.summary ?? 'none (it ran out of budget)'}

Read:
${reads(w)}
- ${A}/drafts/${w.contract}.rs (the draft to fix)

Run exactly this check, and no other command, with the Bash tool's timeout set to 600000 (never in the background):
  ${check(w, slot)}
Fix the draft until the check exits 0. Change only that file; keep \`pub struct ${w.struct}\`, \`new(${w.newArgs})\` and \`show\` as they are. If a test cannot pass on the desktop as written, say which and why in needs.`

const queue = [...args.screens]
const done = []
await Promise.all(Array.from({ length: args.slots }, async (_, slot) => {
  while (queue.length) {
    const w = queue.shift()
    const first = await agent(build(w, slot), { ...OPTS, schema: RESULT, label: `build ${w.contract} (slot ${slot})`, phase: 'Build' })
    let result = first
    if (!first?.passed) {
      result = await agent(fix(w, slot, first), { ...OPTS, schema: RESULT, label: `fix ${w.contract} (slot ${slot})`, phase: 'Fix' })
    }
    done.push({ screen: w.contract, slot, passed: !!result?.passed, summary: result?.summary ?? '', needs: [first?.needs, result?.needs].filter(Boolean).join(' / ') })
    log(`${done.filter((d) => d.passed).length}/${done.length} passed (${w.contract}: ${result?.passed ? 'pass' : 'not yet'})`)
  }
}))
return done
