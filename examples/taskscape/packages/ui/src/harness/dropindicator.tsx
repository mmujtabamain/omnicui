import { Fragment } from "react";
import { DropIndicator } from "../components/ui/drop-indicator";

const TASKS = ["Draft the quarterly plan", "Review onboarding copy", "Ship the export fix"];

function DropIndicatorHarness({ emit, target = "row", edge = "before", visible = false }: any) {
  // No events: the drop is emitted by the target, so emit is accepted and unused.
  void emit;
  if (target === "tab")
    return (
      <div className="flex h-[2.5rem] w-56 items-center gap-1 rounded-(--radius-panel) border border-edge-1 bg-surface-1 px-2">
        <span className="px-1 text-[0.929rem] font-medium text-content-1">Inbox</span>
        <DropIndicator target="tab" visible={visible} />
        <span className="px-1 text-[0.929rem] text-content-2">Today</span>
        <span className="px-1 text-[0.929rem] text-content-2">Later</span>
      </div>
    );

  const indicator = <DropIndicator target={target} edge={edge} visible={visible} />;

  return (
    <div className="flex w-56 flex-col gap-1 rounded-(--radius-panel) border border-edge-1 bg-surface-1 p-2">
      {TASKS.map((task, i) => (
        <Fragment key={task}>
          {target === "row" && edge === "before" && i === 1 && indicator}
          <div className="flex h-[1.75rem] items-center rounded-(--radius-field) border border-edge-1 bg-surface-3 px-2 text-[0.929rem] text-content-1">
            {task}
          </div>
          {target === "row" && edge === "after" && i === 1 && indicator}
        </Fragment>
      ))}
      {target === "root" && indicator}
    </div>
  );
}

export default { id: "ui.dropindicator", Component: DropIndicatorHarness, events: [] };
