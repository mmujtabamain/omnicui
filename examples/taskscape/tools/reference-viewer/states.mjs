// The screen states the capture records. Each names the feature areas it shows (the
// numbers of the audit's areas) so an area's solver B gets exactly its states.
const settle = (p, ms = 650) => p.waitForTimeout(ms);
const row = (p, title) => p.getByText(title, { exact: true }).first();
const palette = async (p, label) => {
  await p.keyboard.press('Meta+K');
  await settle(p, 400);
  await p.keyboard.type(label, { delay: 15 });
  await settle(p, 300);
  await p.keyboard.press('Enter');
  await settle(p);
};
const toLaunch = async (p) => {
  await p.getByRole('button', { name: /Home/ }).first().click();
  await settle(p, 400);
  await p.getByText('Spring launch', { exact: true }).first().click();
  await settle(p);
};
const select = async (p, title) => {
  await row(p, title).click();
  await settle(p);
};

export const states = [
  // main window
  { id: 'main-default', areas: [1, 2, 9, 11, 18], steps: async () => {} },
  { id: 'main-dark', areas: [11, 18], theme: 'dark', steps: async () => {} },
  { id: 'row-hover', areas: [1], steps: async (p) => row(p, 'Book bike service').hover() },
  { id: 'row-selected', areas: [1, 3], steps: (p) => select(p, 'Book bike service') },
  { id: 'row-selected-dark', areas: [1, 3, 11], theme: 'dark', steps: (p) => select(p, 'Book bike service') },
  { id: 'row-with-subtasks-selected', areas: [1, 3], steps: (p) => select(p, 'Buy groceries for the week') },
  { id: 'subtask-selected', areas: [1, 3], steps: (p) => select(p, 'Sourdough loaf') },
  { id: 'done-row-selected', areas: [1, 3], steps: (p) => select(p, 'Post the parcel to Mum') },
  {
    id: 'multi-select-cmd', areas: [1, 4], steps: async (p) => {
      await select(p, 'Book bike service');
      await row(p, 'Cut a spare key').click({ modifiers: ['Meta'] });
      await settle(p);
    },
  },
  {
    id: 'multi-select-shift', areas: [1, 4], steps: async (p) => {
      await select(p, 'Pick up prescription');
      await row(p, 'Cut a spare key').click({ modifiers: ['Shift'] });
      await settle(p);
    },
  },
  {
    id: 'option-held', areas: [8, 1], steps: async (p) => {
      await p.keyboard.down('Alt');
      await settle(p);
    },
  },
  {
    id: 'option-held-hover', areas: [8, 1], steps: async (p) => {
      await row(p, 'Book bike service').hover();
      await p.keyboard.down('Alt');
      await settle(p);
    },
  },
  {
    id: 'option-held-selected', areas: [8, 1, 3], steps: async (p) => {
      await select(p, 'Book bike service');
      await p.keyboard.down('Alt');
      await settle(p);
    },
  },
  {
    id: 'cmd-held', areas: [8, 2], steps: async (p) => {
      await p.keyboard.down('Meta');
      await settle(p, 1200);
    },
  },
  {
    id: 'shift-held', areas: [8], steps: async (p) => {
      await p.keyboard.down('Shift');
      await settle(p);
    },
  },
  { id: 'row-context-menu', areas: [6, 1, 7], steps: (p) => row(p, 'Book bike service').click({ button: 'right' }) },
  {
    id: 'row-more-menu', areas: [6, 1], steps: async (p) => {
      await row(p, 'Book bike service').hover();
      await p.locator('[title^="More"], [aria-label^="More"]').first().click().catch(() => {});
      await settle(p);
    },
  },
  {
    id: 'multi-context-menu', areas: [6, 4], steps: async (p) => {
      await select(p, 'Book bike service');
      await row(p, 'Cut a spare key').click({ modifiers: ['Meta'] });
      await row(p, 'Cut a spare key').click({ button: 'right' });
      await settle(p);
    },
  },
  { id: 'tab-context-menu', areas: [6, 2, 9], steps: (p) => p.getByText('Reading', { exact: true }).first().click({ button: 'right' }) },
  { id: 'tab-hover', areas: [2], steps: (p) => p.getByText('Reading', { exact: true }).first().hover() },
  {
    id: 'project-menu', areas: [2, 9], steps: async (p) => {
      await p.getByRole('button', { name: /Home/ }).first().click();
      await settle(p);
    },
  },
  { id: 'project-launch', areas: [2, 9], steps: toLaunch },
  {
    id: 'inspector-notes', areas: [3, 4], steps: async (p) => {
      await toLaunch(p);
      await select(p, 'Rewrite the hero section');
    },
  },
  {
    id: 'note-editing', areas: [3], steps: async (p) => {
      await toLaunch(p);
      await select(p, 'Rewrite the hero section');
      await p.getByText('Ask design for two layouts by Friday.').click();
      await settle(p);
    },
  },
  {
    id: 'attachment-lightbox', areas: [4], steps: async (p) => {
      await toLaunch(p);
      await select(p, 'Rewrite the hero section');
      await p.locator('img[src*="hero-sketch"]').first().click();
      await settle(p, 900);
    },
  },
  {
    id: 'attachment-menu', areas: [4, 6], steps: async (p) => {
      await toLaunch(p);
      await select(p, 'Rewrite the hero section');
      await p.getByText('Brand brief').first().click({ button: 'right' });
      await settle(p);
    },
  },
  { id: 'palette', areas: [6, 8], steps: async (p) => { await p.keyboard.press('Meta+K'); await settle(p); } },
  {
    id: 'palette-search', areas: [6, 8], steps: async (p) => {
      await p.keyboard.press('Meta+K');
      await settle(p, 400);
      await p.keyboard.type('new', { delay: 20 });
      await settle(p);
    },
  },
  {
    id: 'palette-no-match', areas: [6], steps: async (p) => {
      await p.keyboard.press('Meta+K');
      await settle(p, 400);
      await p.keyboard.type('zzzz', { delay: 20 });
      await settle(p);
    },
  },
  {
    id: 'search-typed', areas: [5], steps: async (p) => {
      await p.keyboard.press('Meta+F');
      await settle(p, 300);
      await p.keyboard.type('b', { delay: 20 });
      await settle(p);
    },
  },
  {
    id: 'search-no-match', areas: [5], steps: async (p) => {
      await p.keyboard.press('Meta+F');
      await settle(p, 300);
      await p.keyboard.type('zzzz', { delay: 20 });
      await settle(p);
    },
  },
  {
    id: 'filter-panel', areas: [5], steps: async (p) => {
      await p.locator('[title^="Filter"], [aria-label^="Filter"], [title^="Sort"]').first().click();
      await settle(p);
    },
  },
  { id: 'new-task-draft', areas: [4, 1, 8], steps: async (p) => { await p.keyboard.press('Meta+N'); await settle(p); } },
  {
    id: 'new-task-typed', areas: [4, 1], steps: async (p) => {
      await p.keyboard.press('Meta+N');
      await settle(p, 400);
      await p.keyboard.type('Water the plants', { delay: 15 });
      await settle(p);
    },
  },
  {
    id: 'new-task-added', areas: [1, 7], steps: async (p) => {
      await p.keyboard.press('Meta+N');
      await settle(p, 400);
      await p.keyboard.type('Water the plants', { delay: 15 });
      await p.keyboard.press('Enter');
      await settle(p);
    },
  },
  {
    id: 'rename-inline', areas: [1, 8], steps: async (p) => {
      await select(p, 'Book bike service');
      await p.keyboard.press('Enter');
      await settle(p);
    },
  },
  { id: 'sheet-new-list', areas: [6, 9], steps: async (p) => { await p.keyboard.press('Meta+Shift+N'); await settle(p, 900); } },
  { id: 'sheet-new-project', areas: [6, 9], steps: async (p) => { await p.keyboard.press('Meta+Shift+P'); await settle(p, 900); } },
  {
    id: 'sheet-rename-list', areas: [6, 9], steps: async (p) => {
      await p.getByText('Reading', { exact: true }).first().click({ button: 'right' });
      await settle(p, 400);
      await p.getByText(/^Rename/).first().click();
      await settle(p, 900);
    },
  },
  {
    id: 'sheet-delete-list', areas: [6, 9, 7], steps: async (p) => {
      await p.getByText('Reading', { exact: true }).first().click({ button: 'right' });
      await settle(p, 400);
      await p.getByText(/^Delete/).first().click();
      await settle(p, 900);
    },
  },
  {
    id: 'delete-confirm', areas: [6, 7], steps: async (p) => {
      await select(p, 'Book bike service');
      await p.keyboard.press('Meta+Backspace');
      await settle(p, 700);
    },
  },
  {
    id: 'toast-after-delete', areas: [6, 7], steps: async (p) => {
      await select(p, 'Book bike service');
      await p.keyboard.press('Meta+Backspace');
      await settle(p, 700);
      await p.getByRole('button', { name: 'Move', exact: true }).click();
      await settle(p, 500);
    },
  },
  {
    id: 'after-undo', areas: [7], steps: async (p) => {
      await select(p, 'Book bike service');
      await p.keyboard.press('Meta+Backspace');
      await settle(p, 700);
      await p.getByRole('button', { name: 'Move', exact: true }).click();
      await settle(p, 500);
      await p.keyboard.press('Meta+Z');
      await settle(p);
    },
  },
  {
    id: 'multi-select-cmd-two', areas: [1, 4], steps: async (p) => {
      await row(p, 'Book bike service').click({ modifiers: ['Meta'] });
      await row(p, 'Cut a spare key').click({ modifiers: ['Meta'] });
      await settle(p);
    },
  },
  {
    id: 'option-click-check', areas: [1, 7, 8], steps: async (p) => {
      await row(p, 'Book bike service').hover();
      await p.keyboard.down('Alt');
      await settle(p, 300);
      await p.locator('button[title="Mark done"]').nth(1).click({ modifiers: ['Alt'] });
      await settle(p, 900);
    },
  },
  {
    id: 'round-check-click', areas: [1, 7], steps: async (p) => {
      await row(p, 'Book bike service').locator('xpath=ancestor::*[@role or @data-task-id][1]').first().hover().catch(() => {});
      await p.getByText('Book bike service', { exact: true }).first().locator('xpath=preceding::button[1]').click();
      await settle(p, 900);
    },
  },
  { id: 'trash', areas: [7], steps: (p) => palette(p, 'Open Trash') },
  {
    id: 'trash-selected', areas: [7], steps: async (p) => {
      await palette(p, 'Open Trash');
      await p.getByText('Print the old flyer').first().click();
      await settle(p);
    },
  },
  { id: 'completed-hidden', areas: [1, 5], steps: async (p) => { await p.keyboard.press('Meta+Shift+H'); await settle(p); } },
  { id: 'split-view', areas: [9, 1], steps: async (p) => { await p.keyboard.press('Meta+Alt+S'); await settle(p, 900); } },
  { id: 'preview-hidden', areas: [9, 3], steps: async (p) => { await p.keyboard.press('Meta+\\'); await settle(p); } },
  { id: 'collapsed-all', areas: [1], steps: async (p) => { await p.keyboard.press('Meta+Alt+Minus'); await settle(p); } },
  { id: 'list-switch-3', areas: [2, 8, 9], steps: async (p) => { await p.keyboard.press('Meta+3'); await settle(p); } },
  { id: 'narrow-window', areas: [2, 9], viewport: { width: 820, height: 640 }, steps: async () => {} },
  { id: 'empty-list', areas: [1], fixture: 'empty', steps: async () => {} },
  { id: 'no-lists', areas: [9, 2], fixture: 'nolists', steps: async () => {} },
  { id: 'long-list', areas: [1], fixture: 'long', steps: async () => {} },
  {
    id: 'long-list-scrolled', areas: [1], fixture: 'long', steps: async (p) => {
      await p.mouse.move(400, 400);
      await p.mouse.wheel(0, 900);
      await settle(p);
    },
  },

  // Settings window
  { id: 'settings-general', areas: [10], window: 'settings', steps: async () => {} },
  { id: 'settings-dark', areas: [10, 11], window: 'settings', theme: 'dark', steps: async () => {} },
  { id: 'settings-capture', areas: [10, 13], window: 'settings', steps: (p) => p.getByText('Capture', { exact: true }).first().click() },
  { id: 'settings-shortcuts', areas: [10, 8], window: 'settings', steps: (p) => p.getByText('Shortcuts', { exact: true }).first().click() },
  {
    id: 'settings-shortcuts-scrolled', areas: [10, 8], window: 'settings', steps: async (p) => {
      await p.getByText('Shortcuts', { exact: true }).first().click();
      await settle(p, 400);
      await p.mouse.move(700, 400);
      await p.mouse.wheel(0, 1200);
      await settle(p);
    },
  },
  {
    id: 'settings-shortcut-recording', areas: [10, 8], window: 'settings', steps: async (p) => {
      await p.getByText('Shortcuts', { exact: true }).first().click();
      await settle(p, 400);
      await p.getByText('New task', { exact: true }).first().locator('xpath=following::button[1]').click();
      await settle(p);
    },
  },
  { id: 'settings-about', areas: [10], window: 'settings', steps: (p) => p.getByText('About', { exact: true }).first().click() },
  {
    id: 'settings-search', areas: [10], window: 'settings', steps: async (p) => {
      await p.getByPlaceholder('Search').first().fill('motion');
      await settle(p);
    },
  },

  // capture bar
  { id: 'capture-bar', areas: [13], window: 'tray', steps: async () => {} },
  { id: 'capture-bar-dark', areas: [13, 11], window: 'tray', theme: 'dark', steps: async () => {} },
  {
    id: 'capture-bar-typed', areas: [13], window: 'tray', steps: async (p) => {
      await p.keyboard.type('Call the plumber', { delay: 15 });
      await settle(p);
    },
  },
  {
    id: 'capture-bar-notes', areas: [13], window: 'tray', steps: async (p) => {
      await p.keyboard.type('Call the plumber', { delay: 15 });
      await p.keyboard.press('Tab');
      await settle(p);
    },
  },
];
