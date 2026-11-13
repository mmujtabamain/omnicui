export const meta = {
  name: 'taskscape-flow-comments',
  description: 'First comments for the flowchart page: one critic per screen reviews its inventory for gaps, inconsistencies and where the web and desktop versions should differ',
  phases: [{ title: 'Review' }],
}

// args: { brief, screens: [{ id, title, file, features }] }: file holds the screen's merged inventory;
// features is a file of the feature titles of the areas the screen draws on
const TYPE = args.agentTypes ?? { critic: 'ts-critic' }

const COMMENTS = {
  type: 'object',
  properties: {
    comments: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          node: { type: 'string', description: 'What it is about: "screen:<id>" for this screen, or "link:<from>><to>" for a link to another screen. Name the widget in the text.' },
          tag: { type: 'string', enum: ['web', 'desktop', 'both', 'general'] },
          text: { type: 'string', description: 'The comment: what you noticed and what you suggest, in two to four plain sentences.' },
        },
        required: ['node', 'tag', 'text'],
      },
    },
  },
  required: ['comments'],
}

const prompt = (s) => `You review one screen of a task-manager app before it is rebuilt, and write the first comments a design reviewer would leave on it.

The rebuild: ${args.brief}

Screen "${s.id}" (${s.title}). Its inventory (purpose, regions, widgets with variants, links to other screens) is in ${s.file}, and the titles of the features it must keep are in ${s.features}. Read both; open nothing else.

Write 4 to 8 comments, each about one thing:
- a gap: a feature the inventory does not show a widget for, or a widget with no clear purpose;
- an inconsistency: the same job done by different widgets or variants, or a variant that differs for no reason;
- a version difference: where the web version or the desktop version should behave or look differently from the reference, given the rebuild above (tag "web", "desktop" or "both").
Be concrete: name the widget or feature. Never invent features the files don't support.`

phase('Review')
return await parallel(
  args.screens.map((s) => () =>
    agent(prompt(s), { model: 'haiku', effort: 'medium', agentType: TYPE.critic, schema: COMMENTS, label: `comments ${s.id}` }).then((r) => ({ id: s.id, comments: r ? r.comments : [] })))
)
