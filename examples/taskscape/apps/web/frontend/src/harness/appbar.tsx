// The title bar contract (app.appbar): mounts TitleBar signed in as the demo account. Before the
// bar renders, the account's Reduced motion choice is set to the contract's reducedMotion prop
// ("on", "off" or "system"; default "off"), so the indicator shows only for On.
import * as React from "react";
import { useQueryClient } from "@tanstack/react-query";
import { TooltipProvider } from "@taskscape/ui/tooltip";
import { AppHarness } from "./lib/AppHarness";
import * as ep from "../api/endpoints";
import { keys, useWrite } from "../api/queries";
import type { Settings } from "../api/types";
import { TitleBar, type BarEmit } from "../screens/title-bar/TitleBar";

interface AppbarHarnessProps {
  dataset?: string;
  reducedMotion?: string;
  /** The event sink the runner provides: called with the event name and its declared fields. */
  emit?: BarEmit;
}

function AppbarHarness(props: AppbarHarnessProps) {
  return (
    <AppHarness>
      <TooltipProvider>
        <AppbarStage {...props} />
      </TooltipProvider>
    </AppHarness>
  );
}

function AppbarStage({ reducedMotion = "off", emit }: AppbarHarnessProps) {
  const qc = useQueryClient();
  const save = useWrite((patch: Partial<Settings>) => ep.settings.update(patch));
  const [ready, setReady] = React.useState(false);

  // Sets the account's Reduced motion before the bar mounts; the bar reads it from the settings query.
  React.useEffect(() => {
    let live = true;
    const choice = (["on", "off", "system"].includes(reducedMotion) ? reducedMotion : "off") as Settings["reduced_motion"];
    save.mutateAsync([{ reduced_motion: choice }]).then(async (saved) => {
      qc.setQueryData(keys.settings, saved);
      // The bar's projects and each project's lists load before it mounts, so its first render (and every
      // project switch) finds them cached, with no request left to start after the page reports idle.
      const projects = await qc.fetchQuery({ queryKey: keys.projects, queryFn: ep.projects.list });
      await Promise.all(projects.map((p) => qc.fetchQuery({ queryKey: keys.lists(p.id), queryFn: () => ep.lists.list(p.id) })));
      if (live) setReady(true);
    });
    return () => {
      live = false;
    };
    // Set once per mount: the contract's props are fixed for the life of the page.
  }, []);

  if (!ready) return null;
  return <TitleBar emit={emit} />;
}

export default {
  id: "app.appbar",
  Component: AppbarHarness,
  events: [
    "projectSelected",
    "projectRenameRequested",
    "projectDeleteRequested",
    "projectCreateRequested",
    "listCreateRequested",
    "listSelected",
    "listRenamed",
    "listRenameRequested",
    "listDeleteRequested",
    "listExportRequested",
    "splitToggled",
    "panesSwapped",
    "previewToggled",
    "trashOpened",
    "paletteOpened",
    "settingsOpened",
    "signOutRequested",
  ],
};
