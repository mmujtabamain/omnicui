import { useState } from "react";
import { SubtaskRow } from "../components/ui/subtask-row";

function SubtaskRowHarness({ emit, done: initialDone = false, ...props }: any) {
  const [done, setDone] = useState<boolean>(initialDone);
  return (
    <SubtaskRow
      {...props}
      done={done}
      onToggleDone={(next: boolean) => {
        setDone(next);
        emit("toggled", { done: next });
      }}
      onSelectTask={() => emit("selected", {})}
    />
  );
}

export default { id: "ui.subtaskrow", Component: SubtaskRowHarness, events: ["toggled", "selected"] };
