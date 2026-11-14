// Mounts the Filter & Sort screen in the contract's start states (app.filterpanel). The demo data set is
// loaded and the account signed in before this renders. Here the store gets the active list (Errands in
// the standard set) and the panel state, and the Show completed setting is stored off when asked.
import { useEffect, useRef, useState, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import * as ep from "../api/endpoints";
import { keys, useLists, useProjects, useSettings } from "../api/queries";
import { useUi } from "../state/ui";
import { FilterPanel } from "../screens/filter-panel/FilterPanel";
import { AppHarness } from "./lib/AppHarness";

function Prime({ panel, showCompleted, children }: { panel: string; showCompleted: boolean; children: ReactNode }) {
  const qc = useQueryClient();
  const projects = useProjects();
  const home = projects.data?.find((p) => p.name === "Home") ?? projects.data?.[0];
  const lists = useLists(home?.id);
  const settings = useSettings();
  // The account's Show completed setting is stored as the prop says before the screen mounts.
  const save = useMutation({
    mutationFn: (value: boolean) => ep.settings.update({ show_completed: value }),
    onSuccess: (next) => qc.setQueryData(keys.settings, next),
  });
  const { mutate } = save;
  const needsSave = settings.data !== undefined && settings.data.show_completed !== showCompleted;
  const sent = useRef(false);
  useEffect(() => {
    if (needsSave && !sent.current) {
      sent.current = true;
      mutate(showCompleted);
    }
  }, [needsSave, mutate, showCompleted]);

  const ready = projects.isSuccess && (home === undefined || lists.isSuccess) && settings.isSuccess && !needsSave;
  const [applied, setApplied] = useState(false);
  useEffect(() => {
    if (!ready || applied) return;
    const errands = lists.data?.find((l) => l.name === "Errands") ?? lists.data?.[0];
    useUi.setState({
      projectId: home?.id ?? null,
      panes: [errands?.id ?? null],
      activePane: 0,
      selection: [],
      previewId: null,
      aside: panel === "closed" ? "preview" : "filter",
    });
    setApplied(true);
  }, [ready, applied, home?.id, lists.data, panel]);

  if (!applied) return null;
  return <>{children}</>;
}

function FilterPanelHarness(props: { panel?: string; showCompleted?: boolean }) {
  return (
    <AppHarness>
      <Prime panel={props.panel ?? "open"} showCompleted={props.showCompleted ?? true}>
        <FilterPanel />
      </Prime>
    </AppHarness>
  );
}

export default { id: "app.filterpanel", Component: FilterPanelHarness, events: [] };
