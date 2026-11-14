import { useState } from "react";
import { TaskRow } from "../components/ui/task-row";

function TaskRowHarness({ emit, title = "", done = false, ...props }: any) {
  const [isDone, setDone] = useState<boolean>(done);
  return (
    <div role="list">
      <TaskRow
        {...props}
        title={title}
        done={isDone}
        check="done"
        onCheckedChange={(next: boolean) => {
          setDone(next);
          emit("toggled", { done: next });
        }}
        onRemove={() => emit("removed", { title })}
      />
    </div>
  );
}

export default { id: "ui.taskrow", Component: TaskRowHarness, events: ["toggled", "removed"] };
