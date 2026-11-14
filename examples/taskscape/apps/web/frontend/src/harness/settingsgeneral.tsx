import * as React from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import * as ep from "../api/endpoints";
import { keys } from "../api/queries";
import { AppHarness } from "./lib/AppHarness";
import { SettingsDialog } from "../screens/settings/SettingsDialog";

/** The contract's defaults for the demo account: System, System, on. */
const DEFAULTS = { theme: "system", reduced_motion: "system", show_completed: true } as const;

/**
 * Puts the demo account's three settings back to their defaults before the page reads them, so the
 * open-time values (the discard baseline) are the defaults. Children render once that has landed;
 * the harness waits while the request runs.
 */
function ResetSettings({ children }: { children: React.ReactNode }) {
  const qc = useQueryClient();
  const [ready, setReady] = React.useState(false);
  const reset = useMutation({
    mutationFn: async () => {
      await qc.cancelQueries({ queryKey: keys.settings });
      const saved = await ep.settings.update({ ...DEFAULTS });
      qc.setQueryData(keys.settings, saved);
    },
    onSettled: () => setReady(true),
  });
  const { mutate } = reset;
  React.useEffect(() => {
    mutate();
  }, [mutate]);
  return ready ? <>{children}</> : null;
}

function SettingsGeneralHarness() {
  return (
    <AppHarness path="/settings">
      <ResetSettings>
        <SettingsDialog />
      </ResetSettings>
    </AppHarness>
  );
}

export default { id: "app.settingsgeneral", Component: SettingsGeneralHarness, events: [] };
