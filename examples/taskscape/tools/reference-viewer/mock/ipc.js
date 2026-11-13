// Loaded before the reference's own entry: answers its Tauri calls from a fixture.
//   ?fixture=standard|empty|long   which data set
//   ?t=<ms>                        the clock the data is stamped against (fixed for screenshots)
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import { createBackend } from './backend.js';

const params = new URLSearchParams(location.search);
const fixed = Number(params.get('t') ?? 0);
const now = () => (fixed ? fixed : Date.now());
const label = window.__PANEL__ ? 'settings' : 'main';

mockWindows(label, 'main', 'settings');
const backend = createBackend(params.get('fixture') ?? 'standard', now);
mockIPC((cmd, args) => backend.handle(cmd, args), { shouldMockEvents: true });
// Built from a string: Vite rewrites `new URL(template, import.meta.url)` into a glob import.
const files = import.meta.url.replace(/ipc\.js.*$/, 'files/');
window.__TAURI_INTERNALS__.convertFileSrc = (p) => files + p.split('/').pop();

window.__viewer = { backend, emit, log: backend.log };
