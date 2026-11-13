export const meta = {
  name: 'taskscape-features',
  description: 'Feature audit per area: solver A from code, solver B from recorded behaviour or docs, a critic on high-tier areas, then a merger',
  phases: [{ title: 'Solve' }, { title: 'Critique' }, { title: 'Merge' }],
}

// args: { refs, areas: [{ n, name, tier, filesA, statesB, docsB }] }
const EXACT = {
  type: 'array',
  description: 'Every exact value of this feature: keys and modifiers, text as shown, durations, sizes, limits, counts, colours.',
  items: {
    type: 'object',
    properties: { what: { type: 'string' }, value: { type: 'string' } },
    required: ['what', 'value'],
  },
}
const FEATURE = {
  type: 'object',
  properties: {
    title: { type: 'string', description: 'Short name of the behaviour, e.g. "Option-click the check marks the task done".' },
    trigger: { type: 'string', description: 'What the user does, exactly: keys with modifiers, click/double-click/right-click/hover/drag with modifiers, menu path, typing. "none" for something only observed.' },
    context: { type: 'string', description: 'When it applies: window, focus, selection, mode, data state.' },
    result: { type: 'string', description: 'What happens and what the user sees, exactly.' },
    exact: EXACT,
    edge_cases: { type: 'array', items: { type: 'string' } },
    sources: { type: 'array', items: { type: 'string' }, description: 'file:line or file:line-line for code; state:<id> for a recording; doc:<file> for docs.' },
    system: { type: 'boolean', description: 'True when it depends on the operating system: global hotkeys, menu bar, screen capture, file dialogs, Finder, native windows.' },
    uncertain: { type: 'boolean' },
    notes: { type: 'string' },
  },
  required: ['title', 'trigger', 'context', 'result', 'exact', 'sources', 'system', 'uncertain'],
}
const SOLUTION = {
  type: 'object',
  properties: { features: { type: 'array', items: FEATURE } },
  required: ['features'],
}
const CRITIQUE = {
  type: 'object',
  properties: {
    issues: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          kind: { type: 'string', enum: ['only_in_A', 'only_in_B', 'conflict', 'vague', 'duplicate', 'missed_by_both', 'wrong'] },
          feature: { type: 'string', description: 'The feature title it concerns.' },
          verdict: { type: 'string', description: 'What is right, after checking the source.' },
          evidence: { type: 'string', description: 'file:line or state:<id> you checked.' },
          proposed: FEATURE,
        },
        required: ['kind', 'feature', 'verdict', 'evidence'],
      },
    },
  },
  required: ['issues'],
}
const MERGED = {
  type: 'object',
  properties: {
    features: {
      type: 'array',
      items: {
        ...FEATURE,
        properties: {
          ...FEATURE.properties,
          found_by: { type: 'array', items: { type: 'string', enum: ['A', 'B', 'critic'] } },
        },
        required: [...FEATURE.required, 'found_by'],
      },
    },
    dropped: {
      type: 'array',
      items: {
        type: 'object',
        properties: { title: { type: 'string' }, reason: { type: 'string' } },
        required: ['title', 'reason'],
      },
    },
  },
  required: ['features', 'dropped'],
}

const WHAT = `A feature is one thing a user can do or observe in the app:
- an action and its exact trigger: keys with modifiers, click, double-click, right-click, hover, drag, each with its modifiers; menu items; typing;
- what it changes and what appears: exact text, labels, placeholders, tooltips (including the shortcut hints in them), counts, empty states, confirmations, toasts;
- states: selected, focused, hover, done, disabled, editing, empty, dark mode;
- modes: anything that changes while a modifier key is held or a setting is on (for example a control changing shape or meaning while Option is held, or keyboard labels shown instead of icons);
- timing and motion values, limits (lengths, counts), ordering rules, persistence (what is remembered between launches), and edge cases.
One feature per distinct behaviour: never fold two actions into one entry. Leave out implementation detail a user can't observe. Copy every string, key and number exactly as it is. Be exhaustive: a missed feature is the worst outcome.`

const solverA = (a) => `You are solver A for one area of the audit of a desktop task-manager app (the "reference"). You work from its source code.

Area ${a.n}: ${a.name}.
Read ONLY these files and folders (read each file completely; for a folder, every file in it):
${a.filesA.map((f) => `- ${f}`).join('\n')}

${WHAT}

Cite every feature's sources as file:line. Features owned by another area may appear in these files: include them only if this area's files implement them.`

const solverB = (args, a, part, parts) => {
  const states = part.map((s) => `- ${args.refs}/${s}/ (state "${s}")`).join('\n')
  const docs = a.docsB.map((d) => `- ${d}`).join('\n')
  return `You are solver B for one area of the audit of a desktop task-manager app (the "reference"). You do NOT read its source code. You work from ${a.statesB.length ? 'recordings of the running app' : 'its documentation'}.

Area ${a.n}: ${a.name}.${parts > 1 ? ` The recordings for this area are split between ${parts} workers; you have one share of them. Describe everything they show.` : ''}
${a.statesB.length ? `Recorded states. Each folder holds: shot.png (the window: open it, it is an image), tree.txt (React's component tree for that moment: component names, their props, and on-screen boxes as @x,y WxH) and meta.json (the commands the app sent to its backend). The state's name says what was done to reach it, e.g. "option-held" means the Option key is held, "row-context-menu" a right-click on a task row, "sheet-new-list" the new-list shortcut pressed.
${states}
` : ''}${docs ? `Documents:\n${docs}\n` : ''}
Open only these. ${WHAT}

Work from what the recordings show and what their props say (a prop like title="Hide preview panel  ⌘\\\\" is the tooltip text; variant="danger" is a style). Compare states to find behaviour: what differs between main-default and option-held is what Option does. Cite sources as state:<name> (add the tree line when it helps) or doc:<file>. Mark anything you infer rather than see as uncertain.`
}

const critic = (a, A, B) => `You are the critic for area ${a.n} (${a.name}) of the audit of a desktop task-manager app. Two solvers worked independently: A from the source code, B from recordings of the running app${a.docsB.length ? ' and its docs' : ''}.

Solution A:
${JSON.stringify(A.features)}

Solution B:
${JSON.stringify(B.features)}

Find: features only one of them has (and whether the other missed it or it is wrong), exact values that conflict, entries too vague to be exact, duplicates, and features both missed. Settle each question against the sources; you may open only:
${[...a.filesA, ...a.statesB.map((s) => `(recording) ${s}`), ...a.docsB].map((f) => `- ${f}`).join('\n')}
Recordings are folders under the refs directory given in B's sources. For a feature both missed, give it in "proposed".`

const merger = (a, A, B, C) => `You merge two independent solutions of area ${a.n} (${a.name}) of the audit of a desktop task-manager app into one list. A worked from the source code, B from recordings of the running app${a.docsB.length ? ' and its docs' : ''}.${C ? ' A critic checked both against the sources; its findings are verified and take precedence.' : ''}

Solution A:
${JSON.stringify(A.features)}

Solution B:
${JSON.stringify(B.features)}
${C ? `\nCritique:\n${JSON.stringify(C.issues)}\n` : ''}
Rules:
- Keep the union. The same behaviour in both becomes one entry with every exact value, both sets of sources, and found_by ["A","B"].
- A feature only one found stays, with found_by naming it${C ? '; a critic\'s proposed feature gets found_by ["critic"]' : ''}.
- Conflicting values: ${C ? 'use the critique\'s verdict; ' : ''}otherwise keep the value with a file:line source and say what B saw in notes.
- Drop only duplicates${C ? ' and what the critique shows is wrong' : ''}; list every drop in "dropped" with its reason.
- Keep every string, key and number exactly as given. Don't shorten results or edge cases.`

const TYPE = args.agentTypes ?? { solver: 'ts-solver', critic: 'ts-critic', merger: 'ts-merger' }
const OPTS = { agentType: TYPE.solver, model: 'haiku', effort: 'medium' }
// A solver that answers without reading returns an empty list; give it one more try.
const solve = async (prompt, opts) => {
  const r = await agent(prompt, opts)
  if (r && r.features.length >= 3) return r
  return agent(`${prompt}\n\nRead every listed file or folder before answering. This area has many features; an empty or near-empty list is wrong.`, { ...opts, label: `${opts.label} (retry)` })
}

const results = await pipeline(
  args.areas,
  async (a) => {
    // At most 9 recordings per B worker keeps each under its context budget; B is their union.
    const parts = []
    for (let i = 0; i < Math.max(a.statesB.length, 1); i += 9) parts.push(a.statesB.slice(i, i + 9))
    const [A, ...Bs] = await parallel([
      () => solve(solverA(a), { ...OPTS, schema: SOLUTION, label: `A ${a.n} ${a.name}`, phase: 'Solve' }),
      ...parts.map((part, i) => () =>
        solve(solverB(args, a, part, parts.length), { ...OPTS, schema: SOLUTION, label: `B${parts.length > 1 ? i + 1 : ''} ${a.n} ${a.name}`, phase: 'Solve' })),
    ])
    const ok = Bs.filter(Boolean)
    return [A, ok.length ? { features: ok.flatMap((b) => b.features) } : null]
  },
  async ([A, B], a) => {
    if (!A || !B) return { A, B, C: null }
    if (a.tier !== 'high') return { A, B, C: null }
    const copts = { ...OPTS, agentType: TYPE.critic, schema: CRITIQUE, label: `critic ${a.n}`, phase: 'Critique' }
    let C = await agent(critic(a, A, B), copts)
    // Two independent lists of this size always differ somewhere; an empty critique means it didn't look.
    if (C && !C.issues.length)
      C = await agent(`${critic(a, A, B)}\n\nCompare the two lists entry by entry before answering; they differ.`, { ...copts, label: `critic ${a.n} (retry)` })
    return { A, B, C }
  },
  async ({ A, B, C }, a) => {
    if (!A || !B) return { area: a.n, name: a.name, failed: true, A, B }
    const M = await agent(merger(a, A, B, C), { ...OPTS, agentType: TYPE.merger, schema: MERGED, label: `merge ${a.n}`, phase: 'Merge' })
    return { area: a.n, name: a.name, tier: a.tier, A: A.features.length, B: B.features.length, issues: C ? C.issues.length : null, merged: M }
  }
)
return results
