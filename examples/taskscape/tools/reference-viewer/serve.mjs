// Starts the reference's own Vite dev servers (main on 1420, tray on 1421) in the
// prepared copy. Dev mode keeps component names, which the capture's component dump needs.
import { spawn } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';

const copy = process.env.VIEWER_CACHE ?? path.join(os.homedir(), '.cache', 'taskscape-reference');
const vite = path.join(copy, 'node_modules', '.bin', 'vite');
const kids = ['main', 'tray'].map((app) =>
  spawn(vite, ['--clearScreen', 'false'], { cwd: path.join(copy, app), stdio: 'inherit' })
);
const stop = () => kids.forEach((k) => k.kill());
process.on('SIGINT', stop);
process.on('SIGTERM', stop);
