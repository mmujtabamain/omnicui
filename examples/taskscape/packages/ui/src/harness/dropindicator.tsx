import { DropIndicator, type DropIndicatorProps } from "../components/ui/drop-indicator";

type Props = Pick<DropIndicatorProps, "target" | "edge" | "visible">;

/** A task row of the demo list the indicator is drawn against. */
function TaskRow({ children }: { children: string }) {
  return (
    <div className="flex h-[1.75rem] items-center rounded-(--radius-field) border border-edge-1 bg-surface-2 px-2 text-[0.929rem] text-content-1">
      {children}
    </div>
  );
}

function DropIndicatorHarness({ target = "row", edge = "before", visible = false }: Props) {
  if (target === "tab")
    return (
      <div className="flex h-[2.5rem] w-[16rem] items-center gap-1 rounded-(--radius-control) border border-edge-1 bg-surface-1 px-2 text-[0.929rem] text-content-2">
        <span className="px-2">Inbox</span>
        <DropIndicator target="tab" visible={visible} />
        <span className="px-2">Today</span>
        <span className="px-2">Done</span>
      </div>
    );

  if (target === "root")
    return (
      <div className="flex w-[16rem] flex-col gap-1 p-2">
        <TaskRow>Write release notes</TaskRow>
        <TaskRow>Review pull request</TaskRow>
        <DropIndicator target="root" visible={visible} />
      </div>
    );

  const line = <DropIndicator target="row" edge={edge} visible={visible} />;
  return (
    <div className="flex w-[16rem] flex-col gap-1 p-2">
      {edge === "before" && line}
      <TaskRow>Write release notes</TaskRow>
      {edge === "after" && line}
      <TaskRow>Review pull request</TaskRow>
    </div>
  );
}

export default { id: "ui.dropindicator", Component: DropIndicatorHarness, events: [] };
