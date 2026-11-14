// The task inspector as omnicui mounts it. The task is looked up by title in the demo data, its
// list is opened in the store, and the inspector is shown on it.
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { TooltipProvider } from "@taskscape/ui/tooltip";
import * as ep from "../api/endpoints";
import { keys } from "../api/queries";
import { TaskInspector } from "../screens/task-inspector/TaskInspector";
import type { Emit } from "../screens/task-inspector/events";
import { useUi } from "../state/ui";
import { AppHarness } from "./lib/AppHarness";

async function findTask(title: string) {
  const projects = await ep.projects.list();
  const perProject = await Promise.all(projects.map((project) => ep.tasks.list({ project: project.id })));
  for (let i = 0; i < projects.length; i++) {
    const task = perProject[i].find((t) => t.title === title);
    if (task) return { project: projects[i].id, list: task.list, task: task.id };
  }
  throw new Error(`No task titled "${title}" in the data set`);
}

/**
 * Keeps the runner waiting until the inspector is on screen. AppHarness reports busy only while
 * requests run; the gap between the last response and the first render of the inspector is not
 * covered, so the runner can read parts too early. This wraps the busy reporter (AppHarness sets it
 * on every render, which the setter forwards) and reports busy until `ready` is true and the title
 * field is in the DOM.
 */
function useBusyGate(ready: boolean) {
  const readyRef = useRef(ready);
  readyRef.current = ready;
  useLayoutEffect(() => {
    const w = window as unknown as Record<string, unknown>;
    let base = w.omnicuiBusy as (() => boolean) | undefined;
    Object.defineProperty(window, "omnicuiBusy", {
      configurable: true,
      get: () => () =>
        !readyRef.current ||
        document.querySelector('[data-part="app.taskinspector.title"]') === null ||
        (base ? base() : false),
      set: (fn: (() => boolean) | undefined) => {
        base = fn;
      },
    });
    return () => {
      delete w.omnicuiBusy;
    };
  }, []);
}

function Shown({ title, emit }: { title: string; emit: Emit }) {
  const qc = useQueryClient();
  const target = useQuery({
    queryKey: ["harness", "taskinspector", title],
    queryFn: () => findTask(title),
    staleTime: Infinity,
  });
  const [ready, setReady] = useState(false);
  useBusyGate(ready);

  // The data the inspector reads is loaded before it mounts, so its first render already has it.
  useEffect(() => {
    const found = target.data;
    if (!found) return;
    let cancelled = false;
    Promise.all([
      qc.prefetchQuery({
        queryKey: keys.tasks({ list: found.list }),
        queryFn: () => ep.tasks.list({ list: found.list }),
      }),
      qc.prefetchQuery({ queryKey: keys.projects, queryFn: ep.projects.list }),
      qc.prefetchQuery({ queryKey: keys.lists(found.project), queryFn: () => ep.lists.list(found.project) }),
    ]).then(() => {
      if (cancelled) return;
      useUi.setState({
        projectId: found.project,
        panes: [found.list],
        activePane: 0,
        selection: [],
        previewId: found.task,
      });
      setReady(true);
    });
    return () => {
      cancelled = true;
    };
  }, [target.data, qc]);

  if (!ready) return null;
  return <TaskInspector emit={emit} />;
}

function TaskInspectorHarness(props: { dataset?: string; task?: string; emit: Emit }) {
  return (
    <AppHarness>
      <TooltipProvider>
        <Shown title={props.task ?? "Book bike service"} emit={props.emit} />
      </TooltipProvider>
    </AppHarness>
  );
}

export default {
  id: "app.taskinspector",
  Component: TaskInspectorHarness,
  events: [
    "titleChanged",
    "doneChanged",
    "previewOpenChanged",
    "deleteRequested",
    "taskSelected",
    "notesChanged",
    "noteSaved",
    "attachRequested",
    "linkRequested",
    "attachmentAction",
  ],
};
