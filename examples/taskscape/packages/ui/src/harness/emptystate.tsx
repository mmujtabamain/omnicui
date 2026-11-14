import { EmptyState } from "../components/ui/empty-state";

function EmptyStateHarness(props: any) {
  // A pane fills the task pane, so the demo mount gives it the viewport height to fill.
  if (props.scope === "pane")
    return (
      <div className="h-screen w-full">
        <EmptyState {...props} />
      </div>
    );
  return <EmptyState {...props} />;
}

export default { id: "ui.emptystate", Component: EmptyStateHarness, events: [] };
