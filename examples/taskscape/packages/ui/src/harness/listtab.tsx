import { ListTab } from "../components/ui/list-tab";

function ListTabHarness({ emit, ...props }: any) {
  return (
    <ListTab
      {...props}
      onSelect={() => emit("select", {})}
      onRenameStart={() => emit("renameStart", {})}
      onRenameCommit={(name: string) => emit("renameCommit", { name })}
      onRenameCancel={() => emit("renameCancel", {})}
      onDelete={() => emit("delete", {})}
      onContextMenu={() => emit("contextMenu", {})}
      onDropTask={(taskId: string) => emit("dropTask", { taskId })}
      onReorder={(fromIndex: number, toIndex: number) => emit("reorder", { fromIndex, toIndex })}
    />
  );
}

export default {
  id: "ui.listtab",
  Component: ListTabHarness,
  events: ["select", "renameStart", "renameCommit", "renameCancel", "delete", "contextMenu", "dropTask", "reorder"],
};
