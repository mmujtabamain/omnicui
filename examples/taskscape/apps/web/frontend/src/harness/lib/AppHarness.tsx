// Wraps a screen for its contract tests the way main.tsx wraps the app: a fresh query cache, a
// router at `path`, and the account's appearance. omnicui waits while any request is in flight.
import { useEffect, useState, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { useMe, useSettings } from "../../api/queries";
import { useAppearance } from "../../lib/appearance";

export function AppHarness({ path = "/", children }: { path?: string; children: ReactNode }) {
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
