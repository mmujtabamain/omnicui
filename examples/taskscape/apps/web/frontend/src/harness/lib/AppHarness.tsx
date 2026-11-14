// Wraps a screen for its contract tests the way main.tsx wraps the app: a fresh query cache, a
// router at `path`, and the account's appearance. omnicui waits while any request is in flight.
import { useEffect, useState, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { useMe, useSettings } from "../../api/queries";
import { useAppearance } from "../../lib/appearance";

/** The same "random" numbers on every mount (a rolled list name, a shuffled hint), so a test that
 *  compares two runs (a press with its keyboard twin) sees the same screen. */
function seedRandom(seed = 1) {
  let a = seed;
  Math.random = () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function AppHarness({ path = "/", children }: { path?: string; children: ReactNode }) {
  useState(() => seedRandom());
  const [queries] = useState(() => new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } }));
  (window as any).omnicuiBusy = () => queries.isFetching() > 0 || queries.isMutating() > 0;
  return (
    <QueryClientProvider client={queries}>
      <MemoryRouter initialEntries={[path]}>
        <Appearance />
        {children}
      </MemoryRouter>
    </QueryClientProvider>
  );
}

function Appearance() {
  const me = useMe();
  const settings = useSettings();
  useAppearance(me.data ? settings.data : undefined);
  useEffect(() => () => { delete (window as any).omnicuiBusy; }, []);
  return null;
}
