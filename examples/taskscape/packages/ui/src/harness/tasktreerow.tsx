import { useState } from "react";
import { TaskTreeRow } from "../components/ui/task-tree-row";

function TaskTreeRowHarness({
  emit,
  done: initialDone = false,
  selected: initialSelected = false,
  expanded: initialExpanded = false,
  ...props
}: any) {
  const [done, setDone] = useState<boolean>(initialDone);
  const [selected, setSelected] = useState<boolean>(initialSelected);
  const [expanded, setExpanded] = useState<boolean>(initialExpanded);
  return (
    <TaskTreeRow
      {...props}
      done={done}
      selected={selected}
      expanded={expanded}
      onActivate={(event) => emit("activate", { platformModifier: event.platformModifier, shift: event.shift })}
      onCheck={(action: "select" | "done") => {
        emit("check", { action });
        if (action === "done") setDone((value) => !value);
        else setSelected((value) => !value);
      }}
      onToggleDone={() => {
        emit("toggleDone", {});
        setDone((value) => !value);
      }}
      onContextMenu={() => emit("contextMenu", {})}
      onDrop={(position: string) => emit("drop", { position })}
      onToggleExpanded={(next: boolean) => {
        emit("toggleExpanded", { expanded: next });
        setExpanded(next);
      }}
    />
  );
}

export default {
  id: "ui.tasktreerow",
  Component: TaskTreeRowHarness,
  events: ["activate", "check", "toggleDone", "contextMenu", "drop", "toggleExpanded"],
};
