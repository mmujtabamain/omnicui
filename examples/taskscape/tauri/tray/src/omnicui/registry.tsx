// The component registry a test harness mounts the capture bar from — the same
// App main.tsx renders, revealed the way Rust reveals it (`mini-shown`, top
// anchor). Nothing here changes how the bar behaves.
import { emit } from '@tauri-apps/api/event';
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from '../App';
import '../index.css';
import { initTheme } from '../lib/theme';
import '../material-symbols.css';

export type Props = Record<string, string | number | boolean>;

let root: ReactDOM.Root | null = null;

export async function mount(
  component: string,
  _props: Props,
  el: HTMLElement
): Promise<void> {
  if (component !== 'taskscape.capture')
    throw new Error(`no component ${component}`);
  await initTheme();
  root?.unmount();
  root = ReactDOM.createRoot(el);
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
  // Rust emits this on every reveal, once the window is at the cursor.
  await new Promise((r) => requestAnimationFrame(r));
  await emit('mini-shown', 'top');
}

export const COMPONENTS = ['taskscape.capture'];
