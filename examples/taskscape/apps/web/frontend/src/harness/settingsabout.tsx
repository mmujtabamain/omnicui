// How omnicui mounts the About screen (app.settingsabout): the demo account's settings are put into the
// contract's starting state, then the Settings screen opens on About.
import * as React from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import * as ep from "../api/endpoints";
import { keys } from "../api/queries";
import { AppHarness } from "./lib/AppHarness";
import { AboutPage } from "../screens/settings/AboutPage";

/**
 * The starting state the contract needs: appearance at System, and Show completed off. A reset then
 * changes Show completed back on, so the discard control has a change to revert.
 */
const START = { theme: "system", reduced_motion: "system", show_completed: false } as const;

/** Saves the starting state before the page reads it, so the open-time values are these. Children render once it has landed. */
function StartState({ children }: { children: React.ReactNode }) {
  const qc = useQueryClient();
  const [ready, setReady] = React.useState(false);
  const put = useMutation({
    mutationFn: async () => {
      await qc.cancelQueries({ queryKey: keys.settings });
      const saved = await ep.settings.update({ ...START });
      qc.setQueryData(keys.settings, saved);
      // Loaded here too, so the page mounts with its data cached and no request starts after the gate opens.
      await qc.fetchQuery({ queryKey: keys.shortcuts, queryFn: ep.settings.shortcuts });
    },
    onSettled: () => setReady(true),
  });
  const { mutate } = put;
  React.useEffect(() => {
    mutate();
  }, [mutate]);
  return ready ? <>{children}</> : null;
}

interface SettingsAboutHarnessProps {
  dataset?: string;
  /** ctx.version; "" shows an em dash. */
  version?: string;
  /** Desktop only; the web build has no data folder row. */
  dataFolder?: string;
  emit?: (name: string, fields: Record<string, string>) => void;
}

function SettingsAboutHarness(props: SettingsAboutHarnessProps) {
  const version = props.version ?? "0.1.0";
  const { emit } = props;
  return (
    <AppHarness path="/settings">
      <StartState>
        <AboutPage version={version} onResetAll={() => emit?.("resetAll", { scope: "all" })} />
      </StartState>
    </AppHarness>
  );
}

export default { id: "app.settingsabout", Component: SettingsAboutHarness, events: ["resetAll"] };
