// The omnicui target's page (omnicui.html, built only with OMNICUI=1): the web
// app's surfaces mounted one at a time by the registry, on data the server's
// test hooks load per mount (server/src/hooks.rs). It gives the target three
// things on `window`:
//
// - omnicuiMount(component, props): a fresh fixture, then the surface;
// - omnicuiEvents(): the domain events since the last call, the server's (what
//   was written to the store) and the page's (what the user did that writes
//   nothing: focusing a task, opening a draft, asking a question);
// - omnicuiBusy(): true while a request to the server is in flight or a mount
//   is loading, which the target waits out before it answers.
import { requestsInFlight } from '../platform/http';
import '../index.css';
import '../material-symbols.css';

type Fields = Record<string, unknown>;
type Ev = { name: string; fields: Fields };
const local: Ev[] = [];
const record = (name: string, fields: Fields = {}) => local.push({ name, fields });

let keys: Record<string, string> = {};
let names: Record<string, string> = {};
let mounting = false;

// Opening a link or a stored file is a new tab; here it is what the user asked
// for, recorded, and no tab is opened.
window.open = ((url?: string | URL) => {
  record('attachmentOpened', { location: String(url ?? '') });
  return null;
}) as typeof window.open;

async function serverEvents(): Promise<Ev[]> {
  const res = await fetch('/__test/events');
  return res.ok ? res.json() : [];
}

Object.assign(window, {
  omnicuiBusy: () => mounting || requestsInFlight() > 0,
  async omnicuiEvents() {
    const server = await serverEvents();
    return [...local.splice(0), ...server];
  },
  async omnicuiMount(component: string, props: Record<string, string | number | boolean>) {
    mounting = true;
    try {
      const res = await fetch('/__test/fixture', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ seed: String(props.seed ?? 'standard') }),
      });
      if (!res.ok) return `fixture: ${await res.text()}`;
      keys = (await res.json()).keys;
      names = Object.fromEntries(Object.entries(keys).map(([k, id]) => [id, k]));
      const registry = await import('./registry');
      if (!registry.COMPONENTS.includes(component)) return `no component ${component}`;
      await registry.mount(component, props, document.getElementById('root')!, {
        emit: record,
        id: (key: string) => (key ? (keys[key] ?? key) : ''),
        key: (id: string) => names[id] ?? id,
      });
      // What loading wrote (a setting the props asked for) is not the test's.
      local.length = 0;
      await serverEvents();
      return null;
    } catch (e) {
      return `mount ${component}: ${(e as Error)?.message ?? e}`;
    } finally {
      mounting = false;
    }
  },
});
