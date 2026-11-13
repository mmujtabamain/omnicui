// `@tauri-apps/api/event` for a page. In the Tauri app these events carry
// messages between its windows (Settings tells the main window the theme
// changed, the tray asks it to reload). In the browser every surface is drawn
// in one document, so an event goes to this page's listeners; it is also
// posted to the app's other tabs (BroadcastChannel), so two tabs on the same
// data stay in step: a theme or shortcut chosen in one, a change made in one
// (api.ts emits `refresh` after every write), shows up in the other.
// The shape is Tauri's, so the shared stores that listen work unchanged.

export interface Event<T> {
  event: string;
  id: number;
  payload: T;
}
export type EventCallback<T> = (event: Event<T>) => void;
export type UnlistenFn = () => void;

const listeners = new Map<string, Set<EventCallback<unknown>>>();
let nextId = 1;

let channel: BroadcastChannel | null = null;
try {
  channel = new BroadcastChannel('taskscape');
  channel.onmessage = (m: MessageEvent<{ event: string; payload: unknown }>) =>
    deliver(m.data.event, m.data.payload);
} catch {
  // No BroadcastChannel: events stay in this tab.
}

function deliver(event: string, payload: unknown) {
  for (const cb of [...(listeners.get(event) ?? [])])
    cb({ event, id: nextId++, payload });
}

export async function listen<T>(
  event: string,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  let set = listeners.get(event);
  if (!set) listeners.set(event, (set = new Set()));
  const cb = handler as EventCallback<unknown>;
  set.add(cb);
  return () => set.delete(cb);
}

export async function once<T>(
  event: string,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  const off = await listen<T>(event, (e) => {
    off();
    handler(e);
  });
  return off;
}

/** To this page's listeners and the app's other tabs. */
export async function emit<T>(event: string, payload?: T): Promise<void> {
  deliver(event, payload);
  channel?.postMessage({ event, payload: payload ?? null });
}

/** To the app's other tabs only (this page already has the change). */
export function emitToOtherTabs<T>(event: string, payload?: T): void {
  channel?.postMessage({ event, payload: payload ?? null });
}
