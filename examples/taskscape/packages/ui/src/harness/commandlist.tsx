import { useMemo } from "react";
import { CommandList, type CommandListGroup } from "../components/ui/command-list";

const TASKS = "Tasks";
const LISTS = "Lists";

/** Fixture rows cmd-01 ... cmd-NN, the first half in Tasks and the rest in Lists. */
function buildGroups(rowCount: number): CommandListGroup[] {
  const total = Math.min(Math.max(Math.trunc(rowCount), 0), 100);
  const taskCount = Math.ceil(total / 2);
  const row = (i: number, noun: string) => ({
    id: `cmd-${String(i).padStart(2, "0")}`,
    label: `${noun} action ${String(i).padStart(2, "0")}`,
  });
  const tasks = Array.from({ length: taskCount }, (_, k) => row(k + 1, "Task"));
  const lists = Array.from({ length: total - taskCount }, (_, k) => row(taskCount + k + 1, "List"));
  return [
    ...(tasks.length ? [{ heading: TASKS, rows: tasks }] : []),
    ...(lists.length ? [{ heading: LISTS, rows: lists }] : []),
  ];
}

function CommandListHarness({ emit, rowCount = 6, ...props }: any) {
  const groups = useMemo(() => buildGroups(rowCount), [rowCount]);
  return <CommandList {...props} groups={groups} onPick={(id: string) => emit("picked", { id })} />;
}

export default { id: "ui.commandlist", Component: CommandListHarness, events: ["picked"] };
