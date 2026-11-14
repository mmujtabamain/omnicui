import { PaneGroup, PaneGroupAside, PaneGroupSplit, PaneGroupTask } from "../components/ui/pane-group";

function DemoRows({ prefix, count }: { prefix: string; count: number }) {
  return (
    <div className="flex flex-col">
      {Array.from({ length: count }, (_, i) => (
        <div key={i} className="border-b border-edge-1 px-3 py-2 text-[0.929rem] text-content-1">
          {prefix} {i + 1}
        </div>
      ))}
    </div>
  );
}

function PaneGroupHarness({ emit, ...props }: any) {
  return (
    <div className="flex h-[min(36rem,100vh)] w-full min-w-0">
      <PaneGroup {...props} onLayoutChanged={(layout) => emit("layoutChanged", layout)}>
        <PaneGroupTask heading="Tasks">
          <DemoRows prefix="Task" count={12} />
        </PaneGroupTask>
        <PaneGroupSplit heading="Notes">
          <DemoRows prefix="Note" count={8} />
        </PaneGroupSplit>
        <PaneGroupAside heading="Preview">
          <div className="px-3 py-2 text-[0.929rem] text-content-2">Preview of the selected task.</div>
        </PaneGroupAside>
      </PaneGroup>
    </div>
  );
}

export default { id: "ui.panegroup", Component: PaneGroupHarness, events: ["layoutChanged"] };
