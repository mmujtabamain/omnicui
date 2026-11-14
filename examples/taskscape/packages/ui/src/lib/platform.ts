// The platform the page runs on, for the few things that differ by OS (shortcut glyphs).
export const isMac = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

const MAC: Record<string, string> = { Mod: "⌘", Ctrl: "⌃", Alt: "⌥", Shift: "⇧", Enter: "↩", Backspace: "⌫", Escape: "⎋" };

/** "Mod+Shift+K" → "⇧⌘K" on macOS, "Ctrl+Shift+K" elsewhere. */
export function formatAccel(accel: string): string {
  const keys = accel.split("+").filter(Boolean);
  if (!isMac) return keys.map((k) => (k === "Mod" ? "Ctrl" : k)).join("+");
  const order = ["Ctrl", "Alt", "Shift", "Mod"];
  const mods = keys.filter((k) => order.includes(k)).sort((a, b) => order.indexOf(a) - order.indexOf(b));
  return mods.map((k) => MAC[k]).join("") + keys.filter((k) => !order.includes(k)).map((k) => MAC[k] ?? k).join("");
}
