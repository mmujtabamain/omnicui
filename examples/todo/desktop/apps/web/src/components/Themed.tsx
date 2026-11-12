import type { ReactNode } from "react";
import type { Theme } from "../model.ts";

/**
 * The app's colours (todo.css): light, dark, or the system's (prefers-color-scheme). Every screen
 * is drawn inside one, so a standalone screen has the theme too.
 */
export function Themed({ theme, className, children }: { theme: Theme; className?: string; children: ReactNode }) {
  return <div className={["todo", className].filter(Boolean).join(" ")} data-theme={theme}>{children}</div>;
}
