import type { Task } from "../model.ts";

/** The parts of the row at `position` (1 is the top row): todo.list.row<N> and row<N>Check. */
export const rowPart = (position: number) => `todo.list.row${position}`;
export const rowCheckPart = (position: number) => `todo.list.row${position}Check`;

/**
 * One task: a checkbox that marks it done (named by the title), and the title as a button that
 * opens the task. The open task's row is marked as the current one.
 */
export function TaskRow({ task, position, current, onToggle, onOpen }: {
  task: Task; position: number; current: boolean; onToggle: (done: boolean) => void; onOpen: () => void;
}) {
  return (
    <li className={["row", task.done && "row--done", current && "row--current"].filter(Boolean).join(" ")}>
      <input type="checkbox" className="check" checked={task.done} aria-label={task.title} data-part={rowCheckPart(position)} onChange={(e) => onToggle(e.target.checked)} />
      <button type="button" className="row__title" aria-current={current || undefined} data-part={rowPart(position)} onClick={onOpen}>
        {task.title}
      </button>
    </li>
  );
}
