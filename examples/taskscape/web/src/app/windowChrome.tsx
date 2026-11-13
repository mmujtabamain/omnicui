// The web's windowChrome (replaces tauri/main/src/components/windowChrome.tsx):
// a panel's close button sits at its trailing edge, as on the web generally,
// and "is the window focused" is whether the page has focus.
import type { TitleBarControlsSide } from '@taskscape/common-ui/components';
import { useEffect, useState } from 'react';

export const controlsSide: TitleBarControlsSide = 'trailing';

export function useWindowFocused(): boolean {
  const [focused, setFocused] = useState(() => document.hasFocus());
  useEffect(() => {
    const on = () => setFocused(true);
    const off = () => setFocused(false);
    window.addEventListener('focus', on);
    window.addEventListener('blur', off);
    return () => {
      window.removeEventListener('focus', on);
      window.removeEventListener('blur', off);
    };
  }, []);
  return focused;
}
