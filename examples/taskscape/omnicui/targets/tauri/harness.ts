// The page the tauri adapter opens: the app's frontend with Tauri's runtime replaced. Tauri's
// own `mockIPC` routes every `invoke` to the target's Rust server (server/), which answers it
// with taskscape_common's Store on the fixture's database and reports the domain events. The
// window plumbing Rust does natively (presenting, revealing and theming windows, the macOS
// screenshot tool) is answered here. Then the app's registry entry point (`@omnicui-registry`)
// mounts the surface a test asks for.
import { emit } from '@tauri-apps/api/event';
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';

declare const __SURFACE__: 'main' | 'tray';

type Fields = Record<string, unknown>;
const events: { name: string; fields: Fields }[] = [];
const record = (name: string, fields: Fields = {}) => events.push({ name, fields });

let keys: Record<string, string> = {};
let names: Record<string, string> = {};
let pending = 0;
let mounting = false;

// The tray's window label is "main" too (the mini bar).
mockWindows('main');

/** What Rust does without the store: window mechanics and the OS's own tools. */
function local(cmd: string, args: Record<string, unknown>): { value: unknown } | null {
  switch (cmd) {
    case 'present_window':
    case 'reveal_main':
    case 'boot_done':
    case 'set_window_theme':
    case 'set_bar_height':
      return { value: null };
    case 'is_low_power_mode':
      return { value: false };
    case 'open_settings':
      record('settingsOpened');
      return { value: null };
    case 'hide_mini':
      record('dismissed');
      return { value: null };
    case 'open_main':
      record('mainOpened');
      return { value: null };
    case 'capture_and_attach':
      // The tray's background capture: pending, then the macOS screenshot tool's outcome.
      // Off macOS the app's capture fails with this message (common/src/screenshot.rs).
      record('screenshotRequested');
      queueMicrotask(async () => {
        await emit('screenshot-pending');
        await emit('screenshot-error', 'screen capture is currently only implemented on macOS');
      });
      return { value: null };
    case 'plugin:app|version':
      return { value: '0.1.0' };
    case 'plugin:app|name':
      return { value: 'Taskscape' };
    case 'plugin:dialog|open':
    case 'plugin:dialog|save':
      // A file picker the user cancels.
      return { value: null };
  }
  if (cmd.startsWith('plugin:window|')) {
    // Questions about the window get a focused, windowed answer; the rest are actions.
    const method = cmd.slice('plugin:window|'.length);
    const answers: Record<string, unknown> = {
      is_focused: true,
      is_fullscreen: false,
      is_maximized: false,
      is_minimized: false,
      is_visible: true,
      scale_factor: 1,
      theme: matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
      inner_size: { width: innerWidth, height: innerHeight },
      outer_size: { width: innerWidth, height: innerHeight },
      inner_position: { x: 0, y: 0 },
      outer_position: { x: 0, y: 0 },
    };
    if (method in answers) return { value: answers[method] };
    record('windowAction', { action: method });
    return { value: null };
  }
  if (cmd.startsWith('plugin:opener|')) {
    record('opened', { what: String(args.path ?? args.url ?? '') });
    return { value: null };
  }
  return null;
}

mockIPC(
  async (cmd, payload) => {
    const args = (payload ?? {}) as Record<string, unknown>;
    const here = local(cmd, args);
    if (here) return here.value;
    pending++;
    try {
      const res = await fetch('/invoke', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ cmd, args }),
      });
      const body = await res.json();
      for (const e of body.events ?? []) events.push(e);
      // Tauri rejects with the command's error string.
      if ('error' in body) throw body.error;
      return body.ok;
    } finally {
      pending--;
    }
  },
  { shouldMockEvents: true }
);

// `convertFileSrc` (copied attachments, screenshots): served by the server from the data folder.
(window as any).__TAURI_INTERNALS__.convertFileSrc = (path: string) =>
  `/asset?path=${encodeURIComponent(path)}`;

Object.assign(window, {
  /** The adapter waits while this is true: a command in flight, or a mount loading. */
  omnicuiBusy: () => mounting || pending > 0,
  omnicuiEvents: () => events.splice(0),
  async omnicuiMount(component: string, props: Record<string, string | number | boolean>) {
    mounting = true;
    try {
      const res = await fetch('/fixture', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ seed: String(props.seed ?? 'standard') }),
      });
      if (!res.ok) return `fixture: ${await res.text()}`;
      keys = (await res.json()).keys;
      names = Object.fromEntries(Object.entries(keys).map(([k, id]) => [id, k]));
      const registry = await import('@omnicui-registry');
      if (!registry.COMPONENTS.includes(component)) return `no component ${component}`;
      await registry.mount(component, props, document.getElementById('root')!, {
        emit: record,
        id: (key: string) => (key ? (keys[key] ?? key) : ''),
        key: (id: string) => names[id] ?? id,
      });
      events.length = 0;
      return null;
    } catch (e) {
      return `mount ${component}: ${(e as Error)?.message ?? e}`;
    } finally {
      mounting = false;
    }
  },
});
