export const meta = {
  name: 'taskscape-desktop-widgets',
  description: 'Desktop widget library (egui): per widget, a builder ports the web widget against its contract in a build slot of its own; a second builder fixes what still fails',
  phases: [{ title: 'Build' }, { title: 'Fix' }],
}

// args: { root, slots, widgets: [{ name, contract, web, uses }] }
//   root: examples/taskscape; slots: how many build slots exist (.checks/desktop-slots/0..slots-1);
//   name: PaneFooter; contract: panefooter; web: pane-footer (the web widget's file); uses: finished
//   desktop widgets it composes (contract names).
// Each worker owns one slot and builds its widgets one after another in it, so no two builds share a
// slot. A widget that passes stays in drafts/; the promotion into src/widgets/ is done between waves.
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
const D = `${R}/apps/desktop/ui`
const check = (w, slot) => `cd ${R} && python3 apps/desktop/scripts/check-widget.py ${w.contract} --slot ${slot}`
const reads = (w) => [
  `${D}/AGENTS.md (the conventions: follow them exactly)`,
  `${R}/omnicui/contracts/${w.contract}.contract.yaml (the contract; tests tagged only: [web] do not apply)`,
  `${D}/src/omnicui/${w.contract}.rs (the types generated from it: props, events, part ids)`,
  `${R}/packages/ui/src/components/ui/${w.web}.tsx (the web widget you are porting: its look, sizes and behaviour)`,
  `${R}/spec/library/widgets/${w.name}.json (the widget spec, if the web file leaves something open)`,
  `${D}/src/lib.rs and ${D}/src/widgets/checkbox.rs (the library's helpers, and a finished widget)`,
  ...(w.uses ?? []).map((u) => `${D}/src/widgets/${u}.rs (a finished desktop widget you compose; use it as crate::widgets::${u})`),
].map((l) => `- ${l}`).join('\n')

const build = (w, slot) => `You build one widget of a desktop app's egui widget library: ${w.name} (contract ${w.contract}), by porting the web version to egui so that it passes its omnicui contract on the desktop target.

Read these first:
${reads(w)}

Write only ${D}/drafts/${w.contract}.rs. Check it with exactly this command, and run no other command:
  ${check(w, slot)}
It builds your draft with the finished widgets and runs the contract, printing BUILD, CONTRACT and every failing test. Repeat until it exits 0, then look at two or three screenshots it names (Read the PNG) next to the web widget's look, and fix what is wrong. Keep within budget: read each file once.`

const fix = (w, slot, first) => `A first builder ported the widget ${w.name} (contract ${w.contract}) to egui as ${D}/drafts/${w.contract}.rs; its check still fails. Its report: ${first?.summary ?? 'none (it ran out of budget)'}

Read:
${reads(w)}
- ${D}/drafts/${w.contract}.rs (the draft to fix)

Run exactly this check, and no other command:
  ${check(w, slot)}
Fix the draft until the check exits 0. Change only that file. If a test cannot pass on egui as written, say which and why in needs.`

const queue = [...args.widgets]
const done = []
await Promise.all(Array.from({ length: args.slots }, async (_, slot) => {
  while (queue.length) {
    const w = queue.shift()
    const first = await agent(build(w, slot), { ...OPTS, schema: RESULT, label: `build ${w.contract} (slot ${slot})`, phase: 'Build' })
    let result = first
    if (!first?.passed) {
      result = await agent(fix(w, slot, first), { ...OPTS, schema: RESULT, label: `fix ${w.contract} (slot ${slot})`, phase: 'Fix' })
    }
    done.push({ widget: w.contract, slot, passed: !!result?.passed, summary: result?.summary ?? '', needs: [first?.needs, result?.needs].filter(Boolean).join(' / ') })
    log(`${done.filter((d) => d.passed).length}/${done.length} passed (${w.contract}: ${result?.passed ? 'pass' : 'not yet'})`)
  }
}))
return done
