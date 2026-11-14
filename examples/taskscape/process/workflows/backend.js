export const meta = {
  name: 'taskscape-backend',
  description: 'Backend resources: per resource, one builder writes the implementation and another writes tests from the API spec alone; then a repair loop makes the implementation pass the independent tests',
  phases: [{ title: 'Build' }, { title: 'Repair' }],
}

// args: { root, python, pg, resources: [{ name, section }] }: root is the Django project
const TYPE = args.agentTypes ?? { builder: 'ts-builder' }
const OPTS = { model: 'haiku', effort: 'high', agentType: TYPE.builder }
const RESULT = {
  type: 'object',
  properties: { passed: { type: 'boolean' }, summary: { type: 'string', description: 'What you built, and the last check output if it does not pass.' } },
  required: ['passed', 'summary'],
}
const check = (r) => `cd ${args.root} && DATABASE_URL=${args.pg}/taskscape_${r.name} DEBUG=1 ${args.python} -m pytest planner/tests/test_${r.name}.py -q`
const shared = `${args.root}/API.md (the contract; your resource is the section "${'${SECTION}'}"), ${args.root}/planner/models.py, ${args.root}/accounts/models.py, ${args.root}/planner/api/common.py, ${args.root}/conftest.py`

const implementer = (r) => `You implement one resource of a Django REST Framework API for a task manager.

Read: ${shared.replace('${SECTION}', r.section)}, and ${args.root}/config/urls.py (it imports your viewset by name).
Write only ${args.root}/planner/api/${r.name}.py (it holds a placeholder: replace it). Implement every endpoint the section lists, exactly: paths, bodies, status codes, ordering, and the 404-for-other-users rule (use the helpers in common.py). Log one INFO line per write (logger "taskscape.${r.name}"). Keep it plain DRF: serializers and a ViewSet with @action for the extra routes.
You may write a scratch test to check yourself, but delete it before you finish. Your check: ${check(r).replace(`planner/tests/test_${r.name}.py`, 'accounts')} (it must still pass: you must not break auth).`

const tester = (r) => `You write the tests for one resource of a Django REST Framework API for a task manager, from its contract alone. You never read the implementation.

Read: ${shared.replace('${SECTION}', r.section)}. Do not open anything under ${args.root}/planner/api/ except common.py.
Write only ${args.root}/planner/tests/test_${r.name}.py, with pytest and the fixtures in conftest.py (ana, bob, client_for, home). Cover every endpoint in the section: the success path with exact status codes and body fields, validation errors, ordering rules, the cascades the section describes, and that bob gets 404 for each of ana's rows. Each rule of the contract gets its own test with a clear name.
Your check: ${check(r)} --collect-only (it must collect without errors; the tests may fail against the placeholder, that is expected).`

const repairer = (r, impl, tests) => `You make one resource of a Django REST Framework API pass its independent tests.

Read: ${shared.replace('${SECTION}', r.section)}, the implementation ${args.root}/planner/api/${r.name}.py and the tests ${args.root}/planner/tests/test_${r.name}.py.
The implementer said: ${impl}
The test writer said: ${tests}

Run: ${check(r)}
Fix the implementation until every test passes. Change a test only when it contradicts API.md, and say which and why in your summary. Write only those two files. Then run the check again, and also ${check(r).replace(`planner/tests/test_${r.name}.py`, 'accounts')}.`

phase('Build')
return await pipeline(
  args.resources,
  (r) =>
    parallel([
      () => agent(implementer(r), { ...OPTS, schema: RESULT, label: `impl ${r.name}`, phase: 'Build' }),
      () => agent(tester(r), { ...OPTS, schema: RESULT, label: `tests ${r.name}`, phase: 'Build' }),
    ]),
  async ([I, T], r) => {
    if (!I || !T) return { name: r.name, failed: true, I, T }
    let R = await agent(repairer(r, I.summary, T.summary), { ...OPTS, schema: RESULT, label: `repair ${r.name}`, phase: 'Repair' })
    for (let i = 2; R && !R.passed && i <= 3; i++)
      R = await agent(`${repairer(r, I.summary, T.summary)}\n\nThe previous repair did not finish: ${R.summary}`, { ...OPTS, schema: RESULT, label: `repair ${r.name} (${i})`, phase: 'Repair' })
    return { name: r.name, impl: I.summary, tests: T.summary, repair: R }
  }
)
