export const meta = {
  name: 'taskscape-exactness',
  description: 'Check every merged feature for exactness: checker A against the code, checker B against the recordings or docs, then a merger applies the corrections',
  phases: [{ title: 'Check' }, { title: 'Merge' }],
}

// args: { refs, agentTypes?, items: [{ key, file, area: { n, name, filesA, statesB, docsB } }] }
// Each item is a chunk of one area's features, in a JSON file the checkers read; the merger
// returns only the resolutions, which collect.py applies.
const TYPE = args.agentTypes ?? { solver: 'ts-solver', merger: 'ts-merger' }
const OPTS = { model: 'haiku', effort: 'medium' }

const EXACT = {
  type: 'array',
  items: { type: 'object', properties: { what: { type: 'string' }, value: { type: 'string' } }, required: ['what', 'value'] },
}
const VERDICTS = {
  type: 'object',
  properties: {
    checks: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          verdict: { type: 'string', enum: ['exact', 'corrected', 'not_found', 'cannot_check'] },
          corrections: {
            type: 'object',
            description: 'Only the fields that were wrong or missing, with their corrected values.',
            properties: {
              title: { type: 'string' },
              trigger: { type: 'string' },
              context: { type: 'string' },
              result: { type: 'string' },
              exact: EXACT,
              edge_cases: { type: 'array', items: { type: 'string' } },
            },
          },
          evidence: { type: 'string', description: 'file:line or state:<id> you checked.' },
        },
        required: ['id', 'verdict', 'evidence'],
      },
    },
  },
  required: ['checks'],
}
const FINAL = {
  type: 'object',
  properties: {
    resolutions: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          check: { type: 'string', enum: ['confirmed_by_both', 'confirmed_by_one', 'corrected', 'disputed', 'unverified'] },
          corrections: VERDICTS.properties.checks.items.properties.corrections,
          evidence: { type: 'array', items: { type: 'string' } },
          check_notes: { type: 'string' },
        },
        required: ['id', 'check', 'evidence'],
      },
    },
  },
  required: ['resolutions'],
}

const RULES = `For each feature, check every claim: the trigger (exact keys and modifiers, the exact gesture), the context, the result, every exact value (strings character for character, numbers, durations, limits), and the edge cases. A value that is right but incomplete is "corrected" with the missing parts added. Never change a value without evidence; cite the file:line or state you checked.`

const checkerA = (it) => `You check features of a desktop task-manager app (the "reference") for exactness against its source code.

Area ${it.area.n}: ${it.area.name}. The features to check are in ${it.file}; read it first.

You may also open these files and folders, and grep within the reference repo for a symbol a feature cites:
${it.area.filesA.map((f) => `- ${f}`).join('\n')}

${RULES} Use "not_found" when the code shows no such behaviour.`

const checkerB = (args, it) => {
  const src = it.area.statesB.length
    ? `recordings of the running app. Each folder holds shot.png (an image of the window), tree.txt (React's component tree with props and boxes) and meta.json (commands sent):\n${it.area.statesB.map((s) => `- ${args.refs}/${s}/`).join('\n')}`
    : `the app's documentation:\n${it.area.docsB.map((d) => `- ${d}`).join('\n')}`
  return `You check features of a desktop task-manager app (the "reference") for exactness against ${src}

Do not read the source code. Area ${it.area.n}: ${it.area.name}. The features to check are in ${it.file}; read it first.

${RULES} Use "cannot_check" when nothing you have shows the behaviour either way.`
}

const merger = (it, A, B) => `You resolve two independent exactness checks of some features of a desktop task-manager app: checker A worked against the source code, checker B against recordings of the running app (or its docs).

Checker A:
${JSON.stringify(A.checks)}

Checker B:
${JSON.stringify(B.checks)}

Return one resolution per feature id that appears in either check:
- corrections: the corrected fields; where A and B disagree on a value, the source code (A) decides, and say what B saw in check_notes;
- evidence: both checkers' evidence;
- check: "confirmed_by_both" when both found it exact, "confirmed_by_one" when one did and the other could not check, "corrected" when there are corrections, "disputed" when a checker found no such behaviour, "unverified" when neither could check.
Keep every string, key and number exactly.`

const results = await pipeline(
  args.items,
  (it) =>
    parallel([
      () => agent(checkerA(it), { ...OPTS, agentType: TYPE.solver, schema: VERDICTS, label: `A ${it.key}`, phase: 'Check' }),
      () => agent(checkerB(args, it), { ...OPTS, agentType: TYPE.solver, schema: VERDICTS, label: `B ${it.key}`, phase: 'Check' }),
    ]),
  async ([A, B], it) => {
    if (!A || !B) return { key: it.key, failed: true, A, B }
    const M = await agent(merger(it, A, B), { ...OPTS, agentType: TYPE.merger, schema: FINAL, label: `merge ${it.key}`, phase: 'Merge' })
    return { key: it.key, area: it.area.n, final: M }
  }
)
return results
