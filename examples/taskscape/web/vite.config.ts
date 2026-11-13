// Taskscape for the web is the Tauri app's own frontend (../tauri/main/src and
// ../tauri/common-ui/src), built for a browser. Nothing is copied: the shared
// sources are compiled from where they are, and only the modules that talk to
// Tauri or draw desktop chrome are swapped for the browser's versions in
// src/app/ (OVERRIDES below, each file says what differs and why); two shared
// modules get a few exact lines changed (PATCHES).
//
// `OMNICUI=1 vite build` also builds omnicui.html, the page the omnicui target
// mounts single surfaces in (src/omnicui/registry.tsx); a normal build has
// only the app.
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { existsSync } from 'node:fs';
import { dirname, join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig, type Plugin } from 'vite';

const here = dirname(fileURLToPath(import.meta.url));
const tauri = join(here, '../tauri');
const app = (p: string) => join(here, 'src/app', p);

/** Shared module (under ../tauri) → the web's version of it. */
const OVERRIDES: Record<string, string> = {
  // Tauri IPC → HTTP to server/.
  'main/src/api.ts': app('api.ts'),
  // The window: a page header instead of a window title bar, Settings and the
  // capture bar drawn in the page, URLs, the narrow layout.
  'main/src/App.tsx': app('App.tsx'),
  'main/src/components/TitleBar.tsx': app('TitleBar.tsx'),
  'main/src/components/WindowControls.tsx': app('WindowControls.tsx'),
  'main/src/components/windowChrome.tsx': app('windowChrome.tsx'),
  'main/src/windows/settings/PathButton.tsx': app('PathButton.tsx'),
  // Files: uploads and downloads instead of the system's file dialogs and
  // paths; no screenshots in a browser.
  'main/src/commands/lists.ts': app('lists.ts'),
  'main/src/components/preview/AttachmentSection.tsx': app('AttachmentSection.tsx'),
  'main/src/components/preview/DraftInspector.tsx': app('DraftInspector.tsx'),
  'common-ui/src/fileKind.ts': app('fileKind.ts'),
};
/** Shared modules the web needs a line or two changed in, where swapping the
 *  whole module would copy hundreds of lines that are the same. Each hunk is
 *  exact: if the shared source no longer has it, the build stops and says so. */
const PATCHES: Record<string, { why: string; hunks: [string, string][] }> = {
  'main/src/components/TaskRow.tsx': {
    why:
      "the design review's direction (SPEC-PRODUCT.md): the round check marks the task done; " +
      'the Tauri app still selects with it and completes only with ⌥ held',
    hunks: [
      ['const alt = useAltPressed();\n  const doneMode = alt;', 'const doneMode = true;'],
      ["doneMode ? 'rounded-field' : 'rounded-full',", "'rounded-full',"],
      ['if (e.altKey) return onToggleDone();\n        onToggleSelect();', 'onToggleDone();'],
    ],
  },
  'main/src/components/TaskPane.tsx': {
    why: 'the empty state names the new-task shortcut as bound (Alt+N on the web), not ⌘N',
    hunks: [
      [
        "  const captureHint = formatAccel(",
        "  const newTaskHint = formatAccel(\n    useHotkeyStore((s) => s.map['new_task'] ?? '')\n  );\n  const captureHint = formatAccel(",
      ],
      ['`Press + or ⌘N to add a task, or ${captureHint}', '`Press + or ${newTaskHint} to add a task, or ${captureHint}'],
      ["'Press + or ⌘N to add a task'", '`Press + or ${newTaskHint} to add a task`'],
    ],
  },
};

const overrideByPath = new Map(
  Object.entries(OVERRIDES).map(([k, v]) => [join(tauri, k), v])
);

const patchByPath = new Map(
  Object.entries(PATCHES).map(([k, v]) => [join(tauri, k), v])
);

function patches(): Plugin {
  return {
    name: 'taskscape-patches',
    enforce: 'pre',
    transform(code, id) {
      const patch = patchByPath.get(id.split('?')[0]);
      if (!patch) return null;
      for (const [find, replace] of patch.hunks) {
        if (!code.includes(find))
          this.error(`${relative(here, id)} no longer has ${JSON.stringify(find)} (patched for: ${patch.why})`);
        code = code.replace(find, replace);
      }
      return { code, map: null };
    },
  };
}

const isBare = (s: string) =>
  !s.startsWith('.') && !s.startsWith('/') && !s.startsWith('\0') && !/^[a-zA-Z]:/.test(s);

/** Swaps the overridden shared modules, and resolves the shared sources' own
 *  package imports (react, zustand, tiptap…) from this app's node_modules, so
 *  there is one React and the shared folder needs no install of its own. */
function shared(): Plugin {
  const entry = join(here, 'src/main.tsx');
  return {
    name: 'taskscape-shared',
    enforce: 'pre',
    async resolveId(source, importer, options) {
      if (!importer) return null;
      const fromShared = importer.startsWith(tauri + sep);
      if (fromShared && isBare(source) && !source.startsWith('@taskscape/')) {
        if (source.startsWith('@tauri-apps/'))
          this.error(`${relative(here, importer)} imports ${source}: give it a web version in OVERRIDES`);
        return this.resolve(source, entry, { ...options, skipSelf: true });
      }
      const resolved = await this.resolve(source, importer, { ...options, skipSelf: true });
      if (!resolved) return null;
      const file = resolved.id.split('?')[0];
      const swap = overrideByPath.get(file);
      return swap ?? resolved;
    },
  };
}

const omnicui = process.env.OMNICUI === '1';
const harness = join(here, 'omnicui.html');

export default defineConfig({
  plugins: [shared(), patches(), react(), tailwindcss()],
  resolve: {
    alias: [
      { find: '@taskscape/common-ui', replacement: join(tauri, 'common-ui/src') },
      // The Tauri app's own frontend, which the web app is built from.
      { find: /^@shared\//, replacement: join(tauri, 'main/src') + '/' },
      // Tauri's events carried messages between the app's windows; in a page,
      // every "window" is in one document (src/platform/events.ts).
      { find: '@tauri-apps/api/event', replacement: join(here, 'src/platform/events.ts') },
    ],
    dedupe: ['react', 'react-dom', 'zustand', 'tailwind-merge', 'motion'],
  },
  define: { __TASKSCAPE_VERSION__: JSON.stringify(process.env.npm_package_version ?? '0.1.0') },
  server: { fs: { allow: [here, tauri] } },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: join(here, 'index.html'),
        ...(omnicui && existsSync(harness) ? { omnicui: harness } : {}),
      },
    },
  },
});
