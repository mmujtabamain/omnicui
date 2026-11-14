// The harness entry of app.tasklist: mounts the task list pane in the contract's starting state. The page has
// loaded the data set and signed in; this opens the project and the list by name, applies Show completed, then
// renders the pane.
import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import * as ep from "../api/endpoints";
import { keys } from "../api/queries";
import { useUi } from "../state/ui";
import { TaskList } from "../screens/task-list/TaskList";
import { AppHarness } from "./lib/AppHarness";

interface TaskListHarnessProps {
  dataset?: string;
  project?: string;
  list?: string;
  showCompleted?: boolean;
  split?: boolean;
  emit?: (name: string, fields: Record<string, string | number>) => void;
}

function TaskListLoader({ project = "Home", list = "Errands", showCompleted = true, split = false, emit }: TaskListHarnessProps) {
  const qc = useQueryClient();
  const [ready, setReady] = useState(false);

  // The loads go through the query cache, so the page counts them as busy until the pane has its data.
  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const settings = await qc.fetchQuery({ queryKey: keys.settings, queryFn: ep.settings.get, staleTime: 0 });
      if (!showCompleted && settings.show_completed) {
        await qc.fetchQuery({
          queryKey: keys.settings,
          queryFn: () => ep.settings.update({ show_completed: false }),
          staleTime: 0,
        });
      }
      const projects = await qc.fetchQuery({ queryKey: keys.projects, queryFn: ep.projects.list, staleTime: 0 });
      const found = projects.find((p) => p.name === project);
      if (!found) throw new Error(`No project named ${project}`);
      const lists = await qc.fetchQuery({
        queryKey: keys.lists(found.id),
        queryFn: () => ep.lists.list(found.id),
        staleTime: 0,
      });
      const target = lists.find((l) => l.name === list);
      if (!target) throw new Error(`No list named ${list} in ${project}`);
      await qc.fetchQuery({
        queryKey: keys.tasks({ project: found.id }),
        queryFn: () => ep.tasks.list({ project: found.id }),
        staleTime: 0,
      });
      useUi.getState().setProject(found.id);
      useUi.getState().openList(target.id);
      if (!cancelled) setReady(true);
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (!ready) return null;
  return <TaskList split={split} emit={emit} />;
}

function TaskListHarness(props: TaskListHarnessProps) {
  // The pane takes the window's height, as it does in the app's layout, so its tree scrolls within it.
  return (
    <AppHarness>
      <div className="flex h-screen min-h-0 flex-col">
        <TaskListLoader {...props} />
      </div>
    </AppHarness>
  );
}

export default {
  id: "app.tasklist",
  Component: TaskListHarness,
  events: ["taskPreviewed", "listOpened", "copied", "draftStarted", "splitClosed"],
};
