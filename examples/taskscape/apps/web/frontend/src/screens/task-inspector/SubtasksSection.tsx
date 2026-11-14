import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { Checkbox } from "@taskscape/ui/checkbox";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useWrite } from "../../api/queries";
import type { Task } from "../../api/types";
import { useUi } from "../../state/ui";
import type { Emit } from "./events";
import { SectionTitle } from "./SectionTitle";

const P = "app.taskinspector.";

/** One subtask row: a done checkbox and a title button that shows the subtask in the inspector. */
function SubtaskItem({
  task,
  depth,
  tasks,
  first,
  emit,
}: {
  task: Task;
  depth: number;
  tasks: Task[];
  first: boolean;
  emit: Emit;
}) {
  const update = useWrite(ep.tasks.update);
  // Local done state, so a second press before the save returns toggles from what is shown.
  const [done, setDone] = React.useState(task.done);
  React.useEffect(() => setDone(task.done), [task.done]);
  const children = tasks.filter((t) => t.parent === task.id);
  const indent = depth === 0 ? "pl-[0.429rem]" : "pl-[1.714rem]";
  const titlePart =
    task.title === "Sourdough loaf" ? `${P}subtaskSourdough` : first ? `${P}subtaskTitle` : undefined;

  const toggle = (next: boolean) => {
    setDone(next);
    update.mutate([task.id, { done: next }]);
    emit("doneChanged", { title: task.title, done: next });
  };

  return (
    <div className="flex min-w-0 flex-col">
      <div
        className={`group/row flex h-[2rem] min-w-0 items-center gap-2 rounded-(--radius-field) pr-2 hover:bg-wash-1 ${indent}`}
      >
        <Checkbox
          data-part={first ? `${P}subtaskDone` : undefined}
          ariaLabel={done ? "Mark not done" : "Mark done"}
          size="sm"
          intent="done"
          checked={done}
          onCheckedChange={toggle}
          className="shrink-0"
        />
        <Button
          data-part={titlePart}
          variant="ghost"
          size="md"
          width="content"
          tone={done ? "muted" : "strong"}
          label={task.title}
          className={`h-[2rem] min-w-0 flex-1 justify-start px-0 text-left ${done ? "line-through" : ""}`}
          onClick={() => {
            useUi.getState().preview(task.id);
            emit("taskSelected", { title: task.title });
          }}
        />
      </div>
      {children.length > 0 && (
        <div className="flex flex-col">
          {children.map((child) => (
            <SubtaskItem key={child.id} task={child} depth={depth + 1} tasks={tasks} first={false} emit={emit} />
          ))}
        </div>
      )}
    </div>
  );
}

/** The Subtasks section: shown only when the task has children. */
export function SubtasksSection({ subtasks, tasks, emit }: { subtasks: Task[]; tasks: Task[]; emit: Emit }) {
  const done = subtasks.filter((child) => child.done).length;
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <SectionTitle
        label="Subtasks"
        part={`${P}subtasksHeading`}
        trailing={
          <Text data-part={`${P}subtasksCount`} size="xs" tone="muted" numeric="tabular">
            {`${done} done / ${subtasks.length}`}
          </Text>
        }
      />
      <div className="flex min-w-0 flex-col">
        {subtasks.map((child, index) => (
          <SubtaskItem key={child.id} task={child} depth={0} tasks={tasks} first={index === 0} emit={emit} />
        ))}
      </div>
    </div>
  );
}
