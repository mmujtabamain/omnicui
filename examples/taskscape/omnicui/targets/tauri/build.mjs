// Builds the harness pages with the app's own toolchain (its Vite, React plugin and Tailwind,
// from examples/taskscape/tauri/node_modules), for production, into build/dist/{main,tray}/.
// Run by setup.sh, never per check.
import { createRequire } from 'node:module';
import { readdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const app = join(here, '../../../tauri');
const req = createRequire(join(app, 'main/package.json'));
const load = async (name) => (await import(pathToFileURL(req.resolve(name)).href));
const vite = await load('vite');
const react = (await load('@vitejs/plugin-react')).default;
const tailwind = (await load('@tailwindcss/vite')).default;

for (const surface of ['main', 'tray']) {
  const root = join(app, surface);
  const outDir = join(here, 'build/dist', surface);
  // Tailwind finds the app's classes from the working folder, as in the app's own build.
  process.chdir(root);
  await vite.build({
    root,
    configFile: false,
    logLevel: 'warn',
    plugins: [react(), tailwind()],
    define: { __SURFACE__: JSON.stringify(surface), 'process.env.NODE_ENV': '"production"' },
    resolve: {
      alias: [
        { find: '@taskscape/common-ui', replacement: join(app, 'common-ui/src') },
        { find: '@omnicui-registry', replacement: join(root, 'src/omnicui/registry.tsx') },
        { find: /^@tauri-apps\/api/, replacement: join(app, 'node_modules/@tauri-apps/api') },
      ],
      dedupe: ['react', 'react-dom', '@tauri-apps/api', 'tailwind-merge'],
    },
    build: {
      outDir,
      emptyOutDir: true,
      minify: true,
      cssCodeSplit: false,
      lib: { entry: join(here, 'harness.ts'), formats: ['es'], fileName: () => 'harness.js' },
    },
  });
  const css = readdirSync(outDir).filter((f) => f.endsWith('.css'));
  writeFileSync(
    join(outDir, 'index.html'),
    `<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Taskscape (${surface})</title>
${css.map((f) => `<link rel="stylesheet" href="./${f}">`).join('\n')}</head>
<body><div id="root"></div><script type="module" src="./harness.js"></script></body>
</html>
`
  );
  console.error(`tauri: built the ${surface} harness into build/dist/${surface}`);
}
