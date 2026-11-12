import { useState } from "react";
import { emptyMessage, left, useStore, visible, type TodoStore } from "../model.ts";
import { Parts } from "../omnicui/list.ts";
import { AddTaskDialog } from "./AddTaskDialog.tsx";
import { FilterBar } from "./FilterBar.tsx";
import { PlusIcon } from "./icons.tsx";
import { TaskRow } from "./TaskRow.tsx";

/**
 * The task list, the app's left sidebar: "Tasks" and the Add task button, the filter, how many
 * are left, and the tasks. `current` is the task open in the detail pane, if any.
 */
export function TaskListPane({ store, current, onOpen }: { store: TodoStore; current?: string; onOpen?: (id: string) => void }) {
  const state = useStore(store);
  const [adding, setAdding] = useState(false);
  const tasks = visible(state);
  return (
    <section className="sidebar" aria-labelledby="tasks-heading">
      <div className="sidebar__head">
        <h2 id="tasks-heading" className="sidebar__title" data-part={Parts.tasksHeading}>Tasks</h2>
        <button type="button" className="filled-button" onClick={() => setAdding(true)} data-part={Parts.add}>
          <PlusIcon />Add task
        </button>
      </div>
      <FilterBar filter={state.filter} onFilter={(f) => store.select(f)} />
      <p className="left" role="status" data-part={Parts.left}>{left(state)} left</p>
      {tasks.length === 0 ? (
        <p className="empty" role="status" data-part={Parts.empty}>{emptyMessage(state)}</p>
      ) : (
        <ul className="rows">
          {tasks.map((t, i) => (
            <TaskRow key={t.id} task={t} position={i + 1} current={t.id === current}
              onToggle={(done) => store.setDone(t.id, done)}
              onOpen={() => { store.report("opened", { id: t.id }); onOpen?.(t.id); }} />
          ))}
        </ul>
      )}
      {adding && <AddTaskDialog onClose={() => setAdding(false)} onAdd={(title) => { store.add(title); setAdding(false); }} />}
    </section>
  );
}
