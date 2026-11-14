// Applies the account's theme and reduced-motion settings to <html> (the tokens switch on .dark).
import { useEffect } from "react";
import type { Settings } from "../api/types";

export function useAppearance(s: Settings | undefined) {
  useEffect(() => {
    const html = document.documentElement;
    const dark = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const theme = s?.theme ?? "system";
      html.classList.toggle("dark", theme === "dark" || (theme === "system" && dark.matches));
      html.classList.toggle("reduced-motion", s?.reduced_motion === "on");
      html.classList.toggle("motion-on", s?.reduced_motion === "off");
    };
    apply();
    dark.addEventListener("change", apply);
    return () => dark.removeEventListener("change", apply);
  }, [s?.theme, s?.reduced_motion]);
}
