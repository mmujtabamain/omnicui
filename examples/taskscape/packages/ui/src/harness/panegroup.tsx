import { PaneGroup } from "../components/ui/pane-group";

function Demo({ title, lines }: { title: string; lines: string[] }) {
  return (
    <div className="flex h-full min-h-0 flex-col gap-1.5 p-3">
      <div className="truncate text-[1rem] font-semibold text-content-1">{title}</div>
      {lines.map((line) => (
        <div key={line} className="truncate text-[0.929rem] text-content-2">
          {line}
        </div>
      ))}
    </div>
  );
}

function PaneGroupHarness({ emit, ...props }: any) {
  return (
    <div className="flex h-screen min-h-0 w-full">
      <PaneGroup
        {...props}
        taskLabel="Tasks"
        splitLabel="Split"
        asideLabel="Preview"
        taskContent={<Demo title="Tasks" lines={["Write the quarterly plan", "Review the budget", "Call the vendor"]} />}
        splitContent={<Demo title="Split" lines={["Notes for the review", "Open questions"]} />}
        asideContent={<Demo title="Preview" lines={["Selected task", "Due Friday"]} />}
        onLayoutChanged={(layout) => emit("layoutChanged", layout)}
      />
    </div>
  );
}

export default { id: "ui.panegroup", Component: PaneGroupHarness, events: ["layoutChanged"] };
