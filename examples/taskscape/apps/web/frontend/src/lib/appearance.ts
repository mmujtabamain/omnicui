// Applies the account's theme and reduced-motion settings to <html> (the tokens switch on .dark). The theme is
// also kept in localStorage, where index.html reads it before the first paint, so a theme that differs from the
// system's never shows the wrong colours while the app loads (as in the reference).
import { useEffect } from "react";
import type { Settings } from "../api/types";

export const THEME_KEY = "taskscape-theme";

function storedTheme(): Settings["theme"] | null {
  try {
    const t = localStorage.getItem(THEME_KEY);
    return t === "light" || t === "dark" || t === "system" ? t : null;
  } catch {
    return null;
  }
}

export function useAppearance(s: Settings | undefined) {
  useEffect(() => {
    const html = document.documentElement;
    const dark = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const theme = s?.theme ?? storedTheme() ?? "system";
      if (s?.theme) {
        try {
          localStorage.setItem(THEME_KEY, s.theme);
        } catch {
          // private mode: the theme is still applied, just not remembered
        }
      }
      html.classList.toggle("dark", theme === "dark" || (theme === "system" && dark.matches));
      html.classList.toggle("reduced-motion", s?.reduced_motion === "on");
      html.classList.toggle("motion-on", s?.reduced_motion === "off");
    };
    apply();
    dark.addEventListener("change", apply);
    return () => dark.removeEventListener("change", apply);
  }, [s?.theme, s?.reduced_motion]);
}
