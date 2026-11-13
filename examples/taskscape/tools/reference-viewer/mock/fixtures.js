// Data sets for the viewer, stamped relative to `base` so "2d ago" reads the same in
// every capture. Ids are readable so captures and features can name them.
const MIN = 60_000;
const DAY = 24 * 60 * MIN;

export function fixtures(base) {
  const ago = (ms) => base - ms;
  const row = (o) => ({ created_at: ago(o.age ?? DAY), updated_at: ago(o.age ?? DAY), ...o });

  const projects = [
    row({ id: 'p-home', name: 'Home', age: 30 * DAY }),
    row({ id: 'p-launch', name: 'Spring launch', age: 20 * DAY }),
  ];
  const lists = [
    row({ id: 'l-errands', project_id: 'p-home', name: 'Errands', sort_order: 0 }),
    row({ id: 'l-reading', project_id: 'p-home', name: 'Reading', sort_order: 1 }),
    row({ id: 'l-garden', project_id: 'p-home', name: 'Garden', sort_order: 2 }),
    row({ id: 'l-site', project_id: 'p-launch', name: 'Website', sort_order: 0 }),
    row({ id: 'l-press', project_id: 'p-launch', name: 'Press kit', sort_order: 1 }),
  ];
  let order = 0;
  const t = (id, list_id, title, extra = {}) =>
    row({ id, list_id, title, parent_id: null, notes: null, done: false, sort_order: order++, deleted_at: null, ...extra });
  const tasks = [
    t('t-groceries', 'l-errands', 'Buy groceries for the week', { age: 2 * DAY }),
    t('t-milk', 'l-errands', 'Oat milk', { parent_id: 't-groceries', done: true }),
    t('t-bread', 'l-errands', 'Sourdough loaf', { parent_id: 't-groceries' }),
    t('t-apples', 'l-errands', 'Apples', { parent_id: 't-groceries' }),
    t('t-pharmacy', 'l-errands', 'Pick up prescription', { notes: 'Counter closes at 6' }),
    t('t-post', 'l-errands', 'Post the parcel to Mum', { done: true, age: 3 * DAY }),
    t('t-bike', 'l-errands', 'Book bike service', { age: 5 * DAY }),
    t('t-keys', 'l-errands', 'Cut a spare key'),
    t('t-novel', 'l-reading', 'Finish the novel on the nightstand'),
    t('t-article', 'l-reading', 'Read the long article on soil health', { done: true }),
    t('t-library', 'l-reading', 'Return library books'),
    t('t-seeds', 'l-garden', 'Order tomato seeds'),
    t('t-hedge', 'l-garden', 'Trim the hedge'),
    t('t-hero', 'l-site', 'Rewrite the hero section', { age: 4 * DAY }),
    t('t-copy', 'l-site', 'Draft headline options', { parent_id: 't-hero' }),
    t('t-shot', 'l-site', 'Pick the hero image', { parent_id: 't-hero', done: true }),
    t('t-pricing', 'l-site', 'Pricing page: compare plans table'),
    t('t-faq', 'l-site', 'Collect FAQ questions from support'),
    t('t-logo', 'l-press', 'Export logos in SVG and PNG'),
    t('t-bios', 'l-press', 'Team bios, 50 words each', { done: true }),
    t('t-old-flyer', 'l-errands', 'Print the old flyer', { deleted_at: ago(2 * DAY) }),
    t('t-old-call', 'l-site', 'Call the printer about samples', { deleted_at: ago(30 * MIN) }),
  ];
  const notes = [
    row({ id: 'n-1', task_id: 't-hero', sort_order: 0, content: '<p>Lead with the <strong>one-line promise</strong>, then three proof points.</p><ul><li>Fast capture</li><li>Works offline</li><li>Keyboard first</li></ul>' }),
    row({ id: 'n-2', task_id: 't-hero', sort_order: 1, content: '<p>Ask design for two layouts by Friday.</p>' }),
    row({ id: 'n-3', task_id: 't-groceries', sort_order: 0, content: '<p>Check the cupboard before going.</p>' }),
  ];
  const attachments = [
    row({ id: 'a-brief', task_id: 't-hero', name: 'Brand brief', link_type: 'reference', location: 'https://example.com/brand-brief' }),
    row({ id: 'a-sketch', task_id: 't-hero', name: 'hero-sketch.png', link_type: 'copy', location: 'attachments/hero-sketch.png' }),
  ];
  const standard = {
    projects,
    lists,
    tasks,
    notes,
    attachments,
    settings: { last_active_project: 'p-home', last_active_list: 'l-errands' },
  };

  const empty = {
    projects: [row({ id: 'p-home', name: 'Home' })],
    lists: [row({ id: 'l-inbox', project_id: 'p-home', name: 'Inbox', sort_order: 0 })],
    tasks: [],
    settings: { last_active_project: 'p-home', last_active_list: 'l-inbox' },
  };
  const nolists = { projects: [row({ id: 'p-home', name: 'Home' })], lists: [], tasks: [], settings: {} };

  order = 0;
  const long = {
    ...standard,
    tasks: [
      ...tasks,
      ...Array.from({ length: 40 }, (_, i) =>
        t(`t-long-${i + 1}`, 'l-errands', `Errand number ${i + 1}`, { done: i % 5 === 0 })
      ),
    ],
  };
  return { standard, empty, nolists, long };
}
