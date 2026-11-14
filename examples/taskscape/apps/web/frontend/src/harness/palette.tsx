// Mounts the command palette open, in the starting state the app.palette contract names: the
// focused list is Errands in Home, and the task prop, when set, is selected the way a click selects it.
import { useEffect, useState } from "react";
import { AppHarness } from "./lib/AppHarness";
import { AppPalette } from "../screens/palette/AppPalette";
import { useLists, useProjects, useTasks } from "../api/queries";
import { useUi } from "../state/ui";

interface PaletteHarnessProps {
  dataset?: string;
  task?: string;
  emit?: (name: string, fields: { label: string }) => void;
}

function PaletteSetup({ task = "", emit }: PaletteHarnessProps) {
  const projects = useProjects();
  const home = projects.data?.find((p) => p.name === "Home");
  const lists = useLists(home?.id);
  const errands = lists.data?.find((l) => l.name === "Errands");
  const tasks = useTasks({ list: errands?.id });
  const [ready, setReady] = useState(false);

  useEffect(() => {
    if (ready || !home || !errands || (task !== "" && !tasks.data)) return;
    const picked = task === "" ? undefined : tasks.data?.find((t) => t.title === task);
    if (task !== "" && !picked) throw new Error(`palette harness: no task "${task}" in Errands`);
    useUi.setState({
      projectId: home.id,
      panes: [errands.id],
      activePane: 0,
      selection: picked ? [picked.id] : [],
      previewId: picked ? picked.id : null,
      paletteOpen: true,
    });
    setReady(true);
  }, [ready, home, errands, tasks.data, task]);

  if (!ready) return null;
  return <AppPalette onRan={(label) => emit?.("ran", { label })} />;
}

function PaletteHarness(props: PaletteHarnessProps) {
  return (
    <AppHarness>
      <PaletteSetup {...props} />
    </AppHarness>
  );
}

export default { id: "app.palette", Component: PaletteHarness, events: ["ran"] };
