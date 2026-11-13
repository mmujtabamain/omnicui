export const meta = {
  name: 'taskscape-comment-review',
  description: 'Review the flowchart comments: two independent reviewers per group of screens decide each comment, then a merger settles one outcome and reply per comment',
  phases: [{ title: 'Review' }, { title: 'Merge' }],
}

// args: { brief, decisions, ref, featuresDir, items: [{ key, screens, file, inventory, features }] }
const TYPE = args.agentTypes ?? { reviewer: 'ts-critic', merger: 'ts-merger' }
const OPTS = { model: 'haiku', effort: 'high' }

const OUTCOME = {
  type: 'object',
  properties: {
    id: { type: 'string', description: 'The comment id.' },
    outcome: {
      type: 'string',
      enum: ['answered', 'decided', 'needs_mujtaba', 'invalid'],
      description: 'answered: the sources already answer it (cite them). decided: a design call within the brief and the decisions already made. needs_mujtaba: a product decision the brief and decisions do not settle. invalid: the comment is wrong about the reference.',
    },
    reply: { type: 'string', description: 'The reply to post under the comment: the answer or decision, with evidence, in two to four plain sentences.' },
    question: { type: 'string', description: 'For needs_mujtaba only: the decision to make, in one sentence, with two or three options.' },
    evidence: { type: 'string', description: 'file:line, feature id or inventory entry checked.' },
  },
  required: ['id', 'outcome', 'reply', 'evidence'],
}
const REVIEW = { type: 'object', properties: { outcomes: { type: 'array', items: OUTCOME } }, required: ['outcomes'] }

const reviewer = (it, lens) => `You review design comments left on a map of a task-manager app before it is rebuilt, and decide each one.

The rebuild: ${args.brief}

Decisions already made by the product owner: ${args.decisions}

The comments to decide are in ${it.file} (screens: ${it.screens.join(', ')}). Read it first. Each has an id, the node it is about, a tag (web, desktop, both, general) and its text.

To check facts you may open: the screens' inventories (${it.inventory.join(', ')}), the feature titles (${it.features.join(', ')}), the full feature entries in ${args.featuresDir} (search them with Grep by feature id or word), and the reference app's source in ${args.ref} (Grep is the fastest way). Open nothing else.

For every comment, decide one outcome:
- answered: the reference or the feature list already answers it. Many comments claim something is missing; check before agreeing (for example, a control may exist in the command palette or a menu rather than on screen).
- decided: it asks for a design call that the brief, the decisions above, or plain consistency settles (for example "use one icon weight per button size"). State the decision.
- needs_mujtaba: it needs a product decision the brief and the decisions do not settle. Give the question and two or three options.
- invalid: the comment is wrong about the reference. Say why.
Write the reply as it would be posted under the comment. Be concrete, cite evidence, never invent.

Your lens: ${lens}`

const merger = (it, A, B) => `Two reviewers independently decided the same design comments on a task-manager app's rebuild map. Settle one outcome and one reply per comment.

Reviewer A: ${JSON.stringify(A.outcomes)}

Reviewer B: ${JSON.stringify(B.outcomes)}

Rules:
- Same outcome: keep it, and write one reply combining both reviewers' evidence.
- Different outcomes: if one reviewer cites evidence that the other missed, follow the evidence. If they still disagree on whether a product decision is needed, choose needs_mujtaba and write a question that names both views.
- Every comment id in either list gets exactly one entry. Keep replies plain, two to four sentences.`

const results = await pipeline(
  args.items,
  (it) =>
    parallel([
      () => agent(reviewer(it, 'check each claim against the reference source code first, then the feature list.'), { ...OPTS, agentType: TYPE.reviewer, schema: REVIEW, label: `A ${it.key}`, phase: 'Review' }),
      () => agent(reviewer(it, 'check each claim against the feature list and the inventories first, and open the source only to settle a doubt.'), { ...OPTS, agentType: TYPE.reviewer, schema: REVIEW, label: `B ${it.key}`, phase: 'Review' }),
    ]),
  async ([A, B], it) => {
    if (!A || !B) return { key: it.key, failed: true, A, B }
    const M = await agent(merger(it, A, B), { ...OPTS, agentType: TYPE.merger, schema: REVIEW, label: `merge ${it.key}`, phase: 'Merge' })
    return { key: it.key, merged: M, A: A.outcomes.length, B: B.outcomes.length }
  }
)
return results
