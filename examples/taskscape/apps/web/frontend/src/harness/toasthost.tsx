// Mounts the Toast alone (app.toasthost) on the demo account. The props run the deletes through the
// API first, in the order the contract gives: bulkTrash, then trash, then trashAgain. The page's
// request wait covers them because they run inside a React Query query.
import { useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { AppHarness } from "./lib/AppHarness";
import * as ep from "../api/endpoints";
import type { Task } from "../api/types";
import { ToastHost, resetToast, showUndoToast, type Restored, type UndoFn } from "../screens/toast";

interface ToastHarnessProps {
  dataset?: string;
  bulkTrash?: number;
  trash?: string;
  trashAgain?: string;
  undoFails?: boolean;
  emit?: (event: string, payload: Record<string, unknown>) => void;
}

const byOrder = (a: Task, b: Task) => a.sort_order - b.sort_order || a.id - b.id;

/** An Undo that restores the ids. With undoFails, the first Undo request fails; later ones are real. */
function undoer(ids: number[], result: Restored, failNext: { current: boolean }): UndoFn {
  return async () => {
    if (failNext.current) {
      failNext.current = false;
      throw new Error("Could not restore the task");
    }
    await ep.trash.restore(ids);
    return result;
  };
}

async function seed(props: ToastHarnessProps, failNext: { current: boolean }): Promise<true> {
  resetToast();
  const projects = await ep.projects.list();
  const home = projects.find((p) => p.name === "Home");
  const lists = home ? await ep.lists.list(home.id) : [];
  const errands = lists.find((l) => l.name === "Errands");
  if (!errands) return true;

  const loadErrands = async () => (await ep.tasks.list({ list: errands.id })).sort(byOrder);

  if (props.bulkTrash && props.bulkTrash > 0) {
    const roots = (await loadErrands()).filter((t) => t.parent == null).slice(0, props.bulkTrash);
    if (roots.length > 0) {
      const res = await ep.tasks.bulk(roots.map((r) => r.id), "trash");
      const trashed = res.trashed ?? roots.map((r) => r.id);
      showUndoToast(`${roots.length} moved to Trash`, undoer(trashed, { count: roots.length, title: roots[0].title }, failNext));
    }
  }

  const trashOne = async (title: string) => {
    const task = (await loadErrands()).find((t) => t.title === title);
    if (!task) return;
    const res = await ep.tasks.trash(task.id);
    showUndoToast("Moved to Trash", undoer(res.trashed, { count: 1, title: task.title }, failNext));
  };

  if (props.trash) await trashOne(props.trash);
  if (props.trashAgain) await trashOne(props.trashAgain);
  return true;
}

function Seeded(props: ToastHarnessProps) {
  const failNext = useRef(!!props.undoFails);
  const seeded = useQuery({
    queryKey: ["toasthost-seed", props.dataset ?? "standard", props.bulkTrash ?? 0, props.trash ?? "", props.trashAgain ?? ""],
    queryFn: () => seed(props, failNext),
  });
  if (!seeded.isSuccess) return null;
  return (
    <ToastHost
      onUndone={(restored) => props.emit?.("undone", { count: restored.count, title: restored.title })}
      onDismissed={(via) => props.emit?.("dismissed", { via })}
    />
  );
}

function ToastHarness(props: ToastHarnessProps) {
  return (
    <AppHarness>
      <Seeded {...props} />
    </AppHarness>
  );
}

export default { id: "app.toasthost", Component: ToastHarness, events: ["undone", "dismissed"] };
