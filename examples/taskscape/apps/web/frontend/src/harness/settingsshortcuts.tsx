// Mounts the Settings dialog on its Shortcuts tab, signed in as the demo account. Each saved,
// reset, unbound or discarded shortcut is reported as the bindingChanged event.
import * as React from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import * as ep from "../api/endpoints";
import { keys } from "../api/queries";
import { AppHarness } from "./lib/AppHarness";
import { ShortcutsPage } from "../screens/settings/ShortcutsPage";

interface BindingChangedFields {
  label: string;
  accel: string;
}

interface SettingsShortcutsProps {
  emit?: (name: "bindingChanged", fields: BindingChangedFields) => void;
}

/**
 * Puts every shortcut of the demo account back to its default before the page reads them, so the
 * open-time values (the undo baseline) are the defaults. Children render once that has landed; the
 * harness waits while the requests run.
 */
function ResetShortcuts({ children }: { children: React.ReactNode }) {
  const qc = useQueryClient();
  const [ready, setReady] = React.useState(false);
  const reset = useMutation({
    mutationFn: async () => {
      await qc.cancelQueries({ queryKey: keys.shortcuts });
      const list = await ep.settings.shortcuts();
      let latest = list;
      for (const binding of list) latest = await ep.settings.bind(binding.command, null);
      qc.setQueryData(keys.shortcuts, latest);
    },
    onSettled: () => setReady(true),
  });
  const { mutate } = reset;
  React.useEffect(() => {
    mutate();
  }, [mutate]);
  return ready ? <>{children}</> : null;
}

function SettingsShortcutsHarness({ emit }: SettingsShortcutsProps) {
  return (
    <AppHarness>
      <ResetShortcuts>
        <ShortcutsPage onBindingChanged={(label, accel) => emit?.("bindingChanged", { label, accel })} />
      </ResetShortcuts>
    </AppHarness>
  );
}

export default { id: "app.settingsshortcuts", Component: SettingsShortcutsHarness, events: ["bindingChanged"] };
