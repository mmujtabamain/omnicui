// Lets a screen name the parts inside a composed widget for its tests: parts={{ accept: "app.trash.confirm" }}
// puts data-part="app.trash.confirm" on the element whose data-slot is "accept" (the root itself
// under the key "root"). Applied after every render, so parts that appear later are named too.
import * as React from "react";

export type PartNames = Partial<Record<string, string>>;

export function useParts<T extends HTMLElement>(ref: React.RefObject<T | null>, parts: PartNames | undefined) {
  React.useLayoutEffect(() => {
    const root = ref.current;
    if (!root || !parts) return;
    for (const [slot, part] of Object.entries(parts)) {
      if (!part) continue;
      const el = slot === "root" ? root : root.querySelector(`[data-slot="${slot}"]`);
      el?.setAttribute("data-part", part);
    }
  });
}

/** Joins a forwarded ref and a local one. */
export function useMergedRef<T>(forwarded: React.ForwardedRef<T>, local: React.MutableRefObject<T | null>) {
  return React.useCallback((node: T | null) => {
    local.current = node;
    if (typeof forwarded === "function") forwarded(node);
    else if (forwarded) forwarded.current = node;
  }, [forwarded, local]);
}
